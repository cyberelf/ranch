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
    /// Agent ID (used for registration in multi-agent systems)
    pub id: String,

    /// Agent name
    pub name: String,

    /// Agent description
    pub description: String,

    /// Agent capabilities
    pub capabilities: Vec<String>,

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
            id: format!("openai-{}", uuid::Uuid::new_v4()),
            name: "OpenAI Agent".to_string(),
            description: "OpenAI-compatible language model agent".to_string(),
            capabilities: vec![
                "text-generation".to_string(),
                "conversation".to_string(),
                "question-answering".to_string(),
            ],
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
#[allow(dead_code)] // Some fields are for future use or debugging
struct OpenAIResponse {
    id: String,
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Some fields are for future use or debugging
struct OpenAIChoice {
    index: u32,
    message: OpenAIResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Role field is for debugging but not currently used
struct OpenAIResponseMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Usage stats are for monitoring but not currently used
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

impl OpenAIAgent {
    /// Create a new OpenAI agent with the given endpoint and configuration
    pub fn with_config(base_url: String, config: OpenAIAgentConfig) -> Self {
        let agent_info = AgentInfo {
            id: config.id.clone(),
            name: config.name.clone(),
            description: config.description.clone(),
            skills: config.capabilities.iter().map(|c| AgentSkill {
                name: c.clone(),
                description: None,
                category: None,
                tags: vec![],
                examples: vec![],
            }).collect(),
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
    use crate::agent::Agent;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[test]
    fn test_openai_config_default() {
        let config = OpenAIAgentConfig::default();
        assert_eq!(config.name, "OpenAI Agent");
        assert_eq!(config.model, "gpt-3.5-turbo");
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.capabilities.contains(&"text-generation".to_string()));
        assert!(config.capabilities.contains(&"conversation".to_string()));
        assert_eq!(config.temperature, None);
        assert_eq!(config.max_tokens, None);
    }

    #[test]
    fn test_openai_config_custom() {
        let config = OpenAIAgentConfig {
            id: "custom-id".to_string(),
            name: "Custom Agent".to_string(),
            description: "Custom description".to_string(),
            capabilities: vec!["test".to_string()],
            api_key: Some("test-key".to_string()),
            max_retries: 5,
            timeout_seconds: 60,
            model: "gpt-4".to_string(),
            temperature: Some(0.8),
            max_tokens: Some(2000),
        };

        assert_eq!(config.id, "custom-id");
        assert_eq!(config.name, "Custom Agent");
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.temperature, Some(0.8));
        assert_eq!(config.max_tokens, Some(2000));
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.timeout_seconds, 60);
    }

    #[test]
    fn test_openai_config_clone() {
        let config = OpenAIAgentConfig::default();
        let cloned = config.clone();
        assert_eq!(config, cloned);
    }

    #[test]
    fn test_openai_config_debug() {
        let config = OpenAIAgentConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("OpenAIAgentConfig"));
    }

    #[test]
    fn test_openai_agent_creation() {
        let base_url = "https://api.openai.com/v1".to_string();
        let agent = OpenAIAgent::new(base_url.clone());

        assert_eq!(agent.config.model, "gpt-3.5-turbo");
        assert_eq!(agent.config.max_retries, 3);
        assert_eq!(agent.config.timeout_seconds, 30);
        assert_eq!(agent.base_url, base_url);
    }

    #[test]
    fn test_openai_agent_with_config() {
        let base_url = "https://api.openai.com/v1".to_string();
        let config = OpenAIAgentConfig {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            description: "Test description".to_string(),
            capabilities: vec!["test".to_string()],
            api_key: Some("sk-test".to_string()),
            max_retries: 5,
            timeout_seconds: 45,
            model: "gpt-4".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(1000),
        };

        let agent = OpenAIAgent::with_config(base_url.clone(), config.clone());

        assert_eq!(agent.config.model, "gpt-4");
        assert_eq!(agent.config.temperature, Some(0.7));
        assert_eq!(agent.config.max_tokens, Some(1000));
        assert_eq!(agent.config.id, "test-agent");
        assert_eq!(agent.config.name, "Test Agent");
        assert_eq!(agent.base_url, base_url);
    }

    #[tokio::test]
    async fn test_openai_agent_info() {
        let base_url = "https://api.openai.com/v1".to_string();
        let config = OpenAIAgentConfig {
            id: "info-test".to_string(),
            name: "Info Test Agent".to_string(),
            description: "Testing info method".to_string(),
            capabilities: vec!["cap1".to_string(), "cap2".to_string()],
            ..Default::default()
        };

        let agent = OpenAIAgent::with_config(base_url, config);
        let info = agent.info().await.unwrap();

        assert_eq!(info.id, "info-test");
        assert_eq!(info.name, "Info Test Agent");
        assert_eq!(info.description, "Testing info method");
        assert_eq!(info.skills.len(), 2);
        assert!(info.skills.iter().any(|s| s.name == "cap1"));
    }

    #[test]
    fn test_openai_agent_metadata() {
        let base_url = "https://api.openai.com/v1".to_string();
        let config = OpenAIAgentConfig {
            model: "gpt-4".to_string(),
            temperature: Some(0.9),
            max_tokens: Some(1500),
            ..Default::default()
        };

        let agent = OpenAIAgent::with_config(base_url.clone(), config);

        assert_eq!(
            agent.agent_info.metadata.get("model"),
            Some(&"gpt-4".to_string())
        );
        assert_eq!(
            agent.agent_info.metadata.get("endpoint"),
            Some(&base_url)
        );
        assert_eq!(
            agent.agent_info.metadata.get("temperature"),
            Some(&"0.9".to_string())
        );
        assert_eq!(
            agent.agent_info.metadata.get("max_tokens"),
            Some(&"1500".to_string())
        );
    }

    #[test]
    fn test_openai_request_serialization() {
        let request = OpenAIRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            temperature: Some(0.7),
            max_tokens: Some(100),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gpt-3.5-turbo"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_openai_message_serialization() {
        let message = OpenAIMessage {
            role: "user".to_string(),
            content: "Test message".to_string(),
        };

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("user"));
        assert!(json.contains("Test message"));
    }

        #[tokio::test]
    async fn test_openai_agent_info_returns_config() {
        let base_url = "https://test.openai.com/v1".to_string();
        let config = OpenAIAgentConfig {
            id: "test-openai".to_string(),
            name: "Test OpenAI Agent".to_string(),
            description: "Testing OpenAI agent info".to_string(),
            capabilities: vec!["text-gen".to_string(), "qa".to_string()],
            ..Default::default()
        };

        let agent = OpenAIAgent::with_config(base_url, config);
        let info = agent.info().await.unwrap();

        assert_eq!(info.id, "test-openai");
        assert_eq!(info.name, "Test OpenAI Agent");
        assert_eq!(info.description, "Testing OpenAI agent info");
        assert_eq!(info.skills.len(), 2);
        assert!(info.skills.iter().any(|s| s.name == "text-gen"));
        assert!(info.skills.iter().any(|s| s.name == "qa"));
    }

    #[tokio::test]
    async fn test_openai_agent_default_health_check() {
        let base_url = "https://test.openai.com/v1".to_string();
        let agent = OpenAIAgent::new(base_url);

        // Health check with unreachable endpoint returns false
        let healthy = agent.health_check().await;
        assert!(!healthy);
    }

    #[tokio::test]
    async fn test_openai_agent_with_custom_config() {
        let base_url = "https://test.openai.com/v1".to_string();
        let config = OpenAIAgentConfig {
            id: "custom-agent".to_string(),
            name: "Custom Agent".to_string(),
            description: "Custom test agent".to_string(),
            capabilities: vec!["reasoning".to_string()],
            model: "gpt-4".to_string(),
            ..Default::default()
        };

        let agent = OpenAIAgent::with_config(base_url, config.clone());
        
        // Verify config is stored
        let info = agent.info().await.unwrap();
        assert_eq!(info.id, config.id);
        assert_eq!(info.name, config.name);
        assert_eq!(info.description, config.description);
    }

    #[tokio::test]
    async fn test_openai_agent_timeout_config() {
        let base_url = "https://test.openai.com/v1".to_string();
        let config = OpenAIAgentConfig {
            timeout_seconds: 5,
            max_retries: 2,
            ..Default::default()
        };

        let agent = OpenAIAgent::with_config(base_url, config);
        
        // Just verify the agent can be created with custom timeout
        let _ = agent.info().await;
    }

    /// Test OpenAI agent process with successful response
    #[tokio::test]
    async fn test_openai_process_success() {
        let mock_server = MockServer::start().await;
        
        // Mock OpenAI chat completion endpoint
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "chatcmpl-123",
                "object": "chat.completion",
                "created": 1234567890,
                "model": "gpt-4",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "OpenAI response"
                    },
                    "finish_reason": "stop"
                }]
            })))
            .mount(&mock_server)
            .await;
        
        let agent = OpenAIAgent::new(mock_server.uri());
        let message = Message::user_text("Test query");
        let response = agent.process(message).await.unwrap();
        
        assert_eq!(response.role, MessageRole::Agent);
        let text = crate::adapters::extract_text(&response);
        assert_eq!(text, Some("OpenAI response".to_string()));
    }

    /// Test OpenAI agent health check with successful API
    #[tokio::test]
    async fn test_openai_health_check_success() {
        let mock_server = MockServer::start().await;
        
        // Mock models endpoint for health check
        Mock::given(method("GET"))
            .and(path("/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "object": "list",
                "data": []
            })))
            .mount(&mock_server)
            .await;
        
        let agent = OpenAIAgent::new(mock_server.uri());
        let healthy = agent.health_check().await;
        
        assert!(healthy);
    }

    /// Test OpenAI agent health check with failed API
    #[tokio::test]
    async fn test_openai_health_check_failure() {
        // Use unreachable endpoint
        let agent = OpenAIAgent::new("http://localhost:99999".to_string());
        let healthy = agent.health_check().await;
        
        assert!(!healthy);
    }

    /// Test OpenAI agent with error handling
    #[tokio::test]
    async fn test_openai_error_handling() {
        let mock_server = MockServer::start().await;
        
        // Mock with error response
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "error": {
                    "message": "Invalid request",
                    "type": "invalid_request_error",
                    "code": "invalid_api_key"
                }
            })))
            .mount(&mock_server)
            .await;
        
        let agent = OpenAIAgent::new(mock_server.uri());
        let message = Message::user_text("Test error");
        let result = agent.process(message).await;
        
        // Should get an error
        assert!(result.is_err());
    }

    /// Test send_message private function with API key
    #[tokio::test]
    async fn test_send_message_with_api_key() {
        let mock_server = MockServer::start().await;
        
        // Mock OpenAI endpoint
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "chatcmpl-test",
                "object": "chat.completion",
                "created": 1234567890,
                "model": "gpt-4",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Response with API key"
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": 10,
                    "completion_tokens": 20,
                    "total_tokens": 30
                }
            })))
            .mount(&mock_server)
            .await;
        
        let config = OpenAIAgentConfig {
            api_key: Some("sk-test-key-123".to_string()),
            ..Default::default()
        };
        
        let agent = OpenAIAgent::with_config(mock_server.uri(), config);
        let message = Message::user_text("Test with key");
        
        let result = agent.send_message(&message).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.id, "chatcmpl-test");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.content, "Response with API key");
    }

    /// Test send_message with custom temperature and max_tokens
    #[tokio::test]
    async fn test_send_message_with_custom_params() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "chatcmpl-custom",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Custom params response"
                    },
                    "finish_reason": "length"
                }]
            })))
            .mount(&mock_server)
            .await;
        
        let config = OpenAIAgentConfig {
            temperature: Some(0.9),
            max_tokens: Some(500),
            model: "gpt-4".to_string(),
            ..Default::default()
        };
        
        let agent = OpenAIAgent::with_config(mock_server.uri(), config);
        let message = Message::user_text("Test custom params");
        
        let result = agent.send_message(&message).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.choices[0].message.content, "Custom params response");
    }

    /// Test send_message with 500 server error
    #[tokio::test]
    async fn test_send_message_server_error() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;
        
        let agent = OpenAIAgent::new(mock_server.uri());
        let message = Message::user_text("Test error");
        
        let result = agent.send_message(&message).await;
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("OpenAI API error") || error_msg.contains("Internal Server Error"));
    }

    /// Test send_message with empty response choices
    #[tokio::test]
    async fn test_send_message_empty_choices() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "chatcmpl-empty",
                "choices": []
            })))
            .mount(&mock_server)
            .await;
        
        let agent = OpenAIAgent::new(mock_server.uri());
        let message = Message::user_text("Test");
        
        // Process will handle empty choices and return error
        let result = agent.process(message).await;
        assert!(result.is_err());
    }

    /// Test health_check_api with successful response
    #[tokio::test]
    async fn test_health_check_api_success() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "object": "list",
                "data": [{"id": "gpt-3.5-turbo"}, {"id": "gpt-4"}]
            })))
            .mount(&mock_server)
            .await;
        
        let agent = OpenAIAgent::new(mock_server.uri());
        let healthy = agent.health_check_api().await;
        
        assert!(healthy);
    }

    /// Test health_check_api with authentication
    #[tokio::test]
    async fn test_health_check_api_with_auth() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "object": "list",
                "data": []
            })))
            .mount(&mock_server)
            .await;
        
        let config = OpenAIAgentConfig {
            api_key: Some("sk-auth-test".to_string()),
            ..Default::default()
        };
        
        let agent = OpenAIAgent::with_config(mock_server.uri(), config);
        let healthy = agent.health_check_api().await;
        
        assert!(healthy);
    }

    /// Test health_check_api with 401 unauthorized
    #[tokio::test]
    async fn test_health_check_api_unauthorized() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/models"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;
        
        let agent = OpenAIAgent::new(mock_server.uri());
        let healthy = agent.health_check_api().await;
        
        assert!(!healthy);
    }

    /// Test health_check_api with network timeout
    #[tokio::test]
    async fn test_health_check_api_timeout() {
        // Use an unreachable endpoint
        let agent = OpenAIAgent::new("http://10.255.255.1:12345".to_string());
        let healthy = agent.health_check_api().await;
        
        assert!(!healthy);
    }

    /// Test send_message with multipart message content
    #[tokio::test]
    async fn test_send_message_complex_message() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "chatcmpl-complex",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Complex message response"
                    },
                    "finish_reason": "stop"
                }]
            })))
            .mount(&mock_server)
            .await;
        
        let agent = OpenAIAgent::new(mock_server.uri());
        
        // Create a message with multiple text parts using add_text
        let message = Message::user_text("First part")
            .add_text("Second part");
        
        let result = agent.send_message(&message).await;
        assert!(result.is_ok());
    }

    /// Test OpenAI agent with custom timeout configuration
    #[tokio::test]
    async fn test_send_message_with_custom_timeout() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "chatcmpl-timeout",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Timeout test response"
                    },
                    "finish_reason": "stop"
                }]
            })))
            .mount(&mock_server)
            .await;
        
        let config = OpenAIAgentConfig {
            timeout_seconds: 60,
            ..Default::default()
        };
        
        let agent = OpenAIAgent::with_config(mock_server.uri(), config);
        let message = Message::user_text("Test timeout");
        
        let result = agent.send_message(&message).await;
        assert!(result.is_ok());
    }

}
