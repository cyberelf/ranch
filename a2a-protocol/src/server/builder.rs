//! Server builder for simplified server creation
//!
//! This module provides a fluent builder API for creating A2A servers with minimal boilerplate.
//! It hides the complexity of axum, tokio, and JSON-RPC setup, allowing users to focus on
//! implementing their agent logic.

#![cfg(feature = "json-rpc")]

use crate::{
    server::{handler::A2aHandler, json_rpc::axum::JsonRpcRouter},
    A2aError, A2aResult,
};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

/// Builder for creating an A2A server with fluent configuration
///
/// This builder provides a simple, ergonomic way to create and configure an A2A server
/// without dealing directly with axum, tokio, or JSON-RPC implementation details.
///
/// # Example
///
/// ```no_run
/// # use a2a_protocol::prelude::*;
/// # use a2a_protocol::server::{Agent, ServerBuilder, TaskAwareHandler};
/// # use url::Url;
/// # use std::sync::Arc;
/// # use async_trait::async_trait;
/// # struct MyAgent { profile: AgentProfile }
/// # #[async_trait]
/// # impl Agent for MyAgent {
/// #     async fn profile(&self) -> A2aResult<AgentProfile> { Ok(self.profile.clone()) }
/// #     async fn process_message(&self, msg: Message) -> A2aResult<Message> { Ok(Message::agent_text("response")) }
/// # }
/// # async fn example() {
/// // Create your agent
/// let agent_id = AgentId::new("my-agent".to_string()).unwrap();
/// let profile = AgentProfile::new(
///     agent_id,
///     "My Agent",
///     Url::parse("https://example.com").unwrap()
/// );
/// let agent = Arc::new(MyAgent { profile });
/// let handler = TaskAwareHandler::new(agent);
///
/// // Build and run server in one expression
/// ServerBuilder::new(handler)
///     .with_port(3000)
///     .run()
///     .await.ok();
/// # }
/// ```
pub struct ServerBuilder<H: A2aHandler> {
    handler: H,
    address: SocketAddr,
}

impl<H: A2aHandler + 'static> ServerBuilder<H> {
    /// Create a new server builder with the given handler
    ///
    /// By default, the server will listen on `127.0.0.1:3000`.
    ///
    /// # Example
    ///
    /// ```
    /// # use a2a_protocol::prelude::*;
    /// # use a2a_protocol::server::{Agent, ServerBuilder, TaskAwareHandler};
    /// # use url::Url;
    /// # use std::sync::Arc;
    /// # use async_trait::async_trait;
    /// # struct MyAgent { profile: AgentProfile }
    /// # #[async_trait]
    /// # impl Agent for MyAgent {
    /// #     async fn profile(&self) -> A2aResult<AgentProfile> { Ok(self.profile.clone()) }
    /// #     async fn process_message(&self, msg: Message) -> A2aResult<Message> { Ok(Message::agent_text("response")) }
    /// # }
    /// let agent_id = AgentId::new("my-agent".to_string()).unwrap();
    /// let profile = AgentProfile::new(
    ///     agent_id,
    ///     "My Agent",
    ///     Url::parse("https://example.com").unwrap()
    /// );
    /// let agent = Arc::new(MyAgent { profile });
    /// let handler = TaskAwareHandler::new(agent);
    /// let builder = ServerBuilder::new(handler);
    /// ```
    pub fn new(handler: H) -> Self {
        Self {
            handler,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000),
        }
    }

    /// Set the socket address to bind to
    ///
    /// # Example
    ///
    /// ```
    /// # use a2a_protocol::server::{Agent, ServerBuilder, TaskAwareHandler};
    /// # use a2a_protocol::prelude::*;
    /// # use url::Url;
    /// # use std::{net::SocketAddr, sync::Arc};
    /// # use async_trait::async_trait;
    /// # struct MyAgent { profile: AgentProfile }
    /// # #[async_trait]
    /// # impl Agent for MyAgent {
    /// #     async fn profile(&self) -> A2aResult<AgentProfile> { Ok(self.profile.clone()) }
    /// #     async fn process_message(&self, msg: Message) -> A2aResult<Message> { Ok(Message::agent_text("response")) }
    /// # }
    /// # let agent_id = AgentId::new("my-agent".to_string()).unwrap();
    /// # let profile = AgentProfile::new(agent_id, "My Agent", Url::parse("https://example.com").unwrap());
    /// # let agent = Arc::new(MyAgent { profile });
    /// # let handler = TaskAwareHandler::new(agent);
    /// let builder = ServerBuilder::new(handler)
    ///     .with_address("0.0.0.0:8080".parse().unwrap());
    /// ```
    pub fn with_address(mut self, address: SocketAddr) -> Self {
        self.address = address;
        self
    }

    /// Set the host and port separately
    ///
    /// # Example
    ///
    /// ```
    /// # use a2a_protocol::server::{Agent, ServerBuilder, TaskAwareHandler};
    /// # use a2a_protocol::prelude::*;
    /// # use url::Url;
    /// # use std::sync::Arc;
    /// # use async_trait::async_trait;
    /// # struct MyAgent { profile: AgentProfile }
    /// # #[async_trait]
    /// # impl Agent for MyAgent {
    /// #     async fn profile(&self) -> A2aResult<AgentProfile> { Ok(self.profile.clone()) }
    /// #     async fn process_message(&self, msg: Message) -> A2aResult<Message> { Ok(Message::agent_text("response")) }
    /// # }
    /// # let agent_id = AgentId::new("my-agent".to_string()).unwrap();
    /// # let profile = AgentProfile::new(agent_id, "My Agent", Url::parse("https://example.com").unwrap());
    /// # let agent = Arc::new(MyAgent { profile });
    /// # let handler = TaskAwareHandler::new(agent);
    /// let builder = ServerBuilder::new(handler)
    ///     .with_host_port("0.0.0.0", 8080);
    /// ```
    pub fn with_host_port(mut self, host: &str, port: u16) -> Self {
        let ip: IpAddr = host.parse().unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        self.address = SocketAddr::new(ip, port);
        self
    }

    /// Set only the port (keeps the default host 127.0.0.1)
    ///
    /// This is the most common configuration method for local development.
    ///
    /// # Example
    ///
    /// ```
    /// # use a2a_protocol::server::{Agent, ServerBuilder, TaskAwareHandler};
    /// # use a2a_protocol::prelude::*;
    /// # use url::Url;
    /// # use std::sync::Arc;
    /// # use async_trait::async_trait;
    /// # struct MyAgent { profile: AgentProfile }
    /// # #[async_trait]
    /// # impl Agent for MyAgent {
    /// #     async fn profile(&self) -> A2aResult<AgentProfile> { Ok(self.profile.clone()) }
    /// #     async fn process_message(&self, msg: Message) -> A2aResult<Message> { Ok(Message::agent_text("response")) }
    /// # }
    /// # let agent_id = AgentId::new("my-agent".to_string()).unwrap();
    /// # let profile = AgentProfile::new(agent_id, "My Agent", Url::parse("https://example.com").unwrap());
    /// # let agent = Arc::new(MyAgent { profile });
    /// # let handler = TaskAwareHandler::new(agent);
    /// let builder = ServerBuilder::new(handler)
    ///     .with_port(8080);
    /// ```
    pub fn with_port(mut self, port: u16) -> Self {
        self.address.set_port(port);
        self
    }

    /// Build and run the server
    ///
    /// This method consumes the builder, creates the axum router, binds to the
    /// configured address, and starts serving requests. This is an async function
    /// that will run until the server is shut down.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The address is already in use
    /// - The address cannot be bound
    /// - The server encounters a fatal error
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use a2a_protocol::prelude::*;
    /// # use a2a_protocol::server::{Agent, ServerBuilder, TaskAwareHandler};
    /// # use url::Url;
    /// # use std::sync::Arc;
    /// # use async_trait::async_trait;
    /// # struct MyAgent { profile: AgentProfile }
    /// # #[async_trait]
    /// # impl Agent for MyAgent {
    /// #     async fn profile(&self) -> A2aResult<AgentProfile> { Ok(self.profile.clone()) }
    /// #     async fn process_message(&self, msg: Message) -> A2aResult<Message> { Ok(Message::agent_text("response")) }
    /// # }
    /// # async fn example() {
    /// let agent_id = AgentId::new("my-agent".to_string()).unwrap();
    /// let profile = AgentProfile::new(
    ///     agent_id,
    ///     "My Agent",
    ///     Url::parse("https://example.com").unwrap()
    /// );
    /// let agent = Arc::new(MyAgent { profile });
    /// let handler = TaskAwareHandler::new(agent);
    ///
    /// // This will run forever (until interrupted)
    /// ServerBuilder::new(handler)
    ///     .with_port(3000)
    ///     .run()
    ///     .await.ok();
    /// # }
    /// ```
    pub async fn run(self) -> A2aResult<()> {
        // Create JSON-RPC router
        let router = JsonRpcRouter::new(self.handler).into_router();

        // Print server information
        println!("🚀 A2A Server starting on {}", self.address);
        println!();
        println!("Endpoints:");
        println!("  POST http://{}/rpc - JSON-RPC 2.0 endpoint", self.address);
        #[cfg(feature = "streaming")]
        println!("  GET  http://{}/stream - SSE streaming endpoint", self.address);
        println!();

        // Bind and serve
        let listener = tokio::net::TcpListener::bind(self.address)
            .await
            .map_err(|e| A2aError::Internal(format!("Failed to bind to {}: {}", self.address, e)))?;

        axum::serve(listener, router)
            .await
            .map_err(|e| A2aError::Internal(format!("Server error: {}", e)))?;

        Ok(())
    }

    /// Build the server and return the router without running it
    ///
    /// This is useful if you need to integrate the A2A server into an existing
    /// axum application or need more control over the server lifecycle.
    ///
    /// # Example
    ///
    /// ```
    /// # use a2a_protocol::prelude::*;
    /// # use a2a_protocol::server::{Agent, ServerBuilder, TaskAwareHandler};
    /// # use url::Url;
    /// # use std::sync::Arc;
    /// # use async_trait::async_trait;
    /// # struct MyAgent { profile: AgentProfile }
    /// # #[async_trait]
    /// # impl Agent for MyAgent {
    /// #     async fn profile(&self) -> A2aResult<AgentProfile> { Ok(self.profile.clone()) }
    /// #     async fn process_message(&self, msg: Message) -> A2aResult<Message> { Ok(Message::agent_text("response")) }
    /// # }
    /// # let agent_id = AgentId::new("my-agent".to_string()).unwrap();
    /// # let profile = AgentProfile::new(agent_id, "My Agent", Url::parse("https://example.com").unwrap());
    /// # let agent = Arc::new(MyAgent { profile });
    /// # let handler = TaskAwareHandler::new(agent);
    /// let router = ServerBuilder::new(handler)
    ///     .with_port(3000)
    ///     .build();
    ///
    /// // Now you can merge this router with other routes
    /// // or control the server lifecycle manually
    /// ```
    pub fn build(self) -> axum::Router {
        JsonRpcRouter::new(self.handler).into_router()
    }

    /// Get the configured address
    ///
    /// Useful for testing or logging purposes.
    pub fn address(&self) -> SocketAddr {
        self.address
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AgentId, AgentProfile, Message, server::{Agent, TaskAwareHandler}};
    use async_trait::async_trait;
    use std::sync::Arc;
    use url::Url;

    struct TestAgent {
        profile: AgentProfile,
    }

    #[async_trait]
    impl Agent for TestAgent {
        async fn profile(&self) -> crate::A2aResult<AgentProfile> {
            Ok(self.profile.clone())
        }

        async fn process_message(&self, msg: Message) -> crate::A2aResult<Message> {
            Ok(Message::agent_text(format!("Echo: {}", msg.text_content().unwrap_or(""))))
        }
    }

    fn create_test_handler() -> TaskAwareHandler {
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let agent_profile = AgentProfile::new(
            agent_id,
            "Test Agent",
            Url::parse("https://example.com").unwrap(),
        );
        let agent = Arc::new(TestAgent {
            profile: agent_profile,
        });
        TaskAwareHandler::new(agent)
    }

    #[test]
    fn test_builder_defaults() {
        let handler = create_test_handler();
        let builder = ServerBuilder::new(handler);
        
        assert_eq!(builder.address.ip().to_string(), "127.0.0.1");
        assert_eq!(builder.address.port(), 3000);
    }

    #[test]
    fn test_builder_with_port() {
        let handler = create_test_handler();
        let builder = ServerBuilder::new(handler).with_port(8080);
        
        assert_eq!(builder.address.port(), 8080);
        assert_eq!(builder.address.ip().to_string(), "127.0.0.1");
    }

    #[test]
    fn test_builder_with_host_port() {
        let handler = create_test_handler();
        let builder = ServerBuilder::new(handler).with_host_port("0.0.0.0", 9000);
        
        assert_eq!(builder.address.port(), 9000);
        assert_eq!(builder.address.ip().to_string(), "0.0.0.0");
    }

    #[test]
    fn test_builder_with_address() {
        let handler = create_test_handler();
        let addr: SocketAddr = "192.168.1.1:4000".parse().unwrap();
        let builder = ServerBuilder::new(handler).with_address(addr);
        
        assert_eq!(builder.address, addr);
    }

    #[test]
    fn test_builder_build() {
        let handler = create_test_handler();
        let router = ServerBuilder::new(handler)
            .with_port(3000)
            .build();
        
        // Router should be created successfully
        // We can't easily test its internal structure, but we can verify it compiles
        drop(router);
    }
}
