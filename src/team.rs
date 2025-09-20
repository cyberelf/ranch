use crate::agent::{AgentMessage, AgentResponse};
use crate::manager::{AgentManager, AgentRef};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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
    pub fn new(
        config: TeamConfig,
        agent_manager: Arc<AgentManager>,
    ) -> Self {
        let scheduler: Arc<dyn Scheduler> = match &config.scheduler_config {
            SchedulerConfig::Supervisor(supervisor_config) => {
                Arc::new(SupervisorScheduler::new(supervisor_config.clone()))
            },
            SchedulerConfig::Workflow(workflow_config) => {
                Arc::new(WorkflowScheduler::new(workflow_config.clone()))
            },
        };

        Self {
            config,
            agent_manager,
            scheduler,
        }
    }

    pub async fn process_message(&self, message: AgentMessage) -> Result<AgentResponse, TeamError> {
        self.process_messages(vec![message]).await
    }

    pub async fn process_messages(&self, initial_messages: Vec<AgentMessage>) -> Result<AgentResponse, TeamError> {
        let mut current_messages = initial_messages;
        let mut last_response: Option<AgentResponse> = None;
        let mut context = HashMap::new();

        loop {
            let recipient = self.scheduler.determine_next_recipient(
                &self.config,
                &self.agent_manager,
                current_messages.clone(),
                last_response.clone(),
                &context,
            ).await?;

            // Apply context updates from scheduler
            context.extend(recipient.context_updates);

            if recipient.should_return_to_user {
                if let Some(response) = last_response {
                    return Ok(response);
                } else {
                    return Err(TeamError::Scheduling("No response generated".to_string()));
                }
            }

            let agent_id = recipient.agent_id
                .ok_or_else(|| TeamError::Scheduling("No agent ID provided".to_string()))?;

            let agent = self.agent_manager.get_agent(&agent_id).await
                .ok_or_else(|| TeamError::Agent(format!("Agent {} not found", agent_id)))?;

            last_response = Some(agent.send_message(current_messages.clone()).await
                .map_err(|e| TeamError::Agent(e.to_string()))?);

            // Prepare messages for next iteration
            current_messages = vec![AgentMessage {
                id: uuid::Uuid::new_v4().to_string(),
                role: "assistant".to_string(),
                content: last_response.as_ref().unwrap().content.clone(),
                metadata: context.clone(),
            }];
        }
    }

    pub fn get_config(&self) -> &TeamConfig {
        &self.config
    }

    pub async fn health_check(&self) -> Vec<(String, bool)> {
        let mut results = Vec::new();
        
        for agent_config in &self.config.agents {
            if let Some(agent) = self.agent_manager.get_agent(&agent_config.agent_id).await {
                let healthy = agent.health_check().await.unwrap_or(false);
                results.push((agent_config.agent_id.clone(), healthy));
            } else {
                results.push((agent_config.agent_id.clone(), false));
            }
        }
        
        results
    }
}

#[async_trait]
pub trait Scheduler: Send + Sync {
    async fn determine_next_recipient(
        &self,
        team_config: &TeamConfig,
        agent_manager: &AgentManager,
        messages: Vec<AgentMessage>,
        last_response: Option<AgentResponse>,
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

    async fn get_supervisor_agent(&self, agent_manager: &AgentManager) -> Option<AgentRef> {
        agent_manager.get_agent(&self.config.supervisor_agent_id).await
    }
}

#[async_trait]
impl Scheduler for SupervisorScheduler {
    async fn determine_next_recipient(
        &self,
        _team_config: &TeamConfig,
        agent_manager: &AgentManager,
        _messages: Vec<AgentMessage>,
        _last_response: Option<AgentResponse>,
        _context: &HashMap<String, String>,
    ) -> Result<Recipient, TeamError> {
        let supervisor = self.get_supervisor_agent(agent_manager).await
            .ok_or_else(|| TeamError::Configuration("No supervisor agent configured".to_string()))?;

        Ok(Recipient::agent(supervisor.get_config().id.clone()))
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
        _messages: Vec<AgentMessage>,
        last_response: Option<AgentResponse>,
        _context: &HashMap<String, String>,
    ) -> Result<Recipient, TeamError> {
        let mut current_step = self.current_step.write().await;

        if *current_step >= self.config.steps.len() {
            return Ok(Recipient::user());
        }

        let step = &self.config.steps[*current_step];

        // Check if there's a condition for this step
        if let Some(condition) = &step.condition {
            if let Some(response) = &last_response {
                // Simple condition checking - in a real implementation, this would be more sophisticated
                if !response.content.contains(condition) {
                    *current_step += 1;
                    return self.determine_next_recipient(_team_config, _agent_manager, _messages, last_response, _context).await;
                }
            }
        }

        let agent_id = step.agent_id.clone();
        *current_step += 1;

        // If this was the last step, return to user next time
        if *current_step >= self.config.steps.len() {
            let mut context_updates = HashMap::new();
            context_updates.insert("workflow_complete".to_string(), "true".to_string());
            return Ok(Recipient::agent(agent_id).with_context_updates(context_updates));
        }

        Ok(Recipient::agent(agent_id))
    }
}