//! Supervisor Team Example
//!
//! This example demonstrates the supervisor pattern where a supervisor agent
//! intelligently routes messages to specialist agents based on the message content.
//!
//! ## What this example demonstrates:
//! - Creating a supervisor agent that makes routing decisions
//! - Creating specialist agents with different capabilities
//! - Using supervisor mode for intelligent delegation
//! - Message routing based on content analysis
//!
//! ## Running this example:
//! ```bash
//! cargo run --example supervisor_team
//! ```

use a2a_protocol::prelude::Message;
use async_trait::async_trait;
use multi_agent::team::{RouterConfig, TeamAgentConfig, TeamConfig};
use multi_agent::Agent;
use multi_agent::*;
use std::sync::Arc;

/// Supervisor agent that analyzes messages and delegates to specialists
struct SupervisorAgent {
    id: String,
}

impl SupervisorAgent {
    fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

#[async_trait]
impl Agent for SupervisorAgent {
    async fn info(&self) -> A2aResult<AgentInfo> {
        Ok(AgentInfo {
            id: self.id.clone(),
            name: "Supervisor".to_string(),
            description: "Analyzes queries and routes to appropriate specialist".to_string(),
            capabilities: vec!["routing".to_string(), "coordination".to_string()],
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn process(&self, message: Message) -> A2aResult<Message> {
        let input = extract_text(&message).unwrap_or_default();

        println!("  [Supervisor] Analyzing query: \"{}\"", input);

        // Analyze the message content and determine which specialist to route to
        let specialist = if input.to_lowercase().contains("code")
            || input.to_lowercase().contains("bug")
            || input.to_lowercase().contains("implement")
        {
            "coding-specialist"
        } else if input.to_lowercase().contains("document")
            || input.to_lowercase().contains("write")
            || input.to_lowercase().contains("explain")
        {
            "documentation-specialist"
        } else if input.to_lowercase().contains("test")
            || input.to_lowercase().contains("verify")
            || input.to_lowercase().contains("quality")
        {
            "testing-specialist"
        } else {
            "coding-specialist" // default
        };

        println!("  [Supervisor] Decision: Route to {}", specialist);

        // Return a message indicating the routing decision
        // In a real implementation, this would trigger the actual routing
        Ok(Message::agent_text(format!(
            "Supervisor: Routing to {} (This is the supervisor's routing decision)",
            specialist
        )))
    }
}

/// Coding specialist agent
struct CodingSpecialist {
    id: String,
}

impl CodingSpecialist {
    fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

#[async_trait]
impl Agent for CodingSpecialist {
    async fn info(&self) -> A2aResult<AgentInfo> {
        Ok(AgentInfo {
            id: self.id.clone(),
            name: "Coding Specialist".to_string(),
            description: "Expert in code implementation and debugging".to_string(),
            capabilities: vec!["coding".to_string(), "debugging".to_string()],
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn process(&self, message: Message) -> A2aResult<Message> {
        let input = extract_text(&message).unwrap_or_default();
        println!("  [Coding Specialist] Processing: {}", input);
        Ok(Message::agent_text(
            "I can help you implement that feature with clean, efficient code!",
        ))
    }
}

/// Documentation specialist agent
struct DocumentationSpecialist {
    id: String,
}

impl DocumentationSpecialist {
    fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

#[async_trait]
impl Agent for DocumentationSpecialist {
    async fn info(&self) -> A2aResult<AgentInfo> {
        Ok(AgentInfo {
            id: self.id.clone(),
            name: "Documentation Specialist".to_string(),
            description: "Expert in writing clear documentation".to_string(),
            capabilities: vec!["documentation".to_string(), "writing".to_string()],
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn process(&self, message: Message) -> A2aResult<Message> {
        let input = extract_text(&message).unwrap_or_default();
        println!("  [Documentation Specialist] Processing: {}", input);
        Ok(Message::agent_text(
            "I'll create comprehensive documentation with examples and best practices!",
        ))
    }
}

/// Testing specialist agent
struct TestingSpecialist {
    id: String,
}

impl TestingSpecialist {
    fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

#[async_trait]
impl Agent for TestingSpecialist {
    async fn info(&self) -> A2aResult<AgentInfo> {
        Ok(AgentInfo {
            id: self.id.clone(),
            name: "Testing Specialist".to_string(),
            description: "Expert in quality assurance and testing".to_string(),
            capabilities: vec!["testing".to_string(), "quality-assurance".to_string()],
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn process(&self, message: Message) -> A2aResult<Message> {
        let input = extract_text(&message).unwrap_or_default();
        println!("  [Testing Specialist] Processing: {}", input);
        Ok(Message::agent_text(
            "I'll create comprehensive test coverage with unit, integration, and edge case tests!",
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Supervisor Team Example");
    println!("==========================\n");

    // Create agent manager
    println!("📋 Setting up agent manager...");
    let manager = Arc::new(AgentManager::new());

    // Create supervisor and specialist agents
    println!("🔧 Creating supervisor and specialist agents...");
    let supervisor = Arc::new(SupervisorAgent::new("supervisor"));
    let coding_specialist = Arc::new(CodingSpecialist::new("coding-specialist"));
    let doc_specialist = Arc::new(DocumentationSpecialist::new("documentation-specialist"));
    let test_specialist = Arc::new(TestingSpecialist::new("testing-specialist"));

    // Register all agents
    println!("📝 Registering agents...");
    let supervisor_id = manager.register(supervisor as Arc<dyn Agent>).await?;
    let coding_id = manager
        .register(coding_specialist as Arc<dyn Agent>)
        .await?;
    let doc_id = manager.register(doc_specialist as Arc<dyn Agent>).await?;
    let test_id = manager.register(test_specialist as Arc<dyn Agent>).await?;

    println!("  ✓ Registered: {}", supervisor_id);
    println!("  ✓ Registered: {}", coding_id);
    println!("  ✓ Registered: {}", doc_id);
    println!("  ✓ Registered: {}", test_id);

    // Create team with supervisor mode
    println!("\n🏢 Creating supervisor team...");
    let team_config = TeamConfig {
        id: "dev-team".to_string(),
        name: "Development Team".to_string(),
        description: "Team with supervisor routing to specialists".to_string(),
        agents: vec![
            TeamAgentConfig {
                agent_id: supervisor_id.clone(),
                role: "supervisor".to_string(),
                capabilities: vec!["routing".to_string()],
            },
            TeamAgentConfig {
                agent_id: coding_id,
                role: "coding-specialist".to_string(),
                capabilities: vec!["coding".to_string()],
            },
            TeamAgentConfig {
                agent_id: doc_id,
                role: "documentation-specialist".to_string(),
                capabilities: vec!["documentation".to_string()],
            },
            TeamAgentConfig {
                agent_id: test_id,
                role: "testing-specialist".to_string(),
                capabilities: vec!["testing".to_string()],
            },
        ],
        router_config: RouterConfig {
            default_agent_id: supervisor_id,
            max_routing_hops: 10,
        },
    };

    let team = Arc::new(Team::new(team_config, manager));

    // Display team information
    println!("\n📊 Team Information:");
    let team_info = team.info().await?;
    println!("  ID: {}", team_info.id);
    println!("  Name: {}", team_info.name);
    println!("  Capabilities: {}", team_info.capabilities.join(", "));
    println!(
        "  Members: {}",
        team_info
            .metadata
            .get("member_count")
            .unwrap_or(&"0".to_string())
    );

    // Test different types of queries
    println!("\n💬 Testing Supervisor Routing:\n");

    // Query 1: Coding task
    println!("Query 1: Coding task");
    println!("  User: Can you help me implement a new API endpoint?");
    let msg1 = Message::user_text("Can you help me implement a new API endpoint?");
    let response1 = team.process(msg1).await?;
    println!(
        "  Response: {}\n",
        extract_text(&response1).unwrap_or_default()
    );

    // Query 2: Documentation task
    println!("Query 2: Documentation task");
    println!("  User: I need to document this feature for users");
    let msg2 = Message::user_text("I need to document this feature for users");
    let response2 = team.process(msg2).await?;
    println!(
        "  Response: {}\n",
        extract_text(&response2).unwrap_or_default()
    );

    // Query 3: Testing task
    println!("Query 3: Testing task");
    println!("  User: We need to verify the quality of this module");
    let msg3 = Message::user_text("We need to verify the quality of this module");
    let response3 = team.process(msg3).await?;
    println!(
        "  Response: {}\n",
        extract_text(&response3).unwrap_or_default()
    );

    println!("✅ Example complete!");
    println!("\n💡 Key Takeaway:");
    println!("   The supervisor analyzes each query and routes it to the");
    println!("   appropriate specialist based on keywords and context.");

    Ok(())
}
