use multi_agent::*;
use std::sync::Arc;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
    let config = Config::from_file(&config_path)?;

    let agent_manager = Arc::new(AgentManager::new());

    // Register agents from config
    for agent_config in config.to_agent_configs() {
        // Create A2A client for the agent endpoint
        let transport = Arc::new(JsonRpcTransport::new(&agent_config.endpoint)?);
        let client = A2aClient::new(transport);
        
        // Create remote agent
        let agent = Arc::new(RemoteAgent::new(client));
        
        // Register with agent manager (automatically extracts ID from info)
        let _id = agent_manager.register(agent).await?;
    }

    let teams: Vec<Arc<Team>> = config
        .to_team_configs()
        .into_iter()
        .map(|team_config| Arc::new(Team::new(team_config, agent_manager.clone())))
        .collect();

    if teams.is_empty() {
        eprintln!("No teams configured");
        return Ok(());
    }

    let team = teams[0].clone();

    // Create a simple test message
    let message = Message::user_text("What are the latest developments in Rust async programming?");

    match team.process_message(message).await {
        Ok(response) => {
            // Extract text from response message parts
            for part in response.parts {
                if let Part::Text(TextPart { text, .. }) = part {
                    println!("Response: {}", text);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    let server = TeamServer::new(team);
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);

    println!("Starting server on port {}", port);
    server.start(port).await?;

    Ok(())
}
