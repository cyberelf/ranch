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
impl Agent for Team {
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
    context: RwLock<HashMap<String, String>>,
}

impl SupervisorScheduler {
    pub fn new(config: SupervisorSchedulerConfig) -> Self {
        Self {
            config,
            context: RwLock::new(HashMap::new()),
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
        _last_response: Option<Message>,
        _context: &HashMap<String, String>,
    ) -> Result<Recipient, TeamError> {
        let _supervisor = self
            .get_supervisor_agent(agent_manager)
            .await
            .ok_or_else(|| {
                TeamError::Configuration("No supervisor agent configured".to_string())
            })?;

        Ok(Recipient::agent(self.config.supervisor_agent_id.clone()))
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
