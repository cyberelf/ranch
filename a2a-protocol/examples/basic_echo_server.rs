//! Example using the simplified AgentLogic trait
//!
//! This demonstrates how easy it is to create an agent using the AgentLogic trait
//! instead of implementing the full A2aHandler.
//!
//! Run with: cargo run --example basic_echo_server --features streaming

use a2a_protocol::{
    prelude::*,
    server::{AgentLogic, ServerBuilder, TaskAwareHandler},
};
use async_trait::async_trait;
use url::Url;

/// A simple echo agent that uppercases the input
struct UppercaseEchoAgent;

#[async_trait]
impl AgentLogic for UppercaseEchoAgent {
    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        // Extract text from the message
        let text = message
            .text_content()
            .ok_or_else(|| A2aError::Validation("No text content in message".to_string()))?;

        // Process it (uppercase) and return
        let response = format!("ECHO: {}", text.to_uppercase());
        Ok(Message::agent_text(response))
    }

    async fn initialize(&self) -> A2aResult<()> {
        println!("🤖 UppercaseEchoAgent initialized!");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create agent card
    let agent_id = AgentId::new("uppercase-echo".to_string())?;
    let agent_card = AgentCard::new(
        agent_id,
        "Uppercase Echo Agent",
        Url::parse("https://example.com")?,
    );

    // 2. Create our simple agent logic
    let agent = UppercaseEchoAgent;

    // 3. Wrap it in TaskAwareHandler to get full A2A support
    let handler = TaskAwareHandler::with_logic(agent_card, agent);

    // 4. Initialize the agent
    // Note: In a real app, you'd call this in your startup code
    // handler.logic.initialize().await?;

    // 5. Build and run server
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
