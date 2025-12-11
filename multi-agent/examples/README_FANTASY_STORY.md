# Fantasy Story Writing Multi-Agent Example

This example demonstrates the collaborative power of the multi-agent framework by creating fantasy stories using specialized AI agents with different roles and capabilities.

## 🎭 Overview

The fantasy story writer uses three specialized agents working together in a coordinated workflow:

1. **Story Orchestrator (OpenAI GPT-4)** - Plans the narrative structure, themes, and direction
2. **Story Composer (OpenAI GPT-4)** - Writes the actual prose and creates vivid scenes
3. **Story Advisor (A2A Agent)** - Reviews the work and provides improvement suggestions

## 🏗️ Architecture

### Agent Roles

#### 🎯 Story Orchestrator
- **Protocol**: OpenAI
- **Model**: GPT-4
- **Role**: Creative director and story planner
- **Responsibilities**:
  - Creates story concepts and themes
  - Designs characters and world-building
  - Establishes plot structure
  - Identifies key scenes
  - Coordinates between other agents

#### ✍️ Story Composer
- **Protocol**: OpenAI
- **Model**: GPT-4
- **Role**: Prose writer and scene creator
- **Responsibilities**:
  - Writes immersive fantasy scenes
  - Creates vivid descriptions
  - Crafts compelling dialogue
  - Implements the orchestrator's vision

#### 🔍 Story Advisor
- **Protocol**: A2A
- **Role**: Editor and quality controller
- **Responsibilities**:
  - Reviews story consistency
  - Checks pacing and flow
  - Suggests improvements
  - Ensures narrative coherence

### Workflow Process

The agents work in a sequential workflow:

```
🎯 Orchestrator → ✍️ Composer → 🔍 Advisor
     ↓              ↓             ↓
  Story Plan   →  Prose Writing → Review & Edit
```

1. **Orchestrator**: Receives the story topic and creates a comprehensive plan
2. **Composer**: Takes the plan and writes the actual story prose
3. **Advisor**: Reviews the complete work and provides feedback

## 🚀 Getting Started

### Prerequisites

#### Simple Version (Recommended for Getting Started)
1. **OpenAI API Key**: Required for both orchestrator and composer agents
   ```bash
   export OPENAI_API_KEY="your-openai-api-key-here"
   ```

2. **OpenAI Base URL (Optional)**: Customize the OpenAI-compatible endpoint
   ```bash
   # Default: https://api.openai.com/v1
   # Custom example:
   export OPENAI_BASE_URL="https://your-custom-endpoint.com/v1"

   # Azure OpenAI example:
   export OPENAI_BASE_URL="https://your-resource.openai.azure.com/openai/deployments/your-deployment/chat/completions?api-version=2023-12-01-preview"
   ```

3. **Agent Models (Optional)**: Customize models for each agent
   ```bash
   # Default: gpt-4 for both agents
   # Orchestrator model:
   export ORCHESTRATOR_MODEL="gpt-4"

   # Composer model:
   export COMPOSER_MODEL="gpt-4"

   # Examples with different models:
   export ORCHESTRATOR_MODEL="gpt-3.5-turbo"
   export COMPOSER_MODEL="gpt-4-turbo"

   # Claude models (with compatible endpoint):
   export ORCHESTRATOR_MODEL="claude-3-opus-20240229"
   export COMPOSER_MODEL="claude-3-sonnet-20240229"
   ```

4. **Agent Timeouts (Optional)**: Configure timeout for each agent
   ```bash
   # Default: 90s for orchestrator, 120s for composer
   # Orchestrator timeout:
   export ORCHESTRATOR_TIMEOUT="90"

   # Composer timeout:
   export COMPOSER_TIMEOUT="120"

   # For slow connections or complex requests:
   export ORCHESTRATOR_TIMEOUT="180"
   export COMPOSER_TIMEOUT="240"
   ```

#### Full Version (with A2A Agent)
1. **OpenAI API Key**: For the orchestrator and composer agents
   ```bash
   export OPENAI_API_KEY="your-openai-api-key-here"
   ```

2. **A2A Agent Server**: For the advisor agent
   - Run an A2A-compatible agent on `http://localhost:8081/rpc`
   - Example: Use any A2A protocol server with storytelling capabilities

3. **Configuration**: Update the `fantasy_story_config.toml` if needed

### Running the Example

#### Quick Start (Simple Version - 2 OpenAI Agents)
```bash
# Requires only OPENAI_API_KEY
cargo run --example simple_fantasy_writer
```

#### Full Version (Configuration-based - Mixed A2A + OpenAI)
```bash
# Requires OPENAI_API_KEY and A2A server running
CONFIG_PATH=multi-agent/fantasy_story_config.toml cargo run --example fantasy_story_writer
```

2. **Interactive Commands**:
   - `story <topic>` - Create a fantasy story about the given topic
   - `health` - Check all agent health status
   - `agents` - List available agents and their capabilities
   - `quit` - Exit the program

### Example Usage

#### Simple Version (with Default OpenAI)
```bash
$ export OPENAI_API_KEY="your-api-key-here"
$ cargo run --example simple_fantasy_writer
📚 Simple Fantasy Story Writer Example
====================================
🔗 Using OpenAI endpoint: https://api.openai.com/v1
🎯 Orchestrator model: gpt-4
✍️ Composer model: gpt-4
✅ Registered Story Orchestrator: openai-agent-uuid-1
✅ Registered Story Composer: openai-agent-uuid-2

📝 What fantasy story would you like to create? story a dragon's lost treasure

📖 Writing Fantasy Story About: 'a dragon's lost treasure'
=========================================
🎯 Step 1: Creating story plan...
✅ Story plan created:
[Story plan details...]

✍️ Step 2: Writing the story...
📚 Complete Fantasy Story:
========================
[Full fantasy story...]
⏱️ Story completed in 42.15 seconds
```

#### With Custom OpenAI-Compatible Endpoint
```bash
$ export OPENAI_API_KEY="your-api-key-here"
$ export OPENAI_BASE_URL="https://api.anthropic.com/v1"
$ cargo run --example simple_fantasy_writer
🔗 Using OpenAI endpoint: https://api.anthropic.com/v1
```

#### With Azure OpenAI
```bash
$ export OPENAI_API_KEY="your-azure-api-key"
$ export OPENAI_BASE_URL="https://your-resource.openai.azure.com/openai/deployments/your-deployment"
$ cargo run --example simple_fantasy_writer
🔗 Using OpenAI endpoint: https://your-resource.openai.azure.com/openai/deployments/your-deployment
```

#### With Different Models for Each Agent
```bash
$ export OPENAI_API_KEY="your-api-key-here"
$ export ORCHESTRATOR_MODEL="gpt-3.5-turbo"  # Faster planning
$ export COMPOSER_MODEL="gpt-4"             # Higher quality writing
$ cargo run --example simple_fantasy_writer
🔗 Using OpenAI endpoint: https://api.openai.com/v1
🎯 Orchestrator model: gpt-3.5-turbo
✍️ Composer model: gpt-4
```

#### With Claude Models (Anthropic)
```bash
$ export OPENAI_API_KEY="your-anthropic-api-key"
$ export OPENAI_BASE_URL="https://api.anthropic.com/v1"
$ export ORCHESTRATOR_MODEL="claude-3-opus-20240229"  # Strategic planning
$ export COMPOSER_MODEL="claude-3-sonnet-20240229"     # Creative writing
$ cargo run --example simple_fantasy_writer
🔗 Using OpenAI endpoint: https://api.anthropic.com/v1
🎯 Orchestrator model: claude-3-opus-20240229
✍️ Composer model: claude-3-sonnet-20240229
```

## 📁 File Structure

```
multi-agent/
├── examples/
│   ├── fantasy_story_writer.rs          # Main example implementation
│   └── README_FANTASY_STORY.md          # This documentation
├── fantasy_story_config.toml            # Agent configuration
└── src/                                 # Framework source code
```

## 🔧 Troubleshooting

### Common Issues and Solutions

#### **Timeouts**
```bash
# Increase timeouts for slow connections
export ORCHESTRATOR_TIMEOUT="180"
export COMPOSER_TIMEOUT="240"
```

#### **API Key Issues**
```bash
# Verify your OpenAI API key
curl -H "Authorization: Bearer $OPENAI_API_KEY" \
     -H "Content-Type: application/json" \
     -d '{"model":"gpt-3.5-turbo","messages":[{"role":"user","content":"test"}],"max_tokens":5}' \
     https://api.openai.com/v1/chat/completions
```

#### **Agent Health**
```bash
# Check if agents are healthy
cargo run --example simple_fantasy_writer
health
```

**Error Messages and Solutions:**
- `Operation timed out` → Increase timeout values or check network
- `Authentication failed` → Verify OPENAI_API_KEY is valid
- `Agent not found` → Check agent registration logs
- `Empty response` → Check model availability and API quota

## ⚙️ Configuration

### Agent Configuration

Each agent can be customized in `fantasy_story_config.toml`:

```toml
[[agents]]
id = "orchestrator"
name = "Story Orchestrator"
endpoint = "https://api.openai.com/v1"
protocol = "openai"
capabilities = ["story_planning", "creative_direction"]

[agents.metadata]
model = "gpt-4"
temperature = "0.8"
max_tokens = "1500"
system_prompt = "You are a master storyteller..."
```

### Team Configuration

The workflow is configured as a team:

```toml
[[teams]]
id = "fantasy-story-team"
mode = "workflow"

[teams.scheduler_config]
type = "workflow"

[[teams.scheduler_config.steps]]
agent_id = "orchestrator"
order = 1

[[teams.scheduler_config.steps]]
agent_id = "composer"
order = 2

[[teams.scheduler_config.steps]]
agent_id = "advisor"
order = 3
```

## 🔧 Customization

### Adding New Agents

1. **Update the config file** with new agent definitions
2. **Choose appropriate protocol** (OpenAI or A2A)
3. **Define capabilities** and roles
4. **Add to the workflow** with proper ordering

### Modifying the Workflow

1. **Change team mode** from "workflow" to "supervisor" if needed
2. **Reorder steps** in the scheduler configuration
3. **Add conditions** for conditional workflow execution
4. **Adjust timeouts and retries** for each agent

### Prompts and Instructions

Customize the agent behavior by modifying the `system_prompt` in each agent's metadata:

```toml
[agents.metadata]
system_prompt = "You are a specialized agent focused on..."
```

## 🎯 Use Cases

This example demonstrates several powerful concepts:

- **Collaborative AI**: Multiple agents working together
- **Specialized Roles**: Each agent has distinct capabilities
- **Mixed Protocols**: Combining OpenAI and A2A agents
- **Workflow Orchestration**: Sequential processing with handoffs
- **Error Handling**: Robust error management across agents
- **Configuration Management**: TOML-based agent and team definitions

## 🛠️ Troubleshooting

### Common Issues

1. **Missing OpenAI API Key**
   ```
   Error: No API key provided
   ```
   **Solution**: Set `OPENAI_API_KEY` environment variable

2. **A2A Agent Unavailable**
   ```
   Agent advisor is unhealthy
   ```
   **Solution**: Ensure A2A server is running on localhost:8081

3. **Configuration Errors**
   ```
   No fantasy story team found
   ```
   **Solution**: Check CONFIG_PATH points to the correct TOML file

4. **Network Timeouts**
   ```
   Operation timed out
   ```
   **Solution**: Increase timeout values in agent configuration

### Debug Mode

Enable debug logging:
```bash
RUST_LOG=debug CONFIG_PATH=multi-agent/fantasy_story_config.toml cargo run --example fantasy_story_writer
```

## 🚀 Extensions

### Ideas for Enhancement

1. **Additional Agents**:
   - Character Designer
   - World Builder
   - Dialogue Specialist
   - Pacing Expert

2. **Advanced Workflows**:
   - Parallel processing for scene creation
   - Iterative review cycles
   - Dynamic agent selection based on content

3. **Output Formats**:
   - Markdown export
   - PDF generation
   - Chapter organization
   - Character sheets

4. **Interactive Features**:
   - Real-time collaboration
   - Story continuation
   - Alternative plot branches

## 📚 Learn More

- [Multi-Agent Framework Documentation](../README.md)
- [Agent Configuration Guide](../config.rs)
- [A2A Protocol Specification](../../a2a-protocol/README.md)
- [OpenAI API Documentation](https://platform.openai.com/docs)

---

**Happy Story Writing! 📚✨**