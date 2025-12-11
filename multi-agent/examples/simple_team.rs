//! Simple Team Example
//!
//! This example demonstrates the basics of creating a team with two agents
//! and processing a message through the team.
//!
//! ## What this example demonstrates:
//! - Creating mock agents for testing/demonstration
//! - Registering agents with AgentManager
//! - Creating a simple two-agent team
//! - Processing messages through the team
//! - Team capability aggregation
//!
//! ## Running this example:
//! ```bash
//! cargo run --example simple_team
//! ```

use multi_agent::*;
use multi_agent::team::{TeamAgentConfig, TeamConfig, TeamMode, SupervisorSchedulerConfig, SchedulerConfig};
use a2a_protocol::prelude::Message;
use multi_agent::Agent;
use async_trait::async_trait;
use std::sync::Arc;

/// A simple mock agent for demonstration
struct SimpleAgent {
    id: String,
    name: String,
    response: String,
}

impl SimpleAgent {
    fn new(id: &str, name: &str, response: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            response: response.to_string(),
        }
    }
}

#[async_trait]
impl Agent for SimpleAgent {
    async fn info(&self) -> A2aResult<AgentInfo> {
        Ok(AgentInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            description: format!("A simple agent named {}", self.name),
            capabilities: vec!["general".to_string()],
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn process(&self, message: Message) -> A2aResult<Message> {
        let input = extract_text(&message)
            .unwrap_or_default();
        
        println!("  [{}] Received: {}", self.name, input);
        println!("  [{}] Responding: {}", self.name, self.response);
        
        Ok(Message::agent_text(&self.response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Simple Team Example");
    println!("=====================\n");

    // Create agent manager
    println!("📋 Creating agent manager...");
    let manager = Arc::new(AgentManager::new());
    
    // Create two simple agents
    println!("🔧 Creating agents...");
    let agent1 = Arc::new(SimpleAgent::new(
        "agent-1",
        "Agent One",
        "Hello! I'm Agent One, nice to meet you!"
    ));
    
    let agent2 = Arc::new(SimpleAgent::new(
        "agent-2",
        "Agent Two",
        "Greetings! I'm Agent Two, here to help!"
    ));
    
    // Register agents
    println!("📝 Registering agents...");
    let agent1_id = manager.register(agent1 as Arc<dyn Agent>).await?;
    let agent2_id = manager.register(agent2 as Arc<dyn Agent>).await?;
    println!("  ✓ Registered: {}", agent1_id);
    println!("  ✓ Registered: {}", agent2_id);
    
    // Create team configuration
    println!("\n🏢 Creating team...");
    let team_config = TeamConfig {
        id: "simple-team".to_string(),
        name: "Simple Team".to_string(),
        description: "A simple two-agent team".to_string(),
        mode: TeamMode::Supervisor,
        agents: vec![
            TeamAgentConfig {
                agent_id: agent1_id.clone(),
                role: "coordinator".to_string(),
                capabilities: vec!["general".to_string()],
            },
            TeamAgentConfig {
                agent_id: agent2_id.clone(),
                role: "assistant".to_string(),
                capabilities: vec!["general".to_string()],
            },
        ],
        scheduler_config: SchedulerConfig::Supervisor(SupervisorSchedulerConfig {
            supervisor_agent_id: agent1_id,
        }),
    };
    
    let team = Arc::new(Team::new(team_config, manager));
    
    // Display team information
    println!("\n📊 Team Information:");
    let team_info = team.info().await?;
    println!("  ID: {}", team_info.id);
    println!("  Name: {}", team_info.name);
    println!("  Description: {}", team_info.description);
    println!("  Capabilities: {}", team_info.capabilities.join(", "));
    println!("  Team Type: {}", team_info.metadata.get("type").unwrap_or(&"unknown".to_string()));
    println!("  Team Mode: {}", team_info.metadata.get("mode").unwrap_or(&"unknown".to_string()));
    println!("  Members: {}", team_info.metadata.get("member_count").unwrap_or(&"0".to_string()));
    
    // Process a message through the team
    println!("\n💬 Processing message through team...");
    let message = Message::user_text("Hello team!");
    println!("  User: Hello team!");
    
    let response = team.process(message).await?;
    let response_text = extract_text(&response)
        .unwrap_or_default();
    
    println!("\n✨ Final Response:");
    println!("  {}", response_text);
    
    println!("\n✅ Example complete!");
    
    Ok(())
}
