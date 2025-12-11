//! Remote Agents Example
//!
//! This example demonstrates how to coordinate multiple remote A2A agents
//! distributed across different endpoints using the A2A protocol.
//!
//! ## What this example demonstrates:
//! - Creating A2AAgent instances pointing to remote endpoints
//! - Coordinating distributed agents in a team
//! - Handling network errors and timeouts gracefully
//! - Using the A2A protocol for remote agent communication
//!
//! ## Prerequisites:
//! This example requires remote A2A agents to be running. For testing purposes,
//! you can start multiple instances of the `basic_echo_server` example:
//!
//! ```bash
//! # Terminal 1: Start first agent on port 8081
//! cargo run --example basic_echo_server -- --port 8081
//!
//! # Terminal 2: Start second agent on port 8082
//! cargo run --example basic_echo_server -- --port 8082
//!
//! # Terminal 3: Run this example
//! cargo run --example remote_agents
//! ```
//!
//! ## Running this example:
//! ```bash
//! cargo run --example remote_agents
//! ```

use a2a_protocol::prelude::*;
use async_trait::async_trait;
use multi_agent::agent::{A2AAgent, A2AAgentConfig, TaskHandling};
use multi_agent::team::{
    SchedulerConfig, SupervisorSchedulerConfig, TeamAgentConfig, TeamConfig, TeamMode,
};
use multi_agent::Agent;
use multi_agent::*;
use std::sync::Arc;
use std::time::Duration;

/// Supervisor agent that coordinates remote agents
struct RemoteSupervisor {
    id: String,
}

impl RemoteSupervisor {
    fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

#[async_trait]
impl Agent for RemoteSupervisor {
    async fn info(&self) -> A2aResult<AgentInfo> {
        Ok(AgentInfo {
            id: self.id.clone(),
            name: "Remote Supervisor".to_string(),
            description: "Coordinates distributed remote agents".to_string(),
            capabilities: vec!["coordination".to_string(), "routing".to_string()],
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn process(&self, message: Message) -> A2aResult<Message> {
        let input = extract_text(&message).unwrap_or_default();
        println!("  [Remote Supervisor] Analyzing request: \"{}\"", input);

        // Simple routing logic based on message content
        let target = if input.contains("agent1") || input.contains("first") {
            "agent1"
        } else if input.contains("agent2") || input.contains("second") {
            "agent2"
        } else {
            "agent1" // default to first agent
        };

        println!("  [Remote Supervisor] Routing to {}", target);

        Ok(Message::agent_text(&format!(
            "Supervisor: Routing to remote {} for processing",
            target
        )))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Remote Agents Example");
    println!("========================\n");

    println!("⚠️  Note: This example requires remote A2A agents to be running.");
    println!("    For testing, start the basic_echo_server example on ports 8081 and 8082:\n");
    println!("    Terminal 1: cargo run --example basic_echo_server -- --port 8081");
    println!("    Terminal 2: cargo run --example basic_echo_server -- --port 8082\n");

    // Create agent manager
    println!("📋 Setting up agent manager...");
    let manager = Arc::new(AgentManager::new());

    // Create supervisor agent
    println!("🔧 Creating supervisor agent...");
    let supervisor = Arc::new(RemoteSupervisor::new("supervisor"));
    let supervisor_id = manager.register(supervisor as Arc<dyn Agent>).await?;
    println!("  ✓ Registered: {}", supervisor_id);

    // Configure remote A2A agents
    println!("\n🌐 Configuring remote A2A agents...");

    // Agent 1 configuration
    let agent1_endpoint = "http://localhost:8081/rpc";
    println!("  Agent 1: {}", agent1_endpoint);

    let transport1 = Arc::new(JsonRpcTransport::new(agent1_endpoint)?);
    let client1 = A2aClient::new(transport1);
    let config1 = A2AAgentConfig {
        max_retries: 3,
        task_handling: TaskHandling::PollUntilComplete,
    };
    let agent1 = Arc::new(A2AAgent::with_config(client1, config1));

    // Agent 2 configuration
    let agent2_endpoint = "http://localhost:8082/rpc";
    println!("  Agent 2: {}", agent2_endpoint);

    let transport2 = Arc::new(JsonRpcTransport::new(agent2_endpoint)?);
    let client2 = A2aClient::new(transport2);
    let config2 = A2AAgentConfig {
        max_retries: 3,
        task_handling: TaskHandling::PollUntilComplete,
    };
    let agent2 = Arc::new(A2AAgent::with_config(client2, config2));

    // Register remote agents
    println!("\n📝 Registering remote agents...");

    // Try to register agents with connection timeout handling
    let agent1_id = match tokio::time::timeout(
        Duration::from_secs(5),
        manager.register(agent1 as Arc<dyn Agent>),
    )
    .await
    {
        Ok(Ok(id)) => {
            println!("  ✓ Registered remote agent1: {}", id);
            id
        }
        Ok(Err(e)) => {
            println!("  ✗ Failed to register agent1: {}", e);
            println!("    Make sure the agent is running at {}", agent1_endpoint);
            return Ok(());
        }
        Err(_) => {
            println!("  ✗ Timeout registering agent1");
            println!("    Make sure the agent is running at {}", agent1_endpoint);
            return Ok(());
        }
    };

    let agent2_id = match tokio::time::timeout(
        Duration::from_secs(5),
        manager.register(agent2 as Arc<dyn Agent>),
    )
    .await
    {
        Ok(Ok(id)) => {
            println!("  ✓ Registered remote agent2: {}", id);
            id
        }
        Ok(Err(e)) => {
            println!("  ✗ Failed to register agent2: {}", e);
            println!("    Make sure the agent is running at {}", agent2_endpoint);
            return Ok(());
        }
        Err(_) => {
            println!("  ✗ Timeout registering agent2");
            println!("    Make sure the agent is running at {}", agent2_endpoint);
            return Ok(());
        }
    };

    // Create team coordinating remote agents
    println!("\n🏢 Creating distributed team...");
    let team_config = TeamConfig {
        id: "distributed-team".to_string(),
        name: "Distributed Agent Team".to_string(),
        description: "Team coordinating remote A2A agents".to_string(),
        mode: TeamMode::Supervisor,
        agents: vec![
            TeamAgentConfig {
                agent_id: supervisor_id.clone(),
                role: "supervisor".to_string(),
                capabilities: vec!["coordination".to_string()],
            },
            TeamAgentConfig {
                agent_id: agent1_id,
                role: "remote-worker-1".to_string(),
                capabilities: vec!["processing".to_string()],
            },
            TeamAgentConfig {
                agent_id: agent2_id,
                role: "remote-worker-2".to_string(),
                capabilities: vec!["processing".to_string()],
            },
        ],
        scheduler_config: SchedulerConfig::Supervisor(SupervisorSchedulerConfig {
            supervisor_agent_id: supervisor_id,
        }),
    };

    let team = Arc::new(Team::new(team_config, manager));

    // Display team information
    println!("\n📊 Team Information:");
    let team_info = team.info().await?;
    println!("  ID: {}", team_info.id);
    println!("  Name: {}", team_info.name);
    println!("  Mode: Distributed Coordination");
    println!(
        "  Members: {} (1 local supervisor + 2 remote agents)",
        team_info
            .metadata
            .get("member_count")
            .unwrap_or(&"0".to_string())
    );

    // Test communication with remote agents
    println!("\n💬 Testing Distributed Communication:\n");

    // Test 1: Send to first agent
    println!("Test 1: Route to first remote agent");
    println!("  User: Please use agent1 to process this");

    match tokio::time::timeout(
        Duration::from_secs(10),
        team.process(Message::user_text("Please use agent1 to process this")),
    )
    .await
    {
        Ok(Ok(response)) => {
            let response_text = extract_text(&response).unwrap_or_default();
            println!("  Response: {}\n", response_text);
        }
        Ok(Err(e)) => {
            println!("  Error: {}\n", e);
        }
        Err(_) => {
            println!("  Timeout - remote agent may be slow or unavailable\n");
        }
    }

    // Test 2: Send to second agent
    println!("Test 2: Route to second remote agent");
    println!("  User: I want agent2 to handle this request");

    match tokio::time::timeout(
        Duration::from_secs(10),
        team.process(Message::user_text("I want agent2 to handle this request")),
    )
    .await
    {
        Ok(Ok(response)) => {
            let response_text = extract_text(&response).unwrap_or_default();
            println!("  Response: {}\n", response_text);
        }
        Ok(Err(e)) => {
            println!("  Error: {}\n", e);
        }
        Err(_) => {
            println!("  Timeout - remote agent may be slow or unavailable\n");
        }
    }

    println!("✅ Example complete!");
    println!("\n💡 Key Takeaways:");
    println!("   • A2AAgent wraps remote agents using the A2A protocol");
    println!("   • Teams can coordinate distributed agents across different endpoints");
    println!("   • Network errors and timeouts are handled gracefully");
    println!("   • The A2A protocol enables true multi-agent distribution");
    println!("\n📚 Next Steps:");
    println!("   • Configure authentication for production remote agents");
    println!("   • Implement retry logic for transient network failures");
    println!("   • Add monitoring and observability for distributed systems");

    Ok(())
}
