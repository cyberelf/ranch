//! Example showing push notification concepts
//!
//! This example demonstrates how push notifications work in the A2A protocol
//! by setting up a simple webhook receiver and explaining the concepts.
//!
//! Note: This is a simplified conceptual example. In production, you would:
//! - Use proper webhook authentication
//! - Implement retry logic  
//! - Use HTTPS for webhook endpoints
//! - Validate webhook signatures
//!
//! Run the webhook_server example first:
//!   cargo run --example webhook_server --features streaming

use axum::{extract::Json, routing::post, Router};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Webhook receiver state
struct WebhookReceiver {
    received_events: Mutex<Vec<Value>>,
}

impl WebhookReceiver {
    fn new() -> Self {
        Self {
            received_events: Mutex::new(Vec::new()),
        }
    }

    async fn handle_webhook(&self, payload: Value) {
        println!("\n🔔 Webhook received!");
        println!(
            "📦 Payload: {}",
            serde_json::to_string_pretty(&payload).unwrap()
        );

        // Store the event
        let mut events = self.received_events.lock().await;
        events.push(payload);
    }

    async fn get_event_count(&self) -> usize {
        self.received_events.lock().await.len()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n� Push Notification Concepts Example");
    println!("======================================\n");

    // Start a webhook receiver server
    let receiver = Arc::new(WebhookReceiver::new());
    let receiver_clone = receiver.clone();

    println!("1️⃣  Starting webhook receiver on port 8080...");

    let app = Router::new().route(
        "/webhook",
        post({
            let receiver = receiver_clone;
            move |Json(payload): Json<Value>| {
                let receiver = receiver.clone();
                async move {
                    receiver.handle_webhook(payload).await;
                    "OK"
                }
            }
        }),
    );

    // Start the webhook receiver in the background
    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
            .await
            .unwrap();
        println!("   ✅ Webhook receiver listening on http://127.0.0.1:8080/webhook\n");
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    println!("📚 Push Notification Configuration\n");
    println!("Push notifications in A2A protocol allow you to receive");
    println!("webhook callbacks when tasks change state, instead of polling.\n");

    println!("🔧 Configuration Structure (PushNotificationConfig):");
    println!("```rust");
    println!("PushNotificationConfig {{");
    println!("    url: Url::parse(\"https://your-endpoint.com/webhook\")?,");
    println!("    events: vec![");
    println!("        TaskEvent::Completed,");
    println!("        TaskEvent::Failed,");
    println!("        TaskEvent::Cancelled,");
    println!("        TaskEvent::StatusChanged,");
    println!("    ],");
    println!("    authentication: Some(PushNotificationAuth::Bearer {{");
    println!("        token: \"your-bearer-token\".to_string(),");
    println!("    }}),");
    println!("}}");
    println!("```\n");

    println!("📡 Available RPC Methods:");
    println!("   pushNotification/set    - Configure webhook for a task");
    println!("   pushNotification/get    - Get webhook configuration");
    println!("   pushNotification/list   - List all webhooks");
    println!("   pushNotification/delete - Remove webhook\n");

    println!("💡 Example: Setting up a webhook via JSON-RPC:\n");
    println!("curl -X POST http://localhost:3003/rpc \\");
    println!("  -H 'Content-Type: application/json' \\");
    println!("  -d '{{");
    println!("    \"jsonrpc\": \"2.0\",");
    println!("    \"id\": 1,");
    println!("    \"method\": \"pushNotification/set\",");
    println!("    \"params\": {{");
    println!("      \"taskId\": \"task-123\",");
    println!("      \"config\": {{");
    println!("        \"url\": \"http://localhost:8080/webhook\",");
    println!("        \"events\": [\"completed\", \"failed\"],");
    println!("        \"authentication\": {{");
    println!("          \"type\": \"bearer\",");
    println!("          \"token\": \"secret-token\"");
    println!("        }}");
    println!("      }}");
    println!("    }}");
    println!("  }}'\n");

    println!("📦 Webhook Payload Format:");
    println!("When a task event occurs, the server sends:");
    println!("{{");
    println!("  \"event\": \"completed\",");
    println!("  \"task\": {{ /* Full task object */ }},");
    println!("  \"agentId\": \"agent-123\",");
    println!("  \"timestamp\": \"2025-11-10T12:00:00Z\"");
    println!("}}\n");

    println!("🔐 Authentication Options:");
    println!("   1. Bearer Token:");
    println!("      {{");
    println!("        \"type\": \"bearer\",");
    println!("        \"token\": \"your-secret-token\"");
    println!("      }}");
    println!();
    println!("   2. Custom Headers:");
    println!("      {{");
    println!("        \"type\": \"customHeaders\",");
    println!("        \"headers\": {{");
    println!("          \"X-Webhook-Secret\": \"secret\",");
    println!("          \"X-API-Key\": \"api-key\"");
    println!("        }}");
    println!("      }}\n");

    println!("✅ Event Types:");
    println!("   completed      - Task finished successfully");
    println!("   failed         - Task encountered an error");
    println!("   cancelled      - Task was cancelled by user");
    println!("   statusChanged  - Any status change occurred\n");

    println!("🔄 Workflow:");
    println!("   1. Client sends message → Server creates task");
    println!("   2. Client sets up webhook for task");
    println!("   3. Server processes task");
    println!("   4. Server POSTs to webhook URL when events occur");
    println!("   5. Client receives notifications automatically\n");

    println!("🛡️  Security Best Practices:");
    println!("   • Always use HTTPS for webhook URLs");
    println!("   • Include authentication (Bearer or custom headers)");
    println!("   • Validate webhook signatures on receipt");
    println!("   • Use IP allowlisting when possible");
    println!("   • Implement idempotency for webhook handling");
    println!("   • Set reasonable timeout values\n");

    println!("🚀 Try it out:");
    println!("   1. Start webhook_server: cargo run --example webhook_server --features streaming");
    println!("   2. Send a message to create a task");
    println!("   3. Use pushNotification/set to configure webhook");
    println!("   4. Watch this receiver get notifications!\n");

    println!("⏳ Keeping webhook receiver running for 30 seconds...");
    println!("   Send webhooks to: http://127.0.0.1:8080/webhook\n");

    // Wait for potential webhooks
    for i in 0..30 {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let count = receiver.get_event_count().await;
        if count > 0 {
            println!("   Received {} webhook event(s)!", count);
        }
        if i == 29 {
            println!("\n   Time's up!");
        }
    }

    println!("\n✅ Example complete!");
    println!("\n📖 For complete implementation, see:");
    println!("   - webhook_server.rs - Server with webhook support");
    println!("   - complete_agent.rs - Production agent implementation");

    Ok(())
}
