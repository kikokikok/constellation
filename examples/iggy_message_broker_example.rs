//! Example demonstrating Iggy-based message broker for Constellation.
//!
//! This example shows how to:
//! 1. Create an Iggy message broker
//! 2. Register agent sessions
//! 3. Send and receive messages with priority
//! 4. Broadcast messages to all agents

use constellation_core::message_broker::{
    AgentSession, IggyMessageBrokerBuilder, Message, MessagePriority,
};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("Starting Iggy message broker example...");

    // Create Iggy message broker
    println!("Connecting to Iggy server...");
    let broker = IggyMessageBrokerBuilder::new()
        .server_address("127.0.0.1:8090".to_string())
        .credentials("guest".to_string(), "guest".to_string())
        .build()
        .await?;

    println!("Iggy message broker created successfully!");

    // Register agent sessions
    println!("\nRegistering agent sessions...");
    
    let agent1_session = AgentSession::new(
        "agent_1".to_string(),
        "token_1".to_string(),
        "websocket".to_string(),
        None,
    );
    
    let agent2_session = AgentSession::new(
        "agent_2".to_string(),
        "token_2".to_string(),
        "websocket".to_string(),
        None,
    );

    broker.register_session(agent1_session).await?;
    broker.register_session(agent2_session).await?;
    
    println!("Registered agents: agent_1, agent_2");

    // Send messages with different priorities
    println!("\nSending messages with different priorities...");
    
    // Send critical message to agent_1
    let critical_msg = Message::new(
        "msg_critical".to_string(),
        "system".to_string(),
        "agent_1".to_string(),
        "alert".to_string(),
        "Critical system alert!".to_string(),
    )
    .with_priority(MessagePriority::Critical);
    
    broker.send_message(critical_msg.clone()).await?;
    println!("Sent critical message to agent_1");

    // Send normal message to agent_2
    let normal_msg = Message::new(
        "msg_normal".to_string(),
        "system".to_string(),
        "agent_2".to_string(),
        "notification".to_string(),
        "Normal system notification".to_string(),
    )
    .with_priority(MessagePriority::Normal);
    
    broker.send_message(normal_msg.clone()).await?;
    println!("Sent normal message to agent_2");

    // Receive messages
    println!("\nReceiving messages...");
    
    let agent1_messages = broker.receive_messages("agent_1", 10).await?;
    println!("Agent 1 received {} messages", agent1_messages.len());
    
    let agent2_messages = broker.receive_messages("agent_2", 10).await?;
    println!("Agent 2 received {} messages", agent2_messages.len());

    // Broadcast message to all agents
    println!("\nBroadcasting message to all agents...");
    
    let broadcast_msg = Message::new(
        "msg_broadcast".to_string(),
        "system".to_string(),
        "broadcast".to_string(), // Will be replaced with each agent ID
        "broadcast".to_string(),
        "System maintenance scheduled".to_string(),
    )
    .with_priority(MessagePriority::High);
    
    broker.broadcast(broadcast_msg).await?;
    println!("Broadcast message sent to all connected agents");

    // Get connected agents
    println!("\nGetting connected agents...");
    let connected_agents = broker.get_connected_agents().await;
    println!("Connected agents: {:?}", connected_agents);

    // Run maintenance
    println!("\nRunning maintenance tasks...");
    broker.run_maintenance().await?;
    println!("Maintenance tasks completed");

    println!("\nExample completed successfully!");
    println!("\nNote: This is a placeholder implementation.");
    println!("To run with a real Iggy server:");
    println!("1. Start Iggy server: docker run -p 8090:8090 -p 3000:3000 iggyrs/iggy:latest");
    println!("2. Update the IggyMessageBroker implementation to connect to the server");
    println!("3. Run this example: cargo run --example iggy_message_broker_example");

    Ok(())
}