//! Simple A2A client example
//!
//! This demonstrates how to create a client and send messages to an A2A server.
//! Start the basic_echo_server first, then run this client.
//!
//! Run with: cargo run --example echo_client --features streaming

use a2a_protocol::{client::ClientBuilder, prelude::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 A2A Echo Client");
    println!("================\n");

    // Create client using ClientBuilder
    let client = ClientBuilder::new()
        .with_json_rpc("http://localhost:3000/rpc")
        .build()?;

    println!("✓ Client created for http://localhost:3000/rpc");
    println!("  Config: {:?}\n", client.config());

    // Try to get agent card (this also verifies connectivity)
    println!("📡 Connecting to server...");

    // Create a simple request to test connectivity by sending a message
    let test_message = Message::user_text("ping");
    match client.send_message(test_message).await {
        Ok(response) => {
            println!("✓ Connected successfully!");
            match response {
                SendResponse::Message(msg) => {
                    if let Some(text) = msg.text_content() {
                        println!("  Server response: {}\n", text);
                    }
                }
                SendResponse::Task(_) => {
                    println!("  Server returned a task\n");
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to connect to server!");
            println!("   Error: {}\n", e);
            println!("   Error debug: {:?}\n", e);
            println!("💡 Make sure the server is running in another terminal:");
            println!("   cargo run --example basic_echo_server --features streaming\n");
            return Ok(());
        }
    }

    // Send a simple message
    println!("📤 Sending message: 'hello world'");
    let message = Message::user_text("hello world");

    match client.send_message(message).await {
        Ok(response) => match response {
            SendResponse::Message(msg) => {
                println!("📥 Received immediate response:");
                if let Some(text) = msg.text_content() {
                    println!("   {}\n", text);
                }
            }
            SendResponse::Task(task) => {
                println!("📥 Received task:");
                println!("   ID: {}", task.id);
                println!("   Status: {:?}\n", task.status.state);
            }
        },
        Err(e) => {
            println!("❌ Error sending message: {}\n", e);
        }
    }

    // Send another message
    println!("📤 Sending message: 'rust is awesome'");
    let message = Message::user_text("rust is awesome");

    match client.send_message(message).await {
        Ok(response) => match response {
            SendResponse::Message(msg) => {
                println!("📥 Received immediate response:");
                if let Some(text) = msg.text_content() {
                    println!("   {}\n", text);
                }
            }
            SendResponse::Task(task) => {
                println!("📥 Received task:");
                println!("   ID: {}", task.id);
                println!("   Status: {:?}\n", task.status.state);
            }
        },
        Err(e) => {
            println!("❌ Error sending message: {}\n", e);
        }
    }

    println!("✓ Demo complete!");

    Ok(())
}
