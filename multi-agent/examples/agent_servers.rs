//! Multi-Agent Servers Example
//!
//! This example demonstrates how to start multiple A2A protocol servers with different
//! skills and extension capabilities. These servers represent the actual agent implementations.
//!
//! **Architecture:**
//! - This file starts REAL A2A protocol servers (not mock agents)
//! - Each agent server implements ProtocolAgent from a2a-protocol
//! - Agents declare their skills and extension support via AgentProfile
//!
//! **Usage:**
//! 1. Run this first: `cargo run --example agent_servers`
//! 2. Then run the client: `cargo run --example team_client`
//!
//! The servers will run on:
//! - Router Agent: http://localhost:3000
//! - Worker Agent: http://localhost:3001
//! - Supervisor Agent: http://localhost:3002

use a2a_protocol::prelude::*;
use a2a_protocol::server::{ProtocolAgent, ServerBuilder, TaskAwareHandler};
use a2a_protocol::extensions::client_routing::{
    ClientRoutingExtensionData, Participant,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::task::JoinSet;

/// Router agent that supports the Client Agent Routing Extension
struct RouterAgent {
    profile: AgentProfile,
}

impl RouterAgent {
    fn new() -> Self {
        let agent_id = AgentId::new("router".to_string()).unwrap();
        let capability = AgentCapabilities::new().with_extension::<ClientRoutingExtensionData>();
        
        let profile = AgentProfile::new(
            agent_id,
            "Router Agent",
            url::Url::parse("http://localhost:3000").unwrap(),
        )
            .with_description("Agent that makes dynamic routing decisions")
            .with_capability(capability)
            // Declare extension support as a skill for human-readable metadata
            // (extension is detected via capabilities.extensions, not skills)
            .with_skill(AgentSkill {
                name: ClientRoutingExtensionData::URI.to_string(),
                description: Some(ClientRoutingExtensionData::DESCRIPTION.to_string()),
                category: Some("protocol".to_string()),
                tags: vec!["routing".to_string(), "extension".to_string()],
                examples: vec![],
            })
            .with_skill(AgentSkill {
                name: "coordination".to_string(),
                description: Some("Coordinate team activities".to_string()),
                category: Some("skill".to_string()),
                tags: vec!["coordination".to_string()],
                examples: vec![],
            });

        Self { profile }
    }
}

#[async_trait]
impl ProtocolAgent for RouterAgent {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        Ok(self.profile.clone())
    }

    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        println!("\n[Router] Processing message...");
        
        // Check for extension data
        if let Some(ext_data) = message.get_extension::<ClientRoutingExtensionData>()? {
            println!("[Router] Received extension context");
            
            if let Some(agent_cards) = &ext_data.agent_cards {
                println!("[Router] Available peers:");
                for card in agent_cards {
                    println!("  - {} ({})", card.name, card.id);
                }
            }

            // Simple routing logic: if message contains "help", route to "worker"
            let text = message.text_content().unwrap_or_default();
            
            let (recipient, reason, handoffs) = if text.contains("help") {
                (Participant::agent("worker"), Some("User needs help".to_string()), Some(vec!["supervisor".to_string()]))
            } else {
                (Participant::user(), Some("Task complete".to_string()), None)
            };

            let mut response = Message::agent_text(&format!(
                "[Router] Routing to: {:?} ({})",
                recipient, reason.as_ref().unwrap_or(&"".to_string())
            ));

            // Add routing decision to metadata
            let routing_response = ClientRoutingExtensionData {
                recipient: Some(recipient),
                reason,
                handoffs,
                ..Default::default()
            };

            response.set_extension(routing_response)?;

            return Ok(response);
        }

        println!("[Router] No extension data, processing normally");
        Ok(Message::agent_text("[Router] Processed without routing context"))
    }
}

/// Worker agent that performs tasks
struct WorkerAgent {
    profile: AgentProfile,
}

impl WorkerAgent {
    fn new() -> Self {
        let agent_id = AgentId::new("worker".to_string()).unwrap();
        let mut profile = AgentProfile::new(
            agent_id,
            "Worker Agent",
            url::Url::parse("http://localhost:3001").unwrap(),
        );
        
        profile.description = Some("Agent that performs work tasks".to_string());
        
        profile.skills.push(AgentSkill {
            name: "task-execution".to_string(),
            description: Some("Execute assigned tasks".to_string()),
            category: Some("skill".to_string()),
            tags: vec!["work".to_string(), "execution".to_string()],
            examples: vec![],
        });

        Self { profile }
    }
}

#[async_trait]
impl ProtocolAgent for WorkerAgent {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        Ok(self.profile.clone())
    }

    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        println!("\n[Worker] Processing task...");
        
        // Check if we received handoffs (filtered peer list)
        if let Some(ext_data) = message.get_extension::<ClientRoutingExtensionData>()? {
            if let Some(agent_cards) = &ext_data.agent_cards {
                println!("[Worker] Received filtered peers (handoffs):");
                for card in agent_cards {
                    println!("  - {}", card.id);
                }
            }
        }

        let text = message.text_content().unwrap_or_default();
        Ok(Message::agent_text(&format!("[Worker] Completed task: {}", text)))
    }
}

/// Supervisor agent that oversees work
struct SupervisorAgent {
    profile: AgentProfile,
}

impl SupervisorAgent {
    fn new() -> Self {
        let agent_id = AgentId::new("supervisor".to_string()).unwrap();
        
        // Declare Client Routing Extension capability
        let capability = AgentCapabilities::new().with_extension::<ClientRoutingExtensionData>();
        
        let profile = AgentProfile::new(
            agent_id,
            "Supervisor Agent",
            url::Url::parse("http://localhost:3002").unwrap(),
        )
            .with_description("Agent that supervises and quality checks")
            .with_capability(capability)
            // Declare extension support as a skill for human-readable metadata
            // (extension is detected via capabilities.extensions, not skills)
            .with_skill(AgentSkill {
                name: ClientRoutingExtensionData::URI.to_string(),
                description: Some(ClientRoutingExtensionData::DESCRIPTION.to_string()),
                category: Some("protocol".to_string()),
                tags: vec!["routing".to_string(), "extension".to_string()],
                examples: vec![],
            })
            .with_skill(AgentSkill {
                name: "supervision".to_string(),
                description: Some("Supervise work quality".to_string()),
                category: Some("skill".to_string()),
                tags: vec!["qa".to_string(), "supervision".to_string()],
                examples: vec![],
            });

        Self { profile }
    }
}

#[async_trait]
impl ProtocolAgent for SupervisorAgent {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        Ok(self.profile.clone())
    }

    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        println!("\n[Supervisor] Reviewing work...");
        
        let text = message.text_content().unwrap_or_default();
        
        // Check for extension context
        let mut recipient = Participant::user();
        let mut reason = Some("Work approved".to_string());

        if let Some(ext_data) = message.get_extension::<ClientRoutingExtensionData>()? {
            println!("[Supervisor] Received extension context from {:?}", ext_data.sender);
            
            // If message contains "retry", route back to worker
            if text.contains("retry") {
                recipient = Participant::agent("worker");
                reason = Some("Work needs improvement, retrying...".to_string());
            }
        }

        let mut response = Message::agent_text(&format!(
            "[Supervisor] Reviewed: {}. Decision: {:?}", 
            text, reason.as_ref().unwrap_or(&"None".to_string())
        ));

        // Add routing decision
        let routing_response = ClientRoutingExtensionData {
            recipient: Some(recipient),
            reason,
            ..Default::default()
        };
        response.set_extension(routing_response)?;

        Ok(response)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Multi-Agent Servers");
    println!("================================\n");
    
    let mut join_set = JoinSet::new();

    // Start Router Agent on port 3000
    println!("Starting Router Agent on http://localhost:3000");
    join_set.spawn(async move {
        let agent = Arc::new(RouterAgent::new());
        let handler = TaskAwareHandler::with_immediate_responses(agent);
        ServerBuilder::new(handler)
            .with_port(3000)
            .run()
            .await
            .map_err(|e| format!("Router server error: {}", e))
    });

    // Start Worker Agent on port 3001
    println!("Starting Worker Agent on http://localhost:3001");
    join_set.spawn(async move {
        let agent = Arc::new(WorkerAgent::new());
        let handler = TaskAwareHandler::with_immediate_responses(agent);
        ServerBuilder::new(handler)
            .with_port(3001)
            .run()
            .await
            .map_err(|e| format!("Worker server error: {}", e))
    });

    // Start Supervisor Agent on port 3002
    println!("Starting Supervisor Agent on http://localhost:3002");
    join_set.spawn(async move {
        let agent = Arc::new(SupervisorAgent::new());
        let handler = TaskAwareHandler::with_immediate_responses(agent);
        ServerBuilder::new(handler)
            .with_port(3002)
            .run()
            .await
            .map_err(|e| format!("Supervisor server error: {}", e))
    });

    println!("\n✅ All agent servers started!");
    println!("   Ready to accept connections from team client");
    println!("   Press Ctrl+C to stop\n");

    // Wait for all servers (they run forever until interrupted)
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(())) => println!("Server completed successfully"),
            Ok(Err(e)) => eprintln!("Server error: {}", e),
            Err(e) => eprintln!("Join error: {}", e),
        }
    }

    Ok(())
}
