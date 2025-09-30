use crate::agent::{AgentConfig, AgentMessage, AgentResponse, Usage};
use crate::protocol::{Protocol, ProtocolError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Serialize)]
struct A2ARequest {
    id: String,
    messages: Vec<A2AMessage>,
    context: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
struct A2AMessage {
    role: String,
    content: String,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct A2AResponse {
    id: String,
    response: A2AResponseMessage,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct A2AResponseMessage {
    content: String,
    role: String,
    finish_reason: Option<String>,
    usage: Option<A2AUsage>,
}

#[derive(Debug, Deserialize)]
struct A2AUsage {
    input_tokens: u32,
    output_tokens: u32,
    total_tokens: u32,
}

pub struct A2AProtocol {
    client: Client,
    auth_token: Option<String>,
}

impl A2AProtocol {
    pub fn new(auth_token: Option<String>) -> Self {
        Self {
            client: Client::new(),
            auth_token,
        }
    }
}

#[async_trait]
impl Protocol for A2AProtocol {
    async fn send_message(
        &self,
        config: &AgentConfig,
        messages: Vec<AgentMessage>,
    ) -> Result<AgentResponse, ProtocolError> {
        let a2a_messages: Vec<A2AMessage> = messages
            .into_iter()
            .map(|msg| A2AMessage {
                role: msg.role,
                content: msg.content,
                metadata: msg.metadata,
            })
            .collect();

        let request = A2ARequest {
            id: uuid::Uuid::new_v4().to_string(),
            messages: a2a_messages,
            context: config.metadata.clone(),
        };

        let mut req_builder = self.client
            .post(&format!("{}/v1/chat", config.endpoint))
            .header("Content-Type", "application/json")
            .header("User-Agent", "multi-agent-runtime/0.1.0")
            .json(&request)
            .timeout(Duration::from_secs(config.timeout_seconds));

        if let Some(ref token) = self.auth_token {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", token));
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| ProtocolError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ProtocolError::Protocol(error_text));
        }

        let a2a_response: A2AResponse = response
            .json()
            .await
            .map_err(|e| ProtocolError::Serialization(e.to_string()))?;

        Ok(AgentResponse {
            id: a2a_response.id,
            content: a2a_response.response.content,
            role: a2a_response.response.role,
            finish_reason: a2a_response.response.finish_reason,
            usage: a2a_response.response.usage.map(|usage| Usage {
                prompt_tokens: usage.input_tokens,
                completion_tokens: usage.output_tokens,
                total_tokens: usage.total_tokens,
            }),
            metadata: a2a_response.metadata,
        })
    }

    async fn health_check(&self, config: &AgentConfig) -> Result<bool, ProtocolError> {
        let mut req_builder = self.client
            .get(&format!("{}/v1/health", config.endpoint))
            .header("User-Agent", "multi-agent-runtime/0.1.0")
            .timeout(Duration::from_secs(5));

        if let Some(ref token) = self.auth_token {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", token));
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| ProtocolError::Network(e.to_string()))?;

        Ok(response.status().is_success())
    }
}