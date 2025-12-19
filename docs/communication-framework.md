# Constellation Communication Framework

The Communication Framework provides A2A (Agent-to-Agent) communication patterns built on top of the message broker.

## Architecture

```
┌─────────────────┐
│    Agent SDK    │  ← High-level API for agents
├─────────────────┤
│ Communication   │  ← Pattern implementations
│   Framework     │    (request-response, pub-sub, etc.)
├─────────────────┤
│   Message       │  ← Broker abstraction
│    Broker       │
├─────────────────┤
│  Iggy/Redis/    │  ← Transport layer
│   PostgreSQL    │
└─────────────────┘
```

## Core Components

### 1. CommunicationFramework

The main struct that implements all communication patterns:

```rust
pub struct CommunicationFramework<B>
where
    B: MessageBroker + Send + Sync,
{
    subscriptions: Arc<RwLock<HashMap<String, Vec<Subscription>>>>,
    pending_requests: Arc<RwLock<HashMap<String, PendingRequest>>>,
    message_broker: Arc<B>,
    config: CommunicationConfig,
    metrics: Arc<CommunicationMetrics>,
}
```

### 2. MessageBroker Trait

Abstraction layer for different broker implementations:

```rust
#[async_trait::async_trait]
pub trait MessageBroker {
    async fn send_message(&self, message: BrokerMessage) -> MessageBrokerResult<()>;
    async fn receive_messages(&self, agent_id: &str, limit: usize) -> MessageBrokerResult<Vec<BrokerMessage>>;
    async fn register_session(&self, session: AgentSession) -> MessageBrokerResult<()>;
    async fn get_session(&self, agent_id: &str) -> MessageBrokerResult<Option<AgentSession>>;
    async fn broadcast(&self, message: BrokerMessage) -> MessageBrokerResult<()>;
}
```

### 3. Message Types

#### RequestMessage
```rust
pub struct RequestMessage {
    pub request_id: String,
    pub correlation_id: String,
    pub sender_id: String,
    pub recipient: String,
    pub payload: String,
    pub config: RequestConfig,
    pub priority: MessagePriority,
}
```

#### ResponseMessage
```rust
pub struct ResponseMessage {
    pub correlation_id: String,
    pub sender_id: String,
    pub recipient: String,
    pub payload: String,
    pub status: ResponseStatus,
    pub original_request: Option<RequestMessage>,
}
```

#### PublishMessage
```rust
pub struct PublishMessage {
    pub publisher_id: String,
    pub topic: String,
    pub payload: String,
    pub delivery_guarantee: DeliveryGuarantee,
    pub priority: MessagePriority,
    pub ttl_seconds: Option<u32>,
}
```

#### NotificationMessage
```rust
pub struct NotificationMessage {
    pub sender_id: String,
    pub recipient: String,
    pub payload: String,
    pub delivery_guarantee: DeliveryGuarantee,
    pub priority: MessagePriority,
}
```

## Pattern Implementations

### Request-Response Pattern

#### Flow
1. Client sends `RequestMessage` with correlation ID
2. Framework stores request as pending
3. Recipient processes request and sends `ResponseMessage`
4. Framework matches response to pending request using correlation ID
5. Client receives response or timeout

#### Key Features
- Configurable timeouts
- Automatic retries with exponential backoff
- Correlation ID tracking
- Priority-based queuing

#### Implementation Details

```rust
impl<B> CommunicationFramework<B> {
    pub async fn send_request(&self, request: RequestMessage) -> CommunicationResult<ResponseMessage> {
        // 1. Store pending request
        // 2. Send via message broker
        // 3. Wait for response with timeout
        // 4. Handle retries if configured
    }
    
    pub async fn handle_response(&self, response: ResponseMessage) -> CommunicationResult<()> {
        // 1. Find pending request by correlation ID
        // 2. Send response to waiting task
        // 3. Clean up pending request
    }
}
```

### Publish-Subscribe Pattern

#### Flow
1. Agents subscribe to topic patterns (exact, wildcard, regex)
2. Publisher sends `PublishMessage` to topic
3. Framework finds all matching subscribers
4. Message delivered to each subscriber

#### Topic Patterns
- **Exact**: `"system.alerts"`
- **Wildcard**: `"system.*"` (matches `system.alerts`, `system.metrics`)
- **Regex**: `"agent\.\d+"` (matches `agent.1`, `agent.42`)

#### Implementation Details

```rust
impl<B> CommunicationFramework<B> {
    pub async fn subscribe(&self, agent_id: String, topic_pattern: TopicPattern) -> CommunicationResult<Subscription> {
        // 1. Create subscription
        // 2. Add to subscriptions map
        // 3. Return subscription ID
    }
    
    pub async fn publish(&self, publish_message: PublishMessage) -> CommunicationResult<()> {
        // 1. Find all matching subscribers
        // 2. Send message to each subscriber
        // 3. Track delivery metrics
    }
}
```

### Fire-and-Forget Pattern

#### Flow
1. Sender creates `NotificationMessage`
2. Message sent via broker
3. No response expected or tracked

#### Use Cases
- Logging
- Metrics collection
- Status updates
- Event notifications

## Delivery Guarantees

### Implementation

Each pattern supports configurable delivery guarantees:

```rust
pub enum DeliveryGuarantee {
    BestEffort,     // No guarantees, fastest
    AtLeastOnce,    // Message delivered at least once
    AtMostOnce,     // Message delivered at most once  
    ExactlyOnce,    // Message delivered exactly once
}
```

### Guarantee Implementation

| Guarantee | Implementation | Use Case |
|-----------|----------------|----------|
| `BestEffort` | Single send attempt | Logging, metrics |
| `AtLeastOnce` | Retry until success | Notifications |
| `AtMostOnce` | Deduplication | Idempotent ops |
| `ExactlyOnce` | Transaction + dedup | Financial tx |

## Metrics Collection

### Metrics Structure

```rust
pub struct CommunicationMetrics {
    pub request_response: RequestResponseMetrics,
    pub publish_subscribe: PublishSubscribeMetrics,
    pub fire_and_forget: FireAndForgetMetrics,
    pub delivery_guarantees: DeliveryGuaranteeMetrics,
    pub errors: ErrorMetrics,
}
```

### Key Metrics

#### Request-Response
- `requests_sent`: Total requests sent
- `responses_received`: Total responses received
- `request_timeouts`: Request timeouts
- `avg_response_time_ms`: Average response time
- `requests_by_priority`: Requests by priority level

#### Publish-Subscribe
- `messages_published`: Messages published
- `messages_delivered`: Messages delivered to subscribers
- `active_subscriptions`: Current subscriptions
- `messages_by_topic`: Messages by topic

#### Fire-and-Forget
- `notifications_sent`: Notifications sent
- `notifications_by_guarantee`: By delivery guarantee
- `notifications_by_priority`: By priority level

#### Delivery Guarantees
- Counts by guarantee type
- Delivery failures by type
- Successful deliveries by type

#### Errors
- `total_errors`: Total errors
- `auth_errors`: Authentication errors
- `timeout_errors`: Timeout errors
- `network_errors`: Network errors
- `serialization_errors`: Serialization errors

### Metrics Integration

Metrics are automatically collected in all framework methods:

```rust
// In send_request
self.metrics.request_response.record_request_sent(request.priority);

// In publish
self.metrics.publish_subscribe.record_message_published(&topic);

// In send_notification
self.metrics.fire_and_forget.record_notification_sent(
    notification.delivery_guarantee,
    notification.priority,
);
```

## Configuration

### CommunicationConfig

```rust
pub struct CommunicationConfig {
    pub default_request_timeout: Duration,
    pub default_max_retries: u32,
    pub default_retry_base_delay: Duration,
    pub subscription_cleanup_interval: Duration,
    pub request_cleanup_interval: Duration,
    pub max_pending_requests: usize,
}
```

### Default Values

```rust
impl Default for CommunicationConfig {
    fn default() -> Self {
        Self {
            default_request_timeout: Duration::from_secs(30),
            default_max_retries: 3,
            default_retry_base_delay: Duration::from_secs(1),
            subscription_cleanup_interval: Duration::from_secs(300),
            request_cleanup_interval: Duration::from_secs(60),
            max_pending_requests: 1000,
        }
    }
}
```

## Background Tasks

The framework runs background tasks for:

1. **Subscription cleanup**: Removes inactive subscriptions
2. **Request cleanup**: Removes expired pending requests
3. **Metrics aggregation**: Periodically aggregates metrics

## Error Handling

### Error Types

```rust
pub enum CommunicationError {
    MaxRetriesExceeded(String),
    Timeout(String),
    SubscriptionError(String),
    InvalidResponse(String),
    MessageBrokerError(MessageBrokerError),
}
```

### Error Recovery

1. **Transient errors**: Automatic retry with backoff
2. **Permanent errors**: Return error to caller
3. **Network errors**: Reconnection logic in broker
4. **Timeout errors**: Clean up pending requests

## Testing

### Mock Broker for Testing

```rust
struct MockMessageBroker {
    sent_messages: Arc<tokio::sync::Mutex<Vec<BrokerMessage>>>,
    receive_queue: Arc<tokio::sync::Mutex<HashMap<String, VecDeque<BrokerMessage>>>>,
}

#[async_trait::async_trait]
impl MessageBroker for MockMessageBroker {
    async fn send_message(&self, message: BrokerMessage) -> MessageBrokerResult<()> {
        self.sent_messages.lock().await.push(message);
        Ok(())
    }
    
    async fn receive_messages(&self, agent_id: &str, limit: usize) -> MessageBrokerResult<Vec<BrokerMessage>> {
        // Return queued messages for agent
    }
}
```

### Test Coverage

The framework includes comprehensive tests for:

1. **Pattern validation**: Each communication pattern
2. **Error handling**: All error scenarios
3. **Metrics collection**: Metrics accuracy
4. **Concurrency**: Thread safety
5. **Integration**: End-to-end flows

## Performance Considerations

### Optimizations

1. **Lock granularity**: Fine-grained locking for concurrent access
2. **Memory efficiency**: Arc for shared ownership
3. **Async/await**: Non-blocking operations
4. **Batch operations**: Message batching where possible

### Scaling

1. **Horizontal scaling**: Multiple framework instances
2. **Connection pooling**: Reuse broker connections
3. **Message batching**: Batch sends/receives
4. **Background processing**: Offload to background tasks

## Integration Guide

### With IggyMessageBroker

```rust
// Create Iggy broker
let broker = IggyMessageBrokerBuilder::new()
    .server_address("127.0.0.1:8090")
    .credentials("guest", "guest")
    .build()
    .await?;

let broker = Arc::new(broker);

// Create framework
let framework = CommunicationFramework::new(broker.clone());

// Use patterns
let response = framework.send_request(request).await?;
framework.publish(publish_message).await?;
```

### Custom Broker Implementation

```rust
struct CustomBroker;

#[async_trait::async_trait]
impl MessageBroker for CustomBroker {
    async fn send_message(&self, message: BrokerMessage) -> MessageBrokerResult<()> {
        // Custom implementation
    }
    
    // Implement other methods...
}

// Use with framework
let broker = Arc::new(CustomBroker);
let framework = CommunicationFramework::new(broker);
```

## Best Practices

### Pattern Selection

1. **Request-Response**: When you need a response
2. **Publish-Subscribe**: For broadcasting to multiple agents
3. **Fire-and-Forget**: For notifications without responses

### Configuration

1. **Timeouts**: Set appropriate timeouts for operations
2. **Retries**: Configure retries for transient failures
3. **Limits**: Set reasonable limits for pending requests

### Monitoring

1. **Metrics**: Monitor key metrics for health
2. **Alerts**: Set up alerts for error thresholds
3. **Logging**: Enable debug logging for troubleshooting

## Future Enhancements

### Planned Features

1. **Streaming support**: Bidirectional streams for large data
2. **Compression**: Message compression for efficiency
3. **Encryption**: End-to-end encryption for security
4. **Schema validation**: Message schema validation
5. **Rate limiting**: Per-agent rate limiting

### Extension Points

1. **Custom serialization**: Plug-in serialization formats
2. **Custom routing**: Advanced message routing
3. **Custom metrics**: Additional metrics collection
4. **Custom handlers**: Advanced message processing