//! Example demonstrating A2A agent communication using the Agent SDK
//!
//! This example shows how to:
//! 1. Create an agent client using the SDK
//! 2. Use different communication patterns (request-response, publish-subscribe, fire-and-forget)
//! 3. Handle incoming messages and requests
//! 4. Implement custom message and request handlers

use constellation_agent_sdk::{
    AgentClient, AgentConfig, DeliveryGuarantee, MessageHandler, MessagePriority, RequestHandler,
    RequestMessage, ResponseMessage,
};
use std::time::Duration;
use tracing::{info, warn};
use tracing_subscriber;

/// Custom request handler for CEO agent
struct CeoRequestHandler;

#[async_trait::async_trait]
impl RequestHandler for CeoRequestHandler {
    async fn handle_request(&self, request: RequestMessage) -> ResponseMessage {
        info!("CEO handling request: {}", request.request_id);

        // Parse the request payload
        let payload: serde_json::Value = match serde_json::from_str(&request.payload) {
            Ok(p) => p,
            Err(e) => {
                return ResponseMessage::error(
                    request.correlation_id,
                    request.recipient,
                    request.sender_id,
                    format!("Invalid JSON payload: {}", e),
                    Some(request),
                );
            }
        };

        // Handle different request types
        let response = match payload.get("type").and_then(|t| t.as_str()) {
            Some("budget_approval") => {
                info!("Processing budget approval request");
                // Simulate CEO decision making
                let approved = payload
                    .get("amount")
                    .and_then(|a| a.as_f64())
                    .map(|amount| amount <= 100000.0) // CEO approves up to $100k
                    .unwrap_or(false);

                if approved {
                    ResponseMessage::success(
                        request.correlation_id,
                        request.recipient,
                        request.sender_id,
                        serde_json::json!({
                            "status": "approved",
                            "message": "Budget request approved by CEO",
                            "amount": payload.get("amount").cloned().unwrap_or(serde_json::Value::Null)
                        })
                        .to_string(),
                        Some(request),
                    )
                } else {
                    ResponseMessage::error(
                        request.correlation_id,
                        request.recipient,
                        request.sender_id,
                        "Budget request denied - exceeds approval limit".to_string(),
                        Some(request),
                    )
                }
            }
            Some("strategic_decision") => {
                info!("Processing strategic decision request");
                // CEO makes strategic decisions
                ResponseMessage::success(
                    request.correlation_id,
                    request.recipient,
                    request.sender_id,
                    serde_json::json!({
                        "status": "decision_made",
                        "decision": "Proceed with initiative",
                        "rationale": "Aligned with company vision and growth strategy"
                    })
                    .to_string(),
                    Some(request),
                )
            }
            Some("ping") => {
                // Simple ping/pong
                ResponseMessage::success(
                    request.correlation_id,
                    request.recipient,
                    request.sender_id,
                    serde_json::json!({
                        "status": "pong",
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "agent": "ceo_agent"
                    })
                    .to_string(),
                    Some(request),
                )
            }
            _ => {
                warn!("Unknown request type");
                ResponseMessage::error(
                    request.correlation_id,
                    request.recipient,
                    request.sender_id,
                    format!("Unknown request type: {:?}", payload.get("type")),
                    Some(request),
                )
            }
        };

        info!("CEO request handled, sending response");
        response
    }
}

/// Custom message handler for system messages
struct SystemMessageHandler;

#[async_trait::async_trait]
impl MessageHandler for SystemMessageHandler {
    async fn handle_message(&self, message: constellation_agent_sdk::Message) -> Option<constellation_agent_sdk::Message> {
        info!("System handler received: {} - {}", message.message_type, message.payload);

        // Handle system alerts
        if message.message_type == "publish" {
            if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&message.payload) {
                if let Some(topic) = payload.get("topic").and_then(|t| t.as_str()) {
                    if topic.contains("system.alert") {
                        warn!("SYSTEM ALERT: {:?}", payload);
                    }
                }
            }
        }

        None // Don't send a response
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting CEO Agent Example with Agent SDK");
    info!("===========================================");

    // Create agent configuration
    let config = AgentConfig::new("ceo_agent")
        .with_broker_url("127.0.0.1:8090")
        .with_broker_credentials("guest", "guest")
        .with_default_request_timeout(Duration::from_secs(30))
        .with_auto_reconnect(true)
        .with_log_level("debug");

    info!("CEO Agent Configuration:");
    info!("  Agent ID: {}", config.agent_id);
    info!("  Broker: {}", config.broker_url);
    info!("  Default timeout: {}s", config.default_request_timeout.as_secs());

    // Connect to the broker
    info!("\nConnecting to message broker...");
    let mut client = AgentClient::connect(config).await?;

    // Set up custom handlers
    client = client
        .with_request_handler(CeoRequestHandler)
        .with_message_handler(SystemMessageHandler);

    // Start background message processing
    info!("Starting background message processing...");
    client.start().await?;

    // Subscribe to relevant topics
    info!("\nSubscribing to topics...");
    client.subscribe("system.*").await?;
    client.subscribe("finance.budget.*").await?;
    client.subscribe("strategy.*").await?;
    client.subscribe("agent.ceo_agent").await?;

    let subscriptions = client.get_subscriptions().await;
    info!("Active subscriptions: {}", subscriptions.len());

    // Publish CEO status
    info!("\nPublishing CEO status...");
    client
        .publish(
            "system.status",
            serde_json::json!({
                "agent": "ceo_agent",
                "status": "online",
                "role": "Chief Executive Officer",
                "capabilities": ["budget_approval", "strategic_decisions"],
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
            .to_string(),
            DeliveryGuarantee::AtLeastOnce,
        )
        .await?;

    // Send a notification to CFO
    info!("Sending notification to CFO...");
    client
        .notify(
            "cfo_agent",
            serde_json::json!({
                "type": "greeting",
                "message": "CEO is online and ready for budget reviews",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
            .to_string(),
            DeliveryGuarantee::BestEffort,
        )
        .await?;

    // Demonstrate request-response if another agent is available
    info!("\nTesting request-response pattern...");
    info!("(Start another agent to test actual communication)");
    
    // Try to ping a test agent
    match client
        .request(
            "test_agent",
            serde_json::json!({
                "type": "ping",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
            .to_string(),
            Duration::from_secs(5),
        )
        .await
    {
        Ok(response) => {
            info!("Received response from test_agent: {}", response.payload);
        }
        Err(e) => {
            info!("No test_agent available (expected): {}", e);
        }
    }

    // Broadcast leadership announcement
    info!("\nBroadcasting leadership announcement...");
    client
        .broadcast(
            "leadership.announcement",
            serde_json::json!({
                "type": "announcement",
                "from": "ceo_agent",
                "message": "Focus on innovation and customer satisfaction this quarter",
                "priority": "high",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
            .to_string(),
            MessagePriority::High,
        )
        .await?;

    // Check connected agents
    info!("\nChecking connected agents...");
    let connected_agents = client.get_connected_agents().await;
    info!("Connected agents: {:?}", connected_agents);

    // Run for a while to receive messages
    info!("\nCEO Agent is running and ready to receive messages!");
    info!("You can send messages to:");
    info!("  - Topic: 'agent.ceo_agent' for direct messages");
    info!("  - Topic: 'finance.budget.request' for budget approvals");
    info!("  - Topic: 'strategy.request' for strategic decisions");
    info!("\nPress Ctrl+C to exit...");

    // Keep running
    let mut seconds = 0;
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        seconds += 1;

        // Periodically check for messages
        if seconds % 10 == 0 {
            match client.receive(10).await {
                Ok(messages) => {
                    if !messages.is_empty() {
                        info!("Received {} messages in last 10 seconds", messages.len());
                    }
                }
                Err(e) => {
                    warn!("Error receiving messages: {}", e);
                }
            }

            // Update status
            client
                .publish(
                    "system.heartbeat",
                    serde_json::json!({
                        "agent": "ceo_agent",
                        "uptime_seconds": seconds,
                        "status": "healthy",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })
                    .to_string(),
                    DeliveryGuarantee::BestEffort,
                )
                .await?;
        }

        // Exit after 5 minutes for demo purposes
        if seconds >= 300 {
            info!("Demo complete - shutting down after 5 minutes");
            break;
        }
    }

    // Shutdown gracefully
    info!("\nShutting down CEO agent...");
    client.shutdown().await?;

    info!("CEO Agent example completed successfully!");
    Ok(())
}