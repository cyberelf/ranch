//! Axum adapter for JSON-RPC 2.0 A2A server

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use axum::http::StatusCode;
use axum::body::Bytes;
use std::sync::Arc;
use crate::server::A2aHandler;
use super::dispatcher::dispatch_bytes;

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
        let router = Router::new()
            .route("/rpc", post(handle_rpc))
            .with_state(handler);
        Self { router }
    }

    pub fn into_router(self) -> Router { self.router }
}

async fn handle_rpc(
    State(handler): State<Arc<dyn A2aHandler>>,
    body: Bytes,
) -> Response {
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
            let error = crate::transport::json_rpc::JsonRpcError::internal_error()
                .with_data(serde_json::json!({ "message": e.to_string() }));
            let response = crate::transport::json_rpc::JsonRpcResponse::<serde_json::Value> {
                jsonrpc: "2.0".to_string(),
                id: serde_json::Value::Null,
                result: None,
                error: Some(error),
            };
            let bytes = serde_json::to_vec(&response).unwrap_or_default();
            (StatusCode::OK, [("Content-Type", "application/json")], bytes).into_response()
        }
    }
}
