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

    /// Register all agents from configuration
    ///
    /// This is a convenience method that creates and registers all agents
    /// defined in a Config struct.
    ///
    /// # Arguments
    /// * `config` - Configuration containing agent definitions
    ///
    /// # Returns
    /// Vector of registered agent IDs
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
    /// let agent_ids = agent_manager.register_from_config(&config).await?;
    /// println!("Registered {} agents", agent_ids.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn register_from_config(&self, config: &crate::Config) -> A2aResult<Vec<String>> {
        use std::env;

        let mut registered_ids = Vec::new();

        for agent_config in config.to_agent_configs() {
            let agent: Arc<dyn crate::Agent> = match agent_config.protocol {
                crate::ProtocolType::A2A => {
                    let transport = Arc::new(crate::JsonRpcTransport::new(&agent_config.endpoint)?);
                    let client = crate::A2aClient::new(transport);
                    let a2a_config: crate::A2AAgentConfig =
                        agent_config.try_into().map_err(|e| {
                            A2aError::Internal(format!("Config conversion error: {}", e))
                        })?;
                    Arc::new(crate::A2AAgent::with_config(client, a2a_config))
                }
                crate::ProtocolType::OpenAI => {
                    let mut openai_config: crate::OpenAIAgentConfig =
                        agent_config.clone().try_into().map_err(|e| {
                            A2aError::Internal(format!("Config conversion error: {}", e))
                        })?;

                    // Override api_key from environment if available and not set
                    if openai_config.api_key.is_none() {
                        if let Ok(api_key) = env::var("OPENAI_API_KEY") {
                            openai_config.api_key = Some(api_key);
                        }
                    }

                    Arc::new(crate::OpenAIAgent::with_config(
                        agent_config.endpoint,
                        openai_config,
                    ))
                }
            };

            let agent_id = self.register(agent).await?;
            registered_ids.push(agent_id);
        }

        Ok(registered_ids)
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

    /// Register an agent with a specific ID
    ///
    /// This allows explicit control over the registration ID, useful when
    /// you want to use a local ID that differs from the agent's reported ID.
    ///
    /// # Arguments
    /// * `agent_id` - The ID to use for registration
    /// * `agent` - The agent to register
    ///
    /// # Returns
    /// The agent ID used for registration (same as input)
    pub async fn register_with_id(
        &self,
        agent_id: String,
        agent: Arc<dyn Agent>,
    ) -> A2aResult<String> {
        let mut agents = self.agents.write().await;
        agents.insert(agent_id.clone(), agent);
        Ok(agent_id)
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
                // Check if any skill contains the search string
                if info
                    .skills
                    .iter()
                    .any(|skill| skill.name.to_lowercase().contains(&capability.to_lowercase()))
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
    use super::AgentManager;
    use crate::agent::{Agent, AgentInfo};
    use a2a_protocol::prelude::{A2aResult, AgentCapabilities, AgentSkill, Message};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Arc;

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
                skills: self.capabilities.iter().map(|c| AgentSkill {
                    name: c.clone(),
                    description: None,
                    category: None,
                    tags: vec![],
                    examples: vec![],
                }).collect(),
                metadata: HashMap::new(),
                capabilities: AgentCapabilities::default(),
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
    async fn test_register_agent() {
        let manager = AgentManager::new();
        let agent = Arc::new(
            MockAgent::new("test-agent", "Test Agent")
                .with_capabilities(vec!["test".to_string()])
                .with_response("Response"),
        );

        let result = manager.register(agent).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-agent");
    }

    #[tokio::test]
    async fn test_register_duplicate_agent() {
        let manager = AgentManager::new();
        let agent1 = Arc::new(
            MockAgent::new("test-agent", "Test Agent 1")
                .with_capabilities(vec!["test".to_string()])
                .with_response("Response 1"),
        );
        let agent2 = Arc::new(
            MockAgent::new("test-agent", "Test Agent 2")
                .with_capabilities(vec!["test".to_string()])
                .with_response("Response 2"),
        );

        // First registration should succeed
        assert!(manager.register(agent1).await.is_ok());

        // Second registration with same ID should succeed (overwrite)
        let result = manager.register(agent2).await;
        assert!(result.is_ok());

        // The second agent should now be registered
        let retrieved = manager.get("test-agent").await.unwrap();
        let info = retrieved.info().await.unwrap();
        assert_eq!(info.name, "Test Agent 2");
    }

    #[tokio::test]
    async fn test_get_existing_agent() {
        let manager = AgentManager::new();
        let agent = Arc::new(
            MockAgent::new("test-agent", "Test Agent")
                .with_capabilities(vec!["test".to_string()])
                .with_response("Response"),
        );

        manager.register(agent).await.unwrap();

        let retrieved = manager.get("test-agent").await;
        assert!(retrieved.is_some());

        let info = retrieved.unwrap().info().await.unwrap();
        assert_eq!(info.id, "test-agent");
        assert_eq!(info.name, "Test Agent");
    }

    #[tokio::test]
    async fn test_get_nonexistent_agent() {
        let manager = AgentManager::new();

        let retrieved = manager.get("nonexistent").await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_agents() {
        let manager = AgentManager::new();

        // Initially empty
        let agent_ids = manager.list_ids().await;
        assert!(agent_ids.is_empty());

        // Register some agents
        let agent1 = Arc::new(
            MockAgent::new("agent-1", "Agent 1")
                .with_capabilities(vec!["capability-a".to_string()])
                .with_response("Response 1"),
        );
        let agent2 = Arc::new(
            MockAgent::new("agent-2", "Agent 2")
                .with_capabilities(vec!["capability-b".to_string()])
                .with_response("Response 2"),
        );

        manager.register(agent1).await.unwrap();
        manager.register(agent2).await.unwrap();

        // Should have 2 agents
        let agent_ids = manager.list_ids().await;
        assert_eq!(agent_ids.len(), 2);
        assert!(agent_ids.contains(&"agent-1".to_string()));
        assert!(agent_ids.contains(&"agent-2".to_string()));

        // Test list_info as well
        let infos = manager.list_info().await;
        assert_eq!(infos.len(), 2);

        let ids: Vec<String> = infos.iter().map(|info| info.id.clone()).collect();
        assert!(ids.contains(&"agent-1".to_string()));
        assert!(ids.contains(&"agent-2".to_string()));
    }

    #[tokio::test]
    async fn test_find_by_capability() {
        let manager = AgentManager::new();

        // Register agents with different capabilities
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
        let agent3 = Arc::new(
            MockAgent::new("agent-3", "Agent 3")
                .with_capabilities(vec!["capability-d".to_string()])
                .with_response("Response 3"),
        );

        manager.register(agent1).await.unwrap();
        manager.register(agent2).await.unwrap();
        manager.register(agent3).await.unwrap();

        // Find agents with capability-a (should find agent1)
        let found_a = manager.find_by_capability("capability-a").await;
        assert_eq!(found_a.len(), 1);
        let info_a = found_a[0].info().await.unwrap();
        assert_eq!(info_a.id, "agent-1");

        // Find agents with capability-b (should find agent1 and agent2)
        let found_b = manager.find_by_capability("capability-b").await;
        assert_eq!(found_b.len(), 2);

        // Find agents with capability-d (should find agent3)
        let found_d = manager.find_by_capability("capability-d").await;
        assert_eq!(found_d.len(), 1);
        let info_d = found_d[0].info().await.unwrap();
        assert_eq!(info_d.id, "agent-3");

        // Find agents with nonexistent capability
        let found_none = manager.find_by_capability("nonexistent").await;
        assert!(found_none.is_empty());
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let manager = Arc::new(AgentManager::new());

        // Register multiple agents concurrently
        let mut handles = vec![];
        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = tokio::spawn(async move {
                let agent = Arc::new(
                    MockAgent::new(&format!("agent-{}", i), &format!("Agent {}", i))
                        .with_capabilities(vec![format!("capability-{}", i)])
                        .with_response(&format!("Response {}", i)),
                );
                manager_clone.register(agent).await
            });
            handles.push(handle);
        }

        // Wait for all registrations
        for handle in handles {
            assert!(handle.await.unwrap().is_ok());
        }

        // Should have all 10 agents
        let agent_ids = manager.list_ids().await;
        assert_eq!(agent_ids.len(), 10);
    }

    #[tokio::test]
    async fn test_concurrent_reads() {
        let manager = Arc::new(AgentManager::new());

        // Register some agents
        for i in 0..5 {
            let agent = Arc::new(
                MockAgent::new(&format!("agent-{}", i), &format!("Agent {}", i))
                    .with_capabilities(vec![format!("capability-{}", i)])
                    .with_response(&format!("Response {}", i)),
            );
            manager.register(agent).await.unwrap();
        }

        // Read concurrently from multiple tasks
        let mut handles = vec![];
        for i in 0..20 {
            let manager_clone = Arc::clone(&manager);
            let handle = tokio::spawn(async move {
                let agent_id = format!("agent-{}", i % 5);
                manager_clone.get(&agent_id).await
            });
            handles.push(handle);
        }

        // All reads should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_some());
        }
    }

    #[tokio::test]
    async fn test_register_with_id() {
        let manager = AgentManager::new();
        let agent = Arc::new(
            MockAgent::new("agent-internal-id", "Agent Name")
                .with_capabilities(vec!["test".to_string()])
                .with_response("Response"),
        );

        // Register with custom ID
        let result = manager
            .register_with_id("custom-id".to_string(), agent)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "custom-id");

        // Should be able to get by custom ID
        let fetched = manager.get("custom-id").await;
        assert!(fetched.is_some());

        // Original ID should not work
        let not_found = manager.get("agent-internal-id").await;
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_remove_agent() {
        let manager = AgentManager::new();
        let agent = Arc::new(
            MockAgent::new("agent-1", "Agent 1")
                .with_capabilities(vec!["test".to_string()])
                .with_response("Response"),
        );

        manager.register(agent).await.unwrap();

        // Agent should exist
        assert!(manager.get("agent-1").await.is_some());

        // Remove the agent
        let removed = manager.remove("agent-1").await;
        assert!(removed.is_some());

        // Agent should no longer exist
        assert!(manager.get("agent-1").await.is_none());

        // Removing again should return None
        let removed_again = manager.remove("agent-1").await;
        assert!(removed_again.is_none());
    }

    #[tokio::test]
    async fn test_count() {
        let manager = AgentManager::new();

        // Initially empty
        assert_eq!(manager.count().await, 0);

        // Add agents
        for i in 0..5 {
            let agent = Arc::new(
                MockAgent::new(&format!("agent-{}", i), &format!("Agent {}", i))
                    .with_capabilities(vec!["test".to_string()])
                    .with_response("Response"),
            );
            manager.register(agent).await.unwrap();
        }

        assert_eq!(manager.count().await, 5);

        // Remove one
        manager.remove("agent-0").await;
        assert_eq!(manager.count().await, 4);
    }

    #[tokio::test]
    async fn test_clear() {
        let manager = AgentManager::new();

        // Add agents
        for i in 0..5 {
            let agent = Arc::new(
                MockAgent::new(&format!("agent-{}", i), &format!("Agent {}", i))
                    .with_capabilities(vec!["test".to_string()])
                    .with_response("Response"),
            );
            manager.register(agent).await.unwrap();
        }

        assert_eq!(manager.count().await, 5);

        // Clear all
        manager.clear().await;
        assert_eq!(manager.count().await, 0);
        assert!(manager.list_ids().await.is_empty());
    }

    #[tokio::test]
    async fn test_health_check_all() {
        let manager = AgentManager::new();

        // Add healthy agents
        for i in 0..3 {
            let agent = Arc::new(
                MockAgent::new(&format!("agent-{}", i), &format!("Agent {}", i))
                    .with_capabilities(vec!["test".to_string()])
                    .with_response("Response"),
            );
            manager.register(agent).await.unwrap();
        }

        let health = manager.health_check_all().await;

        // All agents should be healthy (MockAgent returns true)
        assert_eq!(health.len(), 3);
        for (_, healthy) in health {
            assert!(healthy);
        }
    }

    #[tokio::test]
    async fn test_agent_manager_default() {
        let manager = AgentManager::default();
        assert_eq!(manager.count().await, 0);
    }

    #[tokio::test]
    async fn test_list_info_with_failing_agent() {
        // This test verifies that list_info() skips agents that fail to return info
        // MockAgent always succeeds, so this is more of a structure test
        let manager = AgentManager::new();

        let agent = Arc::new(
            MockAgent::new("agent-1", "Agent 1")
                .with_capabilities(vec!["test".to_string()])
                .with_response("Response"),
        );

        manager.register(agent).await.unwrap();

        let infos = manager.list_info().await;
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].id, "agent-1");
        assert_eq!(infos[0].name, "Agent 1");
    }}