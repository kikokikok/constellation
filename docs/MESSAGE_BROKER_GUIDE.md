# Constellation A2A Message Broker Guide

## Overview

The Constellation A2A Message Broker is a PostgreSQL-based message routing system that enables reliable agent-to-agent communication with delivery guarantees, priority-based queuing, and real-time WebSocket support.

## Architecture

### Core Components

1. **Database Layer** (`message_broker/database.rs`)
   - PostgreSQL schema with tables for messages, queues, delivery status, and agent sessions
   - SQL functions for queue operations and maintenance
   - Transactional operations for data consistency

2. **Service Layer** (`message_broker/service.rs`)
   - Business logic for message routing and delivery
   - Agent connection management
   - Priority-based queuing with delivery guarantees
   - Dead letter queue management

3. **WebSocket Layer** (`message_broker/websocket.rs`)
   - Real-time bidirectional communication
   - Agent authentication and session management
   - Message broadcasting and targeted delivery

4. **Data Models** (`models/message_broker.rs`)
   - Rust types mapping to database schema
   - A2A protocol message formats
   - Error types and result handling

## Database Schema

### Key Tables

#### `messages`
- Stores all A2A protocol messages with metadata
- Supports message expiration (TTL)
- Includes correlation IDs for request-response patterns

#### `queues`
- Priority-based message queuing (Low, Normal, High, Critical)
- Ensures message ordering within priority levels
- Supports multiple named queues

#### `delivery_status`
- Tracks message delivery state with retry logic
- Implements exponential backoff for failed deliveries
- Records acknowledgments from recipients

#### `agent_sessions`
- Manages agent connection state
- Supports WebSocket and HTTP protocol bindings
- Automatic session expiration and cleanup

#### `dead_letter_queue`
- Stores messages that failed delivery after max retries
- Enables manual review and retry of failed messages

### Database Functions

- `get_next_queue_message()` - Atomically dequeues next message
- `expire_old_messages()` - Moves expired messages to dead letter queue
- `update_updated_at_column()` - Automatic timestamp updates

## Installation & Setup

### 1. Database Setup

```bash
# Create PostgreSQL database
createdb constellation

# Run migrations
psql constellation < migrations/001_initial_message_broker_schema.sql
```

### 2. Configuration

Environment variables:
```bash
DATABASE_URL=postgresql://localhost/constellation
HOST=127.0.0.1
PORT=8080
```

### 3. Running the Server

```bash
# Run example server
cargo run --example message_broker_server

# Or build and run
cargo build --release
./target/release/examples/message_broker_server
```

## API Reference

### REST Endpoints

#### Health Check
```
GET /api/v1/health
```
Returns service health status.

#### Dashboard Statistics
```
GET /api/v1/dashboard
```
Returns message and queue statistics.

#### Queue Statistics
```
GET /api/v1/queues/{queue_name}/stats
```
Returns statistics for a specific queue.

### WebSocket Endpoint

```
WS /ws
```
Real-time message delivery with authentication.

**Connection Headers:**
```http
Authorization: Bearer <session-token>
```

**Query Parameters:**
```
?token=<session-token>
```

## Usage Examples

### 1. Basic Message Broker Setup

```rust
use constellation_core::message_broker::{
    MessageBrokerService, MessageBrokerServiceBuilder
};

let service = MessageBrokerServiceBuilder::new()
    .database_url("postgresql://localhost/constellation")
    .default_queue("agent-communications")
    .max_message_size(10 * 1024 * 1024) // 10MB
    .default_ttl(86400) // 24 hours
    .max_retries(3)
    .build()
    .await?;
```

### 2. Sending Messages

```rust
use constellation_core::message_broker::{A2AMessage, MessagePriority};

let message = A2AMessage::new(
    "msg-001".to_string(),
    "architect-001".to_string(),
    "engineer-002".to_string(),
    "request".to_string(),
    serde_json::json!({"task": "design"}).to_string(),
)
.with_priority(MessagePriority::High)
.with_ttl(3600); // 1 hour TTL

let message_id = service.send_message(message).await?;
```

### 3. Receiving Messages

```rust
let messages = service.receive_messages("engineer-002", 10).await?;

for message in messages {
    println!("Received: {}", message.message_id);
    
    // Process message...
    
    // Acknowledge delivery
    service.acknowledge_message(&message.message_id, acknowledgment).await?;
}
```

### 4. Agent Connection

```rust
use constellation_core::message_broker::AgentConnectionRequest;

let request = AgentConnectionRequest {
    agent_id: "architect-001".to_string(),
    protocol_binding: "websocket".to_string(),
    capabilities: Some(serde_json::json!({
        "name": "System Architect",
        "skills": ["design", "architecture"]
    })),
    auth_token: Some("secret-token".to_string()),
};

let response = service.connect_agent(request).await?;
println!("Session token: {}", response.session_token);
```

### 5. WebSocket Communication

```javascript
// Client-side WebSocket example
const ws = new WebSocket('ws://localhost:8080/ws?token=session-token');

ws.onopen = () => {
    // Send message
    ws.send(JSON.stringify({
        type: 'send_message',
        message: {
            message_id: 'msg-001',
            sender_id: 'client-001',
            recipient_id: 'server-001',
            message_type: 'request',
            payload: JSON.stringify({ action: 'ping' })
        }
    }));
};

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log('Received:', data);
};
```

## Message Priorities

| Priority | Use Case | Delivery Order |
|----------|----------|----------------|
| Critical | System alerts, failures | First |
| High | Time-sensitive requests | Second |
| Normal | Regular communication | Third |
| Low | Background tasks, logs | Last |

## Delivery Guarantees

### At-Least-Once Delivery
- Messages are persisted to database before acknowledgment
- Automatic retry with exponential backoff on failure
- Dead letter queue for messages that exceed max retries

### Message Ordering
- Messages are delivered in order within each sender-recipient pair
- Priority-based ordering across different senders
- Sequence numbers ensure ordering within same priority

### Message Expiration
- Time-to-live (TTL) configurable per message
- Expired messages moved to dead letter queue
- Automatic cleanup of expired data

## Error Handling

### Common Errors

1. **MessageTooLarge** - Exceeds configured maximum size
2. **InvalidMessage** - Malformed or invalid message format  
3. **AuthenticationFailed** - Invalid or expired session token
4. **MessageNotFound** - Requested message doesn't exist
5. **DeliveryFailed** - Message delivery failed after retries

### Retry Logic

```rust
// Exponential backoff formula
let delay_seconds = 2^current_retry * base_delay;

// Example retry schedule:
// Retry 1: 5 seconds
// Retry 2: 10 seconds  
// Retry 3: 20 seconds
// Retry 4: 40 seconds (moves to dead letter if max_retries = 3)
```

## Monitoring & Metrics

### Dashboard Statistics

```json
{
  "total_messages": 150,
  "pending_messages": 25,
  "delivered_messages": 120,
  "failed_messages": 5,
  "active_sessions": 8,
  "queues": [
    {"name": "default", "size": 15},
    {"name": "high-priority", "size": 10}
  ]
}
```

### Queue Statistics

```json
{
  "queue_name": "default",
  "total_messages": 100,
  "pending_messages": 15,
  "delivered_messages": 80,
  "failed_messages": 5,
  "avg_delivery_time_ms": 45.2
}
```

## Performance Considerations

### Database Optimization
- Indexes on frequently queried columns (message_id, status, timestamps)
- Connection pooling for high concurrency
- Regular maintenance tasks (expiration cleanup)

### Message Size Limits
- Default maximum: 10MB per message
- Configurable via `max_message_size` parameter
- Consider chunking for large payloads

### Concurrency
- WebSocket connections: ~10,000 concurrent connections
- Message throughput: ~10,000 messages/second (depends on hardware)
- Database connections: Configured via connection pool size

## Security

### Authentication
- JWT or session token-based authentication
- Token expiration and renewal
- Rate limiting per agent

### Authorization
- Agent-based permission checking
- Message validation and sanitization
- Input validation for all API endpoints

### Data Protection
- Message payload encryption (client responsibility)
- Secure WebSocket connections (wss://)
- Database connection encryption

## Deployment

### Docker Example

```dockerfile
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq5
COPY --from=builder /app/target/release/message_broker_server /usr/local/bin/
CMD ["message_broker_server"]
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: constellation-message-broker
spec:
  replicas: 3
  selector:
    matchLabels:
      app: message-broker
  template:
    metadata:
      labels:
        app: message-broker
    spec:
      containers:
      - name: broker
        image: constellation/message-broker:latest
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: url
        ports:
        - containerPort: 8080
```

## Troubleshooting

### Common Issues

1. **Database Connection Errors**
   - Check PostgreSQL is running
   - Verify connection string format
   - Ensure database user has proper permissions

2. **WebSocket Connection Failures**
   - Verify session token is valid
   - Check CORS configuration
   - Ensure WebSocket protocol is supported

3. **Message Delivery Failures**
   - Check recipient agent is connected
   - Verify message TTL hasn't expired
   - Review dead letter queue for patterns

### Logging

Enable debug logging:
```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

## Related Documentation

- [A2A Protocol Specification](../openspec/changes/add-a2a-message-broker/specs/agent-a2a-protocol/spec.md)
- [Database Schema](../migrations/001_initial_message_broker_schema.sql)
- [API Examples](../examples/message_broker_example.rs)
- [Server Implementation](../examples/message_broker_server.rs)