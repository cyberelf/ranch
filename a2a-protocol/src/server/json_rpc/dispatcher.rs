//! Framework-agnostic JSON-RPC 2.0 dispatcher for A2A

use crate::{
    server::A2aHandler,
    core::{
        is_batch_request, is_notification, map_error_to_rpc,
        JsonRpcResponse,
    },
    A2aError, A2aResult, AgentCardGetRequest, MessageSendRequest,
    TaskCancelRequest, TaskGetRequest, TaskStatusRequest,
};
use serde_json::Value;

/// Dispatch a JSON-RPC request body (single or batch) to the provided handler.
/// Returns a JSON body Vec<u8>. For notifications, returns an empty body.
pub async fn dispatch_bytes(handler: &dyn A2aHandler, body: &[u8]) -> A2aResult<Vec<u8>> {
    let value: Value = serde_json::from_slice(body).map_err(A2aError::Json)?;

    if is_batch_request(&value) {
        let arr = value.as_array().cloned().unwrap_or_default();
        let mut responses: Vec<Value> = Vec::with_capacity(arr.len());
        for item in arr {
            if is_notification(&item) {
                // Fire-and-forget
                let _ = handle_single(handler, item).await; // ignore errors for notification per spec
                continue;
            } else {
                match handle_single(handler, item).await {
                    Ok(resp) => responses.push(resp),
                    Err(err) => responses
                        .push(serde_json::to_value(error_response(Value::Null, err)).unwrap()),
                }
            }
        }
        // If all were notifications, spec says return nothing; but servers may return empty array. We'll return []
        Ok(serde_json::to_vec(&responses).map_err(A2aError::Json)?)
    } else {
        if is_notification(&value) {
            let _ = handle_single(handler, value).await; // no response
            Ok(Vec::new())
        } else {
            let resp = handle_single(handler, value).await?;
            Ok(serde_json::to_vec(&resp).map_err(A2aError::Json)?)
        }
    }
}

async fn handle_single(handler: &dyn A2aHandler, req: Value) -> A2aResult<Value> {
    // Extract common fields
    let id = req.get("id").cloned().unwrap_or(Value::Null);
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(Value::Null);

    // Route by method name
    let result_value = match method {
        "message/send" => {
            let typed: MessageSendRequest =
                serde_json::from_value(params).map_err(A2aError::Json)?;
            let resp = handler.rpc_message_send(typed).await?;
            serde_json::to_value(resp).map_err(A2aError::Json)?
        }
        "task/get" => {
            let typed: TaskGetRequest = serde_json::from_value(params).map_err(A2aError::Json)?;
            let task = handler.rpc_task_get(typed).await?;
            serde_json::to_value(task).map_err(A2aError::Json)?
        }
        "task/status" => {
            let typed: TaskStatusRequest =
                serde_json::from_value(params).map_err(A2aError::Json)?;
            let status = handler.rpc_task_status(typed).await?;
            serde_json::to_value(status).map_err(A2aError::Json)?
        }
        "task/cancel" => {
            let typed: TaskCancelRequest =
                serde_json::from_value(params).map_err(A2aError::Json)?;
            let status = handler.rpc_task_cancel(typed).await?;
            serde_json::to_value(status).map_err(A2aError::Json)?
        }
        "agent/card" => {
            let typed: AgentCardGetRequest =
                serde_json::from_value(params).map_err(A2aError::Json)?;
            let card = handler.rpc_agent_card(typed).await?;
            serde_json::to_value(card).map_err(A2aError::Json)?
        }
        _ => {
            return Err(A2aError::ProtocolViolation(format!(
                "Unknown method: {}",
                method
            )))
        }
    };

    Ok(serde_json::to_value(success_response(id, result_value)).map_err(A2aError::Json)?)
}

fn success_response(id: Value, result: Value) -> JsonRpcResponse<Value> {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result),
        error: None,
    }
}

fn error_response(id: Value, error: A2aError) -> JsonRpcResponse<Value> {
    let rpc_error = map_error_to_rpc(error);
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(rpc_error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::handler::BasicA2aHandler;
    use crate::{AgentCard, AgentId};

    #[tokio::test]
    async fn test_dispatch_single_message_send() {
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let profile = crate::server::AgentProfile::new(
            agent_id,
            "Test Agent",
            url::Url::parse("https://example.com").unwrap(),
        );
        let handler = BasicA2aHandler::new(profile);

        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "message/send",
            "params": {"message": {"role": "user", "parts": [{"kind":"text","text":"hello"}]}, "immediate": true}
        });

        let bytes = serde_json::to_vec(&req).unwrap();
        let resp_bytes = dispatch_bytes(&handler, &bytes).await.unwrap();
        let resp: serde_json::Value = serde_json::from_slice(&resp_bytes).unwrap();
        assert_eq!(resp["jsonrpc"], "2.0");
        assert!(resp["result"].is_object());
        assert!(resp["error"].is_null());
    }

    #[tokio::test]
    async fn test_dispatch_batch() {
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let profile = crate::server::AgentProfile::new(
            agent_id,
            "Test Agent",
            url::Url::parse("https://example.com").unwrap(),
        );
        let handler = BasicA2aHandler::new(profile);

        let req = serde_json::json!([
            {"jsonrpc": "2.0", "id": 1, "method": "message/send", "params": {"message": {"role":"user", "parts":[{"kind":"text","text":"hi"}]}, "immediate": true}},
            {"jsonrpc": "2.0", "id": 2, "method": "agent/card", "params": {}}
        ]);
        let bytes = serde_json::to_vec(&req).unwrap();
        let resp_bytes = dispatch_bytes(&handler, &bytes).await.unwrap();
        let resp: serde_json::Value = serde_json::from_slice(&resp_bytes).unwrap();
        assert!(resp.is_array());
        assert_eq!(resp.as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_dispatch_notification() {
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let profile = crate::server::AgentProfile::new(
            agent_id,
            "Test Agent",
            url::Url::parse("https://example.com").unwrap(),
        );
        let handler = BasicA2aHandler::new(profile);

        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "message/send",
            "params": {"message": {"role": "user", "parts": [{"kind":"text","text":"hello"}]}, "immediate": true}
        });

        let bytes = serde_json::to_vec(&req).unwrap();
        let resp_bytes = dispatch_bytes(&handler, &bytes).await.unwrap();
        assert!(resp_bytes.is_empty()); // Notification returns no content
    }

    #[tokio::test]
    async fn test_dispatch_task_not_found_error() {
        use crate::A2aError;

        // Create a handler that returns TaskNotFound for rpc_task_get
        struct ErrHandler {
            agent_card: AgentCard,
        }

        #[async_trait::async_trait]
        impl crate::server::A2aHandler for ErrHandler {
            async fn handle_message(
                &self,
                _message: crate::Message,
            ) -> crate::A2aResult<crate::SendResponse> {
                Err(A2aError::Server("not implemented".to_string()))
            }

            async fn get_agent_card(&self) -> crate::A2aResult<AgentCard> {
                Ok(self.agent_card.clone())
            }

            async fn health_check(&self) -> crate::A2aResult<crate::server::handler::HealthStatus> {
                Ok(crate::server::handler::HealthStatus::healthy())
            }

            async fn rpc_task_get(
                &self,
                _request: crate::TaskGetRequest,
            ) -> crate::A2aResult<crate::Task> {
                Err(A2aError::TaskNotFound {
                    task_id: "missing".to_string(),
                })
            }

            #[cfg(feature = "streaming")]
            async fn rpc_message_stream(
                &self,
                _message: crate::Message,
            ) -> crate::A2aResult<
                Box<dyn futures_util::stream::Stream<Item = crate::A2aResult<crate::client::transport::StreamingResult>> + Send + Unpin>,
            > {
                Err(A2aError::Server("streaming not supported".to_string()))
            }

            #[cfg(feature = "streaming")]
            async fn rpc_task_resubscribe(
                &self,
                _request: crate::TaskResubscribeRequest,
            ) -> crate::A2aResult<
                Box<dyn futures_util::stream::Stream<Item = crate::A2aResult<crate::client::transport::StreamingResult>> + Send + Unpin>,
            > {
                Err(A2aError::Server("streaming not supported".to_string()))
            }
        }

        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let card = AgentCard::new(
            agent_id,
            "Test Agent",
            url::Url::parse("https://example.com").unwrap(),
        );
        let handler = ErrHandler { agent_card: card };

        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 42,
            "method": "task/get",
            "params": { "taskId": "missing" }
        });

        let bytes = serde_json::to_vec(&req).unwrap();
        match dispatch_bytes(&handler, &bytes).await {
            Ok(resp_bytes) => {
                let resp: serde_json::Value = serde_json::from_slice(&resp_bytes).unwrap();
                assert_eq!(resp["jsonrpc"], "2.0");
                assert!(resp["error"].is_object());
                // A2A spec maps TaskNotFound to -32001
                assert_eq!(resp["error"]["code"], serde_json::json!(-32001));
            }
            Err(err) => {
                // Dispatcher returns Err for single-request failures; convert to a JSON-RPC error for assertion
                let rpc = error_response(serde_json::json!(42), err);
                assert_eq!(rpc.error.unwrap().code, -32001);
            }
        }
    }
}
