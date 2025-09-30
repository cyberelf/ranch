use crate::agent::{AgentConfig, AgentMessage, AgentResponse};
use crate::protocol::{ProtocolAdapter, ProtocolError};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait Agent: Send + Sync {
    async fn send_message(&self, messages: Vec<AgentMessage>) -> Result<AgentResponse, AgentError>;
    async fn health_check(&self) -> Result<bool, AgentError>;
    fn get_config(&self) -> &AgentConfig;
}

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    
    #[error("Agent not found")]
    NotFound,
    
    #[error("Agent unhealthy")]
    Unhealthy,
    
    #[error("Configuration error: {0}")]
    Configuration(String),
}

pub struct RemoteAgent {
    config: AgentConfig,
    protocol: ProtocolAdapter,
}

impl RemoteAgent {
    pub fn new(config: AgentConfig, protocol: ProtocolAdapter) -> Self {
        Self { config, protocol }
    }
}

#[async_trait]
impl Agent for RemoteAgent {
    async fn send_message(&self, messages: Vec<AgentMessage>) -> Result<AgentResponse, AgentError> {
        self.protocol.send_message(&self.config, messages).await.map_err(AgentError::from)
    }

    async fn health_check(&self) -> Result<bool, AgentError> {
        self.protocol.health_check(&self.config).await.map_err(AgentError::from)
    }

    fn get_config(&self) -> &AgentConfig {
        &self.config
    }
}

pub type AgentRef = Arc<dyn Agent>;

pub struct AgentManager {
    agents: RwLock<std::collections::HashMap<String, AgentRef>>,
}

impl AgentManager {
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(std::collections::HashMap::new()),
        }
    }

    pub async fn register_agent(&self, agent: AgentRef) -> Result<(), AgentError> {
        let config = agent.get_config();
        let mut agents = self.agents.write().await;
        agents.insert(config.id.clone(), agent);
        Ok(())
    }

    pub async fn get_agent(&self, agent_id: &str) -> Option<AgentRef> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    pub async fn remove_agent(&self, agent_id: &str) -> Option<AgentRef> {
        let mut agents = self.agents.write().await;
        agents.remove(agent_id)
    }

    pub async fn list_agents(&self) -> Vec<AgentConfig> {
        let agents = self.agents.read().await;
        agents
            .values()
            .map(|agent| agent.get_config().clone())
            .collect()
    }

    pub async fn find_agents_by_capability(&self, capability: &str) -> Vec<AgentRef> {
        let agents = self.agents.read().await;
        agents
            .values()
            .filter(|agent| agent.get_config().capabilities.contains(&capability.to_string()))
            .cloned()
            .collect()
    }

    pub async fn health_check_all(&self) -> Vec<(String, bool)> {
        let agents = self.agents.read().await;
        let mut results = Vec::new();
        
        for (id, agent) in agents.iter() {
            let healthy = agent.health_check().await.unwrap_or(false);
            results.push((id.clone(), healthy));
        }
        
        results
    }
}

impl Default for AgentManager {
    fn default() -> Self {
        Self::new()
    }
}