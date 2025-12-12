use multi_agent::*;
use std::env;
use std::io::{self, Write};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
    let config = Config::from_file(&config_path)?;

    let agent_manager = Arc::new(AgentManager::new());

    // Register all agents from config
    let agent_ids = agent_manager.register_from_config(&config).await?;
    
    for id in &agent_ids {
        println!("Registered agent: {}", id);
    }

    // Check if any agents were registered
    if agent_ids.is_empty() {
        eprintln!("No agents configured. Please check your config.toml file.");
        return Ok(());
    }

    println!("{} agent(s) registered successfully!", agent_ids.len());

    // Interactive CLI loop
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "quit" || input == "exit" {
            println!("Goodbye!");
            break;
        }

        if input == "agents" {
            let agents = agent_manager.list_info().await;
            println!("Available agents:");
            for agent in agents {
                println!("  - {} ({})", agent.name, agent.id);
                println!("    Capabilities: {}", agent.capabilities.join(", "));
                println!();
            }
            continue;
        }

        if input == "health" {
            let health_results = agent_manager.health_check_all().await;
            println!("Agent health status:");
            for (id, healthy) in health_results {
                let status = if healthy {
                    "✓ Healthy"
                } else {
                    "✗ Unhealthy"
                };
                println!("  {}: {}", id, status);
            }
            continue;
        }

        // Create a message from user input
        let message = Message::user_text(input);

        // Get the first available agent
        let agent_ids = agent_manager.list_ids().await;
        if let Some(agent_id) = agent_ids.first() {
            if let Some(agent) = agent_manager.get(agent_id).await {
                match agent.process(message).await {
                    Ok(response) => {
                        let response_text = extract_text(&response)
                            .unwrap_or_else(|| "No response content".to_string());
                        println!("\nAgent response:\n{}", response_text);
                    }
                    Err(e) => {
                        eprintln!("Error processing message: {}", e);
                    }
                }
            } else {
                eprintln!("Agent {} not found", agent_id);
            }
        } else {
            eprintln!("No agents available");
        }

        println!();
    }

    Ok(())
}
