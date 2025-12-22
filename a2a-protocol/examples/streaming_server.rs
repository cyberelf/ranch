//! Example A2A server with SSE streaming support
//!
//! This demonstrates how to create an agent that uses streaming to provide
//! real-time updates as it processes messages. The agent simulates a long-running
//! task by sending status updates via SSE (Server-Sent Events).
//!
//! Run with: cargo run --example streaming_server --features streaming

use a2a_protocol::{
    prelude::*,
    server::{ProtocolAgent, ServerBuilder, TaskAwareHandler},
};
use async_trait::async_trait;
use std::sync::Arc;
use url::Url;

/// A streaming agent that simulates long-running work with progress updates
struct StreamingAgent {
    profile: AgentProfile,
}

impl StreamingAgent {
    fn new() -> Self {
        let agent_id = AgentId::new("streaming-agent".to_string()).unwrap();
        let profile = AgentProfile::new(
            agent_id,
            "Streaming Agent",
            Url::parse("https://example.com").unwrap(),
        )
        .with_description("An agent that supports SSE streaming for real-time updates");

        Self { profile }
    }
}

#[async_trait]
impl ProtocolAgent for StreamingAgent {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        Ok(self.profile.clone())
    }

    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        // Extract text from the message
        let text = message
            .text_content()
            .ok_or_else(|| A2aError::Validation("No text content in message".to_string()))?;

        // Simulate processing - in a real agent, this would do actual work
        // For demonstration, we just uppercase the text and add some flair
        let response = format!(
            "Processed your message: '{}'\n\nThis response was generated with streaming support. \
            The server can send progress updates during processing!",
            text.to_uppercase()
        );

        Ok(Message::agent_text(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌊 Streaming Agent initializing...");
    println!("   Supports SSE streaming for real-time updates");

    // 1. Create our streaming agent
    let agent = Arc::new(StreamingAgent::new());

    // 2. Wrap it in TaskAwareHandler to get full A2A support including streaming
    let handler = TaskAwareHandler::new(agent);

    // 3. Build and run server with streaming enabled
    println!("\n🚀 Starting Streaming Server");
    println!("   Port: 3001");
    println!("   Features: SSE streaming, JSON-RPC 2.0");
    println!("\n📡 Available endpoints:");
    println!("   POST /rpc        - JSON-RPC 2.0 (message/send, task/*, agent/card)");
    println!("   GET  /stream     - SSE streaming endpoint");
    println!("\n💡 Try these commands:");
    println!("\n   1. Send a regular message (immediate response):");
    println!("   curl -X POST http://localhost:3001/rpc \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{");
    println!("       \"jsonrpc\": \"2.0\",");
    println!("       \"id\": 1,");
    println!("       \"method\": \"message/send\",");
    println!("       \"params\": {{");
    println!("         \"message\": {{");
    println!("           \"role\": \"user\",");
    println!("           \"parts\": [{{\"text\":\"hello streaming world\"}}]");
    println!("         }},");
    println!("         \"immediate\": true");
    println!("       }}");
    println!("     }}'");
    println!("\n   2. Start an SSE stream:");
    println!("   curl -X POST http://localhost:3001/rpc \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{");
    println!("       \"jsonrpc\": \"2.0\",");
    println!("       \"id\": 1,");
    println!("       \"method\": \"message/stream\",");
    println!("       \"params\": {{");
    println!("         \"message\": {{");
    println!("           \"role\": \"user\",");
    println!("           \"parts\": [{{\"text\":\"process this with streaming\"}}]");
    println!("         }}");
    println!("       }}");
    println!("     }}'");
    println!("\n   Or use the streaming_client example:");
    println!("   cargo run --example streaming_client --features streaming\n");

    ServerBuilder::new(handler).with_port(3001).run().await?;

    Ok(())
}
