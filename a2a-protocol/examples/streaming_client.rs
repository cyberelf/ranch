//! Example A2A streaming client using SSE
//!
//! This demonstrates how to use the streaming client to receive real-time
//! updates from an A2A server. Make sure to run streaming_server first.
//!
//! Run with: cargo run --example streaming_client --features streaming

use a2a_protocol::{
    client::{
        transport::{JsonRpcTransport, StreamingResult},
        A2aStreamingClient,
    },
    prelude::*,
};
use futures_util::StreamExt;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌊 A2A Streaming Client");
    println!("======================\n");

    // Create a streaming-capable transport
    let transport = Arc::new(JsonRpcTransport::new("http://localhost:3001/rpc")?);

    // Create a type-safe streaming client
    let client = A2aStreamingClient::new(transport);

    println!("✓ Streaming client created for http://localhost:3001/rpc");
    println!("  Transport type: {}\n", client.transport_type());

    // First, test connectivity with a regular (non-streaming) message
    println!("📡 Testing connectivity with regular message...");
    let test_message = Message::user_text("ping");
    match client.send_message(test_message).await {
        Ok(response) => {
            println!("✓ Connected successfully!");
            match response {
                SendResponse::Message(msg) => {
                    if let Some(text) = msg.text_content() {
                        println!("  Server response: {}\n", text);
                    }
                }
                SendResponse::Task(_) => {
                    println!("  Server returned a task\n");
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to connect to server!");
            println!("   Error: {}\n", e);
            println!("💡 Make sure the streaming server is running in another terminal:");
            println!("   cargo run --example streaming_server --features streaming\n");
            return Ok(());
        }
    }

    // Now demonstrate streaming
    println!("🌊 Starting streaming request...");
    println!("   Message: 'process this data with streaming'");

    let message = Message::user_text("process this data with streaming");

    match client.stream_message(message).await {
        Ok(mut stream) => {
            println!("✓ Stream established! Receiving events...\n");

            let mut event_count = 0;
            while let Some(result) = stream.next().await {
                event_count += 1;
                match result {
                    Ok(streaming_result) => match streaming_result {
                        StreamingResult::Message(msg) => {
                            println!("📥 Event #{}: Message", event_count);
                            if let Some(text) = msg.text_content() {
                                println!("   Content: {}\n", text);
                            }
                        }
                        StreamingResult::Task(task) => {
                            println!("📥 Event #{}: Task", event_count);
                            println!("   ID: {}", task.id);
                            println!("   Status: {:?}\n", task.status.state);
                        }
                        StreamingResult::TaskStatusUpdate(update) => {
                            println!("📥 Event #{}: Task Status Update", event_count);
                            println!("   Task ID: {}", update.task_id);
                            println!("   Status: {:?}", update.status.state);
                            if let Some(msg) = &update.status.message {
                                if let Some(text) = msg.text_content() {
                                    println!("      Message: {}", text);
                                }
                            }
                            println!();
                        }
                        StreamingResult::TaskArtifactUpdate(artifact) => {
                            println!("📥 Event #{}: Task Artifact Update", event_count);
                            println!("   Task ID: {}", artifact.task_id);
                            println!("   Artifact ID: {}", artifact.artifact_id);
                            println!("   Type: {}", artifact.artifact_type);
                            if let Some(metadata) = artifact.metadata {
                                println!("   Metadata: {}", metadata);
                            }
                            println!();
                        }
                    },
                    Err(e) => {
                        println!("❌ Error receiving event: {}", e);
                        break;
                    }
                }
            }

            println!("✓ Stream complete! Received {} events", event_count);
        }
        Err(e) => {
            println!("❌ Failed to establish stream: {}", e);
            println!("\n💡 Troubleshooting:");
            println!("   - Make sure the server supports streaming");
            println!("   - Check that the 'streaming' feature is enabled");
            println!("   - Verify the server endpoint is correct");
        }
    }

    println!("\n✓ Demo complete!");

    Ok(())
}
