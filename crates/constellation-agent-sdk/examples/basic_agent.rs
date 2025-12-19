//! Basic agent example using the Constellation Agent SDK

use constellation_agent_sdk::{AgentClient, AgentConfig, DefaultRequestHandler};
use std::time::Duration;
use tracing::{info, warn};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting basic agent example");
    info!("================================");

    // Create agent configuration
    let config = AgentConfig::new("example_agent")
        .with_broker_url("127.0.0.1:8090")
        .with_broker_credentials("guest", "guest")
        .with_default_request_timeout(Duration::from_secs(30))
        .with_auto_reconnect(true);

    info!("Agent configuration:");
    info!("  Agent ID: {}", config.agent_id);
    info!("  Broker URL: {}", config.broker_url);
    info!("  Auto reconnect: {}", config.auto_reconnect);

    // Connect to the broker
    info!("\nConnecting to broker...");
    let mut client = AgentClient::connect(config).await?;

    // Set up request handler
    client = client.with_request_handler(DefaultRequestHandler);

    // Start background message processing
    info!("Starting background message processing...");
    client.start().await?;

    // Subscribe to system topics
    info!("\nSubscribing to topics...");
    client.subscribe("system.*").await?;
    client.subscribe("agent.example_agent").await?;

    // Get active subscriptions
    let subscriptions = client.get_subscriptions().await;
    info!("Active subscriptions: {}", subscriptions.len());
    for sub in subscriptions {
        info!("  - {:?}", sub.topic_pattern);
    }

    // Demonstrate different communication patterns
    info!("\nDemonstrating communication patterns:");
    info!("--------------------------------------");

    // 1. Send a notification (fire-and-forget)
    info!("1. Sending notification...");
    client
        .notify(
            "other_agent",
            "{\"type\": \"greeting\", \"message\": \"Hello from example_agent\"}",
            constellation_agent_sdk::DeliveryGuarantee::BestEffort,
        )
        .await?;
    info!("   Notification sent (fire-and-forget)");

    // 2. Publish to a topic
    info!("2. Publishing to topic...");
    client
        .publish(
            "system.status",
            "{\"agent\": \"example_agent\", \"status\": \"online\"}",
            constellation_agent_sdk::DeliveryGuarantee::AtLeastOnce,
        )
        .await?;
    info!("   Published to 'system.status' topic");

    // 3. Send a request (if there's another agent to respond)
    info!("3. Attempting to send request...");
    match client.request("test_agent", "{\"action\": \"ping\"}").await {
        Ok(response) => {
            info!("   Received response: {}", response.payload);
        }
        Err(e) => {
            warn!(
                "   Request failed (expected if no test_agent is running): {}",
                e
            );
        }
    }

    // 4. Broadcast a message
    info!("4. Broadcasting message...");
    client
        .broadcast(
            "announcement",
            "{\"message\": \"Example agent is running\"}",
            constellation_agent_sdk::MessagePriority::Normal,
        )
        .await?;
    info!("   Broadcast sent to all connected agents");

    // 5. Check connected agents
    info!("5. Checking connected agents...");
    let connected_agents = client.get_connected_agents().await;
    info!("   Connected agents: {:?}", connected_agents);

    // Receive some messages
    info!("\nWaiting for incoming messages (10 seconds)...");
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Try to receive messages
    info!("Checking for received messages...");
    match client.receive(10).await {
        Ok(messages) => {
            info!("   Received {} messages", messages.len());
            for msg in messages {
                info!("   - {}: {}", msg.message_type, msg.payload);
            }
        }
        Err(e) => {
            warn!("   Failed to receive messages: {}", e);
        }
    }

    // Keep running for a bit to demonstrate background processing
    info!("\nAgent running for 30 seconds (press Ctrl+C to exit early)...");
    info!("You can send messages to this agent using the topic 'agent.example_agent'");

    for i in 1..=30 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        if i % 10 == 0 {
            info!("Still running... ({} seconds)", i);
        }
    }

    // Shutdown
    info!("\nShutting down agent...");
    client.shutdown().await?;

    info!("Example completed successfully!");
    Ok(())
}
