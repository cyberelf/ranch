//! Simple A2A server example using ServerBuilder
//!
//! This demonstrates how easy it is to create an A2A server with the new ServerBuilder API.
//! Compare this to the more complex `server.rs` example to see the improvement!
//!
//! Run with: cargo run --example simple_server --features streaming

use a2a_protocol::{
    prelude::*,
    server::{ServerBuilder, TaskAwareHandler},
};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create agent card
    let agent_id = AgentId::new("simple-agent".to_string())?;
    let agent_card = AgentCard::new(
        agent_id,
        "Simple A2A Agent",
        Url::parse("https://example.com")?,
    );

    // 2. Create handler (you can also implement your own A2aHandler)
    let handler = TaskAwareHandler::new(agent_card);

    // 3. Build and run server - that's it!
    ServerBuilder::new(handler)
        .with_port(3000)
        .run()
        .await?;

    Ok(())
}
