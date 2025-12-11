# Config Conversion Traits Contract

**Purpose**: Define `From<AgentConfig>` trait implementations for ergonomic config conversions

## Overview

Enable automatic conversion from generic `AgentConfig` to protocol-specific configs (`A2AAgentConfig`, `OpenAIAgentConfig`) using Rust's `TryFrom` trait for fallible conversions.

**Benefit**: Eliminates 15-20 lines of boilerplate per agent configuration while providing clear error messages for invalid configs.

**Design Decision**: Use `TryFrom` as the primary API since conversions can fail (wrong protocol type, missing required fields, invalid values). The `From` trait would require panic/assert which violates Rust idioms.

## Source Type: AgentConfig

```rust
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub protocol: ProtocolType,      // A2A | OpenAI
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

pub enum ProtocolType {
    A2A,
    OpenAI,
}
```

## Target Type 1: A2AAgentConfig

### Implementation Contract

```rust
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
        
        // Validate timeout
        if config.timeout_seconds == 0 || config.timeout_seconds > 300 {
            return Err(ConfigConversionError::InvalidValue {
                field: "timeout_seconds",
                value: config.timeout_seconds.to_string(),
                reason: "must be between 1 and 300 seconds".to_string(),
            });
        }
        
        // Validate max_retries
        if config.max_retries > 10 {
            return Err(ConfigConversionError::InvalidValue {
                field: "max_retries",
                value: config.max_retries.to_string(),
                reason: "must be 0-10".to_string(),
            });
        }
        
        // Extract authentication from metadata
        let auth = if let Some(api_key) = config.metadata.get("api_key") {
            Some(Authenticator::ApiKey(ApiKeyAuth {
                key: api_key.clone(),
                header: config.metadata
                    .get("auth_header")
                    .cloned()
                    .unwrap_or_else(|| "X-API-Key".to_string()),
            }))
        } else if let Some(bearer_token) = config.metadata.get("bearer_token") {
            Some(Authenticator::Bearer(BearerAuth {
                token: bearer_token.clone(),
            }))
        } else {
            None
        };
        
        // Extract task handling from metadata
        let task_handling = config.metadata
            .get("task_handling")
            .and_then(|s| match s.as_str() {
                "poll" => Some(TaskHandling::PollUntilComplete),
                "return" => Some(TaskHandling::ReturnTaskInfo),
                "reject" => Some(TaskHandling::RejectTasks),
                _ => None,
            })
            .unwrap_or(TaskHandling::PollUntilComplete);
        
        Ok(Self {
            endpoint: config.endpoint,
            agent_id: Some(config.id),
            timeout: Duration::from_secs(config.timeout_seconds),
            max_retries: config.max_retries,
            auth,
            task_handling,
        })
    }
}
```

### Field Mapping

| AgentConfig Field | A2AAgentConfig Field | Transformation |
|-------------------|----------------------|----------------|
| `endpoint` | `endpoint` | Direct copy |
| `id` | `agent_id` | Wrapped in Some() |
| `timeout_seconds` | `timeout` | Convert to Duration |
| `max_retries` | `max_retries` | Direct copy |
| `metadata["api_key"]` | `auth` | Create ApiKeyAuth |
| `metadata["bearer_token"]` | `auth` | Create BearerAuth |
| `metadata["task_handling"]` | `task_handling` | Parse string to enum |

### Validation Rules

- `protocol` MUST be `ProtocolType::A2A` (returns WrongProtocol error otherwise)
- `endpoint` MUST be non-empty (returns MissingField error)
- `timeout_seconds` MUST be 1-300 (returns InvalidValue error)
- `max_retries` MUST be 0-10 (returns InvalidValue error)
- If both `api_key` and `bearer_token` in metadata, prefer `api_key`

### Usage Example

```rust
// Load from TOML
let config: AgentConfig = parse_from_toml("agent.toml")?;

// Convert to A2AAgentConfig (fallible)
let a2a_config = A2AAgentConfig::try_from(config)
    .map_err(|e| format!("Invalid A2A config: {}", e))?;

// Or with ? operator
let a2a_config: A2AAgentConfig = config.try_into()?;

// Create A2AAgent
let transport = JsonRpcTransport::new(&a2a_config.endpoint)?;
let client = A2aClient::new(Arc::new(transport));
let agent = A2AAgent::with_config(client, a2a_config);
```

### TOML Configuration Example

```toml
[[agents]]
id = "research-agent"
name = "Research Agent"
endpoint = "https://research.example.com/rpc"
protocol = "a2a"
capabilities = ["research", "fact-checking"]
timeout_seconds = 30
max_retries = 3

[agents.metadata]
api_key = "sk-abc123"
auth_header = "X-API-Key"
task_handling = "poll"
```

---

## Target Type 2: OpenAIAgentConfig

### Implementation Contract

```rust
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
        
        // Extract required OpenAI-specific fields from metadata
        let api_key = config.metadata
            .get("api_key")
            .cloned()
            .ok_or(ConfigConversionError::MissingField("metadata.api_key"))?;
        
        let model = config.metadata
            .get("model")
            .cloned()
            .unwrap_or_else(|| "gpt-4".to_string());
        
        // Validate timeout
        if config.timeout_seconds == 0 || config.timeout_seconds > 300 {
            return Err(ConfigConversionError::InvalidValue {
                field: "timeout_seconds",
                value: config.timeout_seconds.to_string(),
                reason: "must be between 1 and 300 seconds".to_string(),
            });
        }
        
        // Validate max_retries
        if config.max_retries > 10 {
            return Err(ConfigConversionError::InvalidValue {
                field: "max_retries",
                value: config.max_retries.to_string(),
                reason: "must be 0-10".to_string(),
            });
        }
        
        // Extract optional fields with validation
        let temperature = if let Some(temp_str) = config.metadata.get("temperature") {
            let temp = temp_str.parse::<f32>()
                .map_err(|_| ConfigConversionError::InvalidValue {
                    field: "metadata.temperature",
                    value: temp_str.clone(),
                    reason: "must be a valid float".to_string(),
                })?;
            
            if temp < 0.0 || temp > 2.0 {
                return Err(ConfigConversionError::InvalidValue {
                    field: "metadata.temperature",
                    value: temp.to_string(),
                    reason: "must be between 0.0 and 2.0".to_string(),
                });
            }
            Some(temp)
        } else {
            None
        };
        
        let max_tokens = if let Some(tokens_str) = config.metadata.get("max_tokens") {
            let tokens = tokens_str.parse::<u32>()
                .map_err(|_| ConfigConversionError::InvalidValue {
                    field: "metadata.max_tokens",
                    value: tokens_str.clone(),
                    reason: "must be a valid integer".to_string(),
                })?;
            
            if tokens == 0 || tokens > 4096 {
                return Err(ConfigConversionError::InvalidValue {
                    field: "metadata.max_tokens",
                    value: tokens.to_string(),
                    reason: "must be between 1 and 4096".to_string(),
                });
            }
            Some(tokens)
        } else {
            None
        };
        
        Ok(Self {
            endpoint: config.endpoint,
            api_key,
            model,
            timeout: Duration::from_secs(config.timeout_seconds),
            max_retries: config.max_retries,
            temperature,
            max_tokens,
        })
    }
}
```

### Field Mapping

| AgentConfig Field | OpenAIAgentConfig Field | Transformation |
|-------------------|-------------------------|----------------|
| `endpoint` | `endpoint` | Direct copy |
| `metadata["api_key"]` | `api_key` | Required, panic if missing |
| `metadata["model"]` | `model` | Default "gpt-4" if missing |
| `timeout_seconds` | `timeout` | Convert to Duration |
| `max_retries` | `max_retries` | Direct copy |
| `metadata["temperature"]` | `temperature` | Parse, validate 0.0-2.0 |
| `metadata["max_tokens"]` | `max_tokens` | Parse, validate 1-4096 |

### Validation Rules

- `protocol` MUST be `ProtocolType::OpenAI` (returns WrongProtocol error)
- `metadata["api_key"]` MUST exist and be non-empty (returns MissingField error)
- `endpoint` MUST be non-empty (returns MissingField error)
- `temperature` if present MUST be 0.0-2.0 (returns InvalidValue error)
- `max_tokens` if present MUST be 1-4096 (returns InvalidValue error)
- `timeout_seconds` MUST be 1-300 (returns InvalidValue error)
- `max_retries` MUST be 0-10 (returns InvalidValue error)

### Usage Example

```rust
// Load from TOML
let config: AgentConfig = parse_from_toml("agent.toml")?;

// Convert to OpenAIAgentConfig (fallible)
let openai_config = OpenAIAgentConfig::try_from(config)
    .map_err(|e| format!("Invalid OpenAI config: {}", e))?;

// Or with ? operator
let openai_config: OpenAIAgentConfig = config.try_into()?;

// Create OpenAIAgent
let agent = OpenAIAgent::with_config(openai_config);
```

### TOML Configuration Example

```toml
[[agents]]
id = "writing-agent"
name = "Writing Agent"
endpoint = "https://api.openai.com/v1/chat/completions"
protocol = "openai"
capabilities = ["writing", "editing"]
timeout_seconds = 60
max_retries = 3

[agents.metadata]
api_key = "sk-openai-xyz789"
model = "gpt-4"
temperature = "0.7"
max_tokens = "2000"
```

---

## Error Type

```rust
#[derive(Debug, thiserror::Error)]
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
```

## Testing Contract

### Unit Tests Required

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_a2a_config_from_agent_config() {
        let agent_config = AgentConfig {
            id: "test-agent".to_string(),
            endpoint: "https://example.com/rpc".to_string(),
            protocol: ProtocolType::A2A,
            timeout_seconds: 30,
            max_retries: 3,
            metadata: HashMap::new(),
        };
        
        let a2a_config = A2AAgentConfig::try_from(agent_config)
            .expect("Should convert valid A2A config");
        
        assert_eq!(a2a_config.endpoint, "https://example.com/rpc");
        assert_eq!(a2a_config.timeout, Duration::from_secs(30));
    }
    
    #[test]
    fn test_a2a_config_wrong_protocol_errors() {
        let agent_config = AgentConfig {
            protocol: ProtocolType::OpenAI,
            // ...
        };
        
        let result = A2AAgentConfig::try_from(agent_config);
        assert!(matches!(result, Err(ConfigConversionError::WrongProtocol { .. })));
    }
    
    #[test]
    fn test_openai_config_from_agent_config() {
        let mut metadata = HashMap::new();
        metadata.insert("api_key".to_string(), "sk-test-123".to_string());
        metadata.insert("model".to_string(), "gpt-4".to_string());
        
        let agent_config = AgentConfig {
            id: "openai-agent".to_string(),
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            protocol: ProtocolType::OpenAI,
            timeout_seconds: 60,
            max_retries: 3,
            metadata,
        };
        
        let openai_config = OpenAIAgentConfig::try_from(agent_config)
            .expect("Should convert valid OpenAI config");
        
        assert_eq!(openai_config.api_key, "sk-test-123");
        assert_eq!(openai_config.model, "gpt-4");
    }
    
    #[test]
    fn test_auth_extraction_from_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("api_key".to_string(), "secret-key".to_string());
        metadata.insert("auth_header".to_string(), "X-Custom-Key".to_string());
        
        let agent_config = AgentConfig {
            protocol: ProtocolType::A2A,
            endpoint: "https://example.com/rpc".to_string(),
            metadata,
            // ...
        };
        
        let a2a_config = A2AAgentConfig::try_from(agent_config)
            .expect("Should convert");
        
        assert!(matches!(a2a_config.auth, Some(Authenticator::ApiKey(_))));
    }
    
    #[test]
    fn test_task_handling_parsing() {
        let mut metadata = HashMap::new();
        metadata.insert("task_handling".to_string(), "poll".to_string());
        
        let agent_config = AgentConfig {
            protocol: ProtocolType::A2A,
            endpoint: "https://example.com/rpc".to_string(),
            metadata,
            // ...
        };
        
        let a2a_config = A2AAgentConfig::try_from(agent_config)
            .expect("Should convert");
        
        assert!(matches!(a2a_config.task_handling, TaskHandling::PollUntilComplete));
    }
}
```

### Integration Test

```rust
#[test]
fn test_config_roundtrip_from_toml() {
    let toml_str = r#"
        [[agents]]
        id = "test"
        endpoint = "https://example.com"
        protocol = "a2a"
        timeout_seconds = 30
        max_retries = 3
        
        [agents.metadata]
        api_key = "secret"
    "#;
    
    let config: Config = toml::from_str(toml_str).unwrap();
    let agent_config = &config.agents[0];
    
    // Convert to protocol-specific config (fallible)
    let a2a_config = A2AAgentConfig::try_from(agent_config.clone())
        .expect("Should convert valid config");
    
    // Verify fields
    assert!(a2a_config.auth.is_some());
}
```

## Benefits Summary

### Before (Manual Conversion)
```rust
let a2a_config = A2AAgentConfig {
    endpoint: agent_config.endpoint.clone(),
    agent_id: Some(agent_config.id.clone()),
    timeout: Duration::from_secs(agent_config.timeout_seconds),
    max_retries: agent_config.max_retries,
    auth: agent_config.metadata.get("api_key").map(|key| {
        Authenticator::ApiKey(ApiKeyAuth {
            key: key.clone(),
            header: "X-API-Key".to_string(),
        })
    }),
    task_handling: TaskHandling::PollUntilComplete,
};
// 15 lines of boilerplate
```

### After (Trait Conversion)
```rust
let a2a_config: A2AAgentConfig = agent_config.into();
// 1 line, clear and idiomatic
```

## Implementation Checklist

- [ ] Define `ConfigConversionError` type
  - [ ] WrongProtocol variant
  - [ ] MissingField variant
  - [ ] InvalidValue variant
  - [ ] Implement Display and Error traits (via thiserror)
  
- [ ] Implement `TryFrom<AgentConfig>` for `A2AAgentConfig`
  - [ ] Protocol validation (WrongProtocol error)
  - [ ] Endpoint validation (MissingField error)
  - [ ] Timeout validation (InvalidValue error)
  - [ ] Max retries validation (InvalidValue error)
  - [ ] Auth extraction from metadata
  - [ ] Task handling parsing with default
  
- [ ] Implement `TryFrom<AgentConfig>` for `OpenAIAgentConfig`
  - [ ] Protocol validation (WrongProtocol error)
  - [ ] Endpoint validation (MissingField error)
  - [ ] API key validation (MissingField error)
  - [ ] Timeout validation (InvalidValue error)
  - [ ] Max retries validation (InvalidValue error)
  - [ ] Temperature parsing and validation (InvalidValue error)
  - [ ] Max tokens parsing and validation (InvalidValue error)
  
- [ ] Add unit tests
  - [ ] Successful conversions (valid configs)
  - [ ] Wrong protocol errors
  - [ ] Missing field errors (endpoint, api_key)
  - [ ] Invalid value errors (timeout, temperature, etc.)
  - [ ] Metadata extraction (auth, task_handling)
  - [ ] Edge cases (empty strings, out of range values)
  
- [ ] Update examples to use `.try_into()?`
  - [ ] simple_team.rs
  - [ ] supervisor_team.rs
  - [ ] workflow_team.rs
  - [ ] remote_agents.rs
  
- [ ] Add rustdoc with examples
  - [ ] Usage examples in doc comments
  - [ ] Field mapping tables
  - [ ] TOML configuration examples
