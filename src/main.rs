use multi_agent::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
    let config = Config::from_file(&config_path)?;

    let agent_manager = Arc::new(AgentManager::new());

    let openai_api_key = env::var("OPENAI_API_KEY").ok();
    let a2a_auth_token = env::var("A2A_AUTH_TOKEN").ok();

    for agent_config in config.to_agent_configs() {
        let protocol = protocols::create_protocol_adapter(
            &agent_config.protocol,
            match agent_config.protocol {
                ProtocolType::OpenAI => openai_api_key.clone(),
                ProtocolType::A2A => a2a_auth_token.clone(),
            },
        );

        let agent = Arc::new(RemoteAgent::new(agent_config, protocol));
        agent_manager.register_agent(agent).await?;
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

    let message = AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: "user".to_string(),
        content: "What are the latest developments in Rust async programming?".to_string(),
        metadata: HashMap::new(),
    };

    match team.process_message(message).await {
        Ok(response) => {
            println!("Response: {}", response.content);
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
