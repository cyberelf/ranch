//! Axum adapter for JSON-RPC 2.0 A2A server

use super::dispatcher::dispatch_bytes;
use crate::server::A2aHandler;
use axum::body::Bytes;
use axum::http::StatusCode;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use std::sync::Arc;

#[cfg(feature = "streaming")]
use crate::server::sse::SseResponse;
#[cfg(feature = "streaming")]
use serde_json::Value;

#[derive(Clone)]
pub struct JsonRpcRouter {
    router: Router,
}

impl JsonRpcRouter {
    pub fn new<H>(handler: H) -> Self
    where
        H: A2aHandler + 'static,
    {
        let handler = Arc::new(handler);

        #[cfg(feature = "streaming")]
        let router = Router::new()
            .route("/rpc", post(handle_rpc))
            .route("/stream", post(handle_stream))
            .with_state(handler);

        #[cfg(not(feature = "streaming"))]
        let router = Router::new()
            .route("/rpc", post(handle_rpc))
            .with_state(handler);

        Self { router }
    }

    pub fn into_router(self) -> Router {
        self.router
    }
}

#[cfg(feature = "streaming")]
async fn handle_stream(State(handler): State<Arc<dyn A2aHandler>>, body: Bytes) -> Response {
    // Parse JSON-RPC request
    let value: Result<Value, _> = serde_json::from_slice(&body);
    let value = match value {
        Ok(v) => v,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)).into_response();
        }
    };

    let method = value.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = value.get("params").cloned().unwrap_or(Value::Null);

    // Route streaming methods
    let stream_result = match method {
        "message/stream" => {
            // Extract message from params
            let message_value = params.get("message").cloned().unwrap_or(Value::Null);
            match serde_json::from_value(message_value) {
                Ok(message) => handler.rpc_message_stream(message).await,
                Err(e) => {
                    return (StatusCode::BAD_REQUEST, format!("Invalid message: {}", e))
                        .into_response();
                }
            }
        }
        "task/resubscribe" => match serde_json::from_value(params) {
            Ok(request) => handler.rpc_task_resubscribe(request).await,
            Err(e) => {
                return (StatusCode::BAD_REQUEST, format!("Invalid request: {}", e))
                    .into_response();
            }
        },
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Unknown streaming method: {}", method),
            )
                .into_response();
        }
    };

    match stream_result {
        Ok(stream) => SseResponse::new(stream).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response(),
    }
}

async fn handle_rpc(State(handler): State<Arc<dyn A2aHandler>>, body: Bytes) -> Response {
    match dispatch_bytes(handler.as_ref(), &body).await {
        Ok(resp) => {
            if resp.is_empty() {
                // Notification: 204 No Content
                StatusCode::NO_CONTENT.into_response()
            } else {
                (StatusCode::OK, [("Content-Type", "application/json")], resp).into_response()
            }
        }
        Err(e) => {
            // Map server error into JSON-RPC error envelope with null id
            let error = crate::core::JsonRpcError::internal_error()
                .with_data(serde_json::json!({ "message": e.to_string() }));
            let response = crate::core::JsonRpcResponse::<serde_json::Value> {
                jsonrpc: "2.0".to_string(),
                id: serde_json::Value::Null,
                result: None,
                error: Some(error),
            };
            let bytes = serde_json::to_vec(&response).unwrap_or_default();
            (
                StatusCode::OK,
                [("Content-Type", "application/json")],
                bytes,
            )
                .into_response()
        }
    }
}
