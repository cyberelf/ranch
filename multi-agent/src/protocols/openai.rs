use crate::agent::{AgentConfig, AgentMessage, AgentResponse, Usage};
use crate::protocol::{Protocol, ProtocolError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    id: String,
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    index: u32,
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

pub struct OpenAIProtocol {
    client: Client,
    api_key: Option<String>,
}

impl OpenAIProtocol {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }
}

#[async_trait]
impl Protocol for OpenAIProtocol {
    async fn send_message(
        &self,
        config: &AgentConfig,
        messages: Vec<AgentMessage>,
    ) -> Result<AgentResponse, ProtocolError> {
        let openai_messages: Vec<OpenAIMessage> = messages
            .into_iter()
            .map(|msg| OpenAIMessage {
                role: msg.role,
                content: msg.content,
            })
            .collect();

        let request = OpenAIRequest {
            model: config.metadata.get("model").unwrap_or(&"gpt-3.5-turbo".to_string()).clone(),
            messages: openai_messages,
            temperature: config.metadata.get("temperature").and_then(|v| v.parse().ok()),
            max_tokens: config.metadata.get("max_tokens").and_then(|v| v.parse().ok()),
        };

        let mut req_builder = self.client
            .post(&format!("{}/chat/completions", config.endpoint))
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(Duration::from_secs(config.timeout_seconds));

        if let Some(ref api_key) = self.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
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

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| ProtocolError::Serialization(e.to_string()))?;

        let choice = openai_response.choices.first().ok_or_else(|| {
            ProtocolError::Protocol("No choices in response".to_string())
        })?;

        Ok(AgentResponse {
            id: openai_response.id,
            content: choice.message.content.clone(),
            role: choice.message.role.clone(),
            finish_reason: choice.finish_reason.clone(),
            usage: openai_response.usage.map(|usage| Usage {
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
                total_tokens: usage.total_tokens,
            }),
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn health_check(&self, config: &AgentConfig) -> Result<bool, ProtocolError> {
        let mut req_builder = self.client
            .get(&format!("{}/models", config.endpoint))
            .timeout(Duration::from_secs(5));

        if let Some(ref api_key) = self.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| ProtocolError::Network(e.to_string()))?;

        Ok(response.status().is_success())
    }
}