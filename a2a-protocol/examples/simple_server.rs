//! Simple A2A server example using ServerBuilder
//!
//! This demonstrates how easy it is to create an A2A server with the new ServerBuilder API.
//! Compare this to the more complex `server.rs` example to see the improvement!
//!
//! Run with: cargo run --example simple_server --features streaming

use a2a_protocol::{
    prelude::*,
    server::{Agent, ServerBuilder, TaskAwareHandler},
};
use async_trait::async_trait;
use std::sync::Arc;
use url::Url;

/// A simple echo agent
struct SimpleAgent {
    profile: AgentProfile,
}

impl SimpleAgent {
    fn new() -> Self {
        let agent_id = AgentId::new("simple-agent".to_string()).unwrap();
        let profile = AgentProfile::new(
            agent_id,
            "Simple A2A Agent",
            Url::parse("https://example.com").unwrap(),
        );
        Self { profile }
    }
}

#[async_trait]
impl Agent for SimpleAgent {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        Ok(self.profile.clone())
    }

    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        let text = message.text_content().unwrap_or("No text");
        Ok(Message::agent_text(format!("Echo: {}", text)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create agent
    let agent = Arc::new(SimpleAgent::new());

    // 2. Create handler
    let handler = TaskAwareHandler::new(agent);

    // 3. Build and run server - that's it!
    ServerBuilder::new(handler)
        .with_port(3000)
        .run()
        .await?;

    Ok(())
}
