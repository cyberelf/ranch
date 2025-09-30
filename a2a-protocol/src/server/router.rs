//! A2A protocol router for HTTP servers

use crate::{
    server::{A2aHandler, handler::{BasicA2aHandler, HealthStatus}}, Message, MessageResponse, AgentCard, A2aResult, A2aError,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

/// Router for A2A protocol HTTP endpoints
#[derive(Clone)]
pub struct A2aRouter {
    handler: Arc<dyn A2aHandler>,
    router: Router,
}

impl A2aRouter {
    /// Create a new A2A router
    pub fn new<H>(handler: H) -> Self
    where
        H: A2aHandler + 'static,
    {
        let handler = Arc::new(handler);
        let router = Router::new()
            .route("/messages", post(handle_message))
            .route("/card", get(get_agent_card))
            .route("/health", get(health_check))
            .with_state(handler.clone());

        Self { handler, router }
    }

    /// Get the underlying Axum router
    pub fn into_router(self) -> Router {
        self.router
    }

    /// Get the handler used by this router
    pub fn handler(&self) -> &Arc<dyn A2aHandler> {
        &self.handler
    }
}

/// Handle incoming A2A messages
async fn handle_message(
    State(handler): State<Arc<dyn A2aHandler>>,
    Json(message): Json<Message>,
) -> Result<Json<MessageResponse>, A2aErrorResponse> {
    match handler.handle_message(message).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err(A2aErrorResponse::from_error(e)),
    }
}

/// Handle agent card requests
async fn get_agent_card(
    State(handler): State<Arc<dyn A2aHandler>>,
) -> Result<Json<AgentCard>, A2aErrorResponse> {
    match handler.get_agent_card().await {
        Ok(card) => Ok(Json(card)),
        Err(e) => Err(A2aErrorResponse::from_error(e)),
    }
}

/// Handle health check requests
async fn health_check(
    State(handler): State<Arc<dyn A2aHandler>>,
) -> Result<Json<HealthStatus>, A2aErrorResponse> {
    match handler.health_check().await {
        Ok(status) => Ok(Json(status)),
        Err(e) => Err(A2aErrorResponse::from_error(e)),
    }
}

/// Error response wrapper
#[derive(serde::Serialize)]
struct A2aErrorResponse {
    error: A2aErrorBody,
}

impl A2aErrorResponse {
    fn from_error(error: A2aError) -> Self {
        Self {
            error: A2aErrorBody {
                code: error.status_code().unwrap_or(500),
                message: error.to_string(),
                error_type: Some(error_type_string(&error)),
            },
        }
    }
}

/// Error response body
#[derive(serde::Serialize)]
struct A2aErrorBody {
    /// HTTP status code
    code: u16,

    /// Error message
    message: String,

    /// Error type classification
    error_type: Option<String>,
}

/// Get error type string for classification
fn error_type_string(error: &A2aError) -> String {
    match error {
        A2aError::Authentication(_) => "authentication_error".to_string(),
        A2aError::InvalidMessage(_) => "validation_error".to_string(),
        A2aError::AgentNotFound(_) => "not_found_error".to_string(),
        A2aError::ProtocolViolation(_) => "protocol_error".to_string(),
        A2aError::Network(_) => "network_error".to_string(),
        A2aError::Json(_) => "json_error".to_string(),
        A2aError::Timeout => "timeout_error".to_string(),
        A2aError::Configuration(_) => "configuration_error".to_string(),
        A2aError::Transport(_) => "transport_error".to_string(),
        A2aError::Internal(_) => "internal_error".to_string(),
        A2aError::Server(_) => "server_error".to_string(),
        A2aError::InvalidAgentId(_) => "validation_error".to_string(),
        A2aError::RateLimited(_) => "rate_limit_error".to_string(),
        A2aError::Validation(_) => "validation_error".to_string(),
    }
}

/// Convert A2A errors to Axum responses
impl axum::response::IntoResponse for A2aErrorResponse {
    fn into_response(self) -> axum::response::Response {
        let status = StatusCode::from_u16(self.error.code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (status, Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AgentId, MessageId};
    use url::Url;

    #[tokio::test]
    async fn test_router_creation() {
        let agent_id = crate::AgentId::new("test-agent".to_string()).unwrap();
        let agent_card = crate::AgentCard::new(agent_id, "Test Agent",
            Url::parse("https://example.com").unwrap());

        let handler = BasicA2aHandler::new(agent_card);
        let router = A2aRouter::new(handler);

        assert!(router.handler().get_agent_card().await.is_ok());
    }

    #[test]
    fn test_error_response_creation() {
        let error = A2aError::Authentication("Invalid token".to_string());
        let response = A2aErrorResponse::from_error(error);

        assert_eq!(response.error.code, 401);
        assert_eq!(response.error.message, "Authentication failed: Invalid token");
        assert_eq!(response.error.error_type, Some("authentication_error".to_string()));
    }

    #[test]
    fn test_error_type_classification() {
        let auth_error = A2aError::Authentication("Test".to_string());
        assert_eq!(error_type_string(&auth_error), "authentication_error");

        let validation_error = A2aError::Validation("Test".to_string());
        assert_eq!(error_type_string(&validation_error), "validation_error");

        let internal_error = A2aError::Internal("Test".to_string());
        assert_eq!(error_type_string(&internal_error), "internal_error");
    }
}