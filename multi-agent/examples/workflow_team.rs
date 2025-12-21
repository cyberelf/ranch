//! Workflow Team Example
//!
//! This example demonstrates the workflow pattern where messages flow sequentially
//! through a series of agents, with each agent processing and transforming the message
//! before passing it to the next agent in the workflow.
//!
//! ## What this example demonstrates:
//! - Creating agents for sequential workflow steps
//! - Using workflow mode for sequential processing
//! - State passing between workflow steps
//! - Multi-step message transformation pipeline
//!
//! ## Running this example:
//! ```bash
//! cargo run --example workflow_team
//! ```

use a2a_protocol::prelude::Message;
use async_trait::async_trait;
use multi_agent::team::{RouterConfig, TeamAgentConfig, TeamConfig};
use multi_agent::Agent;
use multi_agent::*;
use std::sync::Arc;

/// Research agent - first step in the workflow
struct ResearchAgent {
    id: String,
}

impl ResearchAgent {
    fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

#[async_trait]
impl Agent for ResearchAgent {
    async fn info(&self) -> A2aResult<AgentInfo> {
        Ok(AgentInfo {
            id: self.id.clone(),
            name: "Research Agent".to_string(),
            description: "Gathers and analyzes information on the topic".to_string(),
            capabilities: vec!["research".to_string(), "analysis".to_string()],
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn process(&self, message: Message) -> A2aResult<Message> {
        let input = extract_text(&message).unwrap_or_default();

        println!("  [Step 1: Research] Analyzing topic: \"{}\"", input);
        println!("  [Step 1: Research] Gathering relevant information...");

        // Simulate research and add context
        let research_output = format!(
            "Research findings on '{}': Key concepts include best practices, \
            common patterns, and potential challenges. Industry standards \
            recommend thorough planning and iterative development.",
            input
        );

        println!("  [Step 1: Research] ✓ Research complete\n");

        Ok(Message::agent_text(&research_output))
    }
}

/// Drafting agent - second step in the workflow
struct DraftAgent {
    id: String,
}

impl DraftAgent {
    fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

#[async_trait]
impl Agent for DraftAgent {
    async fn info(&self) -> A2aResult<AgentInfo> {
        Ok(AgentInfo {
            id: self.id.clone(),
            name: "Draft Agent".to_string(),
            description: "Creates initial content draft based on research".to_string(),
            capabilities: vec!["writing".to_string(), "drafting".to_string()],
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn process(&self, message: Message) -> A2aResult<Message> {
        let _research = extract_text(&message).unwrap_or_default();

        println!("  [Step 2: Draft] Received research findings");
        println!("  [Step 2: Draft] Creating initial draft...");

        // Create a draft based on research
        let draft = "DRAFT CONTENT:\n\
            Based on the research, here's an initial draft:\n\n\
            This document covers the essential aspects identified during research. \
            The approach follows industry best practices and addresses common \
            challenges. Implementation should be iterative and well-planned.\n\n\
            [Note: This is a first draft and will be refined in the next step]"
            .to_string();

        println!("  [Step 2: Draft] ✓ Draft created\n");

        Ok(Message::agent_text(&draft))
    }
}

/// Editing agent - final step in the workflow
struct EditAgent {
    id: String,
}

impl EditAgent {
    fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

#[async_trait]
impl Agent for EditAgent {
    async fn info(&self) -> A2aResult<AgentInfo> {
        Ok(AgentInfo {
            id: self.id.clone(),
            name: "Edit Agent".to_string(),
            description: "Refines and polishes the draft into final content".to_string(),
            capabilities: vec!["editing".to_string(), "proofreading".to_string()],
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn process(&self, message: Message) -> A2aResult<Message> {
        let _draft = extract_text(&message).unwrap_or_default();

        println!("  [Step 3: Edit] Received draft");
        println!("  [Step 3: Edit] Refining content...");
        println!("  [Step 3: Edit] Checking grammar and style...");

        // Polish the draft into final content
        let final_content = "FINAL POLISHED CONTENT:\n\n\
            This comprehensive document addresses all key aspects identified \
            during research. The approach adheres to industry best practices \
            and effectively addresses common challenges.\n\n\
            Implementation Strategy:\n\
            • Follow iterative development methodology\n\
            • Maintain clear documentation throughout\n\
            • Test thoroughly at each stage\n\
            • Incorporate feedback continuously\n\n\
            This refined version is ready for publication and addresses all \
            requirements with clarity and precision."
            .to_string();

        println!("  [Step 3: Edit] ✓ Final content ready\n");

        Ok(Message::agent_text(&final_content))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Workflow Team Example");
    println!("========================\n");

    // Create agent manager
    println!("📋 Setting up agent manager...");
    let manager = Arc::new(AgentManager::new());

    // Create workflow agents
    println!("🔧 Creating workflow agents...");
    let research_agent = Arc::new(ResearchAgent::new("research-agent"));
    let draft_agent = Arc::new(DraftAgent::new("draft-agent"));
    let edit_agent = Arc::new(EditAgent::new("edit-agent"));

    // Register all agents
    println!("📝 Registering agents...");
    let research_id = manager.register(research_agent as Arc<dyn Agent>).await?;
    let draft_id = manager.register(draft_agent as Arc<dyn Agent>).await?;
    let edit_id = manager.register(edit_agent as Arc<dyn Agent>).await?;

    println!("  ✓ Registered: {}", research_id);
    println!("  ✓ Registered: {}", draft_id);
    println!("  ✓ Registered: {}", edit_id);

    // Create team with router (using first agent as default)
    println!("\n🏢 Creating workflow team...");
    let team_config = TeamConfig {
        id: "content-team".to_string(),
        name: "Content Creation Team".to_string(),
        description: "Sequential workflow for content creation".to_string(),
        agents: vec![
            TeamAgentConfig {
                agent_id: research_id.clone(),
                role: "researcher".to_string(),
                capabilities: vec!["research".to_string()],
            },
            TeamAgentConfig {
                agent_id: draft_id.clone(),
                role: "drafter".to_string(),
                capabilities: vec!["drafting".to_string()],
            },
            TeamAgentConfig {
                agent_id: edit_id.clone(),
                role: "editor".to_string(),
                capabilities: vec!["editing".to_string()],
            },
        ],
        router_config: RouterConfig {
            default_agent_id: research_id,
            max_routing_hops: 10,
        },
    };

    let team = Arc::new(Team::new(team_config, manager));

    // Display team information
    println!("\n📊 Team Information:");
    let team_info = team.info().await?;
    println!("  ID: {}", team_info.id);
    println!("  Name: {}", team_info.name);
    println!("  Mode: Workflow (Sequential Processing)");
    println!("  Steps: Research → Draft → Edit");
    println!("  Capabilities: {}", team_info.capabilities.join(", "));

    // Process a message through the workflow
    println!("\n💬 Starting Content Creation Workflow:\n");
    println!("─────────────────────────────────────────────\n");

    let topic = "Building Multi-Agent Systems";
    println!("📥 Input Topic: \"{}\"\n", topic);

    let message = Message::user_text(topic);
    let response = team.process(message).await?;

    println!("─────────────────────────────────────────────\n");
    println!("📤 Workflow Output:\n");
    println!("{}", extract_text(&response).unwrap_or_default());

    println!("\n✅ Example complete!");
    println!("\n💡 Key Takeaway:");
    println!("   In workflow mode, each agent processes the message sequentially,");
    println!("   transforming and refining the content at each step. The output");
    println!("   of one agent becomes the input for the next agent in the sequence.");

    Ok(())
}
