//! A2A Agent Card definition

use crate::AgentId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use url::Url;

use super::extension::AgentExtension;

/// Supported transport types defined by the A2A v0.3.0 specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TransportType {
    /// JSON-RPC 2.0 transport over HTTP
    #[serde(
        rename = "JSONRPC",
        alias = "JSON-RPC",
        alias = "JSON_RPC",
        alias = "json-rpc",
        alias = "json_rpc",
        alias = "jsonrpc"
    )]
    #[default]
    JsonRpc,
    /// gRPC transport
    #[serde(rename = "GRPC", alias = "grpc")]
    Grpc,
    /// HTTP+JSON/REST transport
    #[serde(
        rename = "HTTP+JSON",
        alias = "HTTP_JSON",
        alias = "HTTP-JSON",
        alias = "http+json",
        alias = "http_json",
        alias = "http-json"
    )]
    HttpJson,
}

impl TransportType {
    /// Returns the canonical spec name for this transport
    pub fn as_str(&self) -> &'static str {
        match self {
            TransportType::JsonRpc => "JSONRPC",
            TransportType::Grpc => "GRPC",
            TransportType::HttpJson => "HTTP+JSON",
        }
    }

    /// Parse a transport name into a spec-aligned enum value
    pub fn from_name<S: AsRef<str>>(name: S) -> Self {
        match name.as_ref().to_ascii_uppercase().as_str() {
            "JSONRPC" | "JSON-RPC" | "JSON_RPC" => TransportType::JsonRpc,
            "GRPC" | "G_RPC" => TransportType::Grpc,
            "HTTP+JSON" | "HTTP_JSON" | "HTTP-JSON" | "HTTPJSON" => TransportType::HttpJson,
            _ => TransportType::JsonRpc,
        }
    }
}

impl fmt::Display for TransportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A2A Agent Card - metadata about an agent (A2A v0.3.0 compliant)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentCard {
    /// Agent identifier
    pub id: AgentId,

    /// Agent name
    pub name: String,

    /// Agent description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Agent version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Agent URL or endpoint
    pub url: Url,

    /// A2A Protocol version (REQUIRED per spec) - semver format
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,

    /// Preferred transport protocol (REQUIRED per spec)
    #[serde(rename = "preferredTransport")]
    pub preferred_transport: TransportType,

    /// Additional transport interfaces supported (REQUIRED per spec)
    /// List of alternative transport endpoints
    #[serde(
        rename = "additionalInterfaces",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub additional_interfaces: Vec<TransportInterface>,

    /// Agent provider details (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<AgentProvider>,

    /// URL to an icon representing the agent
    #[serde(rename = "iconUrl", skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<Url>,

    /// URL to documentation for this agent
    #[serde(rename = "documentationUrl", skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<Url>,

    /// Cryptographic signatures for verifying the agent card
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub signatures: Vec<AgentCardSignature>,

    /// Default input MIME types supported by this agent
    #[serde(
        rename = "defaultInputModes",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub default_input_modes: Vec<String>,

    /// Default output MIME types produced by this agent
    #[serde(
        rename = "defaultOutputModes",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub default_output_modes: Vec<String>,

    /// Indicates support for the authenticated extended card endpoint
    #[serde(rename = "supportsAuthenticatedExtendedCard", default)]
    pub supports_authenticated_extended_card: bool,

    /// Agent capabilities
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub capabilities: Vec<AgentCapability>,

    /// Agent skills
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skills: Vec<AgentSkill>,

    /// Authentication requirements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<AuthenticationRequirement>,

    /// Rate limiting information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limits: Option<RateLimit>,

    /// Additional metadata
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Transport interface definition (A2A v0.3.0 spec)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransportInterface {
    /// Transport protocol type (e.g., "json-rpc", "http+json", "grpc")
    #[serde(rename = "type")]
    pub transport_type: TransportType,

    /// Transport endpoint URL
    pub url: Url,

    /// Optional transport-specific configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<HashMap<String, serde_json::Value>>,
}

impl TransportInterface {
    /// Create a new transport interface
    pub fn new(transport_type: TransportType, url: Url) -> Self {
        Self {
            transport_type,
            url,
            config: None,
        }
    }

    /// Add configuration to this transport interface
    pub fn with_config(mut self, config: HashMap<String, serde_json::Value>) -> Self {
        self.config = Some(config);
        self
    }
}

/// Agent provider metadata (optional)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProvider {
    /// Provider or organization name
    pub name: String,

    /// Provider description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// URL to the provider's homepage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,

    /// Contact email address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_email: Option<String>,

    /// Contact URL for support or inquiries
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_url: Option<Url>,

    /// Any additional provider metadata not captured by standard fields
    #[serde(flatten, default)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Agent card signature metadata (optional)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCardSignature {
    /// Signature algorithm identifier (e.g., Ed25519)
    pub algorithm: String,

    /// Base64 or hex-encoded signature value (spec-defined format)
    pub signature: String,

    /// Identifier for the key used to sign
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,

    /// Optional certificate chain for signature verification
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub certificate_chain: Vec<String>,

    /// Additional signature metadata
    #[serde(flatten, default)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Agent transport and protocol capabilities (A2A v0.3.0 spec compliant)
///
/// Defines optional capabilities supported by an agent according to the
/// A2A Protocol specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapability {
    /// Indicates if the agent supports streaming responses
    #[serde(default, skip_serializing_if = "is_false")]
    pub streaming: bool,

    /// Indicates if the agent supports sending push notifications for asynchronous task updates
    #[serde(default, skip_serializing_if = "is_false")]
    pub push_notifications: bool,

    /// Indicates if the agent provides a history of state transitions for a task
    #[serde(default, skip_serializing_if = "is_false")]
    pub state_transition_history: bool,

    /// A list of protocol extensions supported by the agent
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<AgentExtensionInfo>,
}

fn is_false(b: &bool) -> bool {
    !b
}

impl AgentCapability {
    /// Create new empty capabilities
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable streaming support
    pub fn with_streaming(mut self, enabled: bool) -> Self {
        self.streaming = enabled;
        self
    }

    /// Enable push notifications support
    pub fn with_push_notifications(mut self, enabled: bool) -> Self {
        self.push_notifications = enabled;
        self
    }

    /// Enable state transition history support
    pub fn with_state_transition_history(mut self, enabled: bool) -> Self {
        self.state_transition_history = enabled;
        self
    }

    /// Add an extension to the capabilities
    pub fn with_extension_info(mut self, extension: AgentExtensionInfo) -> Self {
        self.extensions.push(extension);
        self
    }

    /// Add a typed extension using the AgentExtension trait
    ///
    /// This automatically populates the extension info from the trait's const values.
    ///
    /// # Example
    ///
    /// ```
    /// use a2a_protocol::core::agent_card::AgentCapability;
    /// use a2a_protocol::core::extension::AgentExtension;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Clone, Serialize, Deserialize)]
    /// struct MyExtension {
    ///     value: String,
    /// }
    ///
    /// impl AgentExtension for MyExtension {
    ///     const URI: &'static str = "https://example.com/ext/my-extension";
    ///     const VERSION: &'static str = "v1";
    ///     const NAME: &'static str = "My Extension";
    ///     const DESCRIPTION: &'static str = "Example extension";
    /// }
    ///
    /// let capabilities = AgentCapability::new()
    ///     .with_extension::<MyExtension>();
    /// ```
    pub fn with_extension<T: AgentExtension>(mut self) -> Self {
        self.extensions.push(AgentExtensionInfo {
            uri: T::URI.to_string(),
            version: Some(T::VERSION.to_string()),
            name: Some(T::NAME.to_string()),
            description: Some(T::DESCRIPTION.to_string()),
        });
        self
    }
}

/// Information about a protocol extension supported by an agent
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentExtensionInfo {
    /// Extension URI (e.g., "https://example.com/extensions/my-extension/v1")
    pub uri: String,

    /// Extension version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Human-readable extension name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Extension description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Agent skill definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentSkill {
    /// Skill name
    pub name: String,

    /// Skill description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Skill category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Skill tags
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Examples of using this skill
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,
}

/// Authentication requirement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthenticationRequirement {
    /// No authentication required
    #[serde(rename = "none")]
    None,

    /// API key authentication
    #[serde(rename = "api_key")]
    ApiKey {
        /// API key location (header, query, cookie)
        location: String,
        /// API key name
        name: String,
    },

    /// Bearer token authentication
    #[serde(rename = "bearer")]
    Bearer {
        /// Token format hint
        format: Option<String>,
    },

    /// OAuth2 authentication
    #[serde(rename = "oauth2")]
    OAuth2 {
        /// OAuth2 flows configuration
        flows: Box<OAuth2Flows>,
    },

    /// Custom authentication
    #[serde(rename = "custom")]
    Custom {
        /// Custom authentication type
        auth_type: String,
        /// Configuration parameters
        parameters: HashMap<String, serde_json::Value>,
    },
}

/// OAuth2 flows configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OAuth2Flows {
    /// Authorization code flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_code: Option<OAuth2AuthorizationCodeFlow>,

    /// Client credentials flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_credentials: Option<OAuth2ClientCredentialsFlow>,

    /// Implicit flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implicit: Option<OAuth2ImplicitFlow>,
}

/// OAuth2 authorization code flow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OAuth2AuthorizationCodeFlow {
    /// Authorization URL
    pub authorization_url: Url,
    /// Token URL
    pub token_url: Url,
    /// Supported scopes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
}

/// OAuth2 client credentials flow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OAuth2ClientCredentialsFlow {
    /// Token URL
    pub token_url: Url,
    /// Supported scopes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
}

/// OAuth2 implicit flow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OAuth2ImplicitFlow {
    /// Authorization URL
    pub authorization_url: Url,
    /// Supported scopes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateLimit {
    /// Maximum requests per time window
    pub max_requests: u32,

    /// Time window in seconds
    pub window_seconds: u32,

    /// Strategy for handling rate limits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<RateLimitStrategy>,
}

/// Rate limit strategy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RateLimitStrategy {
    /// Reject requests when limit is exceeded
    Reject,

    /// Queue requests when limit is exceeded
    Queue,

    /// Return cached responses when limit is exceeded
    Cache,
}

impl AgentCard {
    /// Create a minimal agent card with required A2A v0.3.0 fields
    pub fn new<S: Into<String>>(id: AgentId, name: S, url: Url) -> Self {
        Self {
            id,
            name: name.into(),
            description: None,
            version: None,
            url,
            protocol_version: "0.3.0".to_string(), // Default to A2A v0.3.0
            preferred_transport: TransportType::default(),
            additional_interfaces: Vec::new(),
            provider: None,
            icon_url: None,
            documentation_url: None,
            signatures: Vec::new(),
            default_input_modes: Vec::new(),
            default_output_modes: Vec::new(),
            supports_authenticated_extended_card: false,
            capabilities: Vec::new(),
            skills: Vec::new(),
            authentication: None,
            rate_limits: None,
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

    /// Set the preferred transport from a string, falling back to JSON-RPC on unknown values
    pub fn with_preferred_transport_str<S: AsRef<str>>(mut self, transport: S) -> Self {
        self.preferred_transport = TransportType::from_name(transport);
        self
    }

    /// Add an additional transport interface
    pub fn add_transport_interface(mut self, interface: TransportInterface) -> Self {
        self.additional_interfaces.push(interface);
        self
    }

    /// Set the agent provider metadata
    pub fn with_provider(mut self, provider: AgentProvider) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Set or clear the agent icon URL
    pub fn with_icon_url(mut self, icon_url: Option<Url>) -> Self {
        self.icon_url = icon_url;
        self
    }

    /// Set or clear the agent documentation URL
    pub fn with_documentation_url(mut self, documentation_url: Option<Url>) -> Self {
        self.documentation_url = documentation_url;
        self
    }

    /// Replace all card signatures with the provided collection
    pub fn with_signatures<I>(mut self, signatures: I) -> Self
    where
        I: IntoIterator<Item = AgentCardSignature>,
    {
        self.signatures = signatures.into_iter().collect();
        self
    }

    /// Add a single signature entry to the card
    pub fn add_signature(mut self, signature: AgentCardSignature) -> Self {
        self.signatures.push(signature);
        self
    }

    /// Set the default input MIME types
    pub fn with_default_input_modes<I, S>(mut self, modes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.default_input_modes = modes.into_iter().map(Into::into).collect();
        self
    }

    /// Set the default output MIME types
    pub fn with_default_output_modes<I, S>(mut self, modes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.default_output_modes = modes.into_iter().map(Into::into).collect();
        self
    }

    /// Set whether the agent supports the authenticated extended card endpoint
    pub fn with_supports_authenticated_extended_card(mut self, supported: bool) -> Self {
        self.supports_authenticated_extended_card = supported;
        self
    }

    /// Add a capability
    pub fn with_capability(mut self, capability: AgentCapability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Add a typed extension to the agent capabilities
    ///
    /// This is a convenience method that automatically adds extension information
    /// to the agent's capabilities based on the AgentExtension trait implementation.
    ///
    /// # Example
    ///
    /// ```
    /// use a2a_protocol::core::{AgentCard, AgentId};
    /// use a2a_protocol::core::extension::AgentExtension;
    /// use serde::{Serialize, Deserialize};
    /// use url::Url;
    ///
    /// #[derive(Debug, Clone, Serialize, Deserialize)]
    /// struct ClientRoutingExtension {
    ///     target_agent_id: String,
    /// }
    ///
    /// impl AgentExtension for ClientRoutingExtension {
    ///     const URI: &'static str = "https://a2a-protocol.org/extensions/client-agent/v1";
    ///     const VERSION: &'static str = "v1";
    ///     const NAME: &'static str = "Client Agent Extension";
    ///     const DESCRIPTION: &'static str = "Client-side routing extension";
    /// }
    ///
    /// let url = Url::parse("http://localhost:3000").unwrap();
    /// let card = AgentCard::new(AgentId::new("agent-1".to_string()).unwrap(), "My Agent", url)
    ///     .with_extension::<ClientRoutingExtension>();
    /// ```
    pub fn with_extension<T: AgentExtension>(mut self) -> Self {
        // Find or create a capability to add the extension to
        if let Some(capability) = self.capabilities.first_mut() {
            capability.extensions.push(AgentExtensionInfo {
                uri: T::URI.to_string(),
                version: Some(T::VERSION.to_string()),
                name: Some(T::NAME.to_string()),
                description: Some(T::DESCRIPTION.to_string()),
            });
        } else {
            // No existing capabilities, create one with the extension
            self.capabilities.push(
                AgentCapability::new().with_extension::<T>()
            );
        }
        self
    }

    /// Add a skill
    pub fn with_skill(mut self, skill: AgentSkill) -> Self {
        self.skills.push(skill);
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

    /// Add metadata
    pub fn with_metadata<K: Into<String>, V: Into<serde_json::Value>>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set transport capabilities (push notifications, streaming, etc.)
    pub fn with_transport_capabilities(mut self, capabilities: TransportCapabilities) -> Self {
        self.metadata.insert(
            "transportCapabilities".to_string(),
            serde_json::to_value(capabilities).unwrap(),
        );
        self
    }

    /// Get transport capabilities from metadata
    pub fn transport_capabilities(&self) -> Option<TransportCapabilities> {
        self.metadata
            .get("transportCapabilities")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

/// Transport capabilities for AgentCard
///
/// Describes the transport-level capabilities supported by this agent.
/// This should be included in the AgentCard's metadata field under the "transportCapabilities" key.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransportCapabilities {
    /// Whether push notifications (webhooks) are supported
    #[serde(default)]
    pub push_notifications: bool,

    /// Whether SSE streaming is supported
    #[serde(default)]
    pub streaming: bool,
}

impl TransportCapabilities {
    /// Create new transport capabilities with all features disabled
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable push notifications support
    pub fn with_push_notifications(mut self, enabled: bool) -> Self {
        self.push_notifications = enabled;
        self
    }

    /// Enable streaming support
    pub fn with_streaming(mut self, enabled: bool) -> Self {
        self.streaming = enabled;
        self
    }

    /// Create transport capabilities with push notifications enabled
    pub fn push_notifications_enabled() -> Self {
        Self {
            push_notifications: true,
            streaming: false,
        }
    }

    /// Create transport capabilities with streaming enabled
    pub fn streaming_enabled() -> Self {
        Self {
            push_notifications: false,
            streaming: true,
        }
    }

    /// Create transport capabilities with all features enabled
    pub fn all_enabled() -> Self {
        Self {
            push_notifications: true,
            streaming: true,
        }
    }
}

/// Streaming capabilities for AgentCard metadata
///
/// This describes the streaming support offered by an agent, which can be
/// included in the AgentCard's metadata field under the "streaming" key.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct StreamingCapabilities {
    /// Whether streaming is supported
    pub enabled: bool,

    /// Supported streaming methods (e.g., "message/stream", "task/resubscribe")
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub methods: Vec<String>,

    /// Event buffer size (number of events kept for replay)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buffer_size: Option<usize>,

    /// Connection timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,

    /// Keep-alive interval in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keepalive_seconds: Option<u64>,
}

impl StreamingCapabilities {
    /// Create new streaming capabilities with default settings
    pub fn new() -> Self {
        Self {
            enabled: true,
            methods: vec!["message/stream".to_string(), "task/resubscribe".to_string()],
            buffer_size: Some(100),
            timeout_seconds: Some(300), // 5 minutes
            keepalive_seconds: Some(30),
        }
    }

    /// Create disabled streaming capabilities
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            methods: Vec::new(),
            buffer_size: None,
            timeout_seconds: None,
            keepalive_seconds: None,
        }
    }

    /// Check if a specific method is supported
    pub fn supports_method(&self, method: &str) -> bool {
        self.enabled && self.methods.contains(&method.to_string())
    }

    /// Add a supported streaming method
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.methods.push(method.into());
        self
    }

    /// Set the buffer size
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = Some(size);
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }

    /// Set the keep-alive interval
    pub fn with_keepalive(mut self, seconds: u64) -> Self {
        self.keepalive_seconds = Some(seconds);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_agent_card() {
        let id = AgentId::new("agent1".to_string()).unwrap();
        let url = Url::parse("https://agent.example.com").unwrap();
        let card = AgentCard::new(id, "Test Agent", url);

        assert_eq!(card.name, "Test Agent");
        assert_eq!(card.preferred_transport, TransportType::JsonRpc);
        assert!(card.default_input_modes.is_empty());
        assert!(card.default_output_modes.is_empty());
        assert!(card.provider.is_none());
        assert!(card.icon_url.is_none());
        assert!(card.documentation_url.is_none());
        assert!(card.signatures.is_empty());
        assert!(!card.supports_authenticated_extended_card);
    }

    #[test]
    fn test_agent_card_with_additional_interface() {
        let id = AgentId::new("agent1".to_string()).unwrap();
        let url = Url::parse("https://agent.example.com").unwrap();
        let endpoint = Url::parse("https://agent.example.com/a2a").unwrap();

        let card = AgentCard::new(id, "Test Agent", url).add_transport_interface(
            TransportInterface::new(TransportType::HttpJson, endpoint.clone()),
        );

        assert_eq!(card.additional_interfaces.len(), 1);
        assert_eq!(card.additional_interfaces[0].url, endpoint);
        assert_eq!(
            card.additional_interfaces[0].transport_type,
            TransportType::HttpJson
        );
    }

    #[test]
    fn test_agent_card_with_provider_and_signatures() {
        let id = AgentId::new("agent1".to_string()).unwrap();
        let url = Url::parse("https://agent.example.com").unwrap();
        let icon_url = Url::parse("https://agent.example.com/icon.png").unwrap();
        let docs_url = Url::parse("https://agent.example.com/docs").unwrap();
        let provider_url = Url::parse("https://provider.example.com").unwrap();

        let provider = AgentProvider {
            name: "Example Provider".to_string(),
            description: Some("Provides example agents".to_string()),
            url: Some(provider_url.clone()),
            contact_email: Some("support@example.com".to_string()),
            contact_url: None,
            extra: HashMap::new(),
        };

        let signature = AgentCardSignature {
            algorithm: "Ed25519".to_string(),
            signature: "deadbeef".to_string(),
            key_id: Some("key-1".to_string()),
            certificate_chain: vec![],
            extra: HashMap::new(),
        };

        let card = AgentCard::new(id, "Test Agent", url)
            .with_provider(provider.clone())
            .with_icon_url(Some(icon_url.clone()))
            .with_documentation_url(Some(docs_url.clone()))
            .add_signature(signature.clone());

        assert_eq!(card.provider.as_ref().unwrap().name, "Example Provider");
        assert_eq!(card.icon_url.as_ref().unwrap(), &icon_url);
        assert_eq!(card.documentation_url.as_ref().unwrap(), &docs_url);
        assert_eq!(card.provider.unwrap().url.unwrap(), provider_url);
        assert_eq!(card.signatures.len(), 1);
        assert_eq!(card.signatures[0], signature);
    }

    #[test]
    fn test_streaming_capabilities_new() {
        let caps = StreamingCapabilities::new();
        assert!(caps.enabled);
        assert!(caps.supports_method("message/stream"));
        assert!(caps.supports_method("task/resubscribe"));
        assert!(!caps.supports_method("unknown"));
        assert_eq!(caps.buffer_size, Some(100));
        assert_eq!(caps.timeout_seconds, Some(300));
        assert_eq!(caps.keepalive_seconds, Some(30));
    }

    #[test]
    fn test_streaming_capabilities_disabled() {
        let caps = StreamingCapabilities::disabled();
        assert!(!caps.enabled);
        assert!(!caps.supports_method("message/stream"));
        assert_eq!(caps.methods.len(), 0);
    }

    #[test]
    fn test_streaming_capabilities_builder() {
        let caps = StreamingCapabilities::new()
            .with_method("custom/stream")
            .with_buffer_size(200)
            .with_timeout(600)
            .with_keepalive(60);

        assert!(caps.supports_method("custom/stream"));
        assert_eq!(caps.buffer_size, Some(200));
        assert_eq!(caps.timeout_seconds, Some(600));
        assert_eq!(caps.keepalive_seconds, Some(60));
    }

    #[test]
    fn test_transport_capabilities_default() {
        let caps = TransportCapabilities::new();
        assert!(!caps.push_notifications);
        assert!(!caps.streaming);
    }

    #[test]
    fn test_transport_capabilities_push_notifications() {
        let caps = TransportCapabilities::push_notifications_enabled();
        assert!(caps.push_notifications);
        assert!(!caps.streaming);
    }

    #[test]
    fn test_transport_capabilities_streaming() {
        let caps = TransportCapabilities::streaming_enabled();
        assert!(!caps.push_notifications);
        assert!(caps.streaming);
    }

    #[test]
    fn test_transport_capabilities_all_enabled() {
        let caps = TransportCapabilities::all_enabled();
        assert!(caps.push_notifications);
        assert!(caps.streaming);
    }

    #[test]
    fn test_transport_capabilities_builder() {
        let caps = TransportCapabilities::new()
            .with_push_notifications(true)
            .with_streaming(true);

        assert!(caps.push_notifications);
        assert!(caps.streaming);
    }

    #[test]
    fn test_agent_card_with_transport_capabilities() {
        let id = AgentId::new("agent1".to_string()).unwrap();
        let url = Url::parse("https://agent.example.com").unwrap();

        let caps = TransportCapabilities::all_enabled();
        let card = AgentCard::new(id, "Test Agent", url).with_transport_capabilities(caps.clone());

        let retrieved_caps = card.transport_capabilities().unwrap();
        assert_eq!(retrieved_caps, caps);
        assert!(retrieved_caps.push_notifications);
        assert!(retrieved_caps.streaming);
    }

    #[test]
    fn test_agent_card_transport_capabilities_serialization() {
        let id = AgentId::new("agent1".to_string()).unwrap();
        let url = Url::parse("https://agent.example.com").unwrap();

        let caps = TransportCapabilities::push_notifications_enabled();
        let card = AgentCard::new(id, "Test Agent", url).with_transport_capabilities(caps);

        // Serialize to JSON
        let json = serde_json::to_value(&card).unwrap();

        // Check that transportCapabilities is in metadata
        let metadata = json["metadata"].as_object().unwrap();
        assert!(metadata.contains_key("transportCapabilities"));

        let transport_caps = &metadata["transportCapabilities"];
        assert_eq!(transport_caps["pushNotifications"], true);
        assert_eq!(transport_caps["streaming"], false);

        // Deserialize back
        let deserialized: AgentCard = serde_json::from_value(json).unwrap();
        let retrieved_caps = deserialized.transport_capabilities().unwrap();
        assert!(retrieved_caps.push_notifications);
        assert!(!retrieved_caps.streaming);
    }
}
