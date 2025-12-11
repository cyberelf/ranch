//! Client-side SSE (Server-Sent Events) utilities
//!
//! Re-exports shared SSE types from core.
#![cfg(feature = "streaming")]

// Re-export shared SSE types from core
pub use crate::core::{EventBuffer, SseEvent, SseEventId};
