//! Example using the simplified AgentLogic trait
//!
//! This demonstrates how easy it is to create an agent using the AgentLogic trait
//! instead of implementing the full A2aHandler.
//!
//! Run with: cargo run --example basic_echo_server --features streaming

use a2a_protocol::{
    prelude::*,
    server::{Agent, ServerBuilder, TaskAwareHandler},
};
use async_trait::async_trait;
use std::sync::Arc;
use url::Url;

/// A simple echo agent that uppercases the input
struct UppercaseEchoAgent {
    profile: AgentProfile,
}

impl UppercaseEchoAgent {
    fn new() -> Self {
        let agent_id = AgentId::new("uppercase-echo".to_string()).unwrap();
        let profile = AgentProfile::new(
            agent_id,
            "Uppercase Echo Agent",
            Url::parse("https://example.com").unwrap(),
        )
        .with_description("An agent that echoes messages in uppercase");

        Self { profile }
    }
}

#[async_trait]
impl Agent for UppercaseEchoAgent {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        Ok(self.profile.clone())
    }

    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        // Extract text from the message
        let text = message
            .text_content()
            .ok_or_else(|| A2aError::Validation("No text content in message".to_string()))?;
        println!("Received message: {}", text);
        // Process it (uppercase) and return
        let response = format!("ECHO: {}", text.to_uppercase());
        Ok(Message::agent_text(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 UppercaseEchoAgent initializing...");

    // 1. Create our agent
    let agent = Arc::new(UppercaseEchoAgent::new());

    // 2. Wrap it in TaskAwareHandler to get full A2A support
    let handler = TaskAwareHandler::with_immediate_responses(agent);

    // 3. Build and run server
    println!("\n🚀 Starting Uppercase Echo Server");
    println!("   Try sending a message:");
    println!("   curl -X POST http://localhost:3000/rpc \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{");
    println!("       \"jsonrpc\": \"2.0\",");
    println!("       \"id\": 1,");
    println!("       \"method\": \"message/send\",");
    println!("       \"params\": {{");
    println!("         \"message\": {{");
    println!("           \"role\": \"user\",");
    println!("           \"parts\": [{{\"kind\":\"text\",\"text\":\"hello world\"}}]");
    println!("         }}");
    println!("       }}");
    println!("     }}'\n");

    ServerBuilder::new(handler)
        .with_port(3000)
        .run()
        .await?;

    Ok(())
}
