//! Team Client Example
//!
//! This example demonstrates the CORRECT usage of multi-agent:
//! - Connect to REMOTE A2A agent servers via A2AAgent
//! - Form teams of remote agents for collaboration
//! - Use dynamic routing with Client Agent Extension
//!
//! **Prerequisites:**
//! Start the agent servers first: `cargo run --example agent_servers`
//!
//! **Architecture:**
//! - Agent servers run as separate processes (agent_servers.rs)
//! - This client connects to those servers via A2A protocol
//! - Team coordinates remote agents without implementing any agents locally
//!
//! **Usage:**
//! ```bash
//! # Terminal 1: Start agent servers
//! cargo run --example agent_servers
//!
//! # Terminal 2: Run this client
//! cargo run --example team_client
//! ```

use multi_agent::{
    agent::{A2AAgent, Agent},
    team::{RouterConfig, Team, TeamAgentConfig, TeamConfig},
    AgentManager,
};
use a2a_protocol::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤝 Multi-Agent Team Client");
    println!("===========================\n");

    // Step 1: Connect to remote agent servers via A2A protocol
    println!("📡 Connecting to remote agent servers...\n");

    let router_transport = Arc::new(JsonRpcTransport::new("http://localhost:3000/rpc")?);
    let router_client = A2aClient::new(router_transport);
    let router_agent = Arc::new(A2AAgent::new(router_client));

    let worker_transport = Arc::new(JsonRpcTransport::new("http://localhost:3001/rpc")?);
    let worker_client = A2aClient::new(worker_transport);
    let worker_agent = Arc::new(A2AAgent::new(worker_client));

    let supervisor_transport = Arc::new(JsonRpcTransport::new("http://localhost:3002/rpc")?);
    let supervisor_client = A2aClient::new(supervisor_transport);
    let supervisor_agent = Arc::new(A2AAgent::new(supervisor_client));

    // Step 2: Register remote agents in the manager
    println!("📋 Registering remote agents...");
    let manager = Arc::new(AgentManager::new());
    
    manager.register(router_agent.clone()).await?;
    println!("   ✓ Router Agent registered");
    
    manager.register(worker_agent.clone()).await?;
    println!("   ✓ Worker Agent registered");
    
    manager.register(supervisor_agent.clone()).await?;
    println!("   ✓ Supervisor Agent registered\n");

    // Step 3: Display agent information
    println!("👥 Remote Agent Details:");
    for info in manager.list_info().await {
        println!("   • {} ({})", info.name, info.id);
        println!("     Skills: {}", 
            info.skills.iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!();

    // Step 4: Create team configuration with dynamic routing
    let team_config = TeamConfig {
        id: "remote-team".to_string(),
        name: "Remote Agent Team".to_string(),
        description: "Team coordinating remote A2A agents".to_string(),
        agents: vec![
            TeamAgentConfig {
                agent_id: "router".to_string(),
                role: "coordinator".to_string(),
                capabilities: vec![],
            },
            TeamAgentConfig {
                agent_id: "worker".to_string(),
                role: "worker".to_string(),
                capabilities: vec![],
            },
            TeamAgentConfig {
                agent_id: "supervisor".to_string(),
                role: "supervisor".to_string(),
                capabilities: vec![],
            },
        ],
        router_config: RouterConfig {
            default_agent_id: "router".to_string(),
            max_routing_hops: 10,
        },
    };

    // Step 5: Form the team
    println!("🎯 Forming team with remote agents...");
    let team = Team::new(team_config, manager);
    println!("   ✓ Team formed successfully\n");

    // Step 6: Process messages through the team
    println!("💬 Sending message to team...");
    println!("   Message: 'I need help with a task'\n");
    
    let message = Message::user_text("I need help with a task");
    let response = team.process(message).await?;
    
    println!("📨 Response:");
    println!("   {}\n", response.text_content().unwrap_or("No text content"));

    // Demonstrate another message
    println!("💬 Sending another message...");
    println!("   Message: 'Thank you!'\n");
    
    let message2 = Message::user_text("Thank you!");
    let response2 = team.process(message2).await?;
    
    println!("📨 Response:");
    println!("   {}\n", response2.text_content().unwrap_or("No text content"));

    println!("✅ Demo complete!");
    println!("\nKey Points:");
    println!("  • All agents are REMOTE - running as A2A protocol servers");
    println!("  • multi-agent only provides CLIENT-SIDE coordination");
    println!("  • No local agent implementations in this example");
    println!("  • Routing decisions made by remote router agent");
    println!("  • Extension context injected by team coordinator");

    Ok(())
}
