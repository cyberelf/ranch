//! Example A2A server using JSON-RPC 2.0 (A2A v0.3.0 spec-compliant)
//!
//! This demonstrates how to create a spec-compliant A2A server using JsonRpcRouter.
//! All communication happens via JSON-RPC 2.0 protocol over HTTP POST /rpc
//!
//! Run with: cargo run --example server

use a2a_protocol::{
    server::{JsonRpcRouter, TaskAwareHandler},
    AgentId, AgentCard,
};
use std::net::SocketAddr;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create agent card
    let agent_id = AgentId::new("example-agent".to_string())?;
    let agent_card = AgentCard::new(
        agent_id,
        "Example A2A Agent",
        Url::parse("https://example.com")?,
    );

    // Create a task-aware handler
    let handler = TaskAwareHandler::new(agent_card.clone());

    // Create JSON-RPC 2.0 router (A2A spec-compliant)
    let router = JsonRpcRouter::new(handler).into_router();

    // Bind and serve
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 A2A Server listening on {}", addr);
    println!();
    println!("Specification: A2A v0.3.0");
    println!("Transport: JSON-RPC 2.0 over HTTP");
    println!();
    println!("Available RPC methods:");
    println!("  - message/send     Send a message (returns Task or Message)");
    println!("  - task/get         Get task details and results");
    println!("  - task/status      Get current task status");
    println!("  - task/cancel      Cancel a running task");
    println!("  - agent/card       Get agent card");
    println!();
    println!("Endpoint: POST http://localhost:3000/rpc");
    println!();
    println!("Example request:");
    println!(r#"  curl -X POST http://localhost:3000/rpc \
    -H "Content-Type: application/json" \
    -d '{{
      "jsonrpc": "2.0",
      "id": 1,
      "method": "message/send",
      "params": {{
        "message": {{
          "role": "user",
          "parts": [{{"kind":"text","text":"Hello, agent!"}}]
        }},
        "immediate": true
      }}
    }}'
"#);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

