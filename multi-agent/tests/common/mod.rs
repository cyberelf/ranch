//! Common test utilities for multi-agent integration tests

use a2a_protocol::{A2aResult, Message};
use async_trait::async_trait;
use multi_agent::{Agent, AgentInfo};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

/// Mock agent for testing purposes
pub struct MockAgent {
    id: String,
    name: String,
    capabilities: Vec<String>,
    response_text: String,
    call_count: Arc<AtomicU32>,
}

impl MockAgent {
    /// Create a new mock agent
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            capabilities: Vec::new(),
            response_text: "Mock response".to_string(),
            call_count: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Set capabilities for this agent
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Set the response text
    pub fn with_response(mut self, response: impl Into<String>) -> Self {
        self.response_text = response.into();
        self
    }

    /// Get the number of times process was called
    #[allow(dead_code)] // Used in some tests but not all
    pub fn call_count(&self) -> u32 {
        self.call_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl Agent for MockAgent {
    async fn info(&self) -> A2aResult<AgentInfo> {
        Ok(AgentInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            description: format!("Mock agent {}", self.name),
            capabilities: self.capabilities.clone(),
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn process(&self, _msg: Message) -> A2aResult<Message> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        Ok(Message::user_text(&self.response_text))
    }

    async fn health_check(&self) -> bool {
        true
    }
}

/// Helper to create a test message
#[allow(dead_code)] // Used in some tests but not all
pub fn create_test_message(text: &str) -> Message {
    Message::user_text(text)
}
