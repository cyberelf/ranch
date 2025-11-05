//! Example A2A server with long-running task support
//!
//! This demonstrates how to create an agent that handles long-running tasks.
//! The agent processes messages asynchronously and clients can poll for task status.
//!
//! Run with: cargo run --example task_server --features streaming

use a2a_protocol::{
    prelude::*,
    server::{AgentLogic, ServerBuilder, TaskAwareHandler},
};
use async_trait::async_trait;
use url::Url;
use tokio::time::{sleep, Duration};

/// A long-running task agent that simulates heavy computation
struct TaskAgent;

#[async_trait]
impl AgentLogic for TaskAgent {
    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        // Extract text from the message
        let text = message
            .text_content()
            .ok_or_else(|| A2aError::Validation("No text content in message".to_string()))?;

        // Check if this is a "quick" request (immediate response)
        if text.to_lowercase().contains("quick") {
            return Ok(Message::agent_text(format!("Quick response to: {}", text)));
        }

        // For non-quick messages, simulate heavy computation
        println!("🔄 Processing long-running task: {}", text);
        
        // Simulate work with a delay (configurable in production)
        // In a real agent, this would be actual computation
        const PROCESSING_DELAY_SECS: u64 = 2;
        sleep(Duration::from_secs(PROCESSING_DELAY_SECS)).await;
        
        // Generate result
        let word_count = text.split_whitespace().count();
        let char_count = text.chars().count();
        
        let result = format!(
            "Task completed!\n\n\
            Processed message: '{}'\n\n\
            Analysis:\n\
            - Word count: {}\n\
            - Character count: {}\n\
            - Processing time: ~2 seconds\n\n\
            This demonstrates async task processing in A2A protocol.",
            text, word_count, char_count
        );

        Ok(Message::agent_text(result))
    }

    async fn initialize(&self) -> A2aResult<()> {
        println!("⏳ Task Agent initialized!");
        println!("   Supports async task processing with status polling");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create agent card
    let agent_id = AgentId::new("task-agent".to_string())?;
    let agent_card = AgentCard::new(
        agent_id,
        "Long-Running Task Agent",
        Url::parse("https://example.com")?,
    );

    // 2. Create our task agent logic
    let agent = TaskAgent;

    // 3. Wrap it in TaskAwareHandler for automatic task management
    let handler = TaskAwareHandler::with_logic(agent_card, agent);

    println!("\n🚀 Starting Task Server");
    println!("   Port: 3002");
    println!("   Features: Async tasks, Task polling");
    println!("\n📡 Available RPC methods:");
    println!("   message/send   - Send a message (returns Task or immediate Message)");
    println!("   task/get       - Get task details and results");
    println!("   task/status    - Get current task status");
    println!("   task/cancel    - Cancel a running task");
    println!("   agent/card     - Get agent card");
    println!("\n💡 Try these examples:");
    println!("\n   1. Send a message that gets immediate response:");
    println!("   curl -X POST http://localhost:3002/rpc \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{");
    println!("       \"jsonrpc\": \"2.0\",");
    println!("       \"id\": 1,");
    println!("       \"method\": \"message/send\",");
    println!("       \"params\": {{");
    println!("         \"message\": {{");
    println!("           \"role\": \"user\",");
    println!("           \"parts\": [{{\"kind\":\"text\",\"text\":\"quick question\"}}]");
    println!("         }},");
    println!("         \"immediate\": true");
    println!("       }}");
    println!("     }}'");
    println!("\n   2. Send a message that creates an async task:");
    println!("   curl -X POST http://localhost:3002/rpc \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{");
    println!("       \"jsonrpc\": \"2.0\",");
    println!("       \"id\": 1,");
    println!("       \"method\": \"message/send\",");
    println!("       \"params\": {{");
    println!("         \"message\": {{");
    println!("           \"role\": \"user\",");
    println!("           \"parts\": [{{\"kind\":\"text\",\"text\":\"analyze this long text\"}}]");
    println!("         }}");
    println!("       }}");
    println!("     }}'");
    println!("\n   3. Check task status (replace TASK_ID with actual task ID):");
    println!("   curl -X POST http://localhost:3002/rpc \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{");
    println!("       \"jsonrpc\": \"2.0\",");
    println!("       \"id\": 1,");
    println!("       \"method\": \"task/status\",");
    println!("       \"params\": {{\"taskId\": \"TASK_ID\"}}");
    println!("     }}'");
    println!("\n   4. Get task result (replace TASK_ID with actual task ID):");
    println!("   curl -X POST http://localhost:3002/rpc \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{");
    println!("       \"jsonrpc\": \"2.0\",");
    println!("       \"id\": 1,");
    println!("       \"method\": \"task/get\",");
    println!("       \"params\": {{\"taskId\": \"TASK_ID\"}}");
    println!("     }}'\n");

    ServerBuilder::new(handler)
        .with_port(3002)
        .run()
        .await?;

    Ok(())
}
