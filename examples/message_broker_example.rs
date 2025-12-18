//! Example demonstrating the LLM-optimized message broker
//!
//! Demonstrates the fast, in-memory message broker designed specifically for LLM agents.

use constellation_core::message_broker::{
    AgentSession, LlmMessageBrokerBuilder, Message, MessagePriority,
};
use tokio::time;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🚀 Starting LLM-optimized Message Broker Example");

    // Create LLM-optimized message broker
    let broker = LlmMessageBrokerBuilder::new()
        .max_queue_size(500) // Limit queue size per agent
        .message_ttl(1800) // 30 minute TTL
        .max_retries(3) // 3 retry attempts
        .retry_delay(30) // 30 second retry delay
        .session_timeout(300) // 5 minute session timeout
        .build();

    info!("Created LLM message broker with in-memory queues");

    // Register agent sessions using the constructor
    let ceo_session = AgentSession::new(
        "ceo_agent".to_string(),
        "ceo_token".to_string(),
        "websocket".to_string(),
        None,
    );

    let cfo_session = AgentSession::new(
        "cfo_agent".to_string(),
        "cfo_token".to_string(),
        "websocket".to_string(),
        None,
    );

    let marketing_session = AgentSession::new(
        "marketing_agent".to_string(),
        "marketing_token".to_string(),
        "websocket".to_string(),
        None,
    );

    broker.register_session(ceo_session).await?;
    broker.register_session(cfo_session).await?;
    broker.register_session(marketing_session).await?;

    info!("Registered 3 agent sessions");

    // Example 1: Priority-based message queuing
    info!("\n📨 Example 1: Priority-based message queuing");

    // Create test messages with different priorities using Message::new()
    let critical_msg = Message::new(
        "alert".to_string(),
        "monitoring".to_string(),
        "ceo_agent".to_string(),
        "alert".to_string(),
        r#"{"type": "critical", "system": "down", "urgency": "immediate"}"#.to_string(),
    );

    let normal_msg = Message::new(
        "weekly_report".to_string(),
        "cfo_agent".to_string(),
        "ceo_agent".to_string(),
        "report".to_string(),
        r#"{"type": "report", "period": "weekly", "revenue": 150000}"#.to_string(),
    );

    let low_msg = Message::new(
        "newsletter".to_string(),
        "marketing".to_string(),
        "ceo_agent".to_string(),
        "newsletter".to_string(),
        "<h1>Weekly Newsletter</h1>".to_string(),
    );

    // Set different priorities
    let mut critical_msg = critical_msg;
    critical_msg.priority = MessagePriority::Critical;
    critical_msg.delivery_guarantee = "at-least-once".to_string();
    critical_msg.ttl_seconds = Some(3600);

    let mut normal_msg = normal_msg;
    normal_msg.priority = MessagePriority::Normal;
    normal_msg.delivery_guarantee = "at-least-once".to_string();
    normal_msg.ttl_seconds = Some(86400);

    let mut low_msg = low_msg;
    low_msg.priority = MessagePriority::Low;
    low_msg.delivery_guarantee = "best-effort".to_string();
    low_msg.ttl_seconds = Some(604800);
    low_msg.max_retries = 1;

    // Send messages (low first, then normal, then critical)
    broker.send_message(low_msg.clone()).await?;
    broker.send_message(normal_msg.clone()).await?;
    broker.send_message(critical_msg.clone()).await?;

    info!("Sent 3 messages with different priorities to CEO agent");

    // Receive messages - critical should come first!
    let received = broker.receive_messages("ceo_agent", 10).await?;
    info!("CEO received {} messages", received.len());

    for (i, msg) in received.iter().enumerate() {
        info!(
            "  {}. {} (Priority: {:?})",
            i + 1,
            msg.message_id,
            msg.priority
        );
    }

    // Note: In the LLM broker, messages are automatically removed when received
    // Acknowledgments are optional and mainly for logging/session activity
    info!("Messages automatically removed from queue when received");

    // Example 2: Broadcast messaging
    info!("\n📢 Example 2: Broadcast messaging");

    let broadcast_msg = Message::new(
        "system_announcement".to_string(),
        "system".to_string(),
        "all".to_string(), // Special recipient for broadcast
        "announcement".to_string(),
        r#"{"type": "maintenance", "window": "02:00-04:00 UTC"}"#.to_string(),
    );

    broker.broadcast(broadcast_msg).await?;
    info!("Broadcast system announcement to all connected agents");

    // Example 3: Queue statistics
    info!("\n📊 Example 3: Queue statistics");

    let stats = broker.get_queue_stats("ceo_agent").await?;
    info!("CEO agent queue stats: {:?}", stats);

    let connected_agents = broker.get_connected_agents().await;
    info!("Connected agents: {:?}", connected_agents);

    // Example 4: Performance test
    info!("\n⚡ Example 4: Performance test");

    let start = time::Instant::now();
    let mut message_count = 0;

    for i in 0..100 {
        let perf_msg = Message::new(
            format!("perf_msg_{i}"),
            "perf_sender".to_string(),
            "ceo_agent".to_string(),
            "performance".to_string(),
            format!("{{'iteration': {i}}}"),
        );

        broker.send_message(perf_msg).await?;
        message_count += 1;

        if i % 20 == 0 {
            info!("  Sent {} messages...", i);
        }
    }

    let elapsed = start.elapsed();
    let rate = message_count as f64 / elapsed.as_secs_f64();

    info!(
        "Sent {} messages in {:?} ({:.2} msg/sec)",
        message_count, elapsed, rate
    );

    // Example 5: Session management
    info!("\n👥 Example 5: Session management");

    // Update session activity
    broker.update_session_activity("ceo_agent").await?;
    info!("Updated CEO agent session activity");

    // Run maintenance tasks
    broker.run_maintenance().await?;
    info!("Ran maintenance tasks (session cleanup, dead letter processing)");

    // Example 6: Dead letter queue (conceptual)
    info!("\n💀 Example 6: Dead letter queue (conceptual)");

    info!(
        "The LLM broker includes a dead letter queue for messages that fail delivery after retries."
    );
    info!(
        "When a message delivery fails (e.g., network error, processing error), it can be moved to dead letter."
    );
    info!(
        "The maintenance task automatically retries dead letter messages with exponential backoff."
    );
    info!("After max retries, messages are permanently removed from the dead letter queue.");

    // Note: In a real scenario, dead letter would be populated by failed deliveries
    // For this example, we'll just demonstrate the concept
    info!("(In a real scenario, failed deliveries would populate the dead letter queue)");

    info!("\n✅ All examples completed successfully!");
    info!("The LLM-optimized message broker provides:");
    info!("  • Priority-based queuing (Critical > High > Normal > Low)");
    info!("  • In-memory performance (no database required)");
    info!("  • Agent session management");
    info!("  • Dead letter queue with automatic retry");
    info!("  • Broadcast messaging to all agents");
    info!("  • Queue statistics and monitoring");

    Ok(())
}
