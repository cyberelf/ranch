//! A2A client builder

use crate::{
    client::{
        transport::{JsonRpcTransport, Transport, TransportConfig},
        A2aClient,
    },
    AgentId,
};
use std::sync::Arc;

/// Builder for creating A2A clients with custom configuration
pub struct ClientBuilder {
    agent_id: Option<AgentId>,
    transport_config: TransportConfig,
    transport_type: TransportType,
}

/// Supported transport types
#[derive(Debug, Clone)]
pub enum TransportType {
    /// JSON-RPC transport (A2A spec-compliant)
    JsonRpc { endpoint: String },
    /// Custom transport
    Custom(Arc<dyn Transport>),
}

impl ClientBuilder {
    /// Create a new client builder
    pub fn new() -> Self {
        Self {
            agent_id: None,
            transport_config: TransportConfig::default(),
            transport_type: TransportType::JsonRpc {
                endpoint: String::new(),
            },
        }
    }

    /// Set the agent ID for the client
    pub fn with_agent_id<S: Into<String>>(mut self, agent_id: S) -> Result<Self, crate::A2aError> {
        self.agent_id = Some(AgentId::new(agent_id.into())?);
        Ok(self)
    }

    /// Set the transport configuration
    pub fn with_config(mut self, config: TransportConfig) -> Self {
        self.transport_config = config;
        self
    }

    /// Set the timeout in seconds
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.transport_config.timeout_seconds = timeout_seconds;
        self
    }

    /// Set the maximum number of retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.transport_config.max_retries = max_retries;
        self
    }

    /// Enable or disable compression
    pub fn with_compression(mut self, enable: bool) -> Self {
        self.transport_config.enable_compression = enable;
        self
    }

    /// Add extra configuration
    pub fn with_extra<K: Into<String>, V: Into<serde_json::Value>>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        self.transport_config.extra.insert(key.into(), value.into());
        self
    }

    /// Use JSON-RPC transport (A2A spec-compliant)
    pub fn with_json_rpc<S: Into<String>>(mut self, endpoint: S) -> Self {
        self.transport_type = TransportType::JsonRpc {
            endpoint: endpoint.into(),
        };
        self
    }

    /// Use custom transport
    pub fn with_custom_transport(mut self, transport: Arc<dyn Transport>) -> Self {
        self.transport_type = TransportType::Custom(transport);
        self
    }

    /// Build the non-streaming A2A client
    pub fn build(self) -> Result<A2aClient, crate::A2aError> {
        let transport: Arc<dyn Transport> = match self.transport_type {
            TransportType::JsonRpc { endpoint } => Arc::new(JsonRpcTransport::with_config(
                endpoint,
                self.transport_config,
            )?),
            TransportType::Custom(transport) => transport,
        };

        let agent_id = self.agent_id.unwrap_or_else(|| AgentId::generate());

        Ok(A2aClient::with_agent_id(transport, agent_id))
    }

    /// Build a type-safe streaming client (requires streaming feature)
    ///
    /// This creates an `A2aStreamingClient` that provides compile-time guarantees
    /// that the transport supports streaming. Only works with JSON-RPC transport currently.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use a2a_protocol::client::ClientBuilder;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClientBuilder::new()
    ///     .with_json_rpc("https://agent.example.com/rpc")
    ///     .build_streaming()?;
    ///
    /// // Now you have compile-time streaming guarantees
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "streaming")]
    pub fn build_streaming(
        self,
    ) -> Result<crate::client::A2aStreamingClient<JsonRpcTransport>, crate::A2aError> {
        match self.transport_type {
            TransportType::JsonRpc { endpoint } => {
                let transport = Arc::new(JsonRpcTransport::with_config(
                    endpoint,
                    self.transport_config,
                )?);
                Ok(crate::client::A2aStreamingClient::new(transport))
            }
            TransportType::Custom(_) => Err(crate::A2aError::UnsupportedOperation(
                "build_streaming() only supports JSON-RPC transport. Use build() for custom transports.".to_string()
            )),
        }
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_builder_creation() {
        let builder = ClientBuilder::new();
        assert!(builder.agent_id.is_none());
    }

    #[test]
    fn test_client_builder_with_agent_id() {
        let builder = ClientBuilder::new().with_agent_id("test-agent").unwrap();

        assert!(builder.agent_id.is_some());
    }

    #[test]
    fn test_client_builder_with_config() {
        let config = TransportConfig {
            timeout_seconds: 60,
            max_retries: 5,
            enable_compression: false,
            extra: std::collections::HashMap::new(),
        };

        let builder = ClientBuilder::new().with_config(config);
        assert_eq!(builder.transport_config.timeout_seconds, 60);
    }

    #[test]
    fn test_client_builder_with_json_rpc() {
        let builder = ClientBuilder::new().with_json_rpc("https://example.com/rpc");

        match builder.transport_type {
            TransportType::JsonRpc { endpoint } => {
                assert_eq!(endpoint, "https://example.com/rpc");
            }
            _ => panic!("Expected JSON-RPC transport type"),
        }
    }

    #[tokio::test]
    async fn test_client_builder_build() {
        let client = ClientBuilder::new()
            .with_agent_id("test-agent")
            .unwrap()
            .with_json_rpc("https://example.com/rpc")
            .with_timeout(30)
            .with_max_retries(3)
            .build()
            .unwrap();

        assert_eq!(client.agent_id().as_str(), "test-agent");
        assert_eq!(client.transport_type(), "json-rpc");
        assert_eq!(client.config().timeout_seconds, 30);
    }

    #[cfg(feature = "streaming")]
    #[tokio::test]
    async fn test_client_builder_build_streaming() {
        let client = ClientBuilder::new()
            .with_json_rpc("https://example.com/rpc")
            .with_timeout(30)
            .build_streaming()
            .unwrap();

        assert_eq!(client.base().transport_type(), "json-rpc");
        assert_eq!(client.base().config().timeout_seconds, 30);
    }
}
