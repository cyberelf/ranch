# Team Agent Trait Contract

**Purpose**: Define the contract for Team implementing the multi-agent Agent trait

## Trait Definition

```rust
#[async_trait]
pub trait Agent: Send + Sync {
    /// Get agent information (capabilities, metadata)
    async fn info(&self) -> A2aResult<AgentInfo>;
    
    /// Process a message and return a response
    async fn process(&self, message: Message) -> A2aResult<Message>;
    
    /// Check if agent is healthy (default implementation)
    async fn health_check(&self) -> bool {
        self.info().await.is_ok()
    }
}
```

## Team Implementation Contract

### Method: `info()`

**Signature**:
```rust
async fn info(&self) -> A2aResult<AgentInfo>
```

**Purpose**: Generate AgentInfo describing the team's collective capabilities

**Input**: None (uses self.config and self.agent_manager)

**Output**: 
```rust
AgentInfo {
    id: String,              // Team ID from config
    name: String,            // Team name from config
    description: String,     // Team description from config
    capabilities: Vec<String>, // Aggregated from all member agents
    metadata: HashMap<String, String>, // Team metadata including:
                            // - "type": "team"
                            // - "mode": "supervisor" | "workflow"
                            // - "member_count": number of agents
}
```

**Behavior**:
1. Extract team ID, name, description from self.config
2. Query all member agents from agent_manager
3. For each member agent, call agent.info()
4. Aggregate all unique capabilities from member agents
5. Add team-specific metadata (type, mode, member count)
6. Return AgentInfo

**Error Handling**:
- If member agent.info() fails, log warning but continue (graceful degradation)
- If no member agents available, return empty capabilities list (not an error)
- If agent_manager is empty, return basic info without capabilities

**Example Output**:
```json
{
  "id": "fantasy-writing-team",
  "name": "Fantasy Story Writing Team",
  "description": "Collaborative team for fantasy story creation",
  "capabilities": [
    "world-building",
    "character-development", 
    "plot-generation",
    "prose-writing"
  ],
  "metadata": {
    "type": "team",
    "mode": "workflow",
    "member_count": "4"
  }
}
```

---

### Method: `process(message)`

**Signature**:
```rust
async fn process(&self, message: Message) -> A2aResult<Message>
```

**Purpose**: Orchestrate message processing through team's scheduler and member agents

**Input**: 
- `message: Message` - A2A protocol message to process

**Output**: 
- `Message` - Final response after orchestration completes

**Behavior**:
1. Initialize orchestration context (empty HashMap for state)
2. Set current_messages = vec![input message]
3. Set last_response = None
4. Loop:
   a. Call scheduler.determine_next_recipient(config, manager, messages, last_response, context)
   b. If recipient.should_return_to_user is true:
      - Return last_response (or error if None)
   c. Extract agent_id from recipient
   d. Get agent from manager by agent_id
   e. Call agent.process(last message from current_messages)
   f. Store result as last_response
   g. Apply context_updates from recipient to context
   h. Set current_messages = vec![last_response]
   i. Continue loop

**Error Handling**:
- If scheduler returns error → return A2aError::Internal with scheduler error details
- If agent not found → return A2aError::Internal("Agent {id} not found")
- If agent.process() fails → return agent's error (propagate up)
- If loop iterates >100 times → return A2aError::Internal("Max iterations exceeded, possible cycle")

**Cycle Detection**:
- Track visited (agent_id, message_content) pairs
- If same pair seen twice, return error preventing infinite loops

**Example Flow (Supervisor)**:
```
Input: "Write a fantasy story about dragons"
  → Scheduler selects "research-agent"
  → Research agent processes, returns "Dragon lore: ..."
  → Scheduler selects "writing-agent"  
  → Writing agent processes, returns "Story: Once upon a time..."
  → Scheduler returns to user
Output: "Story: Once upon a time..."
```

**Example Flow (Workflow)**:
```
Input: "Write a fantasy story"
  → Step 1: world-building-agent → "World: Medieval fantasy realm"
  → Step 2: character-agent → "Characters: Brave knight, wise wizard"
  → Step 3: plot-agent → "Plot: Quest to save kingdom"
  → Step 4: writing-agent → "Story: [full narrative]"
Output: "Story: [full narrative]"
```

---

### Method: `health_check()`

**Signature**:
```rust
async fn health_check(&self) -> bool
```

**Purpose**: Verify team and all member agents are healthy

**Implementation** (override default):
```rust
async fn health_check(&self) -> bool {
    // Check if info() succeeds
    if self.info().await.is_err() {
        return false;
    }
    
    // Check all member agents
    for agent_config in &self.config.agents {
        if let Some(agent) = self.agent_manager.get(&agent_config.agent_id).await {
            if !agent.health_check().await {
                return false;
            }
        } else {
            return false; // Agent not found
        }
    }
    
    true
}
```

**Output**: 
- `true` if team and all member agents are healthy
- `false` if any check fails

## Usage Examples

### Example 1: Team as Agent in Code

```rust
use multi_agent::{Team, Agent};

// Create team
let team = Team::new(team_config, agent_manager);

// Use as Agent
let info = team.info().await?;
println!("Team: {} with {} capabilities", info.name, info.capabilities.len());

let message = user_message("Write a fantasy story");
let response = team.process(message).await?;
println!("Response: {}", extract_text(&response).unwrap());
```

### Example 2: Nested Teams

```rust
// Create sub-teams
let research_team = Team::new(research_config, manager.clone());
let writing_team = Team::new(writing_config, manager.clone());

// Register sub-teams as agents
manager.register(Arc::new(research_team)).await?;
manager.register(Arc::new(writing_team)).await?;

// Create parent team that coordinates sub-teams
let parent_team = Team::new(parent_config, manager.clone());

// Use parent team (will delegate to sub-teams)
let response = parent_team.process(message).await?;
```

### Example 3: Health Check

```rust
let healthy = team.health_check().await;
if !healthy {
    eprintln!("Team health check failed - some agents unavailable");
}
```

## Implementation Checklist

- [ ] Implement `info()` method
  - [ ] Extract config fields (id, name, description)
  - [ ] Query all member agents
  - [ ] Aggregate unique capabilities
  - [ ] Add team metadata
  - [ ] Handle member agent failures gracefully
  
- [ ] Implement `process()` method
  - [ ] Initialize orchestration loop
  - [ ] Call scheduler for each iteration
  - [ ] Handle return-to-user signal
  - [ ] Delegate to member agents
  - [ ] Apply context updates
  - [ ] Implement cycle detection
  - [ ] Add max iteration limit
  
- [ ] Override `health_check()` method
  - [ ] Check self.info() succeeds
  - [ ] Check all member agents healthy
  - [ ] Return boolean result
  
- [ ] Add unit tests
  - [ ] Test info() returns correct structure
  - [ ] Test process() with supervisor mode
  - [ ] Test process() with workflow mode
  - [ ] Test cycle detection
  - [ ] Test error propagation
  - [ ] Test health_check() with healthy/unhealthy agents
  
- [ ] Add integration tests
  - [ ] Test nested teams
  - [ ] Test with real A2A agents
  - [ ] Test concurrent processing
  - [ ] Test error recovery

## Notes

- Team implements the **multi-agent** Agent trait, not a2a-protocol's Agent trait
- TeamServer bridges the gap by wrapping Team with TaskAwareHandler
- This design enables recursive composition (teams within teams)
- Orchestration is stateless - each process() call is independent
- For stateful orchestration, context HashMap can store state between iterations
