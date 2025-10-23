//! Integration tests for JSON-RPC method handlers

use a2a_protocol::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use a2a_protocol::server::{handler::BasicA2aHandler, A2aHandler};

    #[tokio::test]
    async fn test_rpc_message_send_handler() {
        // Create a basic handler
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let agent_card = AgentCard::new(
            agent_id,
            "Test Agent",
            url::Url::parse("https://example.com").unwrap(),
        );

        let handler = BasicA2aHandler::new(agent_card);

        // Test message/send RPC method
        let message = Message::user_text("Test message");
        let request = MessageSendRequest {
            message: message.clone(),
            immediate: Some(true), // Request immediate response
        };

        let response = handler.rpc_message_send(request).await.unwrap();

        // Should return an immediate message response
        assert!(
            response.is_message(),
            "Should return immediate message when immediate=true"
        );

        let response_msg = response.as_message().unwrap();
        assert_eq!(response_msg.role, MessageRole::Agent);
        assert!(response_msg.text_content().unwrap().starts_with("Echo:"));
    }

    #[tokio::test]
    async fn test_rpc_task_get_not_implemented() {
        // Create a basic handler
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let agent_card = AgentCard::new(
            agent_id,
            "Test Agent",
            url::Url::parse("https://example.com").unwrap(),
        );

        let handler = BasicA2aHandler::new(agent_card);

        // Test task/get RPC method (should not be implemented in basic handler)
        let request = TaskGetRequest {
            task_id: "task-123".to_string(),
        };

        let result = handler.rpc_task_get(request).await;
        assert!(
            result.is_err(),
            "task/get should not be implemented in basic handler"
        );

        if let Err(e) = result {
            assert!(matches!(e, A2aError::Server(_)));
        }
    }

    #[tokio::test]
    async fn test_rpc_task_cancel_not_implemented() {
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let agent_card = AgentCard::new(
            agent_id,
            "Test Agent",
            url::Url::parse("https://example.com").unwrap(),
        );

        let handler = BasicA2aHandler::new(agent_card);

        let request = TaskCancelRequest {
            task_id: "task-123".to_string(),
            reason: Some("User requested cancellation".to_string()),
        };

        let result = handler.rpc_task_cancel(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rpc_task_status_not_implemented() {
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let agent_card = AgentCard::new(
            agent_id,
            "Test Agent",
            url::Url::parse("https://example.com").unwrap(),
        );

        let handler = BasicA2aHandler::new(agent_card);

        let request = TaskStatusRequest {
            task_id: "task-123".to_string(),
        };

        let result = handler.rpc_task_status(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rpc_agent_card() {
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let agent_card = AgentCard::new(
            agent_id.clone(),
            "Test Agent",
            url::Url::parse("https://example.com").unwrap(),
        );

        let handler = BasicA2aHandler::new(agent_card);

        // Test agent/card RPC method
        let request = AgentCardGetRequest {
            agent_id: None, // Get this handler's agent card
        };

        let result = handler.rpc_agent_card(request).await.unwrap();

        assert_eq!(result.id, agent_id);
        assert_eq!(result.name, "Test Agent");
    }

    #[test]
    fn test_json_rpc_request_creation() {
        let message = Message::user_text("Test");
        let request = MessageSendRequest {
            message,
            immediate: Some(true),
        };

        let rpc_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!("req-123"),
            method: "message/send".to_string(),
            params: Some(request),
        };

        // Verify serialization
        let json = serde_json::to_value(&rpc_request).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["method"], "message/send");
        assert_eq!(json["id"], "req-123");
        assert!(json["params"].is_object());
    }

    #[test]
    fn test_json_rpc_response_success() {
        let message = Message::agent_text("Response");
        let response = SendResponse::message(message);

        let rpc_response: JsonRpcResponse<SendResponse> = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!("req-123"),
            result: Some(response),
            error: None,
        };

        // Verify serialization
        let json = serde_json::to_value(&rpc_response).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], "req-123");
        assert!(json["result"].is_object());
        assert!(json["error"].is_null());
    }

    #[test]
    fn test_json_rpc_response_error() {
        let error = JsonRpcError {
            code: -32603,
            message: "Internal error".to_string(),
            data: Some(serde_json::json!({"details": "Something went wrong"})),
        };

        let rpc_response: JsonRpcResponse<SendResponse> = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!("req-123"),
            result: None,
            error: Some(error),
        };

        // Verify serialization
        let json = serde_json::to_value(&rpc_response).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], "req-123");
        assert!(json["result"].is_null());
        assert_eq!(json["error"]["code"], -32603);
        assert_eq!(json["error"]["message"], "Internal error");
    }
}
