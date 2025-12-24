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
        let mut profile = AgentProfile::new(
            agent_id,
            "Router Agent",
            url::Url::parse("http://localhost:3000").unwrap(),
        );
        
        profile.description = Some("Agent that makes dynamic routing decisions".to_string());
        
        // Declare routing extension capability
        profile.skills.push(AgentSkill {
            name: "ranch:extension:client-agent-routing".to_string(),
            description: Some("Client Agent Routing Extension support".to_string()),
            category: Some("protocol".to_string()),
            tags: vec!["routing".to_string(), "extension".to_string()],
            examples: vec![],
        });
        
        // Declare general skills
        profile.skills.push(AgentSkill {
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
        if let Some(metadata) = &message.metadata {
            if let Some(ext_data) = metadata.get("ranch:extension:client-agent-routing") {
                println!("[Router] Received extension context");
                
                // Parse the extension request
                if let Ok(request) = serde_json::from_value::<serde_json::Value>(ext_data.clone()) {
                    if let Some(agent_cards) = request.get("agent_cards").and_then(|v| v.as_array()) {
                        println!("[Router] Available peers:");
                        for card in agent_cards {
                            if let Some(name) = card.get("name").and_then(|v| v.as_str()) {
                                if let Some(id) = card.get("id").and_then(|v| v.as_str()) {
                                    println!("  - {} ({})", name, id);
                                }
                            }
                        }
                    }
                }

                // Simple routing logic: if message contains "help", route to "worker"
                let text = message.text_content().unwrap_or_default();
                
                let (recipient, reason, handoffs) = if text.contains("help") {
                    ("worker".to_string(), Some("User needs help".to_string()), Some(vec!["supervisor".to_string()]))
                } else {
                    ("user".to_string(), Some("Task complete".to_string()), None)
                };

                let mut response = Message::agent_text(&format!(
                    "[Router] Routing to: {} ({})",
                    recipient, reason.as_ref().unwrap_or(&"".to_string())
                ));

                // Add routing decision to metadata
                let routing_response = serde_json::json!({
                    "recipient": recipient,
                    "reason": reason,
                    "handoffs": handoffs,
                });

                let mut response_metadata = std::collections::HashMap::new();
                response_metadata.insert(
                    "ranch:extension:client-agent-routing".to_string(),
                    routing_response,
                );
                response.metadata = Some(response_metadata);

                return Ok(response);
            }
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
        if let Some(metadata) = &message.metadata {
            if let Some(ext_data) = metadata.get("ranch:extension:client-agent-routing") {
                if let Ok(request) = serde_json::from_value::<serde_json::Value>(ext_data.clone()) {
                    if let Some(agent_cards) = request.get("agent_cards").and_then(|v| v.as_array()) {
                        println!("[Worker] Received filtered peers (handoffs):");
                        for card in agent_cards {
                            if let Some(id) = card.get("id").and_then(|v| v.as_str()) {
                                println!("  - {}", id);
                            }
                        }
                    }
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
        let mut profile = AgentProfile::new(
            agent_id,
            "Supervisor Agent",
            url::Url::parse("http://localhost:3002").unwrap(),
        );
        
        profile.description = Some("Agent that supervises and quality checks".to_string());
        
        profile.skills.push(AgentSkill {
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
        Ok(Message::agent_text(&format!("[Supervisor] Reviewed and approved: {}", text)))
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
        let handler = TaskAwareHandler::new(agent);
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
        let handler = TaskAwareHandler::new(agent);
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
        let handler = TaskAwareHandler::new(agent);
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
