//! Example demonstrating compile-time type-safe streaming with A2aStreamingClient
//!
//! This example shows how to use `A2aStreamingClient<T>` for compile-time guaranteed
//! streaming support, and demonstrates the Deref pattern for accessing base client methods.

#![cfg(feature = "streaming")]

use a2a_protocol::prelude::*;
use a2a_protocol::client::{A2aClient, A2aStreamingClient};
use a2a_protocol::transport::JsonRpcTransport;
use std::sync::Arc;

#[tokio::main]
async fn main() -> A2aResult<()> {
    println!("=== Type-Safe Streaming Client Example ===\n");

    // Create a streaming-capable transport
    let transport = Arc::new(JsonRpcTransport::new("https://example.com/rpc")?);

    // Option 1: Regular A2aClient (no streaming support)
    let _regular_client = A2aClient::new(transport.clone());
    println!("✓ Created regular A2aClient");
    println!("  - Supports: send_message(), get_agent_card(), etc.");
    println!("  - Does NOT support: stream_message() (compile error!)\n");

    // Option 2: A2aStreamingClient with compile-time guarantees
    let streaming_client = A2aStreamingClient::new(transport);
    println!("✓ Created A2aStreamingClient<JsonRpcTransport>");
    println!("  - Supports: stream_message(), stream_text(), resubscribe_task()");
    println!("  - ALSO supports base methods via Deref:\n");

    // Access base client methods directly via Deref (no .base() needed!)
    println!("    streaming_client.transport_type() = {:?}", streaming_client.transport_type());
    println!("    streaming_client.agent_id() = {:?}", streaming_client.agent_id());
    
    // You can still use .base() explicitly if you prefer
    println!("    streaming_client.base().transport_type() = {:?}\n", streaming_client.base().transport_type());

    // Streaming operations - compile-time verified
    let message = Message::user_text("Hello, streaming world!");
    println!("✓ Calling stream_message() - no runtime checks needed!");
    match streaming_client.stream_message(message).await {
        Ok(_stream) => {
            println!("  Stream created successfully (would fail to connect in this example)");
        }
        Err(e) => {
            println!("  Expected error (not a real server): {}", e);
        }
    }

    println!("\n=== Type Safety Benefits ===");
    println!("1. Compile-time verification of streaming support");
    println!("2. No runtime downcasting or capability checks");
    println!("3. Ergonomic API: Deref gives access to base methods");
    println!("4. Clear separation: A2aClient vs A2aStreamingClient");
    println!("5. Generic over StreamingTransport trait");

    Ok(())
}
