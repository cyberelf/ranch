use crate::agent::{AgentConfig, AgentMessage, AgentResponse};
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait Protocol: Send + Sync {
    async fn send_message(
        &self,
        config: &AgentConfig,
        messages: Vec<AgentMessage>,
    ) -> Result<AgentResponse, ProtocolError>;
    
    async fn health_check(&self, config: &AgentConfig) -> Result<bool, ProtocolError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Timeout error")]
    Timeout,
    
    #[error("Too many retries")]
    TooManyRetries,
}

pub type ProtocolAdapter = Arc<dyn Protocol>;