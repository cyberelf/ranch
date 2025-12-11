//! Push notification types for A2A Protocol
//!
//! This module implements push notification/webhook support as defined in the A2A Protocol v0.3.0 specification.
//! It allows agents to receive asynchronous task updates via webhooks instead of polling.
//!
//! This is part of the v0.7.0 release implementing the push notification feature.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

use super::task::TaskState;

/// Configuration for push notifications via webhooks
///
/// This struct defines how and when to send webhook notifications for task updates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PushNotificationConfig {
    /// Webhook endpoint URL - must be HTTPS for security
    pub url: Url,

    /// Events that trigger webhook notifications
    pub events: Vec<TaskEvent>,

    /// Authentication configuration for webhook requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<PushNotificationAuth>,
}

impl PushNotificationConfig {
    /// Create a new push notification configuration
    ///
    /// # Arguments
    /// * `url` - Webhook endpoint URL (must be HTTPS)
    /// * `events` - Events that trigger notifications
    /// * `authentication` - Optional authentication configuration
    ///
    /// # Example
    /// ```
    /// use a2a_protocol::core::push_notification::{PushNotificationConfig, TaskEvent};
    /// use url::Url;
    ///
    /// let config = PushNotificationConfig::new(
    ///     Url::parse("https://example.com/webhook").unwrap(),
    ///     vec![TaskEvent::Completed, TaskEvent::Failed],
    ///     None,
    /// );
    /// ```
    pub fn new(
        url: Url,
        events: Vec<TaskEvent>,
        authentication: Option<PushNotificationAuth>,
    ) -> Self {
        Self {
            url,
            events,
            authentication,
        }
    }

    /// Validate the configuration
    ///
    /// Ensures:
    /// - URL uses HTTPS scheme (required for security)
    /// - Events list is not empty
    /// - URL is not targeting private IP ranges (SSRF protection)
    pub fn validate(&self) -> Result<(), String> {
        // Ensure at least one event is configured
        if self.events.is_empty() {
            return Err("At least one event type must be configured".to_string());
        }

        // SSRF protection - validate webhook URL
        super::ssrf_protection::validate_webhook_url(&self.url)?;

        Ok(())
    }
}

/// Authentication methods for webhook requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PushNotificationAuth {
    /// Bearer token authentication
    Bearer {
        /// The bearer token to include in Authorization header
        token: String,
    },

    /// Custom HTTP headers
    CustomHeaders {
        /// Map of header name to header value
        headers: HashMap<String, String>,
    },
    // OAuth2 support can be added in future versions
}

/// Events that can trigger push notifications
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum TaskEvent {
    /// Task status changed (includes state transitions)
    StatusChanged,

    /// New artifact was added to the task
    ArtifactAdded,

    /// Task completed successfully
    Completed,

    /// Task failed
    Failed,

    /// Task was cancelled
    Cancelled,
}

impl TaskEvent {
    /// Check if this event matches a task state transition
    ///
    /// # Arguments
    /// * `from` - Previous task state
    /// * `to` - New task state
    pub fn matches_transition(&self, from: &TaskState, to: &TaskState) -> bool {
        match self {
            TaskEvent::StatusChanged => from != to,
            TaskEvent::Completed => matches!(to, TaskState::Completed),
            TaskEvent::Failed => matches!(to, TaskState::Failed),
            TaskEvent::Cancelled => matches!(to, TaskState::Cancelled),
            TaskEvent::ArtifactAdded => false, // Handled separately
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_notification_config_creation() {
        let url = Url::parse("https://example.com/webhook").unwrap();
        let events = vec![TaskEvent::Completed, TaskEvent::Failed];
        let config = PushNotificationConfig::new(url.clone(), events.clone(), None);

        assert_eq!(config.url, url);
        assert_eq!(config.events, events);
        assert!(config.authentication.is_none());
    }

    #[test]
    fn test_push_notification_config_validation_https() {
        let http_url = Url::parse("http://example.com/webhook").unwrap();
        let config = PushNotificationConfig::new(http_url, vec![TaskEvent::Completed], None);

        assert!(config.validate().is_err());
        assert!(config.validate().unwrap_err().contains("HTTPS"));
    }

    #[test]
    fn test_push_notification_config_validation_empty_events() {
        let url = Url::parse("https://example.com/webhook").unwrap();
        let config = PushNotificationConfig::new(url, vec![], None);

        assert!(config.validate().is_err());
        assert!(config.validate().unwrap_err().contains("event"));
    }

    #[test]
    fn test_push_notification_config_validation_success() {
        let url = Url::parse("https://example.com/webhook").unwrap();
        let config = PushNotificationConfig::new(url, vec![TaskEvent::Completed], None);

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_bearer_auth() {
        let auth = PushNotificationAuth::Bearer {
            token: "secret-token".to_string(),
        };

        match auth {
            PushNotificationAuth::Bearer { token } => {
                assert_eq!(token, "secret-token");
            }
            _ => panic!("Expected Bearer auth"),
        }
    }

    #[test]
    fn test_custom_headers_auth() {
        let mut headers = HashMap::new();
        headers.insert("X-API-Key".to_string(), "my-key".to_string());

        let auth = PushNotificationAuth::CustomHeaders {
            headers: headers.clone(),
        };

        match auth {
            PushNotificationAuth::CustomHeaders { headers: h } => {
                assert_eq!(h, headers);
            }
            _ => panic!("Expected CustomHeaders auth"),
        }
    }

    #[test]
    fn test_task_event_matches_transition() {
        // Completed event
        assert!(TaskEvent::Completed.matches_transition(&TaskState::Working, &TaskState::Completed));
        assert!(!TaskEvent::Completed.matches_transition(&TaskState::Working, &TaskState::Failed));

        // Failed event
        assert!(TaskEvent::Failed.matches_transition(&TaskState::Working, &TaskState::Failed));
        assert!(!TaskEvent::Failed.matches_transition(&TaskState::Working, &TaskState::Completed));

        // Cancelled event
        assert!(TaskEvent::Cancelled.matches_transition(&TaskState::Working, &TaskState::Cancelled));

        // StatusChanged event
        assert!(
            TaskEvent::StatusChanged.matches_transition(&TaskState::Pending, &TaskState::Working)
        );
        assert!(
            !TaskEvent::StatusChanged.matches_transition(&TaskState::Working, &TaskState::Working)
        );
    }

    #[test]
    fn test_serialization() {
        let url = Url::parse("https://example.com/webhook").unwrap();
        let config = PushNotificationConfig::new(
            url,
            vec![TaskEvent::Completed, TaskEvent::Failed],
            Some(PushNotificationAuth::Bearer {
                token: "secret".to_string(),
            }),
        );

        // Test serialization
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("https://example.com/webhook"));
        assert!(json.contains("completed"));
        assert!(json.contains("failed"));
        assert!(json.contains("bearer"));

        // Test deserialization
        let deserialized: PushNotificationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, config);
    }
}
