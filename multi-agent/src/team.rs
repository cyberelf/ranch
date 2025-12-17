use crate::manager::AgentManager;
use crate::{Agent, AgentInfo};
use a2a_protocol::prelude::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cycle detection for nested teams
#[derive(Debug, thiserror::Error)]
#[error("Cycle detected in team nesting: {0}")]
pub struct CycleError(pub String);

/// Check if a team can be registered without creating a cycle
///
/// # Arguments
/// * `team_id` - ID of the team being registered
/// * `visited` - Set of already visited team IDs in the current path
///
/// # Returns
/// * `Ok(())` if no cycle detected
/// * `Err(CycleError)` if a cycle would be created
pub fn track_team_nesting(team_id: &str, visited: &mut HashSet<String>) -> Result<(), CycleError> {
    if visited.contains(team_id) {
        return Err(CycleError(format!(
            "Team '{}' would create a cycle (path: {})",
            team_id,
            visited.iter().cloned().collect::<Vec<_>>().join(" -> ")
        )));
    }
    visited.insert(team_id.to_string());
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Recipient {
    pub agent_id: Option<String>,
    pub should_return_to_user: bool,
    pub context_updates: HashMap<String, String>,
}

impl Recipient {
    pub fn agent(agent_id: String) -> Self {
        Self {
            agent_id: Some(agent_id),
            should_return_to_user: false,
            context_updates: HashMap::new(),
        }
    }

    pub fn user() -> Self {
        Self {
            agent_id: None,
            should_return_to_user: true,
            context_updates: HashMap::new(),
        }
    }

    pub fn with_context_updates(mut self, updates: HashMap<String, String>) -> Self {
        self.context_updates = updates;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub mode: TeamMode,
    pub agents: Vec<TeamAgentConfig>,
    pub scheduler_config: SchedulerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TeamMode {
    Supervisor,
    Workflow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorSchedulerConfig {
    pub supervisor_agent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSchedulerConfig {
    pub steps: Vec<WorkflowStepConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStepConfig {
    pub agent_id: String,
    pub order: u32,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SchedulerConfig {
    #[serde(rename = "supervisor")]
    Supervisor(SupervisorSchedulerConfig),
    #[serde(rename = "workflow")]
    Workflow(WorkflowSchedulerConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAgentConfig {
    pub agent_id: String,
    pub role: String,
    pub capabilities: Vec<String>,
}

pub struct Team {
    config: TeamConfig,
    agent_manager: Arc<AgentManager>,
    scheduler: Arc<dyn Scheduler>,
}

impl Team {
    /// Create a new team from configuration
    pub fn new(config: TeamConfig, agent_manager: Arc<AgentManager>) -> Self {
        let scheduler: Arc<dyn Scheduler> = match &config.scheduler_config {
            SchedulerConfig::Supervisor(supervisor_config) => {
                Arc::new(SupervisorScheduler::new(supervisor_config.clone()))
            }
            SchedulerConfig::Workflow(workflow_config) => {
                Arc::new(WorkflowScheduler::new(workflow_config.clone()))
            }
        };

        Self {
            config,
            agent_manager,
            scheduler,
        }
    }

    /// Create a team from a team ID in the config, using pre-registered agents
    ///
    /// # Arguments
    /// * `config` - The full configuration containing team definitions
    /// * `team_id` - The ID of the team to create
    /// * `agent_manager` - Agent manager with pre-registered agents
    ///
    /// # Returns
    /// * `Ok(Team)` if team found in config
    /// * `Err` if team not found
    ///
    /// # Example
    /// ```no_run
    /// use multi_agent::*;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::from_file("config.toml")?;
    /// let agent_manager = Arc::new(AgentManager::new());
    ///
    /// // Register agents...
    ///
    /// let team = Team::from_config(&config, "my-team-id", agent_manager)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_config(
        config: &crate::Config,
        team_id: &str,
        agent_manager: Arc<AgentManager>,
    ) -> Result<Self, TeamError> {
        let team_configs = config.to_team_configs();
        let team_config = team_configs
            .iter()
            .find(|team| team.id == team_id)
            .ok_or_else(|| {
                TeamError::Configuration(format!("Team '{}' not found in configuration", team_id))
            })?
            .clone();

        Ok(Self::new(team_config, agent_manager))
    }

    pub async fn process_message(&self, message: Message) -> Result<Message, TeamError> {
        self.process_messages(vec![message]).await
    }

    pub async fn process_messages(
        &self,
        initial_messages: Vec<Message>,
    ) -> Result<Message, TeamError> {
        let mut current_messages = initial_messages;
        let mut last_response: Option<Message> = None;
        let mut context = HashMap::new();

        loop {
            let recipient = self
                .scheduler
                .determine_next_recipient(
                    &self.config,
                    &self.agent_manager,
                    current_messages.clone(),
                    last_response.clone(),
                    &context,
                )
                .await?;

            // Apply context updates from scheduler
            context.extend(recipient.context_updates);

            if recipient.should_return_to_user {
                if let Some(response) = last_response {
                    return Ok(response);
                } else {
                    return Err(TeamError::Scheduling("No response generated".to_string()));
                }
            }

            let agent_id = recipient
                .agent_id
                .ok_or_else(|| TeamError::Scheduling("No agent ID provided".to_string()))?;

            let agent = self
                .agent_manager
                .get(&agent_id)
                .await
                .ok_or_else(|| TeamError::Agent(format!("Agent {} not found", agent_id)))?;

            // Process the last message with the selected agent
            let input_message = current_messages
                .last()
                .ok_or_else(|| TeamError::Scheduling("No messages to process".to_string()))?;

            println!(
                "  📞 Processing message with agent: {} ({})",
                agent_id,
                agent
                    .info()
                    .await
                    .unwrap_or_else(|_| AgentInfo {
                        id: agent_id.clone(),
                        name: "Unknown Agent".to_string(),
                        description: "Agent information unavailable".to_string(),
                        capabilities: vec![],
                        metadata: HashMap::new(),
                    })
                    .name
            );
            last_response = Some(
                agent
                    .process(input_message.clone())
                    .await
                    .map_err(|e| TeamError::Agent(e.to_string()))?,
            );
            println!("  ✅ Agent {} completed processing", agent_id);

            // Prepare messages for next iteration
            current_messages = vec![last_response.clone().unwrap()];
        }
    }

    pub fn get_config(&self) -> &TeamConfig {
        &self.config
    }

    pub async fn health_check(&self) -> Vec<(String, bool)> {
        let mut results = Vec::new();

        for agent_config in &self.config.agents {
            if let Some(agent) = self.agent_manager.get(&agent_config.agent_id).await {
                // Try to use health_check method
                let healthy = agent.health_check().await;
                results.push((agent_config.agent_id.clone(), healthy));
            } else {
                results.push((agent_config.agent_id.clone(), false));
            }
        }

        results
    }
}

// Implement Agent trait for Team to enable teams as agents
#[async_trait]
impl crate::agent::Agent for Team {
    /// Get agent information for the team
    ///
    /// Returns aggregated capabilities from all member agents
    async fn info(&self) -> A2aResult<AgentInfo> {
        let mut capabilities = HashSet::new();

        // Aggregate capabilities from all member agents
        for agent_config in &self.config.agents {
            if let Some(agent) = self.agent_manager.get(&agent_config.agent_id).await {
                match agent.info().await {
                    Ok(info) => {
                        capabilities.extend(info.capabilities);
                    }
                    Err(e) => {
                        // Log warning but continue - graceful degradation
                        eprintln!(
                            "Warning: Failed to get info from agent {}: {}",
                            agent_config.agent_id, e
                        );
                    }
                }
            }
        }

        // Add team-specific capabilities from config
        for agent_config in &self.config.agents {
            capabilities.extend(agent_config.capabilities.clone());
        }

        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), "team".to_string());
        metadata.insert(
            "mode".to_string(),
            match self.config.mode {
                TeamMode::Supervisor => "supervisor".to_string(),
                TeamMode::Workflow => "workflow".to_string(),
            },
        );
        metadata.insert(
            "member_count".to_string(),
            self.config.agents.len().to_string(),
        );

        Ok(AgentInfo {
            id: self.config.id.clone(),
            name: self.config.name.clone(),
            description: self.config.description.clone(),
            capabilities: capabilities.into_iter().collect(),
            metadata,
        })
    }

    /// Process a message through the team's orchestration
    ///
    /// Delegates to scheduler which routes to appropriate member agents
    async fn process(&self, message: Message) -> A2aResult<Message> {
        self.process_message(message)
            .await
            .map_err(|e| A2aError::Internal(e.to_string()))
    }

    /// Check if team is healthy - all member agents responsive
    async fn health_check(&self) -> bool {
        let results = Team::health_check(self).await;
        // Team is healthy if at least one agent is healthy
        results.iter().any(|(_, healthy)| *healthy)
    }
}

#[async_trait]
pub trait Scheduler: Send + Sync {
    async fn determine_next_recipient(
        &self,
        team_config: &TeamConfig,
        agent_manager: &AgentManager,
        messages: Vec<Message>,
        last_response: Option<Message>,
        context: &HashMap<String, String>,
    ) -> Result<Recipient, TeamError>;
}

#[derive(Debug, thiserror::Error)]
pub enum TeamError {
    #[error("Agent error: {0}")]
    Agent(String),

    #[error("No agent available for request")]
    NoAgentAvailable,

    #[error("Scheduling error: {0}")]
    Scheduling(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
}

pub struct SupervisorScheduler {
    config: SupervisorSchedulerConfig,
    #[allow(dead_code)] // Reserved for future stateful scheduling
    context: RwLock<HashMap<String, String>>,
    call_count: RwLock<usize>,
}

impl SupervisorScheduler {
    pub fn new(config: SupervisorSchedulerConfig) -> Self {
        Self {
            config,
            context: RwLock::new(HashMap::new()),
            call_count: RwLock::new(0),
        }
    }

    async fn get_supervisor_agent(&self, agent_manager: &AgentManager) -> Option<Arc<dyn Agent>> {
        agent_manager.get(&self.config.supervisor_agent_id).await
    }
}

#[async_trait]
impl Scheduler for SupervisorScheduler {
    async fn determine_next_recipient(
        &self,
        _team_config: &TeamConfig,
        agent_manager: &AgentManager,
        _messages: Vec<Message>,
        last_response: Option<Message>,
        _context: &HashMap<String, String>,
    ) -> Result<Recipient, TeamError> {
        let _supervisor = self
            .get_supervisor_agent(agent_manager)
            .await
            .ok_or_else(|| {
                TeamError::Configuration("No supervisor agent configured".to_string())
            })?;

        // Track call count to prevent infinite loops
        let mut count = self.call_count.write().await;
        *count += 1;

        // If this is the first call, route to supervisor
        // After the supervisor responds, return to user
        if *count == 1 {
            Ok(Recipient::agent(self.config.supervisor_agent_id.clone()))
        } else if last_response.is_some() {
            // Reset counter for next message
            *count = 0;
            Ok(Recipient::user())
        } else {
            // Safety check - avoid infinite loop
            *count = 0;
            Ok(Recipient::user())
        }
    }
}

pub struct WorkflowScheduler {
    config: WorkflowSchedulerConfig,
    current_step: RwLock<usize>,
}

impl WorkflowScheduler {
    pub fn new(config: WorkflowSchedulerConfig) -> Self {
        let mut steps = config.steps;
        steps.sort_by_key(|step| step.order);

        Self {
            config: WorkflowSchedulerConfig { steps },
            current_step: RwLock::new(0),
        }
    }
}

#[async_trait]
impl Scheduler for WorkflowScheduler {
    async fn determine_next_recipient(
        &self,
        _team_config: &TeamConfig,
        _agent_manager: &AgentManager,
        _messages: Vec<Message>,
        last_response: Option<Message>,
        _context: &HashMap<String, String>,
    ) -> Result<Recipient, TeamError> {
        let mut current_step = self.current_step.write().await;

        if *current_step >= self.config.steps.len() {
            println!("🏁 Workflow finished - returning result to user");
            return Ok(Recipient::user());
        }

        let step = &self.config.steps[*current_step];

        // Check if there's a condition for this step
        if let Some(condition) = &step.condition {
            if let Some(response) = &last_response {
                // Simple condition checking - check message content
                if let Some(content) = crate::adapters::extract_text(response) {
                    if !content.contains(condition) {
                        *current_step += 1;
                        return self
                            .determine_next_recipient(
                                _team_config,
                                _agent_manager,
                                _messages,
                                last_response,
                                _context,
                            )
                            .await;
                    }
                }
            }
        }

        let agent_id = step.agent_id.clone();
        let step_number = *current_step + 1;
        *current_step += 1;

        // Debug logging for workflow step
        println!(
            "🔄 Workflow Step {}: Calling agent '{}'",
            step_number, agent_id
        );

        // If this was the last step, return to user next time
        if *current_step >= self.config.steps.len() {
            println!("✅ Workflow completed after {} steps", step_number);
            let mut context_updates = HashMap::new();
            context_updates.insert("workflow_complete".to_string(), "true".to_string());
            return Ok(Recipient::agent(agent_id).with_context_updates(context_updates));
        }

        Ok(Recipient::agent(agent_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{Agent, AgentInfo};
    use a2a_protocol::prelude::{A2aResult, Message};
    use async_trait::async_trait;

    struct MockAgent {
        id: String,
        name: String,
        capabilities: Vec<String>,
        response: String,
    }

    impl MockAgent {
        fn new(id: &str, name: &str) -> Self {
            Self {
                id: id.to_string(),
                name: name.to_string(),
                capabilities: vec![],
                response: "Mock response".to_string(),
            }
        }

        fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
            self.capabilities = capabilities;
            self
        }

        fn with_response(mut self, response: &str) -> Self {
            self.response = response.to_string();
            self
        }
    }

    #[async_trait]
    impl Agent for MockAgent {
        async fn info(&self) -> A2aResult<AgentInfo> {
            Ok(AgentInfo {
                id: self.id.clone(),
                name: self.name.clone(),
                description: "Mock agent for testing".to_string(),
                capabilities: self.capabilities.clone(),
                metadata: HashMap::new(),
            })
        }

        async fn process(&self, _message: Message) -> A2aResult<Message> {
            Ok(Message::agent_text(&self.response))
        }

        async fn health_check(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn test_team_creation_supervisor_mode() {
        let manager = Arc::new(AgentManager::new());
        let config = TeamConfig {
            id: "supervisor-team".to_string(),
            name: "Supervisor Team".to_string(),
            description: "A supervisor mode team".to_string(),
            mode: TeamMode::Supervisor,
            agents: vec![],
            scheduler_config: SchedulerConfig::Supervisor(SupervisorSchedulerConfig {
                supervisor_agent_id: "supervisor".to_string(),
            }),
        };

        let team = Team::new(config.clone(), manager);
        assert_eq!(team.get_config().id, "supervisor-team");
        assert_eq!(team.get_config().name, "Supervisor Team");
    }

    #[tokio::test]
    async fn test_team_creation_workflow_mode() {
        let manager = Arc::new(AgentManager::new());
        let config = TeamConfig {
            id: "workflow-team".to_string(),
            name: "Workflow Team".to_string(),
            description: "A workflow mode team".to_string(),
            mode: TeamMode::Workflow,
            agents: vec![],
            scheduler_config: SchedulerConfig::Workflow(WorkflowSchedulerConfig {
                steps: vec![
                    WorkflowStepConfig {
                        agent_id: "step1".to_string(),
                        order: 1,
                        condition: None,
                    },
                    WorkflowStepConfig {
                        agent_id: "step2".to_string(),
                        order: 2,
                        condition: None,
                    },
                ],
            }),
        };

        let team = Team::new(config.clone(), manager);
        assert_eq!(team.get_config().id, "workflow-team");
        assert_eq!(team.get_config().name, "Workflow Team");
    }

    #[tokio::test]
    async fn test_team_info_aggregates_capabilities() {
        let manager = Arc::new(AgentManager::new());

        // Create agents with different capabilities
        let agent1 = Arc::new(
            MockAgent::new("agent-1", "Agent 1")
                .with_capabilities(vec!["capability-a".to_string(), "capability-b".to_string()])
                .with_response("Response 1"),
        );
        let agent2 = Arc::new(
            MockAgent::new("agent-2", "Agent 2")
                .with_capabilities(vec!["capability-b".to_string(), "capability-c".to_string()])
                .with_response("Response 2"),
        );

        manager.register(agent1).await.unwrap();
        manager.register(agent2).await.unwrap();

        let config = TeamConfig {
            id: "capability-team".to_string(),
            name: "Capability Team".to_string(),
            description: "Testing capability aggregation".to_string(),
            mode: TeamMode::Supervisor,
            agents: vec![
                TeamAgentConfig {
                    agent_id: "agent-1".to_string(),
                    role: "worker".to_string(),
                    capabilities: vec!["capability-a".to_string(), "capability-b".to_string()],
                },
                TeamAgentConfig {
                    agent_id: "agent-2".to_string(),
                    role: "worker".to_string(),
                    capabilities: vec!["capability-b".to_string(), "capability-c".to_string()],
                },
            ],
            scheduler_config: SchedulerConfig::Supervisor(SupervisorSchedulerConfig {
                supervisor_agent_id: "agent-1".to_string(),
            }),
        };

        let team = Team::new(config, manager);
        let info = team.info().await.unwrap();

        // Should aggregate capabilities from all members
        assert!(info.capabilities.len() >= 3);
        assert!(info.capabilities.contains(&"capability-a".to_string()));
        assert!(info.capabilities.contains(&"capability-b".to_string()));
        assert!(info.capabilities.contains(&"capability-c".to_string()));
    }

    #[tokio::test]
    async fn test_team_health_check() {
        let manager = Arc::new(AgentManager::new());
        let agent = Arc::new(
            MockAgent::new("agent-1", "Agent 1")
                .with_capabilities(vec!["test".to_string()])
                .with_response("Response"),
        );

        manager.register(agent).await.unwrap();

        let config = TeamConfig {
            id: "health-team".to_string(),
            name: "Health Check Team".to_string(),
            description: "Testing health check".to_string(),
            mode: TeamMode::Supervisor,
            agents: vec![TeamAgentConfig {
                agent_id: "agent-1".to_string(),
                role: "worker".to_string(),
                capabilities: vec!["test".to_string()],
            }],
            scheduler_config: SchedulerConfig::Supervisor(SupervisorSchedulerConfig {
                supervisor_agent_id: "agent-1".to_string(),
            }),
        };

        let team = Team::new(config, manager);
        let results = team.health_check().await;

        // Health check should succeed - returns Vec<(String, bool)>
        assert!(!results.is_empty());
        assert!(results.iter().all(|(_, healthy)| *healthy));
    }

    #[test]
    fn test_supervisor_scheduler_creation() {
        let config = SupervisorSchedulerConfig {
            supervisor_agent_id: "supervisor-1".to_string(),
        };

        // Just test that we can create the config
        assert_eq!(config.supervisor_agent_id, "supervisor-1");
    }

    #[test]
    fn test_workflow_scheduler_creation() {
        let config = WorkflowSchedulerConfig {
            steps: vec![
                WorkflowStepConfig {
                    agent_id: "agent-1".to_string(),
                    order: 1,
                    condition: None,
                },
                WorkflowStepConfig {
                    agent_id: "agent-2".to_string(),
                    order: 2,
                    condition: Some("condition".to_string()),
                },
            ],
        };

        assert_eq!(config.steps.len(), 2);
        assert_eq!(config.steps[0].order, 1);
        assert_eq!(config.steps[1].order, 2);
        assert!(config.steps[1].condition.is_some());
    }

    #[test]
    fn test_team_config_serialization() {
        let config = TeamConfig {
            id: "test-team".to_string(),
            name: "Test Team".to_string(),
            description: "Test description".to_string(),
            mode: TeamMode::Supervisor,
            agents: vec![TeamAgentConfig {
                agent_id: "agent-1".to_string(),
                role: "worker".to_string(),
                capabilities: vec!["test".to_string()],
            }],
            scheduler_config: SchedulerConfig::Supervisor(SupervisorSchedulerConfig {
                supervisor_agent_id: "supervisor".to_string(),
            }),
        };

        // Test serialization
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("test-team"));
        assert!(json.contains("Test Team"));

        // Test deserialization
        let deserialized: TeamConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, config.id);
        assert_eq!(deserialized.name, config.name);
    }

    #[test]
    fn test_team_mode_serialization() {
        // Test Supervisor mode
        let supervisor_mode = TeamMode::Supervisor;
        let json = serde_json::to_string(&supervisor_mode).unwrap();
        assert_eq!(json, "\"supervisor\"");

        // Test Workflow mode
        let workflow_mode = TeamMode::Workflow;
        let json = serde_json::to_string(&workflow_mode).unwrap();
        assert_eq!(json, "\"workflow\"");
    }

    #[test]
    fn test_recipient_agent() {
        let recipient = Recipient::agent("agent-1".to_string());
        assert_eq!(recipient.agent_id, Some("agent-1".to_string()));
        assert!(!recipient.should_return_to_user);
        assert!(recipient.context_updates.is_empty());
    }

    #[test]
    fn test_recipient_user() {
        let recipient = Recipient::user();
        assert_eq!(recipient.agent_id, None);
        assert!(recipient.should_return_to_user);
        assert!(recipient.context_updates.is_empty());
    }

    #[test]
    fn test_recipient_with_context_updates() {
        let mut updates = HashMap::new();
        updates.insert("key1".to_string(), "value1".to_string());
        updates.insert("key2".to_string(), "value2".to_string());

        let recipient = Recipient::agent("agent-1".to_string()).with_context_updates(updates.clone());

        assert_eq!(recipient.agent_id, Some("agent-1".to_string()));
        assert_eq!(recipient.context_updates.len(), 2);
        assert_eq!(
            recipient.context_updates.get("key1"),
            Some(&"value1".to_string())
        );
        assert_eq!(
            recipient.context_updates.get("key2"),
            Some(&"value2".to_string())
        );
    }

    #[test]
    fn test_track_team_nesting_no_cycle() {
        let mut visited = HashSet::new();
        let result = track_team_nesting("team-1", &mut visited);

        assert!(result.is_ok());
        assert!(visited.contains("team-1"));
    }

    #[test]
    fn test_track_team_nesting_detects_cycle() {
        let mut visited = HashSet::new();
        visited.insert("team-1".to_string());

        let result = track_team_nesting("team-1", &mut visited);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("cycle"));
        assert!(err.to_string().contains("team-1"));
    }

    #[test]
    fn test_cycle_error_message() {
        let mut visited = HashSet::new();
        visited.insert("team-a".to_string());
        visited.insert("team-b".to_string());

        let result = track_team_nesting("team-a", &mut visited);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Cycle detected"));
        assert!(err_msg.contains("team-a"));
    }

    #[test]
    fn test_workflow_step_config_creation() {
        let step = WorkflowStepConfig {
            agent_id: "agent-1".to_string(),
            order: 1,
            condition: Some("if ready".to_string()),
        };

        assert_eq!(step.agent_id, "agent-1");
        assert_eq!(step.order, 1);
        assert_eq!(step.condition, Some("if ready".to_string()));
    }

    #[test]
    fn test_team_agent_config_creation() {
        let config = TeamAgentConfig {
            agent_id: "agent-1".to_string(),
            role: "researcher".to_string(),
            capabilities: vec!["research".to_string(), "analysis".to_string()],
        };

        assert_eq!(config.agent_id, "agent-1");
        assert_eq!(config.role, "researcher");
        assert_eq!(config.capabilities.len(), 2);
        assert!(config.capabilities.contains(&"research".to_string()));
        assert!(config.capabilities.contains(&"analysis".to_string()));
    }

    #[test]
    fn test_recipient_clone() {
        let recipient = Recipient::agent("agent-1".to_string());
        let cloned = recipient.clone();

        assert_eq!(recipient.agent_id, cloned.agent_id);
        assert_eq!(
            recipient.should_return_to_user,
            cloned.should_return_to_user
        );
    }

    #[test]
    fn test_team_mode_debug() {
        let supervisor = TeamMode::Supervisor;
        let workflow = TeamMode::Workflow;

        let supervisor_debug = format!("{:?}", supervisor);
        let workflow_debug = format!("{:?}", workflow);

        assert!(supervisor_debug.contains("Supervisor"));
        assert!(workflow_debug.contains("Workflow"));
    }

    #[tokio::test]
    async fn test_from_config_success() {
        let manager = Arc::new(AgentManager::new());
        
        // Create a config with a team
        let config = crate::Config {
            agents: vec![],
            teams: vec![crate::config::TeamConfigFile {
                id: "test-team-1".to_string(),
                name: "Test Team 1".to_string(),
                description: "Test team for from_config".to_string(),
                mode: "supervisor".to_string(),
                agents: vec![],
                scheduler_config: SchedulerConfig::Supervisor(SupervisorSchedulerConfig {
                    supervisor_agent_id: "supervisor-1".to_string(),
                }),
            }],
        };

        let team = Team::from_config(&config, "test-team-1", manager);
        assert!(team.is_ok());
        
        let team = team.unwrap();
        assert_eq!(team.get_config().id, "test-team-1");
        assert_eq!(team.get_config().name, "Test Team 1");
    }

    #[tokio::test]
    async fn test_from_config_team_not_found() {
        let manager = Arc::new(AgentManager::new());
        
        let config = crate::Config {
            agents: vec![],
            teams: vec![crate::config::TeamConfigFile {
                id: "team-a".to_string(),
                name: "Team A".to_string(),
                description: "Test".to_string(),
                mode: "supervisor".to_string(),
                agents: vec![],
                scheduler_config: SchedulerConfig::Supervisor(SupervisorSchedulerConfig {
                    supervisor_agent_id: "sup".to_string(),
                }),
            }],
        };

        let team = Team::from_config(&config, "non-existent-team", manager);
        assert!(team.is_err());
        
        if let Err(err) = team {
            assert!(matches!(err, TeamError::Configuration(_)));
            assert!(err.to_string().contains("not found"));
            assert!(err.to_string().contains("non-existent-team"));
        } else {
            panic!("Expected error, got Ok");
        }
    }

    #[tokio::test]
    async fn test_health_check_with_unhealthy_agent() {
        struct UnhealthyAgent {
            id: String,
        }

        #[async_trait]
        impl Agent for UnhealthyAgent {
            async fn info(&self) -> A2aResult<AgentInfo> {
                Ok(AgentInfo {
                    id: self.id.clone(),
                    name: "Unhealthy Agent".to_string(),
                    description: "Always fails health check".to_string(),
                    capabilities: vec![],
                    metadata: HashMap::new(),
                })
            }

            async fn process(&self, _message: Message) -> A2aResult<Message> {
                Ok(Message::agent_text("Response"))
            }

            async fn health_check(&self) -> bool {
                false // Always unhealthy
            }
        }

        let manager = Arc::new(AgentManager::new());
        let unhealthy_agent = Arc::new(UnhealthyAgent {
            id: "unhealthy-1".to_string(),
        });

        manager.register(unhealthy_agent).await.unwrap();

        let config = TeamConfig {
            id: "health-test-team".to_string(),
            name: "Health Test Team".to_string(),
            description: "Testing health check with unhealthy agent".to_string(),
            mode: TeamMode::Supervisor,
            agents: vec![TeamAgentConfig {
                agent_id: "unhealthy-1".to_string(),
                role: "worker".to_string(),
                capabilities: vec![],
            }],
            scheduler_config: SchedulerConfig::Supervisor(SupervisorSchedulerConfig {
                supervisor_agent_id: "unhealthy-1".to_string(),
            }),
        };

        let team = Team::new(config, manager);
        let results = team.health_check().await;

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "unhealthy-1");
        assert!(!results[0].1); // Should be unhealthy
    }

    #[tokio::test]
    async fn test_health_check_with_missing_agent() {
        let manager = Arc::new(AgentManager::new());

        let config = TeamConfig {
            id: "missing-agent-team".to_string(),
            name: "Missing Agent Team".to_string(),
            description: "Testing health check with missing agent".to_string(),
            mode: TeamMode::Supervisor,
            agents: vec![TeamAgentConfig {
                agent_id: "non-existent-agent".to_string(),
                role: "worker".to_string(),
                capabilities: vec![],
            }],
            scheduler_config: SchedulerConfig::Supervisor(SupervisorSchedulerConfig {
                supervisor_agent_id: "non-existent-agent".to_string(),
            }),
        };

        let team = Team::new(config, manager);
        let results = team.health_check().await;

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "non-existent-agent");
        assert!(!results[0].1); // Should be unhealthy (missing)
    }

    #[tokio::test]
    async fn test_team_as_agent_health_check_all_unhealthy() {
        struct UnhealthyAgent {
            id: String,
        }

        #[async_trait]
        impl Agent for UnhealthyAgent {
            async fn info(&self) -> A2aResult<AgentInfo> {
                Ok(AgentInfo {
                    id: self.id.clone(),
                    name: "Unhealthy".to_string(),
                    description: "Unhealthy agent".to_string(),
                    capabilities: vec![],
                    metadata: HashMap::new(),
                })
            }

            async fn process(&self, _message: Message) -> A2aResult<Message> {
                Ok(Message::agent_text("Response"))
            }

            async fn health_check(&self) -> bool {
                false
            }
        }

        let manager = Arc::new(AgentManager::new());
        let agent1 = Arc::new(UnhealthyAgent {
            id: "unhealthy-1".to_string(),
        });
        let agent2 = Arc::new(UnhealthyAgent {
            id: "unhealthy-2".to_string(),
        });

        manager.register(agent1).await.unwrap();
        manager.register(agent2).await.unwrap();

        let config = TeamConfig {
            id: "all-unhealthy-team".to_string(),
            name: "All Unhealthy Team".to_string(),
            description: "All agents unhealthy".to_string(),
            mode: TeamMode::Supervisor,
            agents: vec![
                TeamAgentConfig {
                    agent_id: "unhealthy-1".to_string(),
                    role: "worker".to_string(),
                    capabilities: vec![],
                },
                TeamAgentConfig {
                    agent_id: "unhealthy-2".to_string(),
                    role: "worker".to_string(),
                    capabilities: vec![],
                },
            ],
            scheduler_config: SchedulerConfig::Supervisor(SupervisorSchedulerConfig {
                supervisor_agent_id: "unhealthy-1".to_string(),
            }),
        };

        let team = Team::new(config, manager);
        
        // Test via Agent trait - health_check returns bool
        // Team is healthy if at least one agent is healthy, but all are unhealthy here
        let healthy = <Team as crate::agent::Agent>::health_check(&team).await;
        assert!(!healthy); // All agents unhealthy, so team is unhealthy
    }

    #[tokio::test]
    async fn test_supervisor_scheduler_determine_next_recipient() {
        let manager = Arc::new(AgentManager::new());
        let supervisor = Arc::new(MockAgent::new("supervisor-1", "Supervisor"));
        manager.register(supervisor).await.unwrap();

        let config = SupervisorSchedulerConfig {
            supervisor_agent_id: "supervisor-1".to_string(),
        };
        let scheduler = SupervisorScheduler::new(config);

        let team_config = TeamConfig {
            id: "test-team".to_string(),
            name: "Test Team".to_string(),
            description: "Test".to_string(),
            mode: TeamMode::Supervisor,
            agents: vec![],
            scheduler_config: SchedulerConfig::Supervisor(SupervisorSchedulerConfig {
                supervisor_agent_id: "supervisor-1".to_string(),
            }),
        };

        let messages = vec![Message::user_text("Test message")];
        let context = HashMap::new();

        // First call should route to supervisor
        let recipient = scheduler
            .determine_next_recipient(&team_config, &manager, messages.clone(), None, &context)
            .await
            .unwrap();

        assert_eq!(recipient.agent_id, Some("supervisor-1".to_string()));
        assert!(!recipient.should_return_to_user);

        // Second call with response should return to user
        let response = Message::agent_text("Supervisor response");
        let recipient = scheduler
            .determine_next_recipient(&team_config, &manager, messages.clone(), Some(response), &context)
            .await
            .unwrap();

        assert!(recipient.should_return_to_user);
        assert_eq!(recipient.agent_id, None);
    }

    #[tokio::test]
    async fn test_supervisor_scheduler_no_supervisor_agent() {
        let manager = Arc::new(AgentManager::new());
        // Don't register the supervisor agent

        let config = SupervisorSchedulerConfig {
            supervisor_agent_id: "missing-supervisor".to_string(),
        };
        let scheduler = SupervisorScheduler::new(config);

        let team_config = TeamConfig {
            id: "test-team".to_string(),
            name: "Test Team".to_string(),
            description: "Test".to_string(),
            mode: TeamMode::Supervisor,
            agents: vec![],
            scheduler_config: SchedulerConfig::Supervisor(SupervisorSchedulerConfig {
                supervisor_agent_id: "missing-supervisor".to_string(),
            }),
        };

        let messages = vec![Message::user_text("Test")];
        let context = HashMap::new();

        let result = scheduler
            .determine_next_recipient(&team_config, &manager, messages, None, &context)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TeamError::Configuration(_)));
    }

    #[tokio::test]
    async fn test_workflow_scheduler_determine_next_recipient() {
        let manager = Arc::new(AgentManager::new());
        let agent1 = Arc::new(MockAgent::new("agent-1", "Agent 1"));
        let agent2 = Arc::new(MockAgent::new("agent-2", "Agent 2"));
        manager.register(agent1).await.unwrap();
        manager.register(agent2).await.unwrap();

        let config = WorkflowSchedulerConfig {
            steps: vec![
                WorkflowStepConfig {
                    agent_id: "agent-1".to_string(),
                    order: 1,
                    condition: None,
                },
                WorkflowStepConfig {
                    agent_id: "agent-2".to_string(),
                    order: 2,
                    condition: None,
                },
            ],
        };
        let scheduler = WorkflowScheduler::new(config);

        let team_config = TeamConfig {
            id: "workflow-team".to_string(),
            name: "Workflow Team".to_string(),
            description: "Test".to_string(),
            mode: TeamMode::Workflow,
            agents: vec![],
            scheduler_config: SchedulerConfig::Workflow(WorkflowSchedulerConfig {
                steps: vec![],
            }),
        };

        let messages = vec![Message::user_text("Start workflow")];
        let context = HashMap::new();

        // First call - should route to agent-1
        let recipient = scheduler
            .determine_next_recipient(&team_config, &manager, messages.clone(), None, &context)
            .await
            .unwrap();

        assert_eq!(recipient.agent_id, Some("agent-1".to_string()));

        // Second call - should route to agent-2 (last step)
        let response = Message::agent_text("Agent 1 response");
        let recipient = scheduler
            .determine_next_recipient(&team_config, &manager, messages.clone(), Some(response), &context)
            .await
            .unwrap();

        assert_eq!(recipient.agent_id, Some("agent-2".to_string()));
        assert!(recipient.context_updates.contains_key("workflow_complete"));

        // Third call - workflow finished, return to user
        let response = Message::agent_text("Agent 2 response");
        let recipient = scheduler
            .determine_next_recipient(&team_config, &manager, messages, Some(response), &context)
            .await
            .unwrap();

        assert!(recipient.should_return_to_user);
    }

    #[tokio::test]
    async fn test_workflow_scheduler_with_condition_not_met() {
        // Test that workflow completes when condition is not met for the last step
        let manager = Arc::new(AgentManager::new());
        let agent1 = Arc::new(MockAgent::new("agent-1", "Agent 1"));
        manager.register(agent1).await.unwrap();

        let config = WorkflowSchedulerConfig {
            steps: vec![
                WorkflowStepConfig {
                    agent_id: "agent-1".to_string(),
                    order: 1,
                    condition: None,
                },
            ],
        };
        let scheduler = WorkflowScheduler::new(config);

        let team_config = TeamConfig {
            id: "simple-workflow".to_string(),
            name: "Simple Workflow".to_string(),
            description: "Test".to_string(),
            mode: TeamMode::Workflow,
            agents: vec![],
            scheduler_config: SchedulerConfig::Workflow(WorkflowSchedulerConfig {
                steps: vec![],
            }),
        };

        let messages = vec![Message::user_text("Start")];
        let context = HashMap::new();

        // First call - route to agent-1
        let recipient = scheduler
            .determine_next_recipient(&team_config, &manager, messages.clone(), None, &context)
            .await
            .unwrap();
        assert_eq!(recipient.agent_id, Some("agent-1".to_string()));
        assert!(recipient.context_updates.contains_key("workflow_complete"));

        // Second call after workflow completes - should return to user
        let response = Message::agent_text("Agent 1 completed");
        let recipient = scheduler
            .determine_next_recipient(&team_config, &manager, messages, Some(response), &context)
            .await
            .unwrap();
        
        assert!(recipient.should_return_to_user);
    }

    #[tokio::test]
    async fn test_workflow_scheduler_condition_met() {
        let manager = Arc::new(AgentManager::new());
        let agent1 = Arc::new(MockAgent::new("agent-1", "Agent 1"));
        let agent2 = Arc::new(MockAgent::new("agent-2", "Agent 2"));
        manager.register(agent1).await.unwrap();
        manager.register(agent2).await.unwrap();

        let config = WorkflowSchedulerConfig {
            steps: vec![
                WorkflowStepConfig {
                    agent_id: "agent-1".to_string(),
                    order: 1,
                    condition: None,
                },
                WorkflowStepConfig {
                    agent_id: "agent-2".to_string(),
                    order: 2,
                    condition: Some("approved".to_string()),
                },
            ],
        };
        let scheduler = WorkflowScheduler::new(config);

        let team_config = TeamConfig {
            id: "approval-workflow".to_string(),
            name: "Approval Workflow".to_string(),
            description: "Test".to_string(),
            mode: TeamMode::Workflow,
            agents: vec![],
            scheduler_config: SchedulerConfig::Workflow(WorkflowSchedulerConfig {
                steps: vec![],
            }),
        };

        let messages = vec![Message::user_text("Start")];
        let context = HashMap::new();

        // First call
        let recipient = scheduler
            .determine_next_recipient(&team_config, &manager, messages.clone(), None, &context)
            .await
            .unwrap();
        assert_eq!(recipient.agent_id, Some("agent-1".to_string()));

        // Second call with "approved" in response - condition met
        let response = Message::agent_text("Request approved, proceed");
        let recipient = scheduler
            .determine_next_recipient(&team_config, &manager, messages, Some(response), &context)
            .await
            .unwrap();
        
        // Should go to agent-2 since condition is met
        assert_eq!(recipient.agent_id, Some("agent-2".to_string()));
        assert!(recipient.context_updates.contains_key("workflow_complete"));
    }
}
