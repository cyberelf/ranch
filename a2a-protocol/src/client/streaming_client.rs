//! Type-safe streaming client for compile-time guarantees

#![cfg(feature = "streaming")]

use crate::{
    client::{
        transport::{StreamingResult, StreamingTransport},
        A2aClient,
    },
    A2aResult, Message, TaskResubscribeRequest,
};
use futures_util::stream::Stream;
use std::ops::Deref;
use std::sync::Arc;

/// A type-safe A2A client that guarantees streaming support at compile time
///
/// Unlike `A2aClient` which doesn't support streaming, this client is generic over
/// a `StreamingTransport` type parameter, ensuring at compile time that the transport
/// supports streaming operations.
///
/// This client implements `Deref<Target = A2aClient>`, which means you can call any
/// base client method directly on the streaming client without needing to call `.base()`.
///
/// # Type Safety Benefits
///
/// - **Compile-time verification**: The compiler ensures your transport implements `StreamingTransport`
/// - **No runtime downcasting**: Direct access to streaming methods without any checks
/// - **Ergonomic API**: Access base client methods via `Deref` automatically
/// - **Clear intent**: Using this type signals that streaming is required for your use case
///
/// # Example
///
/// ```no_run
/// use a2a_protocol::prelude::*;
/// use a2a_protocol::client::{A2aStreamingClient, transport::JsonRpcTransport};
/// use std::sync::Arc;
///
/// # async fn example() -> A2aResult<()> {
/// // Create a streaming transport
/// let transport = Arc::new(JsonRpcTransport::new("https://agent.example.com/rpc")?);
///
/// // Create a typed streaming client - compile-time guarantee of streaming support
/// let client = A2aStreamingClient::new(transport);
///
/// // Access base client operations directly via Deref (no .base() needed!)
/// let message = Message::user_text("Hello");
/// client.send_message(message).await?;
///
/// // Use streaming operations directly - no runtime checks needed
/// let stream_msg = Message::user_text("Stream this");
/// let mut stream = client.stream_message(stream_msg).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct A2aStreamingClient<T: StreamingTransport + ?Sized> {
    /// Base client for non-streaming operations
    base: A2aClient,
    /// Typed transport reference for streaming operations
    transport: Arc<T>,
}

impl<T: StreamingTransport + 'static> Deref for A2aStreamingClient<T> {
    type Target = A2aClient;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<T: StreamingTransport + 'static> A2aStreamingClient<T> {
    /// Create a new streaming client with a typed transport
    ///
    /// The transport type parameter `T` must implement `StreamingTransport`,
    /// which is verified at compile time.
    ///
    /// # Example
    ///
    /// ```
    /// # use a2a_protocol::prelude::*;
    /// # use a2a_protocol::client::{A2aStreamingClient, transport::JsonRpcTransport};
    /// # use std::sync::Arc;
    /// let transport = Arc::new(JsonRpcTransport::new("https://agent.example.com/rpc").unwrap());
    /// let client = A2aStreamingClient::new(transport);
    ///
    /// // Base client methods are accessible via Deref
    /// assert_eq!(client.transport_type(), "json-rpc");
    /// ```
    pub fn new(transport: Arc<T>) -> Self {
        // Create base client with type-erased transport for common operations
        let base = A2aClient::new(transport.clone());
        Self { base, transport }
    }

    /// Get a reference to the underlying base `A2aClient`
    ///
    /// This is rarely needed since `A2aStreamingClient` implements `Deref<Target = A2aClient>`,
    /// which means you can call base client methods directly.
    ///
    /// # Example
    ///
    /// ```
    /// # use a2a_protocol::prelude::*;
    /// # use a2a_protocol::client::{A2aStreamingClient, transport::JsonRpcTransport};
    /// # use std::sync::Arc;
    /// # let transport = Arc::new(JsonRpcTransport::new("https://example.com/rpc").unwrap());
    /// let client = A2aStreamingClient::new(transport);
    ///
    /// // These are equivalent:
    /// assert_eq!(client.transport_type(), "json-rpc");
    /// assert_eq!(client.base().transport_type(), "json-rpc");
    /// ```
    pub fn base(&self) -> &A2aClient {
        &self.base
    }

    /// Get a reference to the typed transport
    ///
    /// This provides direct access to the underlying transport for advanced use cases.
    pub fn transport(&self) -> &Arc<T> {
        &self.transport
    }

    /// Send a message and receive a stream of responses
    ///
    /// This method directly calls the transport's `send_streaming_message()` without
    /// any runtime type checks, as the compiler has already verified the transport
    /// implements `StreamingTransport`.
    ///
    /// # Returns
    ///
    /// A stream that emits:
    /// - `StreamingResult::Message` - Immediate message responses
    /// - `StreamingResult::Task` - Task creation or completion
    /// - `StreamingResult::TaskStatusUpdate` - Task status changes
    /// - `StreamingResult::TaskArtifactUpdate` - Task artifact notifications
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use a2a_protocol::prelude::*;
    /// # use a2a_protocol::prelude::*;
    /// # use a2a_protocol::client::{A2aStreamingClient, transport::{JsonRpcTransport, StreamingResult}};
    /// # use futures_util::StreamExt;
    /// # use std::sync::Arc;
    /// # async fn example() -> A2aResult<()> {
    /// # let transport = Arc::new(JsonRpcTransport::new("https://example.com/rpc")?);
    /// let client = A2aStreamingClient::new(transport);
    ///
    /// // Send streaming message
    /// let message = Message::user_text("Process this data");
    /// let mut stream = client.stream_message(message).await?;
    ///
    /// // Or use base client method directly via Deref
    /// let blocking_msg = Message::user_text("Quick question");
    /// let response = client.send_message(blocking_msg).await?;
    ///
    /// while let Some(result) = stream.next().await {
    ///     match result? {
    ///         StreamingResult::Task(task) => {
    ///             println!("Task created: {}", task.id);
    ///         }
    ///         StreamingResult::TaskStatusUpdate(update) => {
    ///             println!("Status: {:?}", update.status);
    ///         }
    ///         StreamingResult::TaskArtifactUpdate(artifact) => {
    ///             println!("Artifact ID: {}", artifact.artifact_id);
    ///         }
    ///         StreamingResult::Message(msg) => {
    ///             println!("Message: {:?}", msg.text_content());
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn stream_message(
        &self,
        message: Message,
    ) -> A2aResult<Box<dyn Stream<Item = A2aResult<StreamingResult>> + Send + Unpin>> {
        self.transport.send_streaming_message(message).await
    }

    /// Send a text message and receive a stream of responses
    ///
    /// Convenience method that creates a text message and streams it.
    pub async fn stream_text<S: Into<String>>(
        &self,
        text: S,
    ) -> A2aResult<Box<dyn Stream<Item = A2aResult<StreamingResult>> + Send + Unpin>> {
        let message = Message::user_text(text);
        self.stream_message(message).await
    }

    /// Resubscribe to an existing task's event stream
    ///
    /// This method allows you to resume receiving updates for a task, optionally
    /// from a specific event using the `lastEventId` in the request metadata.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use a2a_protocol::prelude::*;
    /// # use a2a_protocol::prelude::*;
    /// # use a2a_protocol::client::{A2aStreamingClient, transport::{JsonRpcTransport, StreamingResult}};
    /// # use futures_util::StreamExt;
    /// # use std::sync::Arc;
    /// # async fn example() -> A2aResult<()> {
    /// # let transport = Arc::new(JsonRpcTransport::new("https://example.com/rpc")?);
    /// let client = A2aStreamingClient::new(transport);
    ///
    /// // Resubscribe with optional last event ID for resumption
    /// let request = TaskResubscribeRequest::new("task-123")
    ///     .with_metadata(serde_json::json!({"lastEventId": "event-456"}));
    ///
    /// let mut stream = client.resubscribe_task(request).await?;
    ///
    /// while let Some(result) = stream.next().await {
    ///     match result? {
    ///         StreamingResult::TaskStatusUpdate(update) => {
    ///             println!("Status: {:?}", update.status);
    ///         }
    ///         _ => {}
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn resubscribe_task(
        &self,
        request: TaskResubscribeRequest,
    ) -> A2aResult<Box<dyn Stream<Item = A2aResult<StreamingResult>> + Send + Unpin>> {
        self.transport.resubscribe_task(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::transport::JsonRpcTransport;

    #[test]
    fn test_streaming_client_creation() {
        let transport = Arc::new(JsonRpcTransport::new("https://example.com/rpc").unwrap());
        let client = A2aStreamingClient::new(transport);

        // Access base client method via Deref
        assert_eq!(client.transport_type(), "json-rpc");
    }

    #[test]
    fn test_deref_to_base_client() {
        let transport = Arc::new(JsonRpcTransport::new("https://example.com/rpc").unwrap());
        let client = A2aStreamingClient::new(transport);

        // These should be equivalent
        assert_eq!(client.transport_type(), "json-rpc");
        assert_eq!(client.base().transport_type(), "json-rpc");
    }

    #[test]
    fn test_compile_time_type_safety() {
        // This test verifies that A2aStreamingClient only accepts StreamingTransport
        let transport = Arc::new(JsonRpcTransport::new("https://example.com/rpc").unwrap());
        
        // This compiles because JsonRpcTransport implements StreamingTransport
        let _client = A2aStreamingClient::new(transport);
        
        // If we tried to use a non-streaming transport, this wouldn't compile:
        // let bad_transport = Arc::new(SomeNonStreamingTransport::new());
        // let _client = A2aStreamingClient::new(bad_transport); // Compile error!
    }
}
