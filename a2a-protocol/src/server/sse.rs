//! Server-side SSE (Server-Sent Events) streaming implementation
//!
//! Provides Axum-based SSE response types for streaming A2A protocol data to clients.

use crate::client::transport::StreamingResult;
use crate::core::{A2aError, A2aResult};
use async_stream::stream;
use axum::response::sse::{Event as AxumSseEvent, KeepAlive};
use axum::response::{IntoResponse, Response, Sse};
use futures_util::stream::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::broadcast;

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
                    StreamingResult::Message(msg) => ("message", serde_json::to_value(&msg)?),
                    StreamingResult::Task(task) => ("task", serde_json::to_value(&task)?),
                    StreamingResult::TaskStatusUpdate(event) => {
                        ("task-status-update", serde_json::to_value(&event)?)
                    }
                    StreamingResult::TaskArtifactUpdate(event) => {
                        ("task-artifact-update", serde_json::to_value(&event)?)
                    }
                };

                let data_str = serde_json::to_string(&data)?;

                Ok(AxumSseEvent::default().event(event_type).data(data_str))
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
        let sse = Sse::new(self.stream);

        if let Some(keepalive) = self.keepalive {
            sse.keep_alive(KeepAlive::new().interval(keepalive))
                .into_response()
        } else {
            sse.into_response()
        }
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
        self.tx
            .send(result)
            .map_err(|_| A2aError::Internal("No active subscribers".to_string()))?;

        let mut counter = self.event_counter.write().unwrap();
        *counter += 1;

        Ok(())
    }

    pub fn event_count(&self) -> u64 {
        *self.event_counter.read().unwrap()
    }

    pub fn subscribe(&self) -> impl Stream<Item = A2aResult<StreamingResult>> + use<> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::streaming_events::TaskStatusUpdateEvent;
    use crate::core::{TaskState, TaskStatus};

    #[test]
    fn test_sse_writer() {
        let writer = SseWriter::new(10);
        assert_eq!(writer.event_count(), 0);
        assert_eq!(writer.subscriber_count(), 0);

        let status = TaskStatus::new(TaskState::Working);
        let event = StreamingResult::TaskStatusUpdate(TaskStatusUpdateEvent::new("task_1", status));

        // Need a subscriber before we can send
        let _stream = writer.subscribe();
        assert_eq!(writer.subscriber_count(), 1);

        writer.send(event).unwrap();
        assert_eq!(writer.event_count(), 1);
    }
}
