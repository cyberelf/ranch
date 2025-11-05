//! Example demonstrating agent-to-agent (A2A) communication
//!
//! This example shows two agents communicating with each other:
//! 1. A "Calculator" agent that performs math operations
//! 2. A "Reporter" agent that uses the Calculator to generate reports
//!
//! Run with: cargo run --example multi_agent --features streaming

use a2a_protocol::{
    prelude::*,
    client::ClientBuilder,
    server::{AgentLogic, ServerBuilder, TaskAwareHandler},
};
use async_trait::async_trait;
use url::Url;
use tokio::time::{sleep, Duration};

// ============================================================================
// CALCULATOR AGENT
// ============================================================================

/// A calculator agent that performs basic math operations
struct CalculatorAgent;

#[async_trait]
impl AgentLogic for CalculatorAgent {
    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        let text = message
            .text_content()
            .ok_or_else(|| A2aError::Validation("No text content".to_string()))?;

        println!("🔢 Calculator received: {}", text);

        // Simple math parser - supports "add X Y", "multiply X Y", etc.
        let parts: Vec<&str> = text.split_whitespace().collect();
        
        let result = if parts.len() >= 3 {
            let operation = parts[0].to_lowercase();
            let a: f64 = parts[1].parse().unwrap_or(0.0);
            let b: f64 = parts[2].parse().unwrap_or(0.0);
            
            match operation.as_str() {
                "add" => format!("Result: {} + {} = {}", a, b, a + b),
                "subtract" => format!("Result: {} - {} = {}", a, b, a - b),
                "multiply" => format!("Result: {} × {} = {}", a, b, a * b),
                "divide" if b != 0.0 => format!("Result: {} ÷ {} = {}", a, b, a / b),
                _ => "Unknown operation. Supported: add, subtract, multiply, divide".to_string(),
            }
        } else {
            "Usage: <operation> <number1> <number2>".to_string()
        };

        Ok(Message::agent_text(result))
    }

    async fn initialize(&self) -> A2aResult<()> {
        println!("🔢 Calculator Agent initialized");
        Ok(())
    }
}

// ============================================================================
// REPORTER AGENT
// ============================================================================

/// A reporter agent that uses the Calculator agent to generate reports
struct ReporterAgent {
    calculator_client: A2aClient,
}

impl ReporterAgent {
    fn new(calculator_url: &str) -> A2aResult<Self> {
        let transport = std::sync::Arc::new(
            a2a_protocol::transport::JsonRpcTransport::new(calculator_url)?
        );
        let calculator_client = ClientBuilder::new()
            .with_custom_transport(transport)
            .build()?;
        
        Ok(Self { calculator_client })
    }
}

#[async_trait]
impl AgentLogic for ReporterAgent {
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

    async fn initialize(&self) -> A2aResult<()> {
        println!("📊 Reporter Agent initialized");
        println!("   Connected to Calculator at http://localhost:3003");
        Ok(())
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
        let agent_id = AgentId::new("calculator".to_string()).unwrap();
        let agent_card = AgentCard::new(
            agent_id,
            "Calculator Agent",
            Url::parse("https://example.com").unwrap(),
        );
        
        let agent = CalculatorAgent;
        let handler = TaskAwareHandler::with_logic(agent_card, agent);
        
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
        let agent_id = AgentId::new("reporter".to_string()).unwrap();
        let agent_card = AgentCard::new(
            agent_id,
            "Reporter Agent",
            Url::parse("https://example.com").unwrap(),
        );
        
        let agent = ReporterAgent::new("http://localhost:3003/rpc").unwrap();
        let handler = TaskAwareHandler::with_logic(agent_card, agent);
        
        println!("📊 Starting Reporter Agent on port 3004\n");
        
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
