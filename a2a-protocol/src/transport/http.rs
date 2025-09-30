//! HTTP transport implementation

use async_trait::async_trait;
use crate::{
    Message, MessageResponse, AgentCard, A2aResult, A2aError,
    transport::{Transport, TransportConfig, RequestInfo},
};
use reqwest::{Client, StatusCode};
use std::time::Duration;

/// HTTP transport for A2A protocol
#[derive(Debug)]
pub struct HttpTransport {
    client: Client,
    config: TransportConfig,
    base_url: String,
}

impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new<S: Into<String>>(base_url: S) -> A2aResult<Self> {
        Self::with_config(base_url, TransportConfig::default())
    }

    /// Create a new HTTP transport with custom configuration
    pub fn with_config<S: Into<String>>(base_url: S, config: TransportConfig) -> A2aResult<Self> {
        let base_url = base_url.into();

        // Ensure base_url ends with a slash
        let base_url = if base_url.ends_with('/') {
            base_url
        } else {
            format!("{}/", base_url)
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| A2aError::Configuration(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            config,
            base_url,
        })
    }

    /// Get the base URL for this transport
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Send an HTTP request with retry logic
    pub async fn send_request_with_retry(
        &self,
        request_info: RequestInfo,
        payload: Option<serde_json::Value>,
    ) -> A2aResult<reqwest::Response> {
        let mut last_error = None;
        let mut retry_count = 0;

        while retry_count <= self.config.max_retries {
            let result = self.send_single_request(&request_info, payload.clone()).await;

            match result {
                Ok(response) => {
                    // Check for rate limiting
                    if response.status() == StatusCode::TOO_MANY_REQUESTS {
                        if let Some(retry_after) = response.headers().get("retry-after") {
                            if let Ok(retry_seconds) = retry_after.to_str() {
                                if let Ok(seconds) = retry_seconds.parse::<u64>() {
                                    tokio::time::sleep(Duration::from_secs(seconds)).await;
                                    retry_count += 1;
                                    continue;
                                }
                            }
                        }

                        // Default exponential backoff
                        let backoff = Duration::from_secs(2u64.pow(retry_count as u32));
                        tokio::time::sleep(backoff).await;
                        retry_count += 1;
                        continue;
                    }
                    return Ok(response);
                }
                Err(e) => {
                    let is_retryable = e.is_retryable();
                    last_error = Some(e);

                    // Retry only on retryable errors
                    if !is_retryable || retry_count >= self.config.max_retries {
                        break;
                    }

                    // Exponential backoff
                    let backoff = Duration::from_secs(2u64.pow(retry_count as u32));
                    tokio::time::sleep(backoff).await;
                    retry_count += 1;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            A2aError::Internal("Unknown error occurred".to_string())
        }))
    }

    /// Send a single HTTP request
    async fn send_single_request(
        &self,
        request_info: &RequestInfo,
        payload: Option<serde_json::Value>,
    ) -> A2aResult<reqwest::Response> {
        let url = format!("{}{}", self.base_url, request_info.endpoint);
        let mut request = self.client.request(
            reqwest::Method::POST,
            &url,
        );

        // Add headers
        for (key, value) in &request_info.headers {
            request = request.header(key, value);
        }

        // Add default headers
        request = request
            .header("Content-Type", "application/json")
            .header("User-Agent", "a2a-protocol-rust/0.1.0");

        // Add payload if provided
        if let Some(payload) = payload {
            request = request.json(&payload);
        }

        let response = request.send().await.map_err(A2aError::Network)?;

        // Handle HTTP errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());

            return Err(match status {
                StatusCode::UNAUTHORIZED => A2aError::Authentication(error_text),
                StatusCode::NOT_FOUND => A2aError::AgentNotFound(crate::AgentId::from(url)),
                StatusCode::TOO_MANY_REQUESTS => A2aError::RateLimited(Duration::from_secs(60)),
                StatusCode::BAD_REQUEST => A2aError::Validation(error_text),
                _ => A2aError::Server(format!("HTTP {}: {}", status.as_u16(), error_text)),
            });
        }

        Ok(response)
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn send_message(&self, message: Message) -> A2aResult<MessageResponse> {
        let request_info = RequestInfo::new("messages")
            .with_method("POST")
            .with_timeout_ms(self.config.timeout_seconds * 1000);

        let payload = serde_json::to_value(message)
            .map_err(|e| A2aError::Json(e))?;

        let response = self.send_request_with_retry(request_info, Some(payload)).await?;

        let response_data: MessageResponse = response.json().await
            .map_err(|e| A2aError::Network(e))?;

        Ok(response_data)
    }

    async fn get_agent_card(&self, agent_id: &crate::AgentId) -> A2aResult<AgentCard> {
        let request_info = RequestInfo::new("card")
            .with_method("GET")
            .with_timeout_ms(self.config.timeout_seconds * 1000);

        let response = self.send_request_with_retry(request_info, None).await?;

        let agent_card: AgentCard = response.json().await
            .map_err(|e| A2aError::Network(e))?;

        Ok(agent_card)
    }

    async fn is_available(&self) -> bool {
        let request_info = RequestInfo::new("health")
            .with_method("GET")
            .with_timeout_ms(5000); // Short timeout for health check

        // Use a single request for health check (no retries)
        match self.send_single_request(&request_info, None).await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    fn config(&self) -> &TransportConfig {
        &self.config
    }

    fn transport_type(&self) -> &'static str {
        "http"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MessageId, AgentId};

    #[test]
    fn test_http_transport_creation() {
        let transport = HttpTransport::new("https://example.com").unwrap();
        assert_eq!(transport.base_url(), "https://example.com/");
    }

    #[test]
    fn test_http_transport_with_config() {
        let config = TransportConfig {
            timeout_seconds: 60,
            max_retries: 5,
            enable_compression: false,
            extra: std::collections::HashMap::new(),
        };

        let transport = HttpTransport::with_config("https://example.com", config).unwrap();
        assert_eq!(transport.config().timeout_seconds, 60);
    }

    #[tokio::test]
    async fn test_request_info_creation() {
        let info = RequestInfo::new("test")
            .with_method("POST")
            .with_header("Authorization", "Bearer token")
            .with_timeout_ms(5000);

        assert_eq!(info.method, Some("POST".to_string()));
        assert_eq!(info.headers.get("Authorization"), Some(&"Bearer token".to_string()));
        assert_eq!(info.timeout_ms, 5000);
    }
}