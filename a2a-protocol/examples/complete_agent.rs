//! Complete Agent Implementation Example
//!
//! This example demonstrates best practices for implementing an A2A agent with:
//! - Dynamic AgentCard generation
//! - Custom message processing
//! - Capability advertisement
//! - Streaming support (when enabled)
//! - Webhook/push notification support
//!
//! Run with: cargo run --example complete_agent --features streaming

use a2a_protocol::{
    prelude::*,
    server::{Agent, ServerBuilder, TaskAwareHandler},
};
use async_trait::async_trait;
use std::sync::Arc;
use url::Url;

/// A complete agent implementation showcasing best practices
struct ResearchAgent {
    base_url: Url,
    agent_id: AgentId,
}

impl ResearchAgent {
    fn new(base_url: Url) -> Self {
        let agent_id = AgentId::new("research-assistant".to_string()).unwrap();
        Self { base_url, agent_id }
    }

    /// Simulate research work
    async fn perform_research(&self, query: &str) -> A2aResult<String> {
        // In a real implementation, this would call APIs, search databases, etc.
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let result = format!(
            "Research Results for: \"{}\"\n\n\
            📊 Key Findings:\n\
            1. Topic is related to: [simulated analysis]\n\
            2. Found {} relevant sources\n\
            3. Confidence level: High\n\n\
            📝 Summary:\n\
            Based on the analysis of '{}', we found significant \
            information across multiple domains. This is a simulated \
            response demonstrating how a real agent would structure \
            its research output.\n\n\
            🔗 Sources: [simulated bibliography]",
            query,
            query.split_whitespace().count() * 3,
            query
        );

        Ok(result)
    }
}

#[async_trait]
impl Agent for ResearchAgent {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        // Build a comprehensive agent profile that reflects the agent's capabilities
        let mut profile = AgentProfile::new(
            self.agent_id.clone(),
            "Research Assistant Agent",
            self.base_url.clone(),
        )
        .with_description("An AI-powered research assistant that conducts in-depth analysis")
        .with_version("2.1.0");

        // Add provider information
        let provider = AgentProvider {
            name: "Example AI Research Lab".to_string(),
            description: Some("Providing AI-powered research assistance".to_string()),
            url: Some(Url::parse("https://example.com").unwrap()),
            contact_email: Some("support@example.com".to_string()),
            contact_url: None,
            extra: std::collections::HashMap::new(),
        };
        profile = profile.with_provider(provider);

        // Add capabilities
        profile = profile
            .with_capability(AgentCapability {
                name: "research".to_string(),
                description: Some("Conduct in-depth research on any topic".to_string()),
                category: Some("analysis".to_string()),
                input_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The research query"
                        }
                    },
                    "required": ["query"]
                })),
                output_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "findings": {
                            "type": "string",
                            "description": "Research findings and analysis"
                        }
                    }
                })),
            })
            .with_capability(AgentCapability {
                name: "summarization".to_string(),
                description: Some("Summarize long-form content".to_string()),
                category: Some("text-processing".to_string()),
                input_schema: None,
                output_schema: None,
            });

        // Add skills
        profile = profile
            .with_skill(AgentSkill {
                name: "Academic Research".to_string(),
                description: Some("Research academic papers and publications".to_string()),
                category: Some("research".to_string()),
                tags: vec!["academic".to_string(), "papers".to_string()],
                examples: vec!["Research quantum computing advances in 2024".to_string()],
            })
            .with_skill(AgentSkill {
                name: "Market Analysis".to_string(),
                description: Some("Analyze market trends and data".to_string()),
                category: Some("business".to_string()),
                tags: vec!["market".to_string(), "trends".to_string()],
                examples: vec!["Analyze AI market trends".to_string()],
            });

        // Set input/output modes
        profile = profile
            .with_default_input_modes(vec!["text/plain".to_string()])
            .with_default_output_modes(vec!["text/plain".to_string(), "text/markdown".to_string()]);

        Ok(profile)
    }

    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        // Extract the query from the message
        let query = message
            .text_content()
            .ok_or_else(|| A2aError::Validation("No text content in message".to_string()))?;

        println!("🔍 Processing research query: {}", query);

        // Perform the research
        let result = self.perform_research(query).await?;

        // Return the result as a message
        Ok(Message::agent_text(result))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🤖 Complete Agent Implementation Example");
    println!("=========================================\n");

    // Create the agent
    let base_url = Url::parse("http://localhost:3004")?;
    let agent = Arc::new(ResearchAgent::new(base_url));

    // Wrap in TaskAwareHandler
    // By default, this returns tasks (async mode)
    let handler = TaskAwareHandler::new(agent.clone());

    println!("🚀 Starting Research Assistant Agent");
    println!("   Port: 3004");
    println!("   Mode: Task-based (async by default)\n");

    println!("📋 Agent Features:");
    println!("   ✅ Dynamic AgentCard generation");
    println!("   ✅ Custom capabilities and skills");
    println!("   ✅ Task-based processing");
    println!("   ✅ Webhook/push notification support");
    #[cfg(feature = "streaming")]
    println!("   ✅ Streaming support (feature enabled)");
    println!("   ✅ Rate limiting metadata");
    println!("   ✅ Authentication requirements\n");

    println!("🔧 Available RPC Methods:");
    println!("   agent/card            - Get agent card with full capabilities");
    println!("   message/send          - Send a message (returns task)");
    println!("   task/get              - Get task details");
    println!("   task/status           - Get task status");
    println!("   task/cancel           - Cancel a task");
    println!("   pushNotification/set  - Set up webhooks");
    println!("   pushNotification/get  - Get webhook config");
    println!("   pushNotification/list - List all webhooks");
    println!("   pushNotification/delete - Delete webhook\n");

    println!("💡 Example Usage:\n");

    println!("1️⃣  Get agent card:");
    println!("curl -X POST http://localhost:3004/rpc \\");
    println!("  -H 'Content-Type: application/json' \\");
    println!("  -d '{{");
    println!("    \"jsonrpc\": \"2.0\",");
    println!("    \"id\": 1,");
    println!("    \"method\": \"agent/card\",");
    println!("    \"params\": {{}}");
    println!("  }}'\n");

    println!("2️⃣  Send a research query:");
    println!("curl -X POST http://localhost:3004/rpc \\");
    println!("  -H 'Content-Type: application/json' \\");
    println!("  -d '{{");
    println!("    \"jsonrpc\": \"2.0\",");
    println!("    \"id\": 2,");
    println!("    \"method\": \"message/send\",");
    println!("    \"params\": {{");
    println!("      \"message\": {{");
    println!("        \"role\": \"user\",");
    println!("        \"parts\": [{{\"kind\":\"text\",\"text\":\"Research AI agents\"}}]");
    println!("      }}");
    println!("    }}");
    println!("  }}'\n");

    println!("3️⃣  Set up webhook for task (replace TASK_ID):");
    println!("curl -X POST http://localhost:3004/rpc \\");
    println!("  -H 'Content-Type: application/json' \\");
    println!("  -d '{{");
    println!("    \"jsonrpc\": \"2.0\",");
    println!("    \"id\": 3,");
    println!("    \"method\": \"pushNotification/set\",");
    println!("    \"params\": {{");
    println!("      \"taskId\": \"TASK_ID\",");
    println!("      \"config\": {{");
    println!("        \"url\": \"http://localhost:8080/webhook\",");
    println!("        \"events\": [\"completed\"]");
    println!("      }}");
    println!("    }}");
    println!("  }}'\n");

    println!("📚 Key Implementation Points:\n");
    println!("   1. Agent trait unifies logic and metadata");
    println!("   2. AgentCard is dynamically generated");
    println!("   3. Capabilities reflect actual implementation");
    println!("   4. Metadata includes runtime features (streaming, webhooks)");
    println!("   5. Handler provides task management automatically\n");

    println!("🚀 Server starting...\n");

    ServerBuilder::new(handler).with_port(3004).run().await?;

    Ok(())
}
