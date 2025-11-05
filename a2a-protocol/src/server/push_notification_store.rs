//! Push notification configuration storage and management
//!
//! This module provides storage for webhook/push notification configurations
//! as part of the A2A Protocol v0.7.0 push notification feature.

use crate::core::push_notification::PushNotificationConfig;
use crate::{A2aError, A2aResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory push notification configuration storage
///
/// Stores webhook configurations for tasks, allowing agents to receive
/// asynchronous notifications about task updates.
#[derive(Debug, Clone)]
pub struct PushNotificationStore {
    configs: Arc<RwLock<HashMap<String, PushNotificationConfig>>>,
}

impl PushNotificationStore {
    /// Create a new push notification store
    pub fn new() -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set push notification configuration for a task
    ///
    /// # Arguments
    /// * `task_id` - The task ID to configure notifications for
    /// * `config` - The push notification configuration
    ///
    /// # Returns
    /// Ok(()) if successful
    pub async fn set(&self, task_id: String, config: PushNotificationConfig) -> A2aResult<()> {
        // Validate configuration before storing
        config.validate().map_err(|e| {
            A2aError::UnsupportedOperation(format!("Invalid push notification config: {}", e))
        })?;

        self.configs.write().await.insert(task_id, config);
        Ok(())
    }

    /// Get push notification configuration for a task
    ///
    /// # Arguments
    /// * `task_id` - The task ID to get configuration for
    ///
    /// # Returns
    /// Some(config) if configuration exists, None otherwise
    pub async fn get(&self, task_id: &str) -> A2aResult<Option<PushNotificationConfig>> {
        Ok(self.configs.read().await.get(task_id).cloned())
    }

    /// List all push notification configurations
    ///
    /// # Returns
    /// Vector of (task_id, config) tuples
    pub async fn list(&self) -> A2aResult<Vec<(String, PushNotificationConfig)>> {
        let configs = self.configs.read().await;
        Ok(configs
            .iter()
            .map(|(id, config)| (id.clone(), config.clone()))
            .collect())
    }

    /// Delete push notification configuration for a task
    ///
    /// # Arguments
    /// * `task_id` - The task ID to delete configuration for
    ///
    /// # Returns
    /// true if configuration was deleted, false if it didn't exist
    pub async fn delete(&self, task_id: &str) -> A2aResult<bool> {
        Ok(self.configs.write().await.remove(task_id).is_some())
    }

    /// Get count of stored configurations
    pub async fn count(&self) -> usize {
        self.configs.read().await.len()
    }

    /// Clear all configurations (for testing)
    #[cfg(test)]
    pub async fn clear(&self) {
        self.configs.write().await.clear();
    }
}

impl Default for PushNotificationStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::push_notification::{PushNotificationAuth, TaskEvent};
    use url::Url;

    fn create_test_config() -> PushNotificationConfig {
        PushNotificationConfig::new(
            Url::parse("https://example.com/webhook").unwrap(),
            vec![TaskEvent::Completed, TaskEvent::Failed],
            None,
        )
    }

    #[tokio::test]
    async fn test_set_and_get_config() {
        let store = PushNotificationStore::new();
        let config = create_test_config();
        let task_id = "task-123".to_string();

        store.set(task_id.clone(), config.clone()).await.unwrap();

        let retrieved = store.get(&task_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().url, config.url);
    }

    #[tokio::test]
    async fn test_get_nonexistent_config() {
        let store = PushNotificationStore::new();
        let result = store.get("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_set_invalid_config_https() {
        let store = PushNotificationStore::new();
        let invalid_config = PushNotificationConfig::new(
            Url::parse("http://example.com/webhook").unwrap(), // HTTP not HTTPS
            vec![TaskEvent::Completed],
            None,
        );

        let result = store.set("task-123".to_string(), invalid_config).await;
        assert!(result.is_err());
        match result {
            Err(A2aError::UnsupportedOperation(msg)) => {
                assert!(msg.contains("HTTPS"));
            }
            _ => panic!("Expected UnsupportedOperation error"),
        }
    }

    #[tokio::test]
    async fn test_set_invalid_config_empty_events() {
        let store = PushNotificationStore::new();
        let invalid_config = PushNotificationConfig::new(
            Url::parse("https://example.com/webhook").unwrap(),
            vec![], // Empty events
            None,
        );

        let result = store.set("task-123".to_string(), invalid_config).await;
        assert!(result.is_err());
        match result {
            Err(A2aError::UnsupportedOperation(msg)) => {
                assert!(msg.contains("event"));
            }
            _ => panic!("Expected UnsupportedOperation error"),
        }
    }

    #[tokio::test]
    async fn test_delete_config() {
        let store = PushNotificationStore::new();
        let config = create_test_config();
        let task_id = "task-123".to_string();

        store.set(task_id.clone(), config).await.unwrap();
        assert_eq!(store.count().await, 1);

        let deleted = store.delete(&task_id).await.unwrap();
        assert!(deleted);
        assert_eq!(store.count().await, 0);
    }

    #[tokio::test]
    async fn test_delete_nonexistent_config() {
        let store = PushNotificationStore::new();
        let deleted = store.delete("nonexistent").await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_list_configs() {
        let store = PushNotificationStore::new();

        let config1 = create_test_config();
        let config2 = PushNotificationConfig::new(
            Url::parse("https://example.com/webhook2").unwrap(),
            vec![TaskEvent::StatusChanged],
            Some(PushNotificationAuth::Bearer {
                token: "secret".to_string(),
            }),
        );

        store.set("task-1".to_string(), config1).await.unwrap();
        store.set("task-2".to_string(), config2).await.unwrap();

        let configs = store.list().await.unwrap();
        assert_eq!(configs.len(), 2);

        let task_ids: Vec<String> = configs.iter().map(|(id, _)| id.clone()).collect();
        assert!(task_ids.contains(&"task-1".to_string()));
        assert!(task_ids.contains(&"task-2".to_string()));
    }

    #[tokio::test]
    async fn test_list_empty() {
        let store = PushNotificationStore::new();
        let configs = store.list().await.unwrap();
        assert_eq!(configs.len(), 0);
    }

    #[tokio::test]
    async fn test_count() {
        let store = PushNotificationStore::new();
        assert_eq!(store.count().await, 0);

        store
            .set("task-1".to_string(), create_test_config())
            .await
            .unwrap();
        assert_eq!(store.count().await, 1);

        store
            .set("task-2".to_string(), create_test_config())
            .await
            .unwrap();
        assert_eq!(store.count().await, 2);
    }

    #[tokio::test]
    async fn test_update_config() {
        let store = PushNotificationStore::new();
        let task_id = "task-123".to_string();

        let config1 = create_test_config();
        store.set(task_id.clone(), config1).await.unwrap();

        let config2 = PushNotificationConfig::new(
            Url::parse("https://example.com/new-webhook").unwrap(),
            vec![TaskEvent::ArtifactAdded],
            None,
        );
        store.set(task_id.clone(), config2.clone()).await.unwrap();

        let retrieved = store.get(&task_id).await.unwrap().unwrap();
        assert_eq!(retrieved.url, config2.url);
        assert_eq!(retrieved.events, config2.events);
        assert_eq!(store.count().await, 1); // Should still be 1, not 2
    }

    #[tokio::test]
    async fn test_clear() {
        let store = PushNotificationStore::new();

        store
            .set("task-1".to_string(), create_test_config())
            .await
            .unwrap();
        store
            .set("task-2".to_string(), create_test_config())
            .await
            .unwrap();
        assert_eq!(store.count().await, 2);

        store.clear().await;
        assert_eq!(store.count().await, 0);
    }
}
