//! Transport-level capabilities for assembling AgentCard
//!
//! This module defines transport-level capabilities that are added by the server/handler
//! layer, separate from the agent's descriptive profile.

use crate::{
    core::{
        agent_card::{
            AgentCardSignature, AuthenticationRequirement, RateLimit, StreamingCapabilities,
            TransportInterface, TransportType,
        },
        AgentCard,
    },
    server::AgentProfile,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Transport-level capabilities managed by the server/handler layer
///
/// This struct contains all the transport-specific details that the handler
/// knows about, independent of the agent's core logic. These are combined
/// with an `AgentProfile` to create a complete `AgentCard`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TransportCapabilities {
    /// A2A Protocol version
    pub protocol_version: String,

    /// Preferred transport protocol
    pub preferred_transport: TransportType,

    /// Additional transport interfaces
    pub additional_interfaces: Vec<TransportInterface>,

    /// Cryptographic signatures for verifying the agent card
    pub signatures: Vec<AgentCardSignature>,

    /// Indicates support for the authenticated extended card endpoint
    pub supports_authenticated_extended_card: bool,

    /// Authentication requirements
    pub authentication: Option<AuthenticationRequirement>,

    /// Rate limiting information
    pub rate_limits: Option<RateLimit>,

    /// Streaming capabilities (if enabled)
    #[cfg(feature = "streaming")]
    pub streaming: Option<StreamingCapabilities>,

    /// Push notification support
    pub push_notifications: Option<PushNotificationSupport>,

    /// Additional transport-level metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Push notification support details
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PushNotificationSupport {
    /// Whether push notifications are enabled
    pub enabled: bool,

    /// Supported event types
    pub supported_events: Vec<String>,

    /// Maximum number of webhooks per task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_webhooks_per_task: Option<usize>,

    /// Webhook retry policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<WebhookRetryPolicy>,
}

/// Webhook retry policy configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebhookRetryPolicy {
    /// Maximum number of retry attempts
    pub max_retries: u32,

    /// Initial retry delay in seconds
    pub initial_delay_seconds: u64,

    /// Maximum retry delay in seconds
    pub max_delay_seconds: u64,

    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for PushNotificationSupport {
    fn default() -> Self {
        Self {
            enabled: true,
            supported_events: vec![
                "task.queued".to_string(),
                "task.started".to_string(),
                "task.progress".to_string(),
                "task.completed".to_string(),
                "task.failed".to_string(),
                "task.cancelled".to_string(),
            ],
            max_webhooks_per_task: Some(5),
            retry_policy: Some(WebhookRetryPolicy::default()),
        }
    }
}

impl Default for WebhookRetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_seconds: 1,
            max_delay_seconds: 60,
            backoff_multiplier: 2.0,
        }
    }
}

impl TransportCapabilities {
    /// Create new transport capabilities with default settings
    pub fn new() -> Self {
        Self {
            protocol_version: "0.3.0".to_string(),
            preferred_transport: TransportType::default(),
            additional_interfaces: Vec::new(),
            signatures: Vec::new(),
            supports_authenticated_extended_card: false,
            authentication: None,
            rate_limits: None,
            #[cfg(feature = "streaming")]
            streaming: None,
            push_notifications: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the A2A protocol version
    pub fn with_protocol_version<S: Into<String>>(mut self, version: S) -> Self {
        self.protocol_version = version.into();
        self
    }

    /// Set the preferred transport protocol
    pub fn with_preferred_transport(mut self, transport: TransportType) -> Self {
        self.preferred_transport = transport;
        self
    }

    /// Add an additional transport interface
    pub fn add_transport_interface(mut self, interface: TransportInterface) -> Self {
        self.additional_interfaces.push(interface);
        self
    }

    /// Set authentication requirement
    pub fn with_authentication(mut self, auth: AuthenticationRequirement) -> Self {
        self.authentication = Some(auth);
        self
    }

    /// Set rate limiting
    pub fn with_rate_limit(mut self, limit: RateLimit) -> Self {
        self.rate_limits = Some(limit);
        self
    }

    /// Enable streaming with default capabilities
    #[cfg(feature = "streaming")]
    pub fn with_streaming(mut self, capabilities: StreamingCapabilities) -> Self {
        self.streaming = Some(capabilities);
        self
    }

    /// Enable push notifications with default configuration
    pub fn with_push_notifications(mut self, support: PushNotificationSupport) -> Self {
        self.push_notifications = Some(support);
        self
    }

    /// Add transport-level metadata
    pub fn with_metadata<K: Into<String>, V: Into<serde_json::Value>>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Assemble a complete `AgentCard` from an `AgentProfile` and these transport capabilities
    pub fn assemble_card(self, profile: AgentProfile) -> AgentCard {
        let mut card = AgentCard {
            id: profile.id,
            name: profile.name,
            description: profile.description,
            version: profile.version,
            url: profile.url,
            protocol_version: self.protocol_version,
            preferred_transport: self.preferred_transport,
            additional_interfaces: self.additional_interfaces,
            provider: profile.provider,
            icon_url: profile.icon_url,
            documentation_url: profile.documentation_url,
            signatures: self.signatures,
            default_input_modes: profile.default_input_modes,
            default_output_modes: profile.default_output_modes,
            supports_authenticated_extended_card: self.supports_authenticated_extended_card,
            capabilities: profile.capabilities,
            skills: profile.skills,
            authentication: self.authentication,
            rate_limits: self.rate_limits,
            metadata: profile.metadata,
        };

        // Build transport capabilities for the spec-compliant field
        let mut transport_caps = crate::core::agent_card::TransportCapabilities::new();

        // Add streaming metadata if enabled
        #[cfg(feature = "streaming")]
        if let Some(streaming) = &self.streaming {
            transport_caps = transport_caps.with_streaming(true);
            card.metadata.insert(
                "streaming".to_string(),
                serde_json::to_value(streaming).unwrap(),
            );
        }

        // Add push notification metadata if enabled
        if let Some(push_notif) = &self.push_notifications {
            transport_caps = transport_caps.with_push_notifications(push_notif.enabled);
            card.metadata.insert(
                "pushNotifications".to_string(),
                serde_json::to_value(push_notif).unwrap(),
            );
        }

        // Add the transport capabilities to metadata (spec-compliant)
        card.metadata.insert(
            "transportCapabilities".to_string(),
            serde_json::to_value(transport_caps).unwrap(),
        );

        // Merge transport-level metadata
        for (key, value) in self.metadata {
            card.metadata.insert(key, value);
        }

        card
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AgentId;
    use url::Url;

    #[test]
    fn test_transport_capabilities_default() {
        let caps = TransportCapabilities::new();
        assert_eq!(caps.protocol_version, "0.3.0");
        assert_eq!(caps.preferred_transport, TransportType::JsonRpc);
        assert!(caps.authentication.is_none());
        assert!(caps.rate_limits.is_none());
    }

    #[test]
    fn test_assemble_card() {
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let url = Url::parse("https://agent.example.com").unwrap();

        let profile =
            AgentProfile::new(agent_id, "Test Agent", url).with_description("A test agent");

        let caps = TransportCapabilities::new()
            .with_protocol_version("0.3.0")
            .with_preferred_transport(TransportType::JsonRpc);

        let card = caps.assemble_card(profile);

        assert_eq!(card.name, "Test Agent");
        assert_eq!(card.description, Some("A test agent".to_string()));
        assert_eq!(card.protocol_version, "0.3.0");
        assert_eq!(card.preferred_transport, TransportType::JsonRpc);
    }

    #[cfg(feature = "streaming")]
    #[test]
    fn test_assemble_card_with_streaming() {
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let url = Url::parse("https://agent.example.com").unwrap();

        let profile = AgentProfile::new(agent_id, "Test Agent", url);

        let streaming = StreamingCapabilities::new();
        let caps = TransportCapabilities::new().with_streaming(streaming.clone());

        let card = caps.assemble_card(profile);

        assert!(card.metadata.contains_key("streaming"));
        let streaming_meta: StreamingCapabilities =
            serde_json::from_value(card.metadata["streaming"].clone()).unwrap();
        assert_eq!(streaming_meta, streaming);
    }

    #[test]
    fn test_assemble_card_with_push_notifications() {
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let url = Url::parse("https://agent.example.com").unwrap();

        let profile = AgentProfile::new(agent_id, "Test Agent", url);

        let push_notif = PushNotificationSupport::default();
        let caps = TransportCapabilities::new().with_push_notifications(push_notif.clone());

        let card = caps.assemble_card(profile);

        assert!(card.metadata.contains_key("pushNotifications"));
        let push_meta: PushNotificationSupport =
            serde_json::from_value(card.metadata["pushNotifications"].clone()).unwrap();
        assert_eq!(push_meta, push_notif);
    }
}
