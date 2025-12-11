//! Agent registry and management
//!
//! This module provides the `AgentManager` for registering, discovering,
//! and managing agents in the multi-agent framework.

use crate::{Agent, AgentInfo};
use a2a_protocol::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Manages a registry of agents
///
/// `AgentManager` provides centralized management of agents, including
/// registration, lookup, and discovery by capabilities.
pub struct AgentManager {
    agents: RwLock<HashMap<String, Arc<dyn Agent>>>,
}

impl AgentManager {
    /// Create a new agent manager
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
        }
    }

    /// Register an agent
    ///
    /// The agent's ID is extracted from its info and used as the registry key.
    ///
    /// # Returns
    /// The agent ID used for registration
    pub async fn register(&self, agent: Arc<dyn Agent>) -> A2aResult<String> {
        let info = agent.info().await?;
        let id = info.id.clone();

        let mut agents = self.agents.write().await;
        agents.insert(id.clone(), agent);

        Ok(id)
    }

    /// Get an agent by ID
    pub async fn get(&self, agent_id: &str) -> Option<Arc<dyn Agent>> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// Remove an agent
    ///
    /// # Returns
    /// The removed agent, if it existed
    pub async fn remove(&self, agent_id: &str) -> Option<Arc<dyn Agent>> {
        let mut agents = self.agents.write().await;
        agents.remove(agent_id)
    }

    /// List all agent IDs
    pub async fn list_ids(&self) -> Vec<String> {
        let agents = self.agents.read().await;
        agents.keys().cloned().collect()
    }

    /// Get info for all agents
    ///
    /// This method fetches the info from each registered agent.
    /// Agents that fail to return info are silently skipped.
    pub async fn list_info(&self) -> Vec<AgentInfo> {
        // 1. Acquire lock, clone references, and release lock immediately
        let agents: Vec<Arc<dyn Agent>> = {
            let guard = self.agents.read().await;
            guard.values().cloned().collect()
        };

        // 2. Perform async operations without holding the lock
        let mut infos = Vec::new();
        for agent in agents {
            if let Ok(info) = agent.info().await {
                infos.push(info);
            }
        }

        infos
    }

    /// Find agents by capability
    ///
    /// Searches for agents whose capabilities contain the specified string
    /// (case-insensitive).
    ///
    /// # Arguments
    /// * `capability` - The capability name to search for
    ///
    /// # Returns
    /// A vector of agents that have matching capabilities
    pub async fn find_by_capability(&self, capability: &str) -> Vec<Arc<dyn Agent>> {
        // 1. Acquire lock, clone references, and release lock immediately
        let agents: Vec<Arc<dyn Agent>> = {
            let guard = self.agents.read().await;
            guard.values().cloned().collect()
        };

        // 2. Perform async operations without holding the lock
        let mut matching = Vec::new();
        for agent in agents {
            if let Ok(info) = agent.info().await {
                // Check if any capability contains the search string
                if info
                    .capabilities
                    .iter()
                    .any(|cap| cap.to_lowercase().contains(&capability.to_lowercase()))
                {
                    matching.push(agent.clone());
                }
            }
        }

        matching
    }

    /// Health check all agents
    ///
    /// Attempts to get the info from each agent as a simple health check.
    ///
    /// # Returns
    /// A HashMap mapping agent IDs to their health status (true = healthy)
    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let agents = self.agents.read().await;
        let mut results = HashMap::new();

        for (id, agent) in agents.iter() {
            let healthy = agent.health_check().await;
            results.insert(id.clone(), healthy);
        }

        results
    }

    /// Get the number of registered agents
    pub async fn count(&self) -> usize {
        let agents = self.agents.read().await;
        agents.len()
    }

    /// Clear all registered agents
    pub async fn clear(&self) {
        let mut agents = self.agents.write().await;
        agents.clear();
    }
}

impl Default for AgentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    // Import from manager module, but be specific about Agent trait
    use super::AgentManager;
    use crate::agent::{Agent, AgentInfo};
    use a2a_protocol::prelude::{A2aResult, Message};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Arc;

    struct MockAgent {
        id: String,
        name: String,
        capabilities: Vec<String>,
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
            Ok(Message::agent_text("Mock response"))
        }

        async fn health_check(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn test_agent_manager_register_and_get() {
        let manager = AgentManager::new();
        let agent = Arc::new(MockAgent {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            capabilities: vec![],
        });

        let id = manager.register(agent.clone()).await.unwrap();
        assert_eq!(id, "test-agent");

        let retrieved = manager.get("test-agent").await;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_agent_manager_find_by_capability() {
        let manager = AgentManager::new();

        let agent1 = Arc::new(MockAgent {
            id: "agent1".to_string(),
            name: "Agent 1".to_string(),
            capabilities: vec!["search".to_string()],
        });

        let agent2 = Arc::new(MockAgent {
            id: "agent2".to_string(),
            name: "Agent 2".to_string(),
            capabilities: vec!["analyze".to_string()],
        });

        manager.register(agent1).await.unwrap();
        manager.register(agent2).await.unwrap();

        let search_agents = manager.find_by_capability("search").await;
        assert_eq!(search_agents.len(), 1);

        let analyze_agents = manager.find_by_capability("analyze").await;
        assert_eq!(analyze_agents.len(), 1);
    }
}
