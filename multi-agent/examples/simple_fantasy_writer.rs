//! Simple Fantasy Story Writer Example
//!
//! This is a simplified version that uses only OpenAI agents to demonstrate
//! the multi-agent workflow without requiring an A2A server setup.

use multi_agent::team::{RouterConfig, TeamAgentConfig, TeamConfig};
use multi_agent::*;
use std::env;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("📚 Simple Fantasy Story Writer Example");
    println!("====================================");

    // Check for required environment variables
    if env::var("OPENAI_API_KEY").is_err() {
        eprintln!("❌ OPENAI_API_KEY environment variable is required");
        eprintln!("Please set your OpenAI API key:");
        eprintln!("export OPENAI_API_KEY=\"your-api-key-here\"");
        return Ok(());
    }

    // Get OpenAI base URL from environment variable or use default
    let openai_base_url =
        env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

    // Get model configurations from environment variables or use defaults
    let orchestrator_model = env::var("ORCHESTRATOR_MODEL").unwrap_or_else(|_| "gpt-4".to_string());
    let composer_model = env::var("COMPOSER_MODEL").unwrap_or_else(|_| "gpt-4".to_string());

    // Get timeout configurations from environment variables or use defaults
    let orchestrator_timeout = env::var("ORCHESTRATOR_TIMEOUT")
        .unwrap_or_else(|_| "90".to_string())
        .parse::<u64>()
        .unwrap_or(90);
    let composer_timeout = env::var("COMPOSER_TIMEOUT")
        .unwrap_or_else(|_| "120".to_string())
        .parse::<u64>()
        .unwrap_or(120);

    println!("🔗 Using OpenAI endpoint: {}", openai_base_url);
    println!(
        "🎯 Orchestrator model: {} (timeout: {}s)",
        orchestrator_model, orchestrator_timeout
    );
    println!(
        "✍️ Composer model: {} (timeout: {}s)",
        composer_model, composer_timeout
    );

    // Initialize agent manager
    let agent_manager: Arc<AgentManager> = Arc::new(AgentManager::new());

    // Create Story Orchestrator
    let orchestrator_config = OpenAIAgentConfig {
        id: "orchestrator".to_string(),
        name: "Story Orchestrator".to_string(),
        description: "Master storyteller and creative director".to_string(),
        capabilities: vec![
            "story_planning".to_string(),
            "creative_direction".to_string(),
        ],
        api_key: Some(env::var("OPENAI_API_KEY")?),
        max_retries: 3,
        timeout_seconds: orchestrator_timeout,
        model: orchestrator_model,
        temperature: Some(0.8),
        max_tokens: Some(1500),
    };

    let orchestrator = Arc::new(OpenAIAgent::with_config(
        openai_base_url.clone(),
        orchestrator_config,
    ));

    let orchestrator_id = agent_manager.register(orchestrator).await?;
    println!("✅ Registered Story Orchestrator: {}", orchestrator_id);

    // Create Story Composer
    let composer_config = OpenAIAgentConfig {
        id: "composer".to_string(),
        name: "Story Composer".to_string(),
        description: "Master prose writer and storyteller".to_string(),
        capabilities: vec!["prose_writing".to_string(), "scene_composition".to_string()],
        api_key: Some(env::var("OPENAI_API_KEY")?),
        max_retries: 3,
        timeout_seconds: composer_timeout,
        model: composer_model,
        temperature: Some(0.9),
        max_tokens: Some(2000),
    };

    let composer = Arc::new(OpenAIAgent::with_config(
        openai_base_url.clone(),
        composer_config,
    ));

    let composer_id = agent_manager.register(composer).await?;
    println!("✅ Registered Story Composer: {}", composer_id);

    // Create a team with router
    let team_config = TeamConfig {
        id: "fantasy-story-team".to_string(),
        name: "Fantasy Story Writing Team".to_string(),
        description: "Team for collaborative fantasy story creation".to_string(),
        agents: vec![
            TeamAgentConfig {
                agent_id: orchestrator_id.clone(),
                role: "story_planner".to_string(),
                capabilities: vec!["story_planning".to_string()],
            },
            TeamAgentConfig {
                agent_id: composer_id.clone(),
                role: "story_writer".to_string(),
                capabilities: vec!["prose_writing".to_string()],
            },
        ],
        router_config: RouterConfig {
            default_agent_id: orchestrator_id.clone(),
            max_routing_hops: 10,
        },
    };

    let team = Arc::new(Team::new(team_config, agent_manager.clone()));
    println!("✅ Created Fantasy Story Writing Team");

    println!("\n🎭 Fantasy Story Writing Session");
    println!("===============================");
    println!("Commands:");
    println!("  'story <topic>' - Write a fantasy story about the given topic");
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
            println!("👋 Thanks for using the Simple Fantasy Story Writer!");
            break;
        }

        if input == "health" {
            println!("🏥 Agent Health Status:");
            let health_results = agent_manager.health_check_all().await;
            let mut all_healthy = true;

            for (id, healthy) in &health_results {
                let status = if *healthy {
                    "✅ Healthy"
                } else {
                    "❌ Unhealthy"
                };
                println!("   {}: {}", id, status);
                if !healthy {
                    all_healthy = false;
                }
            }

            if !all_healthy {
                println!("\n⚠️  Some agents are unhealthy. Check your:");
                println!("   • OPENAI_API_KEY environment variable");
                println!("   • Network connection to OpenAI API");
                println!("   • OPENAI_BASE_URL (if custom endpoint)");
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
            write_fantasy_story_with_team(&team, topic).await?;
            continue;
        }

        // Default: treat as story topic
        write_fantasy_story_with_team(&team, input).await?;
    }

    Ok(())
}

async fn write_fantasy_story_with_team(
    team: &Arc<Team>,
    topic: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📖 Writing Fantasy Story About: '{}'", topic);
    println!(
        "================================{}",
        "=".repeat(topic.len())
    );

    let start_time = Instant::now();

    // Create the story request message for the team
    let story_request = Message::user_text(format!(
        "You are a team of AI assistants creating a fantasy story about '{}'.

        Please work together to create a compelling fantasy story.
        First, plan the story structure and key elements.
        Then, write the complete narrative with vivid descriptions.

        Make it engaging and creative with rich world-building and interesting characters.",
        topic
    ));

    // Let the team process the message through the workflow
    println!("\n🔄 Starting Team Workflow Execution");
    println!("=====================================");
    println!("📝 Input: Story topic '{}'", topic);

    println!("\n🎯 Step 1: Orchestrator creating story plan...");
    match team.process_message(story_request).await {
        Ok(response) => {
            println!("✅ Team workflow completed successfully!");

            println!("\n📋 Final Story Generation:");
            println!("=========================");
            let story_text =
                extract_text(&response).unwrap_or_else(|| "No story content received".to_string());

            if story_text.is_empty() {
                eprintln!("❌ Team returned empty response");
                return Ok(());
            }

            println!("{}", story_text);

            let elapsed = start_time.elapsed();
            println!(
                "\n⏱️  Story completed in {:.2} seconds",
                elapsed.as_secs_f64()
            );
        }
        Err(e) => {
            eprintln!("❌ Error creating fantasy story: {}", e);
            eprintln!("\n💡 Troubleshooting tips:");
            eprintln!("   • Check your OPENAI_API_KEY is valid");
            eprintln!("   • Try running 'health' to check agent status");
            eprintln!("   • Consider increasing timeout environment variables");
            eprintln!("   • Verify your OpenAI API quota isn't exceeded");
        }
    }

    println!();
    Ok(())
}
