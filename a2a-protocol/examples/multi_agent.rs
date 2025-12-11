//! Example demonstrating agent-to-agent (A2A) communication
//!
//! This example shows two agents communicating with each other:
//! 1. A "Calculator" agent that performs math operations
//! 2. A "Reporter" agent that uses the Calculator to generate reports
//!
//! Run with: cargo run --example multi_agent --features streaming

use a2a_protocol::{
    client::ClientBuilder,
    prelude::*,
    server::{Agent, ServerBuilder, TaskAwareHandler},
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use url::Url;

// ============================================================================
// CALCULATOR AGENT
// ============================================================================

/// A calculator agent that performs basic math operations
struct CalculatorAgent {
    profile: AgentProfile,
}

impl CalculatorAgent {
    fn new() -> Self {
        let agent_id = AgentId::new("calculator".to_string()).unwrap();
        let profile = AgentProfile::new(
            agent_id,
            "Calculator Agent",
            Url::parse("https://example.com").unwrap(),
        )
        .with_description("A simple calculator for basic math operations");

        Self { profile }
    }
}

#[async_trait]
impl Agent for CalculatorAgent {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        Ok(self.profile.clone())
    }

    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        let text = message
            .text_content()
            .ok_or_else(|| A2aError::Validation("No text content".to_string()))?;

        println!("🔢 Calculator received: {}", text);

        // Simple math parser - supports "add X Y", "multiply X Y", etc.
        let parts: Vec<&str> = text.split_whitespace().collect();

        let result = if parts.len() >= 3 {
            let operation = parts[0].to_lowercase();

            // Parse numbers with error handling
            let a: f64 = match parts[1].parse() {
                Ok(n) => n,
                Err(_) => {
                    return Ok(Message::agent_text(format!(
                        "Error: '{}' is not a valid number",
                        parts[1]
                    )))
                }
            };
            let b: f64 = match parts[2].parse() {
                Ok(n) => n,
                Err(_) => {
                    return Ok(Message::agent_text(format!(
                        "Error: '{}' is not a valid number",
                        parts[2]
                    )))
                }
            };

            match operation.as_str() {
                "add" => format!("Result: {} + {} = {}", a, b, a + b),
                "subtract" => format!("Result: {} - {} = {}", a, b, a - b),
                "multiply" => format!("Result: {} × {} = {}", a, b, a * b),
                "divide" if b != 0.0 => format!("Result: {} ÷ {} = {}", a, b, a / b),
                "divide" => "Error: Division by zero".to_string(),
                _ => "Unknown operation. Supported: add, subtract, multiply, divide".to_string(),
            }
        } else {
            "Usage: <operation> <number1> <number2>".to_string()
        };

        Ok(Message::agent_text(result))
    }
}

// ============================================================================
// REPORTER AGENT
// ============================================================================

/// A reporter agent that uses the Calculator agent to generate reports
struct ReporterAgent {
    profile: AgentProfile,
    calculator_client: A2aClient,
}

impl ReporterAgent {
    fn new(calculator_url: &str) -> A2aResult<Self> {
        let agent_id = AgentId::new("reporter".to_string()).unwrap();
        let profile = AgentProfile::new(
            agent_id,
            "Reporter Agent",
            Url::parse("https://example.com").unwrap(),
        )
        .with_description("Generates reports using the Calculator agent");

        let transport = std::sync::Arc::new(
            a2a_protocol::client::transport::JsonRpcTransport::new(calculator_url)?,
        );
        let calculator_client = ClientBuilder::new()
            .with_custom_transport(transport)
            .build()?;

        Ok(Self {
            profile,
            calculator_client,
        })
    }
}

#[async_trait]
impl Agent for ReporterAgent {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        Ok(self.profile.clone())
    }

    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        let text = message
            .text_content()
            .ok_or_else(|| A2aError::Validation("No text content".to_string()))?;

        println!("📊 Reporter received: {}", text);

        // Generate a simple report by calling the calculator multiple times
        let mut report = String::from("Financial Report\n================\n\n");

        // Example calculations
        let calculations = vec![
            ("Revenue", "add 100000 50000"),
            ("Expenses", "add 30000 20000"),
        ];

        for (label, operation) in calculations {
            let calc_message = Message::user_text(operation);

            match self.calculator_client.send_message(calc_message).await {
                Ok(SendResponse::Message(response)) => {
                    if let Some(result) = response.text_content() {
                        report.push_str(&format!("{}: {}\n", label, result));
                    }
                }
                Ok(SendResponse::Task(_)) => {
                    report.push_str(&format!("{}: Task created (async)\n", label));
                }
                Err(e) => {
                    report.push_str(&format!("{}: Error - {}\n", label, e));
                }
            }
        }

        // Calculate profit by calling calculator again
        let profit_message = Message::user_text("subtract 150000 50000");
        match self.calculator_client.send_message(profit_message).await {
            Ok(SendResponse::Message(response)) => {
                if let Some(result) = response.text_content() {
                    report.push_str(&format!("\nProfit: {}\n", result));
                }
            }
            _ => {}
        }

        report.push_str("\n✓ Report generated using Calculator agent");

        Ok(Message::agent_text(report))
    }
}

// ============================================================================
// MAIN - START BOTH AGENTS
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🤖 Multi-Agent Communication Example");
    println!("====================================\n");

    // Start Calculator agent in background task
    let calculator_task = tokio::spawn(async {
        let agent = Arc::new(CalculatorAgent::new());
        let handler = TaskAwareHandler::new(agent);

        println!("🔢 Starting Calculator Agent on port 3003");

        ServerBuilder::new(handler)
            .with_port(3003)
            .run()
            .await
            .unwrap();
    });

    // Wait for Calculator to start
    sleep(Duration::from_millis(500)).await;

    // Start Reporter agent
    let reporter_task = tokio::spawn(async {
        let agent = Arc::new(ReporterAgent::new("http://localhost:3003/rpc").unwrap());
        let handler = TaskAwareHandler::new(agent);

        println!("📊 Starting Reporter Agent on port 3004");
        println!("   Connected to Calculator at http://localhost:3003\n");

        ServerBuilder::new(handler)
            .with_port(3004)
            .run()
            .await
            .unwrap();
    });

    // Wait for both agents to start
    sleep(Duration::from_millis(500)).await;

    println!("✓ Both agents running!");
    println!("\n💡 Try these commands:");
    println!("\n   1. Call Calculator directly:");
    println!("   curl -X POST http://localhost:3003/rpc \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{");
    println!("       \"jsonrpc\": \"2.0\",");
    println!("       \"id\": 1,");
    println!("       \"method\": \"message/send\",");
    println!("       \"params\": {{");
    println!("         \"message\": {{");
    println!("           \"role\": \"user\",");
    println!("           \"parts\": [{{\"kind\":\"text\",\"text\":\"add 10 20\"}}]");
    println!("         }},");
    println!("         \"immediate\": true");
    println!("       }}");
    println!("     }}'");
    println!("\n   2. Call Reporter (which calls Calculator internally):");
    println!("   curl -X POST http://localhost:3004/rpc \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{");
    println!("       \"jsonrpc\": \"2.0\",");
    println!("       \"id\": 1,");
    println!("       \"method\": \"message/send\",");
    println!("       \"params\": {{");
    println!("         \"message\": {{");
    println!("           \"role\": \"user\",");
    println!("           \"parts\": [{{\"kind\":\"text\",\"text\":\"generate report\"}}]");
    println!("         }},");
    println!("         \"immediate\": true");
    println!("       }}");
    println!("     }}'\n");

    // Wait for both tasks
    tokio::try_join!(calculator_task, reporter_task)?;

    Ok(())
}
