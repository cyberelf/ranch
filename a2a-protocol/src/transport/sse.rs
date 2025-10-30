//! SSE (Server-Sent Events) streaming implementation
//!
//! This module provides SSE transport infrastructure for streaming over HTTP
//! using the W3C Server-Sent Events protocol. It is protocol-agnostic and can
//! be used with JSON-RPC or other protocols.
//!
//! Provides both client-side parsing and server-side streaming infrastructure.
#![cfg(feature = "streaming")]

use crate::core::{A2aError, A2aResult};
use crate::transport::StreamingResult;
use async_stream::stream;
use axum::response::{IntoResponse, Response, Sse};
use axum::response::sse::{Event as AxumSseEvent, KeepAlive};
use futures_util::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::broadcast;

/// SSE Event ID for replay and ordering
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SseEventId(String);

impl SseEventId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn sequential(counter: u64) -> Self {
        Self(counter.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SseEventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A complete SSE event with metadata (W3C Server-Sent Events format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseEvent {
    /// Event ID for replay
    pub id: Option<SseEventId>,
    
    /// Event type (e.g., "task-status-update")
    pub event_type: Option<String>,
    
    /// Event data (JSON)
    pub data: serde_json::Value,
    
    /// Reconnection timeout in milliseconds
    pub retry: Option<u64>,
}

impl SseEvent {
    /// Create a new SSE event
    pub fn new(data: serde_json::Value) -> Self {
        Self {
            id: None,
            event_type: None,
            data,
            retry: None,
        }
    }

    /// Set the event ID
    pub fn with_id(mut self, id: SseEventId) -> Self {
        self.id = Some(id);
        self
    }

    /// Set the event type
    pub fn with_event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = Some(event_type.into());
        self
    }

    /// Set the retry timeout
    pub fn with_retry(mut self, retry_ms: u64) -> Self {
        self.retry = Some(retry_ms);
        self
    }

    /// Format as W3C SSE text
    pub fn to_sse_format(&self) -> String {
        let mut output = String::new();

        if let Some(ref id) = self.id {
            output.push_str(&format!("id: {}\n", id));
        }

        if let Some(ref event_type) = self.event_type {
            output.push_str(&format!("event: {}\n", event_type));
        }

        if let Some(retry) = self.retry {
            output.push_str(&format!("retry: {}\n", retry));
        }

        // Format data - split multiline JSON into multiple data: lines
        let data_str = serde_json::to_string(&self.data).unwrap_or_default();
        for line in data_str.lines() {
            output.push_str(&format!("data: {}\n", line));
        }

        output.push('\n'); // Double newline to end event
        output
    }

    /// Parse from SSE format text
    pub fn from_sse_format(text: &str) -> Result<Self, String> {
        let mut id = None;
        let mut event_type = None;
        let mut data_lines = Vec::new();
        let mut retry = None;

        for line in text.lines() {
            if line.is_empty() {
                continue;
            }

            if let Some(value) = line.strip_prefix("id:") {
                id = Some(SseEventId::new(value.trim()));
            } else if let Some(value) = line.strip_prefix("event:") {
                event_type = Some(value.trim().to_string());
            } else if let Some(value) = line.strip_prefix("data:") {
                data_lines.push(value.trim());
            } else if let Some(value) = line.strip_prefix("retry:") {
                retry = value.trim().parse().ok();
            }
        }

        let data_str = data_lines.join("\n");
        let data = serde_json::from_str(&data_str)
            .map_err(|e| format!("Failed to parse event data: {}", e))?;

        Ok(Self {
            id,
            event_type,
            data,
            retry,
        })
    }
}

/// SSE response builder for Axum server
pub struct SseResponse {
    stream: Pin<Box<dyn Stream<Item = Result<AxumSseEvent, A2aError>> + Send>>,
    keepalive: Option<Duration>,
}

impl SseResponse {
    /// Create a new SSE response from a stream of StreamingResult
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = A2aResult<StreamingResult>> + Send + 'static,
    {
        let event_stream = stream.map(|result| {
            result.and_then(|sr| {
                let (event_type, data) = match sr {
                    StreamingResult::Message(msg) => {
                        ("message", serde_json::to_value(&msg)?)
                    }
                    StreamingResult::Task(task) => {
                        ("task", serde_json::to_value(&task)?)
                    }
                    StreamingResult::TaskStatusUpdate(event) => {
                        ("task-status-update", serde_json::to_value(&event)?)
                    }
                    StreamingResult::TaskArtifactUpdate(event) => {
                        ("task-artifact-update", serde_json::to_value(&event)?)
                    }
                };

                let data_str = serde_json::to_string(&data)?;

                Ok(AxumSseEvent::default()
                    .event(event_type)
                    .data(data_str))
            })
        });

        Self {
            stream: Box::pin(event_stream),
            keepalive: Some(Duration::from_secs(30)),
        }
    }

    /// Set keep-alive interval
    pub fn with_keepalive(mut self, duration: Duration) -> Self {
        self.keepalive = Some(duration);
        self
    }

    /// Disable keep-alive
    pub fn without_keepalive(mut self) -> Self {
        self.keepalive = None;
        self
    }
}

impl IntoResponse for SseResponse {
    fn into_response(self) -> Response {
        let mut sse = Sse::new(self.stream);
        
        if let Some(keepalive) = self.keepalive {
            sse = sse.keep_alive(KeepAlive::new().interval(keepalive));
        }

        sse.into_response()
    }
}

/// SSE event writer for manual stream control (server-side)
pub struct SseWriter {
    tx: broadcast::Sender<StreamingResult>,
    event_counter: Arc<RwLock<u64>>,
}

impl SseWriter {
    pub fn new(buffer_capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(buffer_capacity);
        Self {
            tx,
            event_counter: Arc::new(RwLock::new(0)),
        }
    }

    pub fn send(&self, result: StreamingResult) -> A2aResult<()> {
        self.tx.send(result)
            .map_err(|_| A2aError::Internal("No active subscribers".to_string()))?;
        
        let mut counter = self.event_counter.write().unwrap();
        *counter += 1;
        
        Ok(())
    }

    pub fn event_count(&self) -> u64 {
        *self.event_counter.read().unwrap()
    }

    pub fn subscribe(&self) -> impl Stream<Item = A2aResult<StreamingResult>> {
        let mut rx = self.tx.subscribe();
        stream! {
            loop {
                match rx.recv().await {
                    Ok(result) => yield Ok(result),
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("SSE stream lagged by {} events", n);
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Clone for SseWriter {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            event_counter: self.event_counter.clone(),
        }
    }
}

/// Event buffer for replay support (Last-Event-ID)
pub struct EventBuffer {
    buffer: Arc<RwLock<VecDeque<(SseEventId, StreamingResult)>>>,
    max_size: usize,
    counter: Arc<RwLock<u64>>,
}

impl EventBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(VecDeque::with_capacity(max_size))),
            max_size,
            counter: Arc::new(RwLock::new(0)),
        }
    }

    pub fn push(&self, result: StreamingResult) -> SseEventId {
        let mut counter = self.counter.write().unwrap();
        *counter += 1;
        let id = SseEventId::sequential(*counter);

        let mut buffer = self.buffer.write().unwrap();
        buffer.push_back((id.clone(), result));

        while buffer.len() > self.max_size {
            buffer.pop_front();
        }

        id
    }

    pub fn get_events_after(&self, last_id: &SseEventId) -> Vec<(SseEventId, StreamingResult)> {
        let buffer = self.buffer.read().unwrap();
        
        let start_pos = buffer.iter().position(|(id, _)| id == last_id)
            .map(|pos| pos + 1)
            .unwrap_or(0);

        buffer.iter()
            .skip(start_pos)
            .cloned()
            .collect()
    }

    pub fn len(&self) -> usize {
        self.buffer.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.read().unwrap().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{TaskStatus, TaskState};
    use crate::core::streaming_events::TaskStatusUpdateEvent;

    #[test]
    fn test_sse_event_id() {
        let id1 = SseEventId::sequential(1);
        let id2 = SseEventId::sequential(2);
        assert_eq!(id1.as_str(), "1");
        assert_eq!(id2.as_str(), "2");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_sse_writer() {
        let writer = SseWriter::new(10);
        assert_eq!(writer.event_count(), 0);
        assert_eq!(writer.subscriber_count(), 0);

        let status = TaskStatus::new(TaskState::Working);
        let event = StreamingResult::TaskStatusUpdate(
            TaskStatusUpdateEvent::new("task_1", status)
        );

        // Need a subscriber before we can send
        let _stream = writer.subscribe();
        assert_eq!(writer.subscriber_count(), 1);

        writer.send(event).unwrap();
        assert_eq!(writer.event_count(), 1);
    }

    #[test]
    fn test_event_buffer() {
        let buffer = EventBuffer::new(3);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());

        let status = TaskStatus::new(TaskState::Working);

        let id1 = buffer.push(StreamingResult::TaskStatusUpdate(
            TaskStatusUpdateEvent::new("task_1", status.clone())
        ));
        let _id2 = buffer.push(StreamingResult::TaskStatusUpdate(
            TaskStatusUpdateEvent::new("task_2", status.clone())
        ));
        
        assert_eq!(buffer.len(), 2);

        let events_after_1 = buffer.get_events_after(&id1);
        assert_eq!(events_after_1.len(), 1);

        // Add more to trigger overflow
        buffer.push(StreamingResult::TaskStatusUpdate(
            TaskStatusUpdateEvent::new("task_3", status.clone())
        ));
        buffer.push(StreamingResult::TaskStatusUpdate(
            TaskStatusUpdateEvent::new("task_4", status)
        ));
        
        assert_eq!(buffer.len(), 3); // Capped at max_size
    }

    #[test]
    fn test_sse_event_format() {
        let event = SseEvent::new(serde_json::json!({"message": "hello"}))
            .with_id(SseEventId::new("123"))
            .with_event_type("test")
            .with_retry(3000);

        let formatted = event.to_sse_format();
        assert!(formatted.contains("id: 123"));
        assert!(formatted.contains("event: test"));
        assert!(formatted.contains("retry: 3000"));
        assert!(formatted.contains("data: {\"message\":\"hello\"}"));
    }

    #[test]
    fn test_sse_event_parse() {
        let text = "id: 456\nevent: update\ndata: {\"status\":\"ok\"}\n\n";
        let event = SseEvent::from_sse_format(text).unwrap();
        
        assert_eq!(event.id.unwrap().as_str(), "456");
        assert_eq!(event.event_type.unwrap(), "update");
        assert_eq!(event.data["status"], "ok");
    }
}
