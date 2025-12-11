//! Team Server Example
//!
//! This example demonstrates how to expose a team as an A2A-compliant
//! HTTP service using TeamServer. The server implements JSON-RPC 2.0
//! and supports all standard A2A protocol methods.
//!
//! ## What this example demonstrates:
//! - Creating and configuring a team
//! - Starting TeamServer to expose the team via HTTP
//! - Handling graceful shutdown (Ctrl+C)
//! - Testing the server with curl commands
//!
//! ## Running this example:
//! ```bash
//! cargo run --example team_server
//! ```
//!
//! ## Testing the server:
//! Once the server is running, you can test it with curl:
//!
//! ```bash
//! # Get agent card (team capabilities)
//! curl -X POST http://localhost:8080/rpc \
//!   -H "Content-Type: application/json" \
//!   -d '{
//!     "jsonrpc": "2.0",
//!     "method": "agent/card",
//!     "params": {},
//!     "id": 1
//!   }'
//!
//! # Send a message to the team
//! curl -X POST http://localhost:8080/rpc \
//!   -H "Content-Type: application/json" \
//!   -d '{
//!     "jsonrpc": "2.0",
//!     "method": "message/send",
//!     "params": {
//!       "message": {
//!         "role": "user",
//!         "content": [{"type": "text", "text": "Hello team!"}]
//!       }
//!     },
//!     "id": 2
//!   }'
//! ```

use a2a_protocol::prelude::Message;
use async_trait::async_trait;
use multi_agent::team::{
    SchedulerConfig, SupervisorSchedulerConfig, TeamAgentConfig, TeamConfig, TeamMode,
};
use multi_agent::Agent;
use multi_agent::*;
use std::sync::Arc;
use tokio::signal;

/// A helpful assistant agent
struct AssistantAgent {
    id: String,
    name: String,
}

impl AssistantAgent {
    fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl Agent for AssistantAgent {
    async fn info(&self) -> A2aResult<AgentInfo> {
        Ok(AgentInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            description: format!("{} - A helpful assistant", self.name),
            capabilities: vec!["assistance".to_string(), "support".to_string()],
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn process(&self, message: Message) -> A2aResult<Message> {
        let input = extract_text(&message).unwrap_or_default();
        println!("  [{}] Processing: {}", self.name, input);

        let response = format!(
            "Hello! I'm {} and I'm happy to help you. Your message was: \"{}\"",
            self.name, input
        );

        Ok(Message::agent_text(&response))
    }
}

/// Coordinator agent that routes requests
struct CoordinatorAgent {
    id: String,
}

impl CoordinatorAgent {
    fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

#[async_trait]
impl Agent for CoordinatorAgent {
    async fn info(&self) -> A2aResult<AgentInfo> {
        Ok(AgentInfo {
            id: self.id.clone(),
            name: "Coordinator".to_string(),
            description: "Routes requests to appropriate team members".to_string(),
            capabilities: vec!["coordination".to_string(), "routing".to_string()],
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn process(&self, message: Message) -> A2aResult<Message> {
        let input = extract_text(&message).unwrap_or_default();
        println!("  [Coordinator] Received request: {}", input);

        // Simple routing decision
        Ok(Message::agent_text(
            "Coordinator: Processing your request and routing to appropriate assistant",
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Team Server Example");
    println!("======================\n");

    // Create agent manager
    println!("📋 Setting up agent manager...");
    let manager = Arc::new(AgentManager::new());

    // Create agents
    println!("🔧 Creating team agents...");
    let coordinator = Arc::new(CoordinatorAgent::new("coordinator"));
    let assistant1 = Arc::new(AssistantAgent::new("assistant-1", "Assistant Alpha"));
    let assistant2 = Arc::new(AssistantAgent::new("assistant-2", "Assistant Beta"));

    // Register agents
    println!("📝 Registering agents...");
    let coordinator_id = manager.register(coordinator as Arc<dyn Agent>).await?;
    let assistant1_id = manager.register(assistant1 as Arc<dyn Agent>).await?;
    let assistant2_id = manager.register(assistant2 as Arc<dyn Agent>).await?;

    println!("  ✓ Registered: {}", coordinator_id);
    println!("  ✓ Registered: {}", assistant1_id);
    println!("  ✓ Registered: {}", assistant2_id);

    // Create team configuration
    println!("\n🏢 Creating team...");
    let team_config = TeamConfig {
        id: "support-team".to_string(),
        name: "Support Team".to_string(),
        description: "A helpful team with coordinator and assistants".to_string(),
        mode: TeamMode::Supervisor,
        agents: vec![
            TeamAgentConfig {
                agent_id: coordinator_id.clone(),
                role: "coordinator".to_string(),
                capabilities: vec!["coordination".to_string()],
            },
            TeamAgentConfig {
                agent_id: assistant1_id,
                role: "assistant".to_string(),
                capabilities: vec!["assistance".to_string()],
            },
            TeamAgentConfig {
                agent_id: assistant2_id,
                role: "assistant".to_string(),
                capabilities: vec!["assistance".to_string()],
            },
        ],
        scheduler_config: SchedulerConfig::Supervisor(SupervisorSchedulerConfig {
            supervisor_agent_id: coordinator_id,
        }),
    };

    let team = Arc::new(Team::new(team_config, manager));

    // Display team information
    println!("\n📊 Team Configuration:");
    let team_info = team.info().await?;
    println!("  ID: {}", team_info.id);
    println!("  Name: {}", team_info.name);
    println!("  Description: {}", team_info.description);
    println!(
        "  Members: {}",
        team_info
            .metadata
            .get("member_count")
            .unwrap_or(&"0".to_string())
    );
    println!("  Capabilities: {}", team_info.capabilities.join(", "));

    // Create TeamServer
    let port = 8080;
    let server = TeamServer::new(team, port);

    println!("\n🧪 Test Commands:");
    println!("─────────────────────────────────────────────────────────────────");
    println!("\n1. Get agent card (team capabilities):");
    println!("   curl -X POST http://localhost:{}/rpc \\", port);
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{");
    println!("       \"jsonrpc\": \"2.0\",");
    println!("       \"method\": \"agent/card\",");
    println!("       \"params\": {{}},");
    println!("       \"id\": 1");
    println!("     }}'");

    println!("\n2. Send a message to the team:");
    println!("   curl -X POST http://localhost:{}/rpc \\", port);
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{");
    println!("       \"jsonrpc\": \"2.0\",");
    println!("       \"method\": \"message/send\",");
    println!("       \"params\": {{");
    println!("         \"message\": {{");
    println!("           \"role\": \"user\",");
    println!("           \"content\": [{{\"type\": \"text\", \"text\": \"Hello team!\"}}]");
    println!("         }}");
    println!("       }},");
    println!("       \"id\": 2");
    println!("     }}'");

    println!("\n3. Get task status (replace TASK_ID with actual task ID from previous response):");
    println!("   curl -X POST http://localhost:{}/rpc \\", port);
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{");
    println!("       \"jsonrpc\": \"2.0\",");
    println!("       \"method\": \"task/status\",");
    println!("       \"params\": {{\"task_id\": \"TASK_ID\"}},");
    println!("       \"id\": 3");
    println!("     }}'");

    println!("\n─────────────────────────────────────────────────────────────────");
    println!("\n💡 Press Ctrl+C to stop the server gracefully\n");

    // Setup graceful shutdown handler
    let shutdown_signal = async {
        signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C signal handler");
        println!("\n\n🛑 Shutdown signal received, stopping server...");
    };

    // Start server with graceful shutdown
    tokio::select! {
        result = server.start() => {
            if let Err(e) = result {
                eprintln!("❌ Server error: {}", e);
                return Err(e);
            }
        }
        _ = shutdown_signal => {
            println!("✅ Server stopped gracefully");
        }
    }

    Ok(())
}
