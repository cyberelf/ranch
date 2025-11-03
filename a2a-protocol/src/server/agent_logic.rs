//! Simplified agent logic trait for easy implementation
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

use crate::{A2aResult, Message};
use async_trait::async_trait;

/// Simplified trait for implementing agent message processing logic
///
/// This trait provides a much simpler interface than `A2aHandler` for users
/// who just want to process messages without dealing with the full A2A protocol
/// complexity. The trait requires only a single method: `process_message`.
///
/// # Example
///
/// ```
/// use a2a_protocol::prelude::*;
/// use a2a_protocol::server::AgentLogic;
/// use async_trait::async_trait;
///
/// struct EchoAgent;
///
/// #[async_trait]
/// impl AgentLogic for EchoAgent {
///     async fn process_message(&self, message: Message) -> A2aResult<Message> {
///         // Simply echo back the input
///         Ok(Message::agent_text(
///             message.text_content().unwrap_or("").to_string()
///         ))
///     }
/// }
/// ```
///
/// # Integration with TaskAwareHandler
///
/// You can wrap any `AgentLogic` implementation in a `TaskAwareHandler` to get
/// full A2A protocol support:
///
/// ```no_run
/// # use a2a_protocol::prelude::*;
/// # use a2a_protocol::server::{AgentLogic, TaskAwareHandler, ServerBuilder};
/// # use async_trait::async_trait;
/// # use url::Url;
/// # struct MyAgent;
/// #[async_trait]
/// # impl AgentLogic for MyAgent {
/// #     async fn process_message(&self, msg: Message) -> A2aResult<Message> {
/// #         Ok(Message::agent_text("response"))
/// #     }
/// # }
/// # async fn example() {
/// let agent_id = AgentId::new("my-agent".to_string()).unwrap();
/// let agent_card = AgentCard::new(
///     agent_id,
///     "My Agent",
///     Url::parse("https://example.com").unwrap()
/// );
///
/// // Your simple agent logic
/// let logic = MyAgent;
///
/// // Wrap it to get full A2A support
/// let handler = TaskAwareHandler::with_logic(agent_card, logic);
///
/// // Start server
/// ServerBuilder::new(handler)
///     .with_port(3000)
///     .run()
///     .await.ok();
/// # }
/// ```
#[async_trait]
pub trait AgentLogic: Send + Sync {
    /// Process an incoming message and return a response
    ///
    /// This is the only method you need to implement to create a working agent.
    /// The message will be processed and a response returned synchronously.
    ///
    /// # Arguments
    ///
    /// * `message` - The incoming message to process
    ///
    /// # Returns
    ///
    /// A message containing the agent's response.
    ///
    /// # Errors
    ///
    /// Return an error if the message cannot be processed. Common error types:
    /// - `A2aError::Validation` - Invalid message format
    /// - `A2aError::Server` - Internal processing error
    /// - `A2aError::UnsupportedOperation` - Operation not supported
    ///
    /// # Example
    ///
    /// ```
    /// # use a2a_protocol::prelude::*;
    /// # use a2a_protocol::server::AgentLogic;
    /// # use async_trait::async_trait;
    /// struct UppercaseAgent;
    ///
    /// #[async_trait]
    /// impl AgentLogic for UppercaseAgent {
    ///     async fn process_message(&self, message: Message) -> A2aResult<Message> {
    ///         let text = message.text_content()
    ///             .ok_or_else(|| A2aError::Validation("No text in message".to_string()))?;
    ///         
    ///         Ok(Message::agent_text(text.to_uppercase()))
    ///     }
    /// }
    /// ```
    async fn process_message(&self, message: Message) -> A2aResult<Message>;

    /// Optional: Handle initialization/warmup
    ///
    /// This method is called once when the agent is started. Use it for:
    /// - Loading models or resources
    /// - Establishing database connections
    /// - Pre-computing data
    ///
    /// Default implementation does nothing.
    async fn initialize(&self) -> A2aResult<()> {
        Ok(())
    }

    /// Optional: Handle shutdown/cleanup
    ///
    /// This method is called when the agent is shutting down. Use it for:
    /// - Closing connections
    /// - Saving state
    /// - Cleaning up resources
    ///
    /// Default implementation does nothing.
    async fn shutdown(&self) -> A2aResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Part;

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
    async fn test_agent_logic_initialization() {
        struct InitAgent {
            initialized: std::sync::Arc<std::sync::atomic::AtomicBool>,
        }

        #[async_trait]
        impl AgentLogic for InitAgent {
            async fn process_message(&self, message: Message) -> A2aResult<Message> {
                Ok(message)
            }

            async fn initialize(&self) -> A2aResult<()> {
                self.initialized.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
        }

        let initialized = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let agent = InitAgent {
            initialized: initialized.clone(),
        };

        agent.initialize().await.unwrap();
        assert!(initialized.load(std::sync::atomic::Ordering::SeqCst));
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
