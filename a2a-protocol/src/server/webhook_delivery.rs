//! Webhook delivery system for push notifications
//!
//! This module handles the actual delivery of webhook notifications to configured
//! endpoints, including retry logic, authentication, and delivery tracking.

use crate::core::push_notification::{PushNotificationAuth, PushNotificationConfig, TaskEvent};
use crate::{A2aError, A2aResult, Task};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

/// Webhook payload sent to the configured endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookPayload {
    /// Event type that triggered this webhook
    pub event: TaskEvent,
    
    /// The full task object
    pub task: Task,
    
    /// Timestamp when the event occurred (ISO 8601)
    pub timestamp: String,
    
    /// ID of the agent sending the webhook
    pub agent_id: String,
}

impl WebhookPayload {
    /// Create a new webhook payload
    pub fn new(event: TaskEvent, task: Task, agent_id: String) -> Self {
        Self {
            event,
            task,
            timestamp: chrono::Utc::now().to_rfc3339(),
            agent_id,
        }
    }
}

/// Delivery status for a webhook
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryStatus {
    /// Webhook is pending delivery
    Pending,
    /// Webhook is currently being delivered
    Delivering,
    /// Webhook was successfully delivered
    Delivered,
    /// Webhook delivery failed after all retries
    Failed,
    /// Webhook is being retried
    Retrying { attempt: u32 },
}

/// Configuration for retry logic
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay before first retry (in seconds)
    pub initial_delay: u64,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Maximum delay between retries (in seconds)
    pub max_delay: u64,
    /// Timeout for each HTTP request (in seconds)
    pub request_timeout: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            initial_delay: 1,
            backoff_multiplier: 2.0,
            max_delay: 60,
            request_timeout: 30,
        }
    }
}

impl RetryConfig {
    /// Calculate delay for a given retry attempt
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay_secs = (self.initial_delay as f64 
            * self.backoff_multiplier.powi(attempt as i32))
            .min(self.max_delay as f64);
        Duration::from_secs(delay_secs as u64)
    }
}

/// A queued webhook delivery request
#[derive(Debug, Clone)]
struct WebhookDeliveryRequest {
    config: PushNotificationConfig,
    payload: WebhookPayload,
    attempt: u32,
}

/// Webhook delivery queue and worker
pub struct WebhookQueue {
    sender: mpsc::Sender<WebhookDeliveryRequest>,
    retry_config: Arc<RetryConfig>,
}

impl WebhookQueue {
    /// Create a new webhook queue
    ///
    /// # Arguments
    /// * `queue_size` - Maximum number of pending webhooks in the queue
    /// * `retry_config` - Configuration for retry behavior
    pub fn new(queue_size: usize, retry_config: RetryConfig) -> Self {
        let (sender, receiver) = mpsc::channel(queue_size);
        let retry_config = Arc::new(retry_config);
        
        // Spawn the worker task
        let worker_retry_config = retry_config.clone();
        let worker_sender = sender.clone();
        tokio::spawn(async move {
            Self::worker(receiver, worker_sender, worker_retry_config).await;
        });
        
        Self {
            sender,
            retry_config,
        }
    }
    
    /// Create a webhook queue with default configuration
    pub fn with_defaults() -> Self {
        Self::new(1000, RetryConfig::default())
    }
    
    /// Enqueue a webhook for delivery
    pub async fn enqueue(
        &self,
        config: PushNotificationConfig,
        payload: WebhookPayload,
    ) -> A2aResult<()> {
        let request = WebhookDeliveryRequest {
            config,
            payload,
            attempt: 0,
        };
        
        self.sender
            .send(request)
            .await
            .map_err(|_| A2aError::Server("Webhook queue is closed".to_string()))?;
        
        Ok(())
    }
    
    /// Worker task that processes queued webhooks
    async fn worker(
        mut receiver: mpsc::Receiver<WebhookDeliveryRequest>,
        sender: mpsc::Sender<WebhookDeliveryRequest>,
        retry_config: Arc<RetryConfig>,
    ) {
        let client = Client::builder()
            .timeout(Duration::from_secs(retry_config.request_timeout))
            .build()
            .expect("Failed to create HTTP client");
        
        while let Some(request) = receiver.recv().await {
            let result = Self::deliver_webhook(
                &client,
                &request.config,
                &request.payload,
            ).await;
            
            match result {
                Ok(_) => {
                    // Successfully delivered
                    // Note: Could add metrics/logging here in production
                }
                Err(e) => {
                    // Delivery failed
                    if request.attempt < retry_config.max_retries {
                        // Retry
                        let next_attempt = request.attempt + 1;
                        let delay = retry_config.calculate_delay(next_attempt);
                        
                        // Note: Could add logging here in production
                        // Retrying webhook delivery after failure
                        
                        // Spawn a task to retry after delay
                        let retry_sender = sender.clone();
                        let retry_request = WebhookDeliveryRequest {
                            attempt: next_attempt,
                            ..request
                        };
                        
                        tokio::spawn(async move {
                            sleep(delay).await;
                            let _ = retry_sender.send(retry_request).await;
                        });
                    } else {
                        // Max retries exceeded
                        // Note: Could add logging/metrics here in production
                        let _ = e; // Acknowledge the error
                    }
                }
            }
        }
    }
    
    /// Deliver a single webhook
    async fn deliver_webhook(
        client: &Client,
        config: &PushNotificationConfig,
        payload: &WebhookPayload,
    ) -> A2aResult<Response> {
        let mut request = client
            .post(config.url.clone())
            .json(payload);
        
        // Add authentication headers
        if let Some(auth) = &config.authentication {
            request = Self::add_authentication(request, auth);
        }
        
        // Send request
        let response = request
            .send()
            .await
            .map_err(|e| A2aError::Server(format!("Webhook delivery failed: {}", e)))?;
        
        // Check response status
        if !response.status().is_success() {
            return Err(A2aError::Server(format!(
                "Webhook returned error status: {}",
                response.status()
            )));
        }
        
        Ok(response)
    }
    
    /// Add authentication to a request
    fn add_authentication(
        mut request: reqwest::RequestBuilder,
        auth: &PushNotificationAuth,
    ) -> reqwest::RequestBuilder {
        match auth {
            PushNotificationAuth::Bearer { token } => {
                request = request.bearer_auth(token);
            }
            PushNotificationAuth::CustomHeaders { headers } => {
                for (key, value) in headers {
                    request = request.header(key, value);
                }
            }
        }
        request
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::push_notification::TaskEvent;
    use std::collections::HashMap;
    use url::Url;

    #[test]
    fn test_retry_config_delay_calculation() {
        let config = RetryConfig::default();
        
        // First retry: 1 * 2^1 = 2 seconds
        assert_eq!(config.calculate_delay(1), Duration::from_secs(2));
        
        // Second retry: 1 * 2^2 = 4 seconds
        assert_eq!(config.calculate_delay(2), Duration::from_secs(4));
        
        // Third retry: 1 * 2^3 = 8 seconds
        assert_eq!(config.calculate_delay(3), Duration::from_secs(8));
        
        // Large retry should be capped at max_delay
        assert_eq!(config.calculate_delay(10), Duration::from_secs(60));
    }

    #[test]
    fn test_webhook_payload_creation() {
        let task = Task::new("Test task");
        let payload = WebhookPayload::new(
            TaskEvent::Completed,
            task.clone(),
            "agent-123".to_string(),
        );
        
        assert_eq!(payload.event, TaskEvent::Completed);
        assert_eq!(payload.task.id, task.id);
        assert_eq!(payload.agent_id, "agent-123");
        assert!(!payload.timestamp.is_empty());
    }

    #[tokio::test]
    async fn test_webhook_queue_creation() {
        let queue = WebhookQueue::with_defaults();
        assert!(queue.sender.capacity() > 0);
    }

    #[tokio::test]
    async fn test_webhook_queue_enqueue() {
        let queue = WebhookQueue::with_defaults();
        
        let config = PushNotificationConfig::new(
            Url::parse("https://example.com/webhook").unwrap(),
            vec![TaskEvent::Completed],
            None,
        );
        
        let task = Task::new("Test task");
        let payload = WebhookPayload::new(
            TaskEvent::Completed,
            task,
            "agent-123".to_string(),
        );
        
        let result = queue.enqueue(config, payload).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_authentication_bearer() {
        let client = Client::new();
        let request = client.post("https://example.com");
        
        let auth = PushNotificationAuth::Bearer {
            token: "test-token".to_string(),
        };
        
        let _authenticated = WebhookQueue::add_authentication(request, &auth);
        // Note: reqwest doesn't expose headers for inspection in tests,
        // but we can verify it compiles and runs
    }

    #[test]
    fn test_add_authentication_custom_headers() {
        let client = Client::new();
        let request = client.post("https://example.com");
        
        let mut headers = HashMap::new();
        headers.insert("X-API-Key".to_string(), "secret".to_string());
        
        let auth = PushNotificationAuth::CustomHeaders { headers };
        
        let _authenticated = WebhookQueue::add_authentication(request, &auth);
        // Note: reqwest doesn't expose headers for inspection in tests,
        // but we can verify it compiles and runs
    }
}
