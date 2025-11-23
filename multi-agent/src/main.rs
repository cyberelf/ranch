use multi_agent::*;
use std::sync::Arc;
use std::env;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
    let config = Config::from_file(&config_path)?;

    let agent_manager = Arc::new(AgentManager::new());

    // Register agents from config with proper protocol support
    for agent_config in config.to_agent_configs() {
        let agent: Arc<dyn Agent> = match agent_config.protocol {
            ProtocolType::A2A => {
                // Create A2A client for the agent endpoint
                let transport = Arc::new(JsonRpcTransport::new(&agent_config.endpoint)?);
                let client = A2aClient::new(transport);

                // Create A2A agent with config
                let a2a_config = A2AAgentConfig {
                    max_retries: agent_config.max_retries,
                    task_handling: TaskHandling::PollUntilComplete,
                };

                Arc::new(A2AAgent::with_config(client, a2a_config))
            }
            ProtocolType::OpenAI => {
                // Create OpenAI agent
                let openai_config = OpenAIAgentConfig {
                    api_key: env::var("OPENAI_API_KEY").ok(),
                    max_retries: agent_config.max_retries,
                    timeout_seconds: agent_config.timeout_seconds,
                    model: agent_config.metadata.get("model")
                        .cloned()
                        .unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
                    temperature: agent_config.metadata.get("temperature")
                        .and_then(|v| v.parse().ok()),
                    max_tokens: agent_config.metadata.get("max_tokens")
                        .and_then(|v| v.parse().ok()),
                };

                Arc::new(OpenAIAgent::new(agent_config.endpoint, openai_config))
            }
        };

        // Register with agent manager
        let _id = agent_manager.register(agent).await?;
        println!("Registered agent: {}", _id);
    }

    // Check if any agents were registered
    let agent_count = agent_manager.count().await;
    if agent_count == 0 {
        eprintln!("No agents configured. Please check your config.toml file.");
        return Ok(());
    }

    println!("{} agent(s) registered successfully!", agent_count);

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
                let status = if healthy { "✓ Healthy" } else { "✗ Unhealthy" };
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
