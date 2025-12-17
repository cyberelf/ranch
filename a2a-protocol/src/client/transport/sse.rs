//! Client-side SSE (Server-Sent Events) utilities
//!
//! Re-exports shared SSE types from core.

// Re-export shared SSE types from core
pub use crate::core::{EventBuffer, SseEvent, SseEventId};
