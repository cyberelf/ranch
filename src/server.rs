use crate::agent::AgentMessage;
use crate::team::Team;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(Debug, Deserialize)]
pub struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIChatMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAIChatResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Serialize)]
struct OpenAIChoice {
    index: u32,
    message: OpenAIResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct OpenAIResponseMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct A2AChatRequest {
    id: String,
    messages: Vec<A2AMessage>,
    context: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct A2AMessage {
    role: String,
    content: String,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
struct A2AChatResponse {
    id: String,
    response: A2AResponseMessage,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
struct A2AResponseMessage {
    content: String,
    role: String,
    finish_reason: Option<String>,
    usage: Option<A2AUsage>,
}

#[derive(Debug, Serialize)]
struct A2AUsage {
    input_tokens: u32,
    output_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: ErrorDetail,
}

#[derive(Debug, Serialize)]
struct ErrorDetail {
    message: String,
    r#type: String,
    code: String,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    agents: Vec<(String, bool)>,
}

pub struct TeamServer {
    team: Arc<Team>,
}

impl TeamServer {
    pub fn new(team: Arc<Team>) -> Self {
        Self { team }
    }

    pub async fn start(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let app = Router::new()
            .route("/v1/chat/completions", post(openai_chat_handler))
            .route("/v1/chat", post(a2a_chat_handler))
            .route("/health", get(health_handler))
            .with_state(self.team.clone())
            .layer(CorsLayer::permissive());

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        
        tracing::info!("Starting server on {}", addr);
        
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        
        Ok(())
    }
}

async fn openai_chat_handler(
    State(team): State<Arc<Team>>,
    Json(request): Json<OpenAIChatRequest>,
) -> Result<Json<OpenAIChatResponse>, (StatusCode, Json<ErrorResponse>)> {
    let messages: Vec<AgentMessage> = request
        .messages
        .into_iter()
        .map(|msg| AgentMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role: msg.role,
            content: msg.content,
            metadata: HashMap::new(),
        })
        .collect();

    match team.process_messages(messages).await {
        Ok(response) => {
            let openai_response = OpenAIChatResponse {
                id: response.id,
                object: "chat.completion".to_string(),
                created: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                model: request.model,
                choices: vec![OpenAIChoice {
                    index: 0,
                    message: OpenAIResponseMessage {
                        role: response.role,
                        content: response.content,
                    },
                    finish_reason: response.finish_reason,
                }],
                usage: OpenAIUsage {
                    prompt_tokens: response.usage.as_ref().map(|u| u.prompt_tokens).unwrap_or(0),
                    completion_tokens: response.usage.as_ref().map(|u| u.completion_tokens).unwrap_or(0),
                    total_tokens: response.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0),
                },
            };
            
            Ok(Json(openai_response))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: ErrorDetail {
                    message: e.to_string(),
                    r#type: "team_error".to_string(),
                    code: "internal_error".to_string(),
                },
            };
            
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

async fn a2a_chat_handler(
    State(team): State<Arc<Team>>,
    Json(request): Json<A2AChatRequest>,
) -> Result<Json<A2AChatResponse>, (StatusCode, Json<ErrorResponse>)> {
    let messages: Vec<AgentMessage> = request
        .messages
        .into_iter()
        .map(|msg| AgentMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role: msg.role,
            content: msg.content,
            metadata: msg.metadata,
        })
        .collect();

    match team.process_messages(messages).await {
        Ok(response) => {
            let a2a_response = A2AChatResponse {
                id: response.id,
                response: A2AResponseMessage {
                    content: response.content,
                    role: response.role,
                    finish_reason: response.finish_reason,
                    usage: response.usage.map(|u| A2AUsage {
                        input_tokens: u.prompt_tokens,
                        output_tokens: u.completion_tokens,
                        total_tokens: u.total_tokens,
                    }),
                },
                metadata: response.metadata,
            };
            
            Ok(Json(a2a_response))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: ErrorDetail {
                    message: e.to_string(),
                    r#type: "team_error".to_string(),
                    code: "internal_error".to_string(),
                },
            };
            
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

async fn health_handler(
    State(team): State<Arc<Team>>,
) -> (StatusCode, Json<HealthResponse>) {
    let health_results = team.health_check().await;
    let all_healthy = health_results.iter().all(|(_, healthy)| *healthy);
    
    let status = if all_healthy {
        "healthy".to_string()
    } else {
        "unhealthy".to_string()
    };
    
    let response = HealthResponse {
        status,
        agents: health_results,
    };
    
    let status_code = if all_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    
    (status_code, Json(response))
}