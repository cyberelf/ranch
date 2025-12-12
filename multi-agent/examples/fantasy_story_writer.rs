//! Fantasy Story Writing Example
//!
//! This example demonstrates how to use the multi-agent framework to collaboratively
//! write a fantasy story using different specialized agents:
//! - Story Orchestrator (OpenAI GPT-4) - Plans the story structure and themes
//! - Story Composer (OpenAI GPT-4) - Writes the prose and scenes
//! - Story Advisor (A2A Agent) - Reviews and provides improvement suggestions

use multi_agent::*;
use std::io::Write;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("📚 Fantasy Story Writing Multi-Agent Example");
    println!("============================================");

    // Load configuration and create the complete multi-agent system
    let config_path = "multi-agent/examples/fantasy_story_config.toml".to_string();
    let config = Config::from_file(&config_path)?;

    println!("\n🔧 Creating multi-agent system from configuration...");
    let (agent_manager, team) = create_team_from_config(&config, "fantasy-story-team").await?;

    println!("\n🎯 Multi-agent system ready!");
    println!("📋 Available agents:");

    // Display available agents
    for agent_info in agent_manager.list_info().await {
        println!("   • {} ({})", agent_info.name, agent_info.id);
        println!("     Capabilities: {}", agent_info.capabilities.join(", "));
        println!("     Description: {}", agent_info.description);
        println!();
    }

    // Check agent health
    println!("🏥 Checking agent health...");
    let health_results = agent_manager.health_check_all().await;
    for (id, healthy) in &health_results {
        let status = if *healthy {
            "✅ Healthy"
        } else {
            "❌ Unhealthy"
        };
        println!("   {}: {}", id, status);
    }

    // Interactive story writing loop
    println!("\n🎭 Fantasy Story Writing Session");
    println!("===============================");
    println!("Commands:");
    println!("  'story <topic>' - Write a story about the given topic");
    println!("  'health' - Check agent health");
    println!("  'agents' - List available agents");
    println!("  'quit' - Exit the program");
    println!();

    loop {
        print!("📝 What fantasy story would you like to create? ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "quit" || input == "exit" {
            println!("👋 Thanks for using the Fantasy Story Writer!");
            break;
        }

        if input == "health" {
            println!("🏥 Agent Health Status:");
            let health_results = agent_manager.health_check_all().await;
            for (id, healthy) in health_results {
                let status = if healthy {
                    "✅ Healthy"
                } else {
                    "❌ Unhealthy"
                };
                println!("   {}: {}", id, status);
            }
            println!();
            continue;
        }

        if input == "agents" {
            println!("🤖 Available Agents:");
            for agent_info in agent_manager.list_info().await {
                println!("   • {} ({})", agent_info.name, agent_info.id);
                println!("     Capabilities: {}", agent_info.capabilities.join(", "));
            }
            println!();
            continue;
        }

        // Check if it's a story request
        if input.starts_with("story ") {
            let topic = input
                .strip_prefix("story ")
                .unwrap_or("a magical adventure");
            write_fantasy_story(&team, topic).await?;
            continue;
        }

        // Default: treat as story topic
        write_fantasy_story(&team, input).await?;
    }

    Ok(())
}

async fn write_fantasy_story(
    team: &Arc<Team>,
    topic: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📖 Writing Fantasy Story About: '{}'", topic);
    println!(
        "================================{}",
        "=".repeat(topic.len())
    );

    let start_time = std::time::Instant::now();

    // Create the initial message for the orchestrator
    let story_prompt = format!(
        "You are orchestrating the creation of a fantasy story about '{}'.
        Please provide:
        1. A brief story concept and theme
        2. Character ideas
        3. Setting and world-building elements
        4. Plot structure overview
        5. Key scenes that should be included

        Your response will guide the composer agent who will write the actual prose.",
        topic
    );

    let message = Message::user_text(&story_prompt);

    match team.process_message(message).await {
        Ok(response) => {
            let response_text =
                extract_text(&response).unwrap_or_else(|| "No response received".to_string());

            println!("🎯 Story Plan Created:");
            println!("====================");
            println!("{}", response_text);
            println!();

            let elapsed = start_time.elapsed();
            println!(
                "⏱️  Story completed in {:.2} seconds",
                elapsed.as_secs_f64()
            );
        }
        Err(e) => {
            eprintln!("❌ Error writing fantasy story: {}", e);
        }
    }

    println!();
    Ok(())
}
