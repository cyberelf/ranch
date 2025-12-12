//! Unified Protocol Compliance Test Suite
//! 
//! This comprehensive test suite validates that our Rust data structures match
//! the official A2A protocol specification by testing serialization format,
//! field names, structure, and behavior.

use a2a_protocol::prelude::*;
use a2a_protocol::Artifact;
use a2a_protocol::core::agent_card::{AgentCardSignature, TransportInterface, TransportType};
use serde_json::Value;

/// Helper to serialize and return JSON for inspection
fn to_json<T: serde::Serialize>(value: &T) -> Value {
    serde_json::to_value(value).expect("Failed to serialize to JSON")
}

// ============================================================================
// Core Data Structure Tests
// ============================================================================

#[test]
fn test_task_status_structure() {
    let status = TaskStatus::new(TaskState::Completed)
        .with_message(Message::agent_text("Task completed successfully"));

    let json = to_json(&status);

    // Required field: state
    assert!(json.get("state").is_some(), "state field is required");
    assert!(json["state"].is_string(), "state should be a string");
    
    // Optional fields: message, timestamp
    if let Some(msg) = json.get("message") {
        assert!(msg.is_object(), "message should be an object");
        assert!(msg.get("role").is_some(), "message.role is required");
        assert!(msg.get("parts").is_some(), "message.parts is required");
    }
    
    // No extra fields allowed
    let obj = json.as_object().unwrap();
    for key in obj.keys() {
        assert!(
            key == "state" || key == "message" || key == "timestamp",
            "TaskStatus has unexpected field '{}'. Only 'state', 'message', and 'timestamp' are allowed",
            key
        );
    }
}

#[test]
fn test_task_structure() {
    let mut task = Task::new("task-123");
    task.context_id = Some("ctx-abc".to_string());
    task.add_message(Message::user_text("Hello"));
    task.add_artifact(Artifact {
        artifact_id: "art-1".to_string(),
        name: Some("Test Artifact".to_string()),
        description: Some("A test artifact".to_string()),
        parts: vec![Part::Text(TextPart::new("content"))],
        metadata: None,
    });

    let json = to_json(&task);

    // Required fields
    assert!(json.get("id").is_some(), "id field is required");
    assert!(json.get("status").is_some(), "status field is required");

    // camelCase field naming
    if json.get("contextId").is_some() {
        assert!(json["contextId"].is_string(), "contextId should be a string");
    }

    // History should be array of messages, not TaskStatus
    if let Some(history) = json.get("history") {
        assert!(history.is_array(), "history should be an array");
        for item in history.as_array().unwrap() {
            assert!(item.is_object(), "history items should be objects (Messages)");
            assert!(item.get("role").is_some(), "history message must have role");
            assert!(item.get("parts").is_some(), "history message must have parts");
            // Should NOT have TaskStatus fields
            assert!(item.get("state").is_none(), "history items should be Messages, not TaskStatus");
        }
    }

    // Artifacts validation
    if let Some(artifacts) = json.get("artifacts") {
        assert!(artifacts.is_array(), "artifacts should be an array");
        for artifact in artifacts.as_array().unwrap() {
            assert!(artifact.get("artifactId").is_some(), "artifact must have artifactId (not 'id')");
            assert!(artifact.get("parts").is_some(), "artifact must have parts array");
            
            // Should NOT have old field names
            assert!(artifact.get("id").is_none(), "artifact should use 'artifactId', not 'id'");
            assert!(artifact.get("type").is_none(), "artifact should not have 'type' field");
            assert!(artifact.get("artifact_type").is_none(), "artifact should not have 'artifact_type' field");
            assert!(artifact.get("uri").is_none(), "artifact should not have standalone 'uri' field");
            assert!(artifact.get("data").is_none(), "artifact should not have standalone 'data' field");
        }
    }

    // No extra fields in Task
    let obj = json.as_object().unwrap();
    for key in obj.keys() {
        assert!(
            key == "id" || key == "contextId" || key == "status" || key == "artifacts" || key == "history" || key == "metadata",
            "Task has unexpected field: {}",
            key
        );
    }
}

#[test]
fn test_artifact_structure() {
    let artifact = Artifact {
        artifact_id: "art-123".to_string(),
        name: Some("Report".to_string()),
        description: Some("Analysis report".to_string()),
        parts: vec![Part::Text(TextPart::new("Content"))],
        metadata: None,
    };

    let json = to_json(&artifact);

    // Required fields
    assert!(json.get("artifactId").is_some(), "artifactId is required");
    assert!(json.get("parts").is_some(), "parts is required");
    assert!(json["parts"].is_array(), "parts should be an array");

    // No old field names
    assert!(json.get("id").is_none(), "should use 'artifactId', not 'id'");
    assert!(json.get("type").is_none(), "should not have 'type' field");
    assert!(json.get("artifact_type").is_none(), "should not have 'artifact_type' field");
    assert!(json.get("uri").is_none(), "should not have standalone 'uri' field");
    assert!(json.get("data").is_none(), "should not have standalone 'data' field");

    // Only allowed fields
    let obj = json.as_object().unwrap();
    for key in obj.keys() {
        assert!(
            key == "artifactId" || key == "name" || key == "description" || key == "parts" || key == "metadata" || key == "extensions",
            "Artifact has unexpected field: {}",
            key
        );
    }
}

#[test]
fn test_message_part_untagged_serialization() {
    let message = Message::user_text("Hello");
    let json = to_json(&message);

    let parts = json.get("parts").expect("parts should exist");
    let parts_arr = parts.as_array().expect("parts should be an array");
    
    for part in parts_arr {
        let part_obj = part.as_object().expect("part should be an object");
        
        // Parts should NOT have a "kind" field (untagged serialization)
        assert!(
            part_obj.get("kind").is_none(),
            "Part should not have 'kind' field. Protocol uses untagged serialization."
        );
        
        // Should have content field (text, file, or data)
        let has_content = part_obj.contains_key("text") 
            || part_obj.contains_key("file") 
            || part_obj.contains_key("data");
        assert!(has_content, "Part should have text, file, or data field");
    }
}

#[test]
fn test_message_structure() {
    let message = Message::user_text("Hello")
        .with_context_id("ctx-123")
        .with_task_id("task-456");

    let json = to_json(&message);

    // Required fields
    assert!(json.get("role").is_some(), "role is required");
    assert!(json.get("parts").is_some(), "parts is required");

    // Optional fields with correct casing
    if json.get("messageId").is_some() {
        assert!(json["messageId"].is_string(), "messageId should be a string");
    }
    if json.get("taskId").is_some() {
        assert!(json["taskId"].is_string(), "taskId should be camelCase");
    }
    if json.get("contextId").is_some() {
        assert!(json["contextId"].is_string(), "contextId should be camelCase");
    }

    // Only allowed fields
    let obj = json.as_object().unwrap();
    for key in obj.keys() {
        assert!(
            key == "role" || key == "parts" || key == "messageId" || key == "taskId" 
            || key == "contextId" || key == "metadata" || key == "referenceTaskIds" || key == "extensions",
            "Message has unexpected field: {}",
            key
        );
    }
}

#[test]
fn test_agent_card_structure() {
    let agent_id = AgentId::new("test-agent".to_owned()).unwrap();
    let url = url::Url::parse("https://example.com").unwrap();
    let http_url = url::Url::parse("https://example.com/http").unwrap();
    let icon_url = url::Url::parse("https://example.com/icon.png").unwrap();
    let docs_url = url::Url::parse("https://example.com/docs").unwrap();
    let provider_url = url::Url::parse("https://provider.example.com").unwrap();

    let card = AgentCard::new(agent_id, "Test Agent", url)
        .with_preferred_transport(TransportType::Grpc)
        .with_default_input_modes(["application/json"])
        .with_default_output_modes(["application/json"])
        .with_supports_authenticated_extended_card(true)
        .with_icon_url(Some(icon_url))
        .with_documentation_url(Some(docs_url))
        .with_provider(AgentProvider {
            name: "Example Provider".to_string(),
            description: Some("Example Org".to_string()),
            url: Some(provider_url),
            contact_email: None,
            contact_url: None,
            extra: std::collections::HashMap::new(),
        })
        .with_signatures([AgentCardSignature {
            algorithm: "Ed25519".to_string(),
            signature: "deadbeef".to_string(),
            key_id: Some("key-1".to_string()),
            certificate_chain: vec!["cert-1".to_string()],
            extra: std::collections::HashMap::new(),
        }])
        .add_transport_interface(TransportInterface::new(
            TransportType::HttpJson,
            http_url,
        ));

    let json = to_json(&card);

    // Required fields
    assert!(json.get("id").is_some(), "id is required");
    assert!(json.get("name").is_some(), "name is required");
    assert!(json.get("url").is_some(), "url is required");
    assert!(json.get("protocolVersion").is_some(), "protocolVersion is required");
    assert!(json.get("preferredTransport").is_some(), "preferredTransport is required");

    // Verify field values
    assert_eq!(json["preferredTransport"], "GRPC");
    assert_eq!(json["defaultInputModes"], serde_json::json!(["application/json"]));
    assert_eq!(json["defaultOutputModes"], serde_json::json!(["application/json"]));
    assert_eq!(json["supportsAuthenticatedExtendedCard"], true);
    assert_eq!(json["protocolVersion"], "0.3.0");
}

// ============================================================================
// Field Naming Tests
// ============================================================================

#[test]
fn test_camel_case_field_naming() {
    let mut task = Task::new("task-123");
    task.context_id = Some("ctx-abc".to_string());
    
    let json_str = serde_json::to_string(&to_json(&task)).unwrap();
    
    // These snake_case fields should NOT appear (should be camelCase)
    assert!(!json_str.contains("\"context_id\""), "Should use 'contextId', not 'context_id'");
    assert!(!json_str.contains("\"task_id\""), "Should use 'taskId', not 'task_id'");
    assert!(!json_str.contains("\"message_id\""), "Should use 'messageId', not 'message_id'");
    assert!(!json_str.contains("\"artifact_id\""), "Should use 'artifactId', not 'artifact_id'");
}

// ============================================================================
// Round-Trip Serialization Tests
// ============================================================================

#[test]
fn test_task_round_trip() {
    let mut original_task = Task::new("task-123");
    original_task.context_id = Some("ctx-abc".to_string());
    original_task.add_message(Message::user_text("Request"));
    original_task.add_message(Message::agent_text("Response"));
    original_task.add_artifact(Artifact {
        artifact_id: "art-1".to_string(),
        name: Some("Result".to_string()),
        description: Some("Task result".to_string()),
        parts: vec![Part::Text(TextPart::new("Output"))],
        metadata: None,
    });
    original_task.status = TaskStatus::new(TaskState::Completed)
        .with_message(Message::agent_text("All done"));

    // Serialize
    let json_str = serde_json::to_string(&original_task).unwrap();
    
    // Deserialize
    let deserialized_task: Task = serde_json::from_str(&json_str).unwrap();

    // Verify
    assert_eq!(original_task.id, deserialized_task.id);
    assert_eq!(original_task.context_id, deserialized_task.context_id);
    assert_eq!(original_task.status.state, deserialized_task.status.state);
    assert_eq!(
        original_task.history.as_ref().unwrap().len(),
        deserialized_task.history.as_ref().unwrap().len()
    );
    assert_eq!(
        original_task.artifacts.as_ref().unwrap().len(),
        deserialized_task.artifacts.as_ref().unwrap().len()
    );
}

#[test]
fn test_message_round_trip() {
    let original = Message::user_text("Test message");
    let serialized = serde_json::to_string(&original).unwrap();
    let deserialized: Message = serde_json::from_str(&serialized).unwrap();

    assert_eq!(original.message_id, deserialized.message_id);
    assert_eq!(original.role, deserialized.role);
    assert_eq!(original.parts.len(), deserialized.parts.len());
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_error_code_mapping() {
    let test_cases = vec![
        (A2aError::Authentication("test".to_string()), 401),
        (A2aError::Validation("test".to_string()), 400),
        (A2aError::ProtocolViolation("test".to_string()), 422),
        (A2aError::RateLimited(std::time::Duration::from_secs(60)), 429),
    ];

    for (error, expected_code) in test_cases {
        assert_eq!(error.status_code(), Some(expected_code));
    }
}

#[test]
fn test_retryable_errors() {
    let retryable = vec![
        A2aError::Timeout,
        A2aError::RateLimited(std::time::Duration::from_secs(60)),
        A2aError::Server("server error".to_string()),
    ];

    for error in retryable {
        assert!(error.is_retryable(), "Should be retryable: {:?}", error);
    }

    let non_retryable = vec![
        A2aError::Authentication("auth error".to_string()),
        A2aError::Validation("validation error".to_string()),
        A2aError::ProtocolViolation("protocol error".to_string()),
    ];

    for error in non_retryable {
        assert!(!error.is_retryable(), "Should not be retryable: {:?}", error);
    }
}

// ============================================================================
// Agent ID Validation Tests
// ============================================================================

#[test]
fn test_agent_id_validation() {
    // Valid
    let valid = vec!["https://agent.example.com", "agent-123", "my_agent", "agent.example.com"];
    for id in valid {
        assert!(AgentId::new(id.to_string()).is_ok(), "Should be valid: {}", id);
    }

    // Invalid
    let invalid = vec!["", "   ", "ht tp://bad-url"];
    for id in invalid {
        assert!(AgentId::new(id.to_string()).is_err(), "Should be invalid: {}", id);
    }
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_serialization_performance() {
    let message = Message::user_text("Test message");
    let start = std::time::Instant::now();
    
    for _ in 0..1000 {
        let _ = serde_json::to_string(&message).unwrap();
    }
    
    let duration = start.elapsed();
    assert!(
        duration.as_millis() < 100,
        "Serialization too slow: {}ms for 1000 messages",
        duration.as_millis()
    );
}
