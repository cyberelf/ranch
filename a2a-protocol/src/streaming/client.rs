//! Streaming client for A2A protocol

use crate::{Message, A2aResult, A2aError};
use tokio::sync::mpsc;

/// Streaming client for real-time message processing
pub struct StreamingClient {
    /// Sender for streaming messages
    sender: mpsc::UnboundedSender<Message>,

    /// Receiver for streaming responses
    receiver: mpsc::UnboundedReceiver<String>,
}

impl StreamingClient {
    /// Create a new streaming client
    pub fn new() -> (Self, mpsc::UnboundedReceiver<Message>) {
        let (sender, client_receiver) = mpsc::unbounded_channel();
        let (_server_sender, receiver) = mpsc::unbounded_channel();

        let client = Self {
            sender,
            receiver,
        };

        (client, client_receiver)
    }

    /// Send a message to be streamed
    pub fn send_message(&self, message: Message) -> A2aResult<()> {
        self.sender.send(message)
            .map_err(|_| A2aError::Transport("Failed to send message".to_string()))
    }

    /// Check if the client is connected
    pub fn is_connected(&self) -> bool {
        !self.sender.is_closed()
    }

    /// Close the streaming connection
    pub fn close(self) -> A2aResult<()> {
        // The connection will be closed when the sender is dropped
        Ok(())
    }
}

/// Builder for creating streaming clients
pub struct StreamingClientBuilder {
    /// Buffer size for message parts
    buffer_size: usize,

    /// Timeout for streaming operations
    timeout_seconds: u64,

    /// Enable WebSocket support
    enable_websocket: bool,
}

impl StreamingClientBuilder {
    /// Create a new streaming client builder
    pub fn new() -> Self {
        Self {
            buffer_size: 1000,
            timeout_seconds: 30,
            enable_websocket: true,
        }
    }

    /// Set the buffer size
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    /// Enable or disable WebSocket support
    pub fn with_websocket(mut self, enable: bool) -> Self {
        self.enable_websocket = enable;
        self
    }

    /// Build the streaming client
    pub fn build(self) -> A2aResult<StreamingClient> {
        let (client, _) = StreamingClient::new();
        Ok(client)
    }
}

impl Default for StreamingClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_streaming_client_creation() {
        let (client, _receiver) = StreamingClient::new();

        assert!(client.is_connected());

        let message = crate::Message::new_text("user", "Hello");
        client.send_message(message).unwrap();
    }

    #[test]
    fn test_streaming_client_builder() {
        let builder = StreamingClientBuilder::new()
            .with_buffer_size(2000)
            .with_timeout(60)
            .with_websocket(false);

        assert_eq!(builder.buffer_size, 2000);
        assert_eq!(builder.timeout_seconds, 60);
        assert!(!builder.enable_websocket);
    }
}