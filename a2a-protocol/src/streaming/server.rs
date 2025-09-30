//! Streaming server for A2A protocol

use crate::{Message, MessagePart, A2aResult, A2aError};
use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use std::pin::Pin;
use tokio::sync::mpsc;

/// Trait for handling streaming A2A messages
#[async_trait]
pub trait StreamingA2aHandler: Send + Sync {
    /// Handle a streaming message request
    async fn handle_streaming_message(
        &self,
        message: Message,
    ) -> A2aResult<Pin<Box<dyn Stream<Item = MessagePart> + Send>>>;

    /// Handle bidirectional streaming
    async fn handle_bidirectional_stream(
        &self,
        request_stream: Pin<Box<dyn Stream<Item = Message> + Send>>,
    ) -> A2aResult<Pin<Box<dyn Stream<Item = MessagePart> + Send>>>;
}

/// Streaming server for A2A protocol
pub struct StreamingServer {
    handler: Box<dyn StreamingA2aHandler>,
}

impl StreamingServer {
    /// Create a new streaming server
    pub fn new<H>(handler: H) -> Self
    where
        H: StreamingA2aHandler + 'static,
    {
        Self {
            handler: Box::new(handler),
        }
    }

    /// Handle a streaming request
    pub async fn handle_streaming_request(
        &self,
        message: Message,
    ) -> A2aResult<Pin<Box<dyn Stream<Item = MessagePart> + Send>>> {
        self.handler.handle_streaming_message(message).await
    }

    /// Handle a bidirectional streaming request
    pub async fn handle_bidirectional_request(
        &self,
        request_stream: Pin<Box<dyn Stream<Item = Message> + Send>>,
    ) -> A2aResult<Pin<Box<dyn Stream<Item = MessagePart> + Send>>> {
        self.handler.handle_bidirectional_stream(request_stream).await
    }
}

/// Basic streaming handler implementation
pub struct BasicStreamingHandler {
    agent_card: crate::AgentCard,
}

impl BasicStreamingHandler {
    /// Create a new basic streaming handler
    pub fn new(agent_card: crate::AgentCard) -> Self {
        Self { agent_card }
    }

    /// Stream text response character by character
    async fn stream_text_response(
        &self,
        text: String,
    ) -> Pin<Box<dyn Stream<Item = MessagePart> + Send>> {
        let (sender, mut receiver) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            // Stream word by word for demonstration
            let words: Vec<&str> = text.split_whitespace().collect();

            for word in words {
                let part = MessagePart {
                    content_type: "text".to_string(),
                    content: format!("{} ", word),
                    metadata: None,
                };

                if sender.send(part).is_err() {
                    break;
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });

        Box::pin(async_stream::stream! {
            while let Some(part) = receiver.recv().await {
                yield part;
            }
        })
    }
}

#[async_trait]
impl StreamingA2aHandler for BasicStreamingHandler {
    async fn handle_streaming_message(
        &self,
        message: Message,
    ) -> A2aResult<Pin<Box<dyn Stream<Item = MessagePart> + Send>>> {
        let response_text = match message.text_content() {
            Some(content) => format!("Streaming response to: {}", content),
            None => "Streaming response to empty message".to_string(),
        };

        Ok(self.stream_text_response(response_text).await)
    }

    async fn handle_bidirectional_stream(
        &self,
        mut request_stream: Pin<Box<dyn Stream<Item = Message> + Send>>,
    ) -> A2aResult<Pin<Box<dyn Stream<Item = MessagePart> + Send>>> {
        let (sender, receiver) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            while let Some(message) = request_stream.next().await {
                if let Some(content) = message.text_content() {
                    let response = MessagePart {
                        content_type: "text".to_string(),
                        content: format!("Echo: {}\n", content),
                        metadata: None,
                    };

                    if sender.send(response).is_err() {
                        break;
                    }
                }
            }
        });

        // TODO: Implement proper streaming with async_stream
      // For now, return empty stream
      Ok(Box::pin(futures_util::stream::empty()))
    }
}

/// WebSocket streaming server
pub struct WebSocketStreamingServer {
    streaming_server: StreamingServer,
}

impl WebSocketStreamingServer {
    /// Create a new WebSocket streaming server
    pub fn new<H>(handler: H) -> Self
    where
        H: StreamingA2aHandler + 'static,
    {
        Self {
            streaming_server: StreamingServer::new(handler),
        }
    }

    /// Handle WebSocket upgrade for streaming
    pub async fn handle_websocket_upgrade(
        &self,
        websocket: tokio_tungstenite::WebSocketStream<hyper::upgrade::Upgraded>,
    ) -> A2aResult<()> {
        use futures_util::SinkExt;
        use tokio_tungstenite::tungstenite::Message as WsMessage;

        let (mut ws_sender, mut ws_receiver) = websocket.split();

        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    // Parse the incoming message
                    let message: Message = serde_json::from_str(&text)
                        .map_err(|e| A2aError::Json(e))?;

                    // Handle the streaming request
                    let response_stream = self.streaming_server
                        .handle_streaming_request(message)
                        .await?;

                    // Stream the response back
                    tokio::pin!(response_stream);

                    while let Some(part) = response_stream.next().await {
                        let ws_msg = WsMessage::Text(serde_json::to_string(&part).unwrap());
                        ws_sender.send(ws_msg).await
                            .map_err(|e| A2aError::Transport(format!("WebSocket send error: {}", e)))?;
                    }
                }
                Ok(WsMessage::Close(_)) => {
                    break;
                }
                Err(e) => {
                    return Err(A2aError::Transport(format!("WebSocket error: {}", e)));
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AgentId, MessageId};
    use url::Url;

    #[tokio::test]
    async fn test_basic_streaming_handler() {
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let agent_card = crate::AgentCard::new(agent_id, "Test Agent",
            Url::parse("https://example.com").unwrap());

        let handler = BasicStreamingHandler::new(agent_card);
        let server = StreamingServer::new(handler);

        let message = Message::new_text("user", "Hello, world!");
        let mut response_stream = server.handle_streaming_request(message).await.unwrap();

        let first_part = response_stream.next().await;
        assert!(first_part.is_some());
    }

    #[tokio::test]
    async fn test_bidirectional_streaming() {
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let agent_card = crate::AgentCard::new(agent_id, "Test Agent",
            Url::parse("https://example.com").unwrap());

        let handler = BasicStreamingHandler::new(agent_card);
        let server = StreamingServer::new(handler);

        let (sender, mut receiver) = mpsc::unbounded_channel();
        sender.send(Message::new_text("user", "Test")).unwrap();

        let request_stream = Box::pin(async_stream::stream! {
            while let Some(msg) = receiver.recv().await {
                yield msg;
            }
        });

        let mut response_stream = server.handle_bidirectional_request(request_stream).await.unwrap();
        let first_part = response_stream.next().await;
        assert!(first_part.is_some());
    }
}