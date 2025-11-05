//! Integration tests for push notification RPC methods

use a2a_protocol::{
    core::push_notification::{PushNotificationAuth, PushNotificationConfig, TaskEvent},
    server::{A2aHandler, TaskAwareHandler},
    AgentCard, AgentId, PushNotificationDeleteRequest, PushNotificationGetRequest,
    PushNotificationListRequest, PushNotificationSetRequest,
};
use url::Url;

fn create_test_handler() -> TaskAwareHandler {
    let agent_id = AgentId::new("test-agent".to_string()).unwrap();
    let agent_card = AgentCard::new(
        agent_id,
        "Test Agent",
        Url::parse("https://example.com").unwrap(),
    );
    TaskAwareHandler::new(agent_card)
}

fn create_test_config() -> PushNotificationConfig {
    PushNotificationConfig::new(
        Url::parse("https://webhook.example.com/notify").unwrap(),
        vec![TaskEvent::Completed, TaskEvent::Failed],
        None,
    )
}

#[tokio::test]
async fn test_push_notification_set_and_get() {
    let handler = create_test_handler();
    let config = create_test_config();

    // Set configuration
    let set_request = PushNotificationSetRequest::new("task-123", config.clone());
    let result = handler.rpc_push_notification_set(set_request).await;
    assert!(result.is_ok());

    // Get configuration
    let get_request = PushNotificationGetRequest::new("task-123");
    let retrieved = handler.rpc_push_notification_get(get_request).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().url, config.url);
}

#[tokio::test]
async fn test_push_notification_get_nonexistent() {
    let handler = create_test_handler();

    let get_request = PushNotificationGetRequest::new("nonexistent");
    let result = handler.rpc_push_notification_get(get_request).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_push_notification_set_invalid_http() {
    let handler = create_test_handler();

    // Try to set config with HTTP (not HTTPS)
    let invalid_config = PushNotificationConfig::new(
        Url::parse("http://webhook.example.com/notify").unwrap(),
        vec![TaskEvent::Completed],
        None,
    );

    let set_request = PushNotificationSetRequest::new("task-123", invalid_config);
    let result = handler.rpc_push_notification_set(set_request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_push_notification_set_private_ip() {
    let handler = create_test_handler();

    // Try to set config with private IP
    let invalid_config = PushNotificationConfig::new(
        Url::parse("https://192.168.1.1/webhook").unwrap(),
        vec![TaskEvent::Completed],
        None,
    );

    let set_request = PushNotificationSetRequest::new("task-123", invalid_config);
    let result = handler.rpc_push_notification_set(set_request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_push_notification_list_empty() {
    let handler = create_test_handler();

    let list_request = PushNotificationListRequest::new();
    let response = handler.rpc_push_notification_list(list_request).await.unwrap();
    assert_eq!(response.configurations.len(), 0);
}

#[tokio::test]
async fn test_push_notification_list_multiple() {
    let handler = create_test_handler();

    // Add multiple configurations
    let config1 = create_test_config();
    let config2 = PushNotificationConfig::new(
        Url::parse("https://webhook2.example.com/notify").unwrap(),
        vec![TaskEvent::StatusChanged],
        Some(PushNotificationAuth::Bearer {
            token: "secret".to_string(),
        }),
    );

    handler
        .rpc_push_notification_set(PushNotificationSetRequest::new("task-1", config1))
        .await
        .unwrap();
    handler
        .rpc_push_notification_set(PushNotificationSetRequest::new("task-2", config2))
        .await
        .unwrap();

    // List configurations
    let list_request = PushNotificationListRequest::new();
    let response = handler.rpc_push_notification_list(list_request).await.unwrap();
    assert_eq!(response.configurations.len(), 2);

    let task_ids: Vec<String> = response
        .configurations
        .iter()
        .map(|c| c.task_id.clone())
        .collect();
    assert!(task_ids.contains(&"task-1".to_string()));
    assert!(task_ids.contains(&"task-2".to_string()));
}

#[tokio::test]
async fn test_push_notification_delete() {
    let handler = create_test_handler();
    let config = create_test_config();

    // Set configuration
    handler
        .rpc_push_notification_set(PushNotificationSetRequest::new("task-123", config))
        .await
        .unwrap();

    // Delete configuration
    let delete_request = PushNotificationDeleteRequest::new("task-123");
    let deleted = handler
        .rpc_push_notification_delete(delete_request)
        .await
        .unwrap();
    assert!(deleted);

    // Verify it's gone
    let get_request = PushNotificationGetRequest::new("task-123");
    let result = handler.rpc_push_notification_get(get_request).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_push_notification_delete_nonexistent() {
    let handler = create_test_handler();

    let delete_request = PushNotificationDeleteRequest::new("nonexistent");
    let deleted = handler
        .rpc_push_notification_delete(delete_request)
        .await
        .unwrap();
    assert!(!deleted);
}

#[tokio::test]
async fn test_push_notification_update() {
    let handler = create_test_handler();

    // Set initial configuration
    let config1 = create_test_config();
    handler
        .rpc_push_notification_set(PushNotificationSetRequest::new("task-123", config1))
        .await
        .unwrap();

    // Update with new configuration
    let config2 = PushNotificationConfig::new(
        Url::parse("https://new-webhook.example.com/notify").unwrap(),
        vec![TaskEvent::ArtifactAdded],
        None,
    );
    handler
        .rpc_push_notification_set(PushNotificationSetRequest::new("task-123", config2.clone()))
        .await
        .unwrap();

    // Verify updated configuration
    let get_request = PushNotificationGetRequest::new("task-123");
    let retrieved = handler.rpc_push_notification_get(get_request).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().url, config2.url);

    // Verify only one configuration exists
    let list_request = PushNotificationListRequest::new();
    let response = handler.rpc_push_notification_list(list_request).await.unwrap();
    assert_eq!(response.configurations.len(), 1);
}
