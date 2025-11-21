use crate::team::{TeamConfig, TeamAgentConfig, TeamMode, SchedulerConfig};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Agent configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub protocol: ProtocolType,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

/// Protocol type for agent communication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolType {
    OpenAI,
    A2A,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub agents: Vec<AgentConfigFile>,
    pub teams: Vec<TeamConfigFile>,
}

#[derive(Debug, Deserialize)]
pub struct AgentConfigFile {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub protocol: String,
    pub capabilities: Vec<String>,
    pub timeout_seconds: Option<u64>,
    pub max_retries: Option<u32>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct TeamConfigFile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub mode: String,
    pub agents: Vec<TeamAgentConfigFile>,
    pub scheduler_config: SchedulerConfig,
}

#[derive(Debug, Deserialize)]
pub struct TeamAgentConfigFile {
    pub agent_id: String,
    pub role: String,
    pub capabilities: Vec<String>,
}



impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn to_agent_configs(&self) -> Vec<AgentConfig> {
        self.agents
            .iter()
            .map(|agent| AgentConfig {
                id: agent.id.clone(),
                name: agent.name.clone(),
                endpoint: agent.endpoint.clone(),
                protocol: match agent.protocol.as_str() {
                    "openai" => ProtocolType::OpenAI,
                    "a2a" => ProtocolType::A2A,
                    _ => ProtocolType::OpenAI,
                },
                capabilities: agent.capabilities.clone(),
                metadata: agent.metadata.clone().unwrap_or_default(),
                timeout_seconds: agent.timeout_seconds.unwrap_or(30),
                max_retries: agent.max_retries.unwrap_or(3),
            })
            .collect()
    }

    pub fn to_team_configs(&self) -> Vec<TeamConfig> {
        self.teams
            .iter()
            .map(|team| TeamConfig {
                id: team.id.clone(),
                name: team.name.clone(),
                description: team.description.clone(),
                mode: match team.mode.as_str() {
                    "supervisor" => TeamMode::Supervisor,
                    "workflow" => TeamMode::Workflow,
                    _ => TeamMode::Supervisor,
                },
                agents: team
                    .agents
                    .iter()
                    .map(|agent| TeamAgentConfig {
                        agent_id: agent.agent_id.clone(),
                        role: agent.role.clone(),
                        capabilities: agent.capabilities.clone(),
                    })
                    .collect(),
                scheduler_config: team.scheduler_config.clone(),
            })
            .collect()
    }
}