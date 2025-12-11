//! OpenAI remote agent implementation
//!
//! This module provides the `OpenAIAgent` struct which communicates with
//! OpenAI-compatible APIs and implements the multi-agent `Agent` trait.

use super::{AgentInfo, MultiAgentError, MultiAgentResult};
use a2a_protocol::prelude::*;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Configuration for OpenAI agent runtime behavior
#[derive(Debug, Clone, PartialEq)]
pub struct OpenAIAgentConfig {
    /// API key for authentication
    pub api_key: Option<String>,

    /// Maximum retry attempts for transient failures
    pub max_retries: u32,

    /// Request timeout in seconds
    pub timeout_seconds: u64,

    /// Model to use for requests
    pub model: String,

    /// Temperature for generation
    pub temperature: Option<f32>,

    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
}

impl Default for OpenAIAgentConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            max_retries: 3,
            timeout_seconds: 30,
            model: "gpt-3.5-turbo".to_string(),
            temperature: None,
            max_tokens: None,
        }
    }
}

/// OpenAI-compatible remote agent
///
/// This agent communicates with OpenAI-compatible REST APIs and provides
/// a simple interface for the multi-agent framework.
pub struct OpenAIAgent {
    /// Agent configuration
    config: OpenAIAgentConfig,

    /// HTTP client for API requests
    client: Client,

    /// Base URL for the API
    base_url: String,

    /// Agent information
    agent_info: AgentInfo,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize)]
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
    message: OpenAIResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponseMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

impl OpenAIAgent {
    /// Create a new OpenAI agent with the given endpoint and configuration
    pub fn with_config(base_url: String, config: OpenAIAgentConfig) -> Self {
        let agent_info = AgentInfo {
            id: format!("openai-{}", uuid::Uuid::new_v4()),
            name: "OpenAI Agent".to_string(),
            description: "OpenAI-compatible language model agent".to_string(),
            capabilities: vec![
                "text-generation".to_string(),
                "conversation".to_string(),
                "question-answering".to_string(),
            ],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("model".to_string(), config.model.clone());
                meta.insert("endpoint".to_string(), base_url.clone());
                if let Some(temp) = config.temperature {
                    meta.insert("temperature".to_string(), temp.to_string());
                }
                if let Some(max_tokens) = config.max_tokens {
                    meta.insert("max_tokens".to_string(), max_tokens.to_string());
                }
                meta
            },
        };

        Self {
            config,
            client: Client::new(),
            base_url,
            agent_info,
        }
    }

    /// Create a new OpenAI agent with default configuration
    pub fn new(base_url: String) -> Self {
        Self::with_config(base_url, OpenAIAgentConfig::default())
    }

    /// Send a message to the OpenAI API
    async fn send_message(&self, message: &Message) -> MultiAgentResult<OpenAIResponse> {
        // Extract text content from message
        let content = crate::adapters::extract_text(message).unwrap_or_default();

        let openai_messages = vec![OpenAIMessage {
            role: "user".to_string(),
            content,
        }];

        let request = OpenAIRequest {
            model: self.config.model.clone(),
            messages: openai_messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
        };

        let mut req_builder = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(Duration::from_secs(self.config.timeout_seconds));

        // Add API key if available
        if let Some(ref api_key) = self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req_builder.send().await?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(MultiAgentError::agent(format!(
                "OpenAI API error: {}",
                error_text
            )));
        }

        let openai_response: OpenAIResponse = response.json().await?;

        Ok(openai_response)
    }

    /// Perform health check on the OpenAI API
    async fn health_check_api(&self) -> bool {
        let mut req_builder = self
            .client
            .get(format!("{}/models", self.base_url))
            .timeout(Duration::from_secs(5));

        if let Some(ref api_key) = self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        req_builder
            .send()
            .await
            .map(|response| response.status().is_success())
            .unwrap_or(false)
    }
}

#[async_trait]
impl super::Agent for OpenAIAgent {
    async fn info(&self) -> A2aResult<AgentInfo> {
        Ok(self.agent_info.clone())
    }

    async fn process(&self, message: Message) -> A2aResult<Message> {
        let openai_response = self
            .send_message(&message)
            .await
            .map_err(|e| A2aError::Internal(format!("OpenAI agent error: {}", e)))?;

        let choice = openai_response
            .choices
            .first()
            .ok_or_else(|| A2aError::Internal("No choices in OpenAI response".to_string()))?;

        let response_content = choice.message.content.clone();

        Ok(Message::agent_text(response_content))
    }

    async fn health_check(&self) -> bool {
        self.health_check_api().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_agent_creation() {
        let base_url = "https://api.openai.com/v1".to_string();
        let agent = OpenAIAgent::new(base_url);

        assert_eq!(agent.config.model, "gpt-3.5-turbo");
        assert_eq!(agent.config.max_retries, 3);
        assert_eq!(agent.config.timeout_seconds, 30);
    }

    #[test]
    fn test_openai_agent_with_config() {
        let base_url = "https://api.openai.com/v1".to_string();
        let config = OpenAIAgentConfig {
            model: "gpt-4".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(1000),
            ..Default::default()
        };

        let agent = OpenAIAgent::with_config(base_url, config);

        assert_eq!(agent.config.model, "gpt-4");
        assert_eq!(agent.config.temperature, Some(0.7));
        assert_eq!(agent.config.max_tokens, Some(1000));
    }
}
