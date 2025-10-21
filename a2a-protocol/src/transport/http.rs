//! Internal HTTP client implementation
//! 
//! This module provides low-level HTTP communication functionality used by
//! transport implementations (e.g., JsonRpcTransport). It is not a public
//! A2A transport itself.
//!
//! **Note:** This is internal infrastructure. For A2A protocol communication,
//! use `JsonRpcTransport` (JSON-RPC 2.0 over HTTP).

use crate::{
    A2aResult, A2aError,
    transport::{TransportConfig, RequestInfo},
};
use reqwest::{Client, StatusCode};
use std::time::Duration;

/// Internal HTTP client for transport implementations
/// 
/// This is not a public A2A transport. It provides HTTP communication
/// primitives used by other transports like `JsonRpcTransport`.
/// 
/// **For A2A communication, use `JsonRpcTransport` instead.**
#[derive(Debug)]
pub(crate) struct HttpClient {
    client: Client,
    pub(crate) config: TransportConfig,
    pub(crate) base_url: String,
}

impl HttpClient {
    /// Create a new HTTP client
    pub(crate) fn new<S: Into<String>>(base_url: S) -> A2aResult<Self> {
        Self::with_config(base_url, TransportConfig::default())
    }

    /// Create a new HTTP client with custom configuration
    pub(crate) fn with_config<S: Into<String>>(base_url: S, config: TransportConfig) -> A2aResult<Self> {
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
    pub(crate) async fn send_request_with_retry(
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

// No Transport implementation - HttpClient is internal only
// Use JsonRpcTransport for A2A protocol communication

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_creation() {
        let client = HttpClient::new("https://example.com").unwrap();
        assert_eq!(client.base_url, "https://example.com/");
    }

    #[test]
    fn test_http_client_with_config() {
        let config = TransportConfig {
            timeout_seconds: 60,
            max_retries: 5,
            enable_compression: false,
            extra: std::collections::HashMap::new(),
        };

        let client = HttpClient::with_config("https://example.com", config).unwrap();
        assert_eq!(client.config.timeout_seconds, 60);
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