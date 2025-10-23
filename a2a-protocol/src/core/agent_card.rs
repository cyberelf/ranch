//! A2A Agent Card definition

use crate::AgentId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use url::Url;

/// Supported transport types defined by the A2A v0.3.0 specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

impl Default for TransportType {
    fn default() -> Self {
        TransportType::JsonRpc
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

/// Agent capability definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentCapability {
    /// Capability name
    pub name: String,

    /// Capability description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Capability category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Input schema for this capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,

    /// Output schema for this capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Examples of using this skill
    #[serde(skip_serializing_if = "Vec::is_empty")]
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
        flows: OAuth2Flows,
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
}
