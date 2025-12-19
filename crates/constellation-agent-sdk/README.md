# Constellation Agent SDK

High-level SDK for building agents that communicate using A2A (Agent-to-Agent) patterns in the Constellation platform.

## Features

- **Request-Response Pattern**: Send requests and await responses with configurable timeouts and retries
- **Publish-Subscribe**: Subscribe to topics and receive published messages with wildcard support
- **Fire-and-Forget**: Send notifications without waiting for responses
- **Delivery Guarantees**: Configurable delivery semantics (BestEffort, AtLeastOnce, AtMostOnce, ExactlyOnce)
- **Priority-Based Queuing**: Message prioritization (Critical, High, Normal, Low)
- **Connection Management**: Automatic reconnection and session management
- **Comprehensive Metrics**: Built-in metrics collection for monitoring
- **Extensible Handlers**: Custom message and request handlers

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
constellation-agent-sdk = { path = "../crates/constellation-agent-sdk" }
```

### Basic Usage

```rust
use constellation_agent_sdk::{AgentClient, AgentConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create agent configuration
    let config = AgentConfig::new("my_agent")
        .with_broker_url("127.0.0.1:8090")
        .with_broker_credentials("guest", "guest");

    // Connect to the broker
    let mut client = AgentClient::connect(config).await?;

    // Start background message processing
    client.start().await?;

    // Subscribe to topics
    client.subscribe("system.*").await?;
    client.subscribe("agent.my_agent").await?;

    // Send a request
    let response = client
        .request(
            "other_agent",
            "{\"action\": \"ping\"}",
            Duration::from_secs(30),
        )
        .await?;

    println!("Received response: {}", response.payload);

    // Send a notification
    client
        .notify(
            "broadcast",
            "{\"event\": \"status_update\"}",
            constellation_agent_sdk::DeliveryGuarantee::BestEffort,
        )
        .await?;

    // Publish to a topic
    client
        .publish(
            "system.status",
            "{\"agent\": \"my_agent\", \"status\": \"online\"}",
            constellation_agent_sdk::DeliveryGuarantee::AtLeastOnce,
        )
        .await?;

    // Keep running
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
```

## Communication Patterns

### 1. Request-Response

Use for synchronous communication where you need a response:

```rust
// Send request with timeout
let response = client
    .request("agent_b", "{\"query\": \"data\"}", Duration::from_secs(30))
    .await?;

// Handle response
match response.status {
    constellation_agent_sdk::ResponseStatus::Success => {
        println!("Success: {}", response.payload);
    }
    constellation_agent_sdk::ResponseStatus::Error => {
        println!("Error: {}", response.payload);
    }
}
```

### 2. Publish-Subscribe

Use for broadcasting messages to multiple subscribers:

```rust
// Subscribe to topics
client.subscribe("finance.*").await?;      // Wildcard subscription
client.subscribe("system.alerts").await?;  // Exact subscription

// Publish to topic
client
    .publish(
        "finance.reports",
        "{\"report\": \"Q1_2024.pdf\"}",
        constellation_agent_sdk::DeliveryGuarantee::AtLeastOnce,
    )
    .await?;
```

### 3. Fire-and-Forget

Use for notifications where no response is needed:

```rust
client
    .notify(
        "logging_service",
        "{\"log\": \"Application started\"}",
        constellation_agent_sdk::DeliveryGuarantee::BestEffort,
    )
    .await?;
```

## Delivery Guarantees

Choose the appropriate delivery guarantee based on your requirements:

| Guarantee | Description | Use Case |
|-----------|-------------|----------|
| `BestEffort` | No delivery guarantees, fastest | Logging, metrics |
| `AtLeastOnce` | Message delivered at least once | Notifications, updates |
| `AtMostOnce` | Message delivered at most once | Idempotent operations |
| `ExactlyOnce` | Message delivered exactly once | Financial transactions |

```rust
use constellation_agent_sdk::DeliveryGuarantee;

// Financial transaction - must be processed exactly once
client
    .notify(
        "payment_service",
        "{\"transaction\": \"tx_123\"}",
        DeliveryGuarantee::ExactlyOnce,
    )
    .await?;

// Log message - best effort is sufficient
client
    .notify(
        "log_aggregator",
        "{\"level\": \"info\", \"message\": \"User logged in\"}",
        DeliveryGuarantee::BestEffort,
    )
    .await?;
```

## Message Priority

Prioritize messages based on importance:

```rust
use constellation_agent_sdk::MessagePriority;

// Critical system alert
client
    .send_message(
        "monitoring",
        "alert",
        "{\"level\": \"critical\", \"message\": \"System down\"}",
        MessagePriority::Critical,
    )
    .await?;

// Normal status update
client
    .send_message(
        "dashboard",
        "update",
        "{\"status\": \"ok\"}",
        MessagePriority::Normal,
    )
    .await?;
```

## Custom Handlers

### Request Handler

Handle incoming requests:

```rust
use constellation_agent_sdk::{RequestHandler, RequestMessage, ResponseMessage};
use async_trait::async_trait;

struct MyRequestHandler;

#[async_trait]
impl RequestHandler for MyRequestHandler {
    async fn handle_request(&self, request: RequestMessage) -> ResponseMessage {
        println!("Received request: {}", request.request_id);
        
        // Process request and create response
        ResponseMessage::success(
            request.correlation_id,
            request.recipient,
            request.sender_id,
            "{\"result\": \"processed\"}".to_string(),
            Some(request),
        )
    }
}

// Use with client
let mut client = AgentClient::connect(config).await?
    .with_request_handler(MyRequestHandler);
```

### Message Handler

Handle incoming messages:

```rust
use constellation_agent_sdk::{MessageHandler, Message};
use async_trait::async_trait;

struct MyMessageHandler;

#[async_trait]
impl MessageHandler for MyMessageHandler {
    async fn handle_message(&self, message: Message) -> Option<Message> {
        println!("Received message: {} - {}", message.message_type, message.payload);
        
        // Return None if no response needed
        None
        
        // Or return a response message
        // Some(Message::new(...))
    }
}

// Use with client
let mut client = AgentClient::connect(config).await?
    .with_message_handler(MyMessageHandler);
```

## Metrics Collection

Monitor communication patterns with built-in metrics:

```rust
// Get metrics from client (requires access to CommunicationFramework)
// In the SDK, metrics are collected internally

// Example: Export metrics as JSON
let metrics_snapshot = client.metrics_snapshot(); // If exposed in SDK
println!("Metrics: {}", serde_json::to_string_pretty(&metrics_snapshot)?);
```

Key metrics collected:
- Request-response: Counts, timeouts, response times
- Publish-subscribe: Messages published, delivered, subscriptions
- Fire-and-forget: Notifications sent by guarantee type
- Delivery guarantees: Success/failure rates
- Errors: By error type (auth, timeout, network, etc.)

## Configuration

### Environment Variables

```bash
export CONSTELLATION_AGENT_ID="my_agent"
export CONSTELLATION_BROKER_URL="127.0.0.1:8090"
export CONSTELLATION_BROKER_USERNAME="guest"
export CONSTELLATION_BROKER_PASSWORD="guest"
```

### Configuration File

```json
{
  "agent_id": "my_agent",
  "broker_url": "127.0.0.1:8090",
  "broker_username": "guest",
  "broker_password": "guest",
  "default_request_timeout_seconds": 30,
  "default_max_retries": 3,
  "auto_reconnect": true,
  "enable_persistence": true
}
```

Load from file:
```rust
let config = AgentConfig::from_file("agent_config.json")?;
```

## Examples

See the `examples/` directory for complete examples:

1. `basic_agent.rs` - Basic agent with all communication patterns
2. `sdk_agent_example.rs` - Complete agent with custom handlers

Run examples:
```bash
cargo run --example basic_agent
cargo run --example sdk_agent_example
```

## Best Practices

1. **Choose the right pattern**:
   - Use request-response for operations needing confirmation
   - Use publish-subscribe for broadcasting to multiple agents
   - Use fire-and-forget for notifications and logging

2. **Set appropriate timeouts**:
   - Short timeouts (1-5s) for fast operations
   - Medium timeouts (30s) for typical requests
   - Long timeouts (5+ minutes) for complex operations

3. **Handle errors gracefully**:
   - Implement retry logic for transient failures
   - Log errors for debugging
   - Provide fallback behavior when possible

4. **Monitor metrics**:
   - Track request success rates
   - Monitor response times
   - Alert on error thresholds

## Troubleshooting

### Common Issues

1. **Connection refused**: Ensure Iggy server is running
2. **Authentication failed**: Check broker credentials
3. **Message not delivered**: Verify recipient agent is connected
4. **Timeout errors**: Increase timeout or check network

### Debugging

Enable debug logging:
```rust
let config = AgentConfig::new("my_agent")
    .with_log_level("debug");
```

Check connection status:
```rust
let connected_agents = client.get_connected_agents().await;
println!("Connected agents: {:?}", connected_agents);
```

## API Reference

See the Rust documentation for complete API details:

```bash
cargo doc --open --package constellation-agent-sdk
```

## License

MIT OR Apache-2.0