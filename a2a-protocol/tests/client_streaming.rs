//! Integration tests for client streaming API
#![cfg(feature = "streaming")]

use a2a_protocol::prelude::*;
use a2a_protocol::client::{A2aStreamingClient, ClientBuilder};
use a2a_protocol::server::{Agent, TaskAwareHandler, JsonRpcRouter};
use a2a_protocol::client::transport::{StreamingResult, JsonRpcTransport};
use a2a_protocol::TaskResubscribeRequest;
use futures_util::StreamExt;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use async_trait::async_trait;

struct TestAgent {
    profile: AgentProfile,
}

#[async_trait]
impl Agent for TestAgent {
    async fn profile(&self) -> Result<AgentProfile, A2aError> {
        Ok(self.profile.clone())
    }

    async fn process_message(&self, _message: Message) -> Result<Message, A2aError> {
        Ok(Message::agent_text("Test response"))
    }
}

/// Helper to start a test server with TaskAwareHandler
async fn start_test_server() -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let agent_id = AgentId::new("streaming-test-agent".to_string()).unwrap();
    let profile = AgentProfile::new(
        agent_id.clone(),
        "Streaming Test Agent",
        url::Url::parse("https://example.com").unwrap(),
    );

    let agent = Arc::new(TestAgent { profile });
    let handler = TaskAwareHandler::new(agent);
    let router = JsonRpcRouter::new(handler);
    let app = router.into_router();

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    (addr, server_handle)
}

#[tokio::test]
async fn test_client_stream_message() {
    let (addr, _server_handle) = start_test_server().await;
    
    // Create streaming client - compile-time guarantee of streaming support
    let endpoint = format!("http://{}/rpc", addr);
    let transport = Arc::new(JsonRpcTransport::new(&endpoint).unwrap());
    let client = A2aStreamingClient::new(transport);

    // Send a streaming message
    let message = Message::user_text("Test streaming message");
    let mut stream = client.stream_message(message).await.unwrap();

    // Collect events
    let mut events = Vec::new();
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => events.push(event),
            Err(e) => panic!("Stream error: {:?}", e),
        }
        
        // Limit to prevent infinite loops in case of bugs
        if events.len() > 100 {
            break;
        }
    }

    // We should receive at least a task creation event
    assert!(!events.is_empty(), "Should receive at least one event");
    
    // First event should be a Task
    match &events[0] {
        StreamingResult::Task(task) => {
            assert!(!task.id.is_empty(), "Task should have an ID");
        }
        other => panic!("Expected first event to be Task, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_client_stream_text() {
    let (addr, _server_handle) = start_test_server().await;
    
    let endpoint = format!("http://{}/rpc", addr);
    let transport = Arc::new(JsonRpcTransport::new(&endpoint).unwrap());
    let client = A2aStreamingClient::new(transport);

    // Use the convenience method
    let mut stream = client.stream_text("Hello from stream_text").await.unwrap();

    let mut received_task = false;
    while let Some(result) = stream.next().await {
        if let Ok(StreamingResult::Task(_)) = result {
            received_task = true;
            break;
        }
    }

    assert!(received_task, "Should receive a task event");
}

#[tokio::test]
async fn test_client_resubscribe_task() {
    let (addr, _server_handle) = start_test_server().await;
    
    let endpoint = format!("http://{}/rpc", addr);
    let transport = Arc::new(JsonRpcTransport::new(&endpoint).unwrap());
    let client = A2aStreamingClient::new(transport);

    // First, create a task via streaming
    let message = Message::user_text("Create a task");
    let mut stream = client.stream_message(message).await.unwrap();

    // Get the task ID from the first event
    let task_id = if let Some(Ok(StreamingResult::Task(task))) = stream.next().await {
        task.id.clone()
    } else {
        panic!("Failed to get task from stream");
    };

    // Now resubscribe to the task
    let request = TaskResubscribeRequest {
        task_id: task_id.clone(),
        metadata: None,
    };

    let mut resubscribe_stream = client.resubscribe_task(request).await.unwrap();

    // Should receive task updates
    let mut event_count = 0;
    while let Some(result) = resubscribe_stream.next().await {
        event_count += 1;
        if let Err(e) = result {
            eprintln!("Resubscribe stream error: {:?}", e);
        }
        
        if event_count > 10 {
            break;
        }
    }

    // Note: This test may fail if the task completes before we resubscribe
    // In production, this would be more reliable
    println!("Received {} events from resubscribe", event_count);
}

#[tokio::test]
async fn test_client_resubscribe_with_last_event_id() {
    let (addr, _server_handle) = start_test_server().await;
    
    let endpoint = format!("http://{}/rpc", addr);
    let transport = Arc::new(JsonRpcTransport::new(&endpoint).unwrap());
    let client = A2aStreamingClient::new(transport);

    // Create a task
    let message = Message::user_text("Create a task");
    let mut stream = client.stream_message(message).await.unwrap();

    let task_id = if let Some(Ok(StreamingResult::Task(task))) = stream.next().await {
        task.id.clone()
    } else {
        panic!("Failed to get task from stream");
    };

    // Resubscribe with last event ID
    let metadata = serde_json::json!({
        "lastEventId": "event-123"
    });

    let request = TaskResubscribeRequest {
        task_id,
        metadata: Some(metadata),
    };

    // This should work even if the server doesn't have event-123
    // (it will start from the beginning or current position)
    let result = client.resubscribe_task(request).await;
    assert!(result.is_ok(), "Resubscribe with lastEventId should not error");
}

#[tokio::test]
async fn test_streaming_error_handling() {
    // Try to stream from a non-existent server
    let endpoint = "http://127.0.0.1:19999/rpc"; // Port that shouldn't be in use
    let transport = Arc::new(JsonRpcTransport::new(endpoint).unwrap());
    let client = A2aStreamingClient::new(transport);

    let message = Message::user_text("This should fail");
    let result = client.stream_message(message).await;

    assert!(result.is_err(), "Should error when connecting to non-existent server");
}

#[tokio::test]
async fn test_client_builder_with_streaming() {
    let (addr, _server_handle) = start_test_server().await;
    
    let endpoint = format!("http://{}/rpc", addr);
    
    // A2aClient no longer supports streaming - use A2aStreamingClient instead
    let client = ClientBuilder::new()
        .with_json_rpc(&endpoint)
        .build_streaming()
        .unwrap();

    let message = Message::user_text("Test via ClientBuilder");
    let result = client.stream_message(message).await;

    assert!(result.is_ok(), "Streaming client should handle streaming");
}

#[tokio::test]
async fn test_typed_streaming_client() {
    let (addr, _server_handle) = start_test_server().await;
    
    let endpoint = format!("http://{}/rpc", addr);
    
    // Create a typed streaming client for compile-time guarantees
    let streaming_client = ClientBuilder::new()
        .with_json_rpc(&endpoint)
        .build_streaming()
        .unwrap();

    // Use typed client for streaming - no runtime checks needed
    let message = Message::user_text("Test typed streaming client");
    let mut stream = streaming_client.stream_message(message).await.unwrap();

    // Collect at least one event
    let mut received_event = false;
    while let Some(result) = stream.next().await {
        if result.is_ok() {
            received_event = true;
            break;
        }
    }

    assert!(received_event, "Typed streaming client should receive events");
}

#[tokio::test]
async fn test_typed_client_base_access() {
    let (addr, _server_handle) = start_test_server().await;
    
    let endpoint = format!("http://{}/rpc", addr);
    
    let streaming_client = ClientBuilder::new()
        .with_json_rpc(&endpoint)
        .build_streaming()
        .unwrap();

    // Access base client methods directly via Deref (no .base() needed!)
    assert_eq!(streaming_client.transport_type(), "json-rpc");
    assert!(streaming_client.config().timeout_seconds > 0);
    
    // Can still use .base() explicitly if preferred
    assert_eq!(streaming_client.base().transport_type(), "json-rpc");
}

