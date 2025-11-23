//! Core traits for the multi-agent framework
//!
//! This module contains the fundamental trait definitions used throughout
//! the multi-agent framework.

use a2a_protocol::prelude::*;
use async_trait::async_trait;
use std::collections::HashMap;

/// Agent profile information for the multi-agent framework
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Agent trait for multi-agent coordination
///
/// This trait defines the interface for agents managed by the multi-agent framework.
/// It is focused on message processing and capability exposure, not on implementing
/// the full A2A protocol server interface.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Get agent information
    async fn info(&self) -> A2aResult<AgentInfo>;

    /// Process a message and return a response
    async fn process(&self, message: Message) -> A2aResult<Message>;

    /// Check if the agent is healthy and responsive
    async fn health_check(&self) -> bool {
        self.info().await.is_ok()
    }
}