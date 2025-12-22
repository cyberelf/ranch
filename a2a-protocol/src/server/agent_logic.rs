//! Trait for defining agent-specific logic
//!
//! This module provides a simplified `AgentLogic` trait that focuses on the core
//! message processing logic, without requiring users to understand all the A2A
//! protocol details like tasks, streaming, or RPC methods.
//!
//! ## When to Use Which Trait?
//!
//! - **Use `AgentLogic`** if:
//!   - You want simple message-in, message-out processing
//!   - You don't need long-running tasks or streaming
//!   - You're building a simple agent or getting started
//!
//! - **Use `A2aHandler`** if:
//!   - You need full control over task management
//!   - You want to implement custom streaming behavior
//!   - You need to handle all RPC methods directly

use crate::{server::AgentProfile, A2aResult, Message};
use async_trait::async_trait;

/// Defines the core logic for an agent to process messages
///
/// This trait can be implemented to create custom agent behaviors that can be
/// used with `TaskAwareHandler::with_logic`.
#[async_trait]
pub trait AgentLogic: Send + Sync {
    /// Process an incoming message and return a response message
    async fn process_message(&self, msg: Message) -> A2aResult<Message>;
}

/// Defines the core behavior and metadata for an agent
///
/// This trait unifies agent logic and metadata, providing a clean separation between
/// descriptive agent attributes (profile) and transport-level capabilities (added by handler).
#[async_trait]
pub trait ProtocolAgent: Send + Sync {
    /// Returns the agent's descriptive profile.
    ///
    /// This method returns only the agent's identity, skills, and capabilities
    /// without transport-level details. The handler layer is responsible for
    /// adding transport capabilities (streaming, push notifications, auth, etc.)
    /// and assembling the complete `AgentCard`.
    async fn profile(&self) -> A2aResult<AgentProfile>;

    /// Processes an incoming message and returns a response.
    async fn process_message(&self, msg: Message) -> A2aResult<Message>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestAgent {
        prefix: String,
    }

    #[async_trait]
    impl AgentLogic for TestAgent {
        async fn process_message(&self, message: Message) -> A2aResult<Message> {
            let text = message.text_content().unwrap_or("");
            Ok(Message::agent_text(format!("{}: {}", self.prefix, text)))
        }
    }

    #[tokio::test]
    async fn test_agent_logic_basic() {
        let agent = TestAgent {
            prefix: "Echo".to_string(),
        };

        let input = Message::user_text("Hello");
        let output = agent.process_message(input).await.unwrap();

        assert_eq!(output.text_content().unwrap(), "Echo: Hello");
    }

    #[tokio::test]
    async fn test_agent_logic_error_handling() {
        struct ErrorAgent;

        #[async_trait]
        impl AgentLogic for ErrorAgent {
            async fn process_message(&self, _message: Message) -> A2aResult<Message> {
                Err(crate::A2aError::Server("Intentional error".to_string()))
            }
        }

        let agent = ErrorAgent;
        let result = agent.process_message(Message::user_text("test")).await;
        assert!(result.is_err());
    }
}
