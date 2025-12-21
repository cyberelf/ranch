use crate::agent::{A2AAgentConfig, OpenAIAgentConfig, TaskHandling};
use crate::team::{RouterConfig, TeamAgentConfig, TeamConfig};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::{env, fs};
use thiserror::Error;

/// Errors that can occur during configuration conversion
#[derive(Debug, Error)]
pub enum ConfigConversionError {
    #[error("Wrong protocol type: expected {expected:?}, found {found:?}")]
    WrongProtocol {
        expected: ProtocolType,
        found: ProtocolType,
    },

    #[error("Missing required field: {0}")]
    MissingField(&'static str),

    #[error("Invalid field value for {field}: {value} ({reason})")]
    InvalidValue {
        field: &'static str,
        value: String,
        reason: String,
    },
}

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
    pub agents: Vec<TeamAgentConfigFile>,
    pub router_config: RouterConfig,
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
                agents: team
                    .agents
                    .iter()
                    .map(|agent| TeamAgentConfig {
                        agent_id: agent.agent_id.clone(),
                        role: agent.role.clone(),
                        capabilities: agent.capabilities.clone(),
                    })
                    .collect(),
                router_config: team.router_config.clone(),
            })
            .collect()
    }
}

// ============================================================================
// TryFrom implementations for protocol-specific config conversion
// ============================================================================

/// Convert from `AgentConfig` to `A2AAgentConfig` with validation.
///
/// # Examples
///
/// ```
/// use multi_agent::{AgentConfig, A2AAgentConfig, ProtocolType};
/// use std::collections::HashMap;
///
/// let mut metadata = HashMap::new();
/// metadata.insert("task_handling".to_string(), "poll".to_string());
///
/// let agent_config = AgentConfig {
///     id: "test-agent".to_string(),
///     name: "Test Agent".to_string(),
///     endpoint: "https://example.com/rpc".to_string(),
///     protocol: ProtocolType::A2A,
///     capabilities: vec!["test".to_string()],
///     metadata,
///     timeout_seconds: 30,
///     max_retries: 3,
/// };
///
/// // Convert using TryFrom
/// let a2a_config: A2AAgentConfig = agent_config.try_into()?;
/// assert_eq!(a2a_config.max_retries, 3);
/// # Ok::<(), multi_agent::ConfigConversionError>(())
/// ```
///
/// # Errors
///
/// Returns `ConfigConversionError` if:
/// - Protocol is not `ProtocolType::A2A`
/// - Endpoint is empty
/// - `timeout_seconds` is not between 1 and 300
/// - `max_retries` is greater than 10
impl TryFrom<AgentConfig> for A2AAgentConfig {
    type Error = ConfigConversionError;

    fn try_from(config: AgentConfig) -> Result<Self, Self::Error> {
        // Validate protocol type
        if config.protocol != ProtocolType::A2A {
            return Err(ConfigConversionError::WrongProtocol {
                expected: ProtocolType::A2A,
                found: config.protocol,
            });
        }

        // Validate endpoint
        if config.endpoint.is_empty() {
            return Err(ConfigConversionError::MissingField("endpoint"));
        }

        // Validate timeout_seconds
        if config.timeout_seconds < 1 || config.timeout_seconds > 300 {
            return Err(ConfigConversionError::InvalidValue {
                field: "timeout_seconds",
                value: config.timeout_seconds.to_string(),
                reason: "must be between 1 and 300".to_string(),
            });
        }

        // Validate max_retries
        if config.max_retries > 10 {
            return Err(ConfigConversionError::InvalidValue {
                field: "max_retries",
                value: config.max_retries.to_string(),
                reason: "must be between 0 and 10".to_string(),
            });
        }

        // Parse task_handling from metadata with default
        let task_handling = config
            .metadata
            .get("task_handling")
            .and_then(|v| match v.as_str() {
                "poll" | "poll_until_complete" => Some(TaskHandling::PollUntilComplete),
                "return" | "return_task_info" => Some(TaskHandling::ReturnTaskInfo),
                "reject" | "reject_tasks" => Some(TaskHandling::RejectTasks),
                _ => None,
            })
            .unwrap_or(TaskHandling::PollUntilComplete);

        Ok(A2AAgentConfig {
            local_id: Some(config.id),
            local_name: Some(config.name),
            max_retries: config.max_retries,
            task_handling,
        })
    }
}

/// Convert from `AgentConfig` to `OpenAIAgentConfig` with validation.
///
/// # Examples
///
/// ```
/// use multi_agent::{AgentConfig, OpenAIAgentConfig, ProtocolType};
/// use std::collections::HashMap;
///
/// let mut metadata = HashMap::new();
/// metadata.insert("api_key".to_string(), "sk-test123".to_string());
/// metadata.insert("model".to_string(), "gpt-4".to_string());
/// metadata.insert("temperature".to_string(), "0.7".to_string());
/// metadata.insert("max_tokens".to_string(), "1000".to_string());
///
/// let agent_config = AgentConfig {
///     id: "test-agent".to_string(),
///     name: "Test Agent".to_string(),
///     endpoint: "https://api.openai.com/v1".to_string(),
///     protocol: ProtocolType::OpenAI,
///     capabilities: vec!["test".to_string()],
///     metadata,
///     timeout_seconds: 30,
///     max_retries: 3,
/// };
///
/// // Convert using TryFrom
/// let openai_config: OpenAIAgentConfig = agent_config.try_into()?;
/// assert_eq!(openai_config.model, "gpt-4");
/// assert_eq!(openai_config.temperature, Some(0.7));
/// assert_eq!(openai_config.max_tokens, Some(1000));
/// # Ok::<(), multi_agent::ConfigConversionError>(())
/// ```
///
/// # Errors
///
/// Returns `ConfigConversionError` if:
/// - Protocol is not `ProtocolType::OpenAI`
/// - Endpoint is empty
/// - `api_key` is missing from metadata
/// - `timeout_seconds` is not between 1 and 300
/// - `max_retries` is greater than 10
/// - `temperature` is not between 0.0 and 2.0 (if provided)
/// - `max_tokens` is not between 1 and 4096 (if provided)
impl TryFrom<AgentConfig> for OpenAIAgentConfig {
    type Error = ConfigConversionError;

    fn try_from(config: AgentConfig) -> Result<Self, Self::Error> {
        // Validate protocol type
        if config.protocol != ProtocolType::OpenAI {
            return Err(ConfigConversionError::WrongProtocol {
                expected: ProtocolType::OpenAI,
                found: config.protocol,
            });
        }

        // Validate endpoint
        if config.endpoint.is_empty() {
            return Err(ConfigConversionError::MissingField("endpoint"));
        }

        // Get api_key from metadata or environment (not required at config time)
        let api_key = config
            .metadata
            .get("api_key")
            .cloned()
            .or_else(|| env::var("OPENAI_API_KEY").ok());

        // Validate timeout_seconds
        if config.timeout_seconds < 1 || config.timeout_seconds > 300 {
            return Err(ConfigConversionError::InvalidValue {
                field: "timeout_seconds",
                value: config.timeout_seconds.to_string(),
                reason: "must be between 1 and 300".to_string(),
            });
        }

        // Validate max_retries
        if config.max_retries > 10 {
            return Err(ConfigConversionError::InvalidValue {
                field: "max_retries",
                value: config.max_retries.to_string(),
                reason: "must be between 0 and 10".to_string(),
            });
        }

        // Parse and validate temperature if present
        let temperature = config
            .metadata
            .get("temperature")
            .map(|v| {
                v.parse::<f32>()
                    .map_err(|_| ConfigConversionError::InvalidValue {
                        field: "temperature",
                        value: v.clone(),
                        reason: "must be a valid float".to_string(),
                    })
            })
            .transpose()?;

        if let Some(temp) = temperature {
            if !(0.0..=2.0).contains(&temp) {
                return Err(ConfigConversionError::InvalidValue {
                    field: "temperature",
                    value: temp.to_string(),
                    reason: "must be between 0.0 and 2.0".to_string(),
                });
            }
        }

        // Parse and validate max_tokens if present
        let max_tokens = config
            .metadata
            .get("max_tokens")
            .map(|v| {
                v.parse::<u32>()
                    .map_err(|_| ConfigConversionError::InvalidValue {
                        field: "max_tokens",
                        value: v.clone(),
                        reason: "must be a valid integer".to_string(),
                    })
            })
            .transpose()?;

        if let Some(tokens) = max_tokens {
            if !(1..=4096).contains(&tokens) {
                return Err(ConfigConversionError::InvalidValue {
                    field: "max_tokens",
                    value: tokens.to_string(),
                    reason: "must be between 1 and 4096".to_string(),
                });
            }
        }

        // Parse model from metadata with default
        let model = config
            .metadata
            .get("model")
            .cloned()
            .unwrap_or_else(|| "gpt-3.5-turbo".to_string());

        Ok(OpenAIAgentConfig {
            id: config.id,
            name: config.name,
            description: config
                .metadata
                .get("system_prompt")
                .cloned()
                .unwrap_or_else(|| "OpenAI-compatible agent".to_string()),
            capabilities: config.capabilities,
            api_key,
            max_retries: config.max_retries,
            timeout_seconds: config.timeout_seconds,
            model,
            temperature,
            max_tokens,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Tests for A2AAgentConfig conversion
    // ========================================================================

    #[test]
    fn test_a2a_config_conversion_success() {
        let agent_config = AgentConfig {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            endpoint: "https://example.com/rpc".to_string(),
            protocol: ProtocolType::A2A,
            capabilities: vec!["test".to_string()],
            metadata: HashMap::new(),
            timeout_seconds: 30,
            max_retries: 3,
        };

        let result: Result<A2AAgentConfig, _> = agent_config.try_into();
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.task_handling, TaskHandling::PollUntilComplete);
    }

    #[test]
    fn test_a2a_config_wrong_protocol() {
        let agent_config = AgentConfig {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            endpoint: "https://example.com/rpc".to_string(),
            protocol: ProtocolType::OpenAI,
            capabilities: vec!["test".to_string()],
            metadata: HashMap::new(),
            timeout_seconds: 30,
            max_retries: 3,
        };

        let result: Result<A2AAgentConfig, _> = agent_config.try_into();
        assert!(matches!(
            result,
            Err(ConfigConversionError::WrongProtocol { .. })
        ));
    }

    #[test]
    fn test_a2a_config_missing_endpoint() {
        let agent_config = AgentConfig {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            endpoint: "".to_string(),
            protocol: ProtocolType::A2A,
            capabilities: vec!["test".to_string()],
            metadata: HashMap::new(),
            timeout_seconds: 30,
            max_retries: 3,
        };

        let result: Result<A2AAgentConfig, _> = agent_config.try_into();
        assert!(matches!(
            result,
            Err(ConfigConversionError::MissingField("endpoint"))
        ));
    }

    #[test]
    fn test_a2a_config_invalid_timeout() {
        let agent_config = AgentConfig {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            endpoint: "https://example.com/rpc".to_string(),
            protocol: ProtocolType::A2A,
            capabilities: vec!["test".to_string()],
            metadata: HashMap::new(),
            timeout_seconds: 0,
            max_retries: 3,
        };

        let result: Result<A2AAgentConfig, _> = agent_config.try_into();
        assert!(matches!(
            result,
            Err(ConfigConversionError::InvalidValue {
                field: "timeout_seconds",
                ..
            })
        ));
    }

    #[test]
    fn test_a2a_config_invalid_max_retries() {
        let agent_config = AgentConfig {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            endpoint: "https://example.com/rpc".to_string(),
            protocol: ProtocolType::A2A,
            capabilities: vec!["test".to_string()],
            metadata: HashMap::new(),
            timeout_seconds: 30,
            max_retries: 11,
        };

        let result: Result<A2AAgentConfig, _> = agent_config.try_into();
        assert!(matches!(
            result,
            Err(ConfigConversionError::InvalidValue {
                field: "max_retries",
                ..
            })
        ));
    }

    #[test]
    fn test_a2a_config_with_task_handling() {
        let mut metadata = HashMap::new();
        metadata.insert("task_handling".to_string(), "return_task_info".to_string());

        let agent_config = AgentConfig {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            endpoint: "https://example.com/rpc".to_string(),
            protocol: ProtocolType::A2A,
            capabilities: vec!["test".to_string()],
            metadata,
            timeout_seconds: 30,
            max_retries: 3,
        };

        let result: Result<A2AAgentConfig, _> = agent_config.try_into();
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.task_handling, TaskHandling::ReturnTaskInfo);
    }

    // ========================================================================
    // Tests for OpenAIAgentConfig conversion
    // ========================================================================

    #[test]
    fn test_openai_config_conversion_success() {
        let mut metadata = HashMap::new();
        metadata.insert("api_key".to_string(), "sk-test123".to_string());

        let agent_config = AgentConfig {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            endpoint: "https://api.openai.com/v1".to_string(),
            protocol: ProtocolType::OpenAI,
            capabilities: vec!["test".to_string()],
            metadata,
            timeout_seconds: 30,
            max_retries: 3,
        };

        let result: Result<OpenAIAgentConfig, _> = agent_config.try_into();
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.api_key, Some("sk-test123".to_string()));
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_openai_config_wrong_protocol() {
        let mut metadata = HashMap::new();
        metadata.insert("api_key".to_string(), "sk-test123".to_string());

        let agent_config = AgentConfig {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            endpoint: "https://api.openai.com/v1".to_string(),
            protocol: ProtocolType::A2A,
            capabilities: vec!["test".to_string()],
            metadata,
            timeout_seconds: 30,
            max_retries: 3,
        };

        let result: Result<OpenAIAgentConfig, _> = agent_config.try_into();
        assert!(matches!(
            result,
            Err(ConfigConversionError::WrongProtocol { .. })
        ));
    }

    #[test]
    fn test_openai_config_missing_api_key() {
        let agent_config = AgentConfig {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            endpoint: "https://api.openai.com/v1".to_string(),
            protocol: ProtocolType::OpenAI,
            capabilities: vec!["test".to_string()],
            metadata: HashMap::new(),
            timeout_seconds: 30,
            max_retries: 3,
        };

        // API key is now optional - can come from environment or be set later
        let result: Result<OpenAIAgentConfig, _> = agent_config.try_into();
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.api_key, None); // No API key set
    }

    #[test]
    fn test_openai_config_with_temperature() {
        let mut metadata = HashMap::new();
        metadata.insert("api_key".to_string(), "sk-test123".to_string());
        metadata.insert("temperature".to_string(), "0.7".to_string());

        let agent_config = AgentConfig {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            endpoint: "https://api.openai.com/v1".to_string(),
            protocol: ProtocolType::OpenAI,
            capabilities: vec!["test".to_string()],
            metadata,
            timeout_seconds: 30,
            max_retries: 3,
        };

        let result: Result<OpenAIAgentConfig, _> = agent_config.try_into();
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.temperature, Some(0.7));
    }

    #[test]
    fn test_openai_config_invalid_temperature() {
        let mut metadata = HashMap::new();
        metadata.insert("api_key".to_string(), "sk-test123".to_string());
        metadata.insert("temperature".to_string(), "3.0".to_string());

        let agent_config = AgentConfig {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            endpoint: "https://api.openai.com/v1".to_string(),
            protocol: ProtocolType::OpenAI,
            capabilities: vec!["test".to_string()],
            metadata,
            timeout_seconds: 30,
            max_retries: 3,
        };

        let result: Result<OpenAIAgentConfig, _> = agent_config.try_into();
        assert!(matches!(
            result,
            Err(ConfigConversionError::InvalidValue {
                field: "temperature",
                ..
            })
        ));
    }

    #[test]
    fn test_openai_config_with_max_tokens() {
        let mut metadata = HashMap::new();
        metadata.insert("api_key".to_string(), "sk-test123".to_string());
        metadata.insert("max_tokens".to_string(), "1000".to_string());

        let agent_config = AgentConfig {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            endpoint: "https://api.openai.com/v1".to_string(),
            protocol: ProtocolType::OpenAI,
            capabilities: vec!["test".to_string()],
            metadata,
            timeout_seconds: 30,
            max_retries: 3,
        };

        let result: Result<OpenAIAgentConfig, _> = agent_config.try_into();
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.max_tokens, Some(1000));
    }

    #[test]
    fn test_openai_config_invalid_max_tokens() {
        let mut metadata = HashMap::new();
        metadata.insert("api_key".to_string(), "sk-test123".to_string());
        metadata.insert("max_tokens".to_string(), "5000".to_string());

        let agent_config = AgentConfig {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            endpoint: "https://api.openai.com/v1".to_string(),
            protocol: ProtocolType::OpenAI,
            capabilities: vec!["test".to_string()],
            metadata,
            timeout_seconds: 30,
            max_retries: 3,
        };

        let result: Result<OpenAIAgentConfig, _> = agent_config.try_into();
        assert!(matches!(
            result,
            Err(ConfigConversionError::InvalidValue {
                field: "max_tokens",
                ..
            })
        ));
    }
    
    
    #[test]
    fn test_protocol_type_equality() {
        assert_eq!(ProtocolType::OpenAI, ProtocolType::OpenAI);
        assert_eq!(ProtocolType::A2A, ProtocolType::A2A);
        assert_ne!(ProtocolType::OpenAI, ProtocolType::A2A);
    }

    #[test]
    fn test_config_conversion_error_display() {
        let err = ConfigConversionError::WrongProtocol {
            expected: ProtocolType::A2A,
            found: ProtocolType::OpenAI,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Wrong protocol type"));
        assert!(msg.contains("A2A"));
        assert!(msg.contains("OpenAI"));

        let err2 = ConfigConversionError::MissingField("endpoint");
        let msg2 = format!("{}", err2);
        assert!(msg2.contains("Missing required field"));
        assert!(msg2.contains("endpoint"));

        let err3 = ConfigConversionError::InvalidValue {
            field: "timeout",
            value: "0".to_string(),
            reason: "must be positive".to_string(),
        };
        let msg3 = format!("{}", err3);
        assert!(msg3.contains("Invalid field value"));
        assert!(msg3.contains("timeout"));
        assert!(msg3.contains("must be positive"));
    }

    #[test]
    fn test_agent_config_clone() {
        let config = AgentConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            endpoint: "https://example.com".to_string(),
            protocol: ProtocolType::OpenAI,
            capabilities: vec!["test".to_string()],
            metadata: HashMap::new(),
            timeout_seconds: 30,
            max_retries: 3,
        };

        let cloned = config.clone();
        assert_eq!(config.id, cloned.id);
        assert_eq!(config.protocol, cloned.protocol);
    }

    #[test]
    fn test_protocol_type_debug() {
        let openai = ProtocolType::OpenAI;
        let a2a = ProtocolType::A2A;

        let openai_debug = format!("{:?}", openai);
        let a2a_debug = format!("{:?}", a2a);

        assert!(openai_debug.contains("OpenAI"));
        assert!(a2a_debug.contains("A2A"));
    }}