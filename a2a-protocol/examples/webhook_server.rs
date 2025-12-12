//! Example A2A server with webhook support for task events
//!
//! This demonstrates how to set up webhooks to receive notifications when tasks
//! change state (start, complete, fail, cancel). This is useful for:
//! - Monitoring long-running tasks without polling
//! - Integration with external systems
//! - Building event-driven workflows
//!
//! Run with: cargo run --example webhook_server --features streaming

use a2a_protocol::{
    prelude::*,
    server::{Agent, ServerBuilder, TaskAwareHandler},
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use url::Url;

/// An agent that processes tasks and supports webhook notifications
struct WebhookAgent {
    profile: AgentProfile,
}

impl WebhookAgent {
    fn new() -> Self {
        let agent_id = AgentId::new("webhook-agent".to_string()).unwrap();
        let profile = AgentProfile::new(
            agent_id,
            "Webhook-Enabled Agent",
            Url::parse("https://example.com/agent").unwrap(),
        )
        .with_description("An agent with webhook support for task events")
        .with_version("1.0.0");

        Self { profile }
    }
}

#[async_trait]
impl Agent for WebhookAgent {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        Ok(self.profile.clone())
    }

    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        let text = message
            .text_content()
            .ok_or_else(|| A2aError::Validation("No text content in message".to_string()))?;

        println!("📨 Processing message: {}", text);

        // Simulate different processing scenarios
        if text.to_lowercase().contains("fail") {
            return Err(A2aError::Validation("Simulated failure".to_string()));
        }

        if text.to_lowercase().contains("slow") {
            println!("⏱️  Simulating slow processing...");
            sleep(Duration::from_secs(5)).await;
        } else {
            sleep(Duration::from_secs(2)).await;
        }

        let result = format!(
            "✅ Processed: {}\n\
            Word count: {}\n\
            Processing complete!",
            text,
            text.split_whitespace().count()
        );

        println!("✅ Processing complete");
        Ok(Message::agent_text(result))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔔 Starting Webhook-Enabled A2A Server");
    println!("   Port: 3003");
    println!("   Features: Tasks, Webhooks, Push Notifications");

    // Create agent and handler
    let agent = Arc::new(WebhookAgent::new());
    let handler = TaskAwareHandler::new(agent);

    println!("\n📋 Usage Guide:");
    println!("\n1️⃣  Create a webhook receiver (simple HTTP server):");
    println!("   python3 -m http.server 8080 --bind 127.0.0.1");
    println!("   (Or use a service like webhook.site for testing)");

    println!("\n2️⃣  Send a message and get a task ID:");
    println!("   curl -X POST http://localhost:3003/rpc \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{");
    println!("       \"jsonrpc\": \"2.0\",");
    println!("       \"id\": 1,");
    println!("       \"method\": \"message/send\",");
    println!("       \"params\": {{");
    println!("         \"message\": {{");
    println!("           \"role\": \"user\",");
    println!("           \"parts\": [{{\"text\":\"process this task\"}}]");
    println!("         }},");
    println!("         \"immediate\": false");
    println!("       }}");
    println!("     }}'");

    println!("\n3️⃣  Set up a webhook for the task (replace TASK_ID):");
    println!("   curl -X POST http://localhost:3003/rpc \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{");
    println!("       \"jsonrpc\": \"2.0\",");
    println!("       \"id\": 2,");
    println!("       \"method\": \"pushNotification/set\",");
    println!("       \"params\": {{");
    println!("         \"taskId\": \"TASK_ID\",");
    println!("         \"config\": {{");
    println!("           \"url\": \"http://localhost:8080/webhook\",");
    println!("           \"events\": [\"completed\", \"failed\", \"cancelled\"],");
    println!("           \"headers\": {{");
    println!("             \"X-Webhook-Secret\": \"my-secret-key\"");
    println!("           }}");
    println!("         }}");
    println!("       }}");
    println!("     }}'");

    println!("\n4️⃣  List all webhook configurations:");
    println!("   curl -X POST http://localhost:3003/rpc \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{");
    println!("       \"jsonrpc\": \"2.0\",");
    println!("       \"id\": 3,");
    println!("       \"method\": \"pushNotification/list\",");
    println!("       \"params\": {{}}");
    println!("     }}'");

    println!("\n5️⃣  Get webhook config for a specific task:");
    println!("   curl -X POST http://localhost:3003/rpc \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{");
    println!("       \"jsonrpc\": \"2.0\",");
    println!("       \"id\": 4,");
    println!("       \"method\": \"pushNotification/get\",");
    println!("       \"params\": {{\"taskId\": \"TASK_ID\"}}");
    println!("     }}'");

    println!("\n6️⃣  Delete a webhook (replace TASK_ID):");
    println!("   curl -X POST http://localhost:3003/rpc \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{");
    println!("       \"jsonrpc\": \"2.0\",");
    println!("       \"id\": 5,");
    println!("       \"method\": \"pushNotification/delete\",");
    println!("       \"params\": {{\"taskId\": \"TASK_ID\"}}");
    println!("     }}'");

    println!("\n📦 Webhook Payload Format:");
    println!("   {{");
    println!("     \"event\": \"completed\" | \"failed\" | \"cancelled\" | \"statusChanged\",");
    println!("     \"task\": {{ /* Full task object */ }},");
    println!("     \"agentId\": \"webhook-agent\",");
    println!("     \"timestamp\": \"2025-11-10T12:00:00Z\"");
    println!("   }}");

    println!("\n💡 Event Types:");
    println!("   - completed: Task finished successfully");
    println!("   - failed: Task encountered an error");
    println!("   - cancelled: Task was cancelled");
    println!("   - statusChanged: Any status transition");

    println!("\n🔒 Security Tips:");
    println!("   - Use HTTPS URLs for webhooks in production");
    println!("   - Include authentication headers (X-Webhook-Secret)");
    println!("   - Validate webhook signatures");
    println!("   - Use IP allowlisting when possible");

    println!("\n🧪 Test Scenarios:");
    println!("   - Send 'slow task' for a 5-second delay");
    println!("   - Send 'fail task' to trigger an error");
    println!("   - Use task/cancel to test cancellation webhooks");

    println!("\n🚀 Server starting...\n");

    ServerBuilder::new(handler).with_port(3003).run().await?;

    Ok(())
}
