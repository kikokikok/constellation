# Design: Hybrid A2A Message Broker

## Context
The Constellation platform needs a message broker that balances extreme performance (300k+ msg/sec for LLM agents) with enterprise reliability (persistence, security, protocol compliance). Existing implementation provides fast in-memory routing but lacks A2A protocol compliance, persistence, and standard interfaces.

## Goals
1. **Performance**: Maintain 300k+ msg/sec for LLM agent communication
2. **Reliability**: Add PostgreSQL persistence for critical messages
3. **Compliance**: Full A2A protocol compatibility
4. **Security**: MCP-integrated authentication and authorization
5. **Interoperability**: HTTP/WebSocket interfaces for broad compatibility

## Non-Goals
1. **Full database-centric architecture**: We keep fast in-memory path
2. **Real-time replication**: Eventual consistency acceptable for most use cases
3. **Legacy protocol support**: Focus on A2A protocol evolution

## Decisions

### Decision 1: Dual-Path Architecture
**What**: Separate fast path (in-memory) and persistent path (PostgreSQL)
**Why**: Maintain 300k+ msg/sec performance while adding persistence
**Alternatives considered**:
- Single PostgreSQL path: Would reduce performance to <10k msg/sec
- Redis persistence: Less durable than PostgreSQL, similar performance impact
- Write-through cache: Complex consistency model

**Implementation**:
```rust
enum DeliveryMode {
    Fast,      // In-memory only, high throughput
    Persistent // PostgreSQL + in-memory, guaranteed delivery
}

struct HybridMessageBroker {
    fast_path: LlmMessageBroker,      // Existing in-memory implementation
    persistent_store: PostgresStore,  // New persistence layer
    router: MessageRouter,            // Routes based on delivery mode
}
```

### Decision 2: A2A Protocol Compliance Layer
**What**: Protocol validation and transformation layer
**Why**: Ensure compatibility with A2A specification while allowing extensions
**Implementation**:
```rust
struct A2AProtocolLayer {
    version: ProtocolVersion,  // 1.0, 1.1, 1.2, 2.0
    validator: MessageValidator,
    transformer: HeaderTransformer,
}

// All messages pass through this layer
message → A2AProtocolLayer → HybridMessageBroker
```

### Decision 3: Trait-Based Persistence
**What**: `PersistenceBackend` trait with multiple implementations
**Why**: Flexibility to switch storage backends, test with in-memory mock
**Implementation**:
```rust
trait PersistenceBackend {
    async fn store_message(&self, message: &Message) -> Result<MessageId>;
    async fn retrieve_message(&self, id: MessageId) -> Result<Message>;
    async fn list_messages(&self, filter: MessageFilter) -> Result<Vec<Message>>;
}

struct PostgresBackend { /* PostgreSQL implementation */ }
struct InMemoryBackend { /* For testing */ }
struct RedisBackend { /* Future option */ }
```

### Decision 4: HTTP/WebSocket Gateway Pattern
**What**: Separate gateway service from core broker
**Why**: Clean separation of concerns, independent scaling
**Implementation**:
```
HTTP/WebSocket Gateway (Axum)
        ↓
  Protocol Adapter (A2A)
        ↓
  Hybrid Message Broker
```

### Decision 5: MCP Security Integration
**What**: Use existing MCP cryptography for all security
**Why**: Avoid duplicate crypto implementations, ensure consistency
**Implementation**:
```rust
// Reuse MCP crypto types
use crate::mcp::crypto::{sign_message, verify_signature, encrypt_message};

struct SecurityLayer {
    crypto: McpCrypto,
    auth: JwtValidator,
    acl: AccessControlList,
}
```

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    HTTP/WebSocket Gateway               │
│  • Axum HTTP server                                    │
│  • WebSocket with A2A framing                          │
│  • Protocol negotiation (HTTP → WS)                    │
│  • Request/response, pub/sub endpoints                 │
└───────────────┬─────────────────────────────────────────┘
                │
┌───────────────▼─────────────────────────────────────────┐
│                A2A Protocol Layer                       │
│  • Protocol version negotiation (1.0, 1.1, 1.2, 2.0)   │
│  • Message validation against A2A schema                │
│  • Header transformation and preservation               │
│  • Extension point for gossip, TOON, etc.               │
└───────────────┬─────────────────────────────────────────┘
                │
┌───────────────▼─────────────────────────────────────────┐
│              Security Layer (MCP)                       │
│  • JWT validation using MCP crypto                     │
│  • Message signing/verification                         │
│  • Role-based access control                            │
│  • Audit logging integrated with MCP                    │
└───────────────┬─────────────────────────────────────────┘
                │
┌───────────────▼─────────────────────────────────────────┐
│              Hybrid Message Router                      │
│  • Routes based on delivery_mode                        │
│  • Fast path: LlmMessageBroker (in-memory)              │
│  • Persistent path: → Persistence Layer → Fast path     │
│  • Dead letter queue with retry logic                   │
└───────────────┬─────────────────────────────────────────┘
                │
     ┌──────────┴──────────┐
     │                     │
┌────▼─────┐        ┌─────▼─────┐
│ Fast Path│        │Persistence│
│In-memory │        │  Layer    │
│  Queues  │        │           │
│300k msg/s│        │ PostgreSQL│
└──────────┘        │  Adapter  │
                    └───────────┘
```

## Data Flow

### Fast Path Message (delivery_mode: fast)
```
1. Agent → HTTP/WS Gateway → A2A Protocol Layer
2. A2A Layer → Security Layer (auth validation)
3. Security Layer → Hybrid Router (route to fast path)
4. Fast Path → In-memory queues → Recipient agent
```

### Persistent Path Message (delivery_mode: persistent)
```
1. Agent → HTTP/WS Gateway → A2A Protocol Layer
2. A2A Layer → Security Layer (auth + signing)
3. Security Layer → Hybrid Router (route to persistent)
4. Persistence Layer → Store in PostgreSQL
5. Persistence Layer → Forward to Fast Path
6. Fast Path → In-memory queues → Recipient agent
```

## Data Models

### Message with Delivery Mode
```rust
struct Message {
    id: MessageId,
    sender: AgentId,
    recipient: AgentId,
    payload: MessagePayload,
    headers: A2AHeaders,
    delivery_mode: DeliveryMode,  // New field
    priority: MessagePriority,
    created_at: DateTime<Utc>,
}
```

### A2A Protocol Headers
```rust
struct A2AHeaders {
    protocol_version: String,      // "1.1", "1.2", "2.0"
    message_type: String,          // "request", "response", "broadcast"
    correlation_id: Option<String>,
    requires_ack: bool,
    ttl_seconds: Option<u32>,
    extensions: HashMap<String, serde_json::Value>,
}
```

### Persistence Record
```sql
CREATE TABLE messages (
    id UUID PRIMARY KEY,
    sender_id VARCHAR(255) NOT NULL,
    recipient_id VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL,
    headers JSONB NOT NULL,
    delivery_mode VARCHAR(20) NOT NULL,
    priority INTEGER NOT NULL,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    delivered_at TIMESTAMPTZ,
    INDEX idx_recipient_status (recipient_id, status),
    INDEX idx_created_at (created_at)
);
```

## Migration Plan

### Phase 1: Add Persistence Layer (Non-breaking)
1. Create `PersistenceBackend` trait
2. Implement `PostgresBackend`
3. Add `delivery_mode` field to `Message` (default: `Fast`)
4. Update `LlmMessageBroker` to accept persistence layer

### Phase 2: Add A2A Protocol Layer (Non-breaking)
1. Create `A2AProtocolLayer` with validation
2. Add protocol version negotiation
3. Update message headers to include A2A fields

### Phase 3: Add HTTP/WebSocket Gateway (New service)
1. Create Axum HTTP server
2. Implement WebSocket with A2A framing
3. Add authentication using MCP

### Phase 4: Security Integration (Non-breaking)
1. Integrate MCP crypto for JWT validation
2. Add role-based access control
3. Implement audit logging

### Phase 5: Protocol Evolution (Breaking)
1. Release A2A protocol 1.1 specification
2. Update all agents to use new protocol
3. Deprecate old message formats

## Risks & Trade-offs

### Risk 1: Performance Degradation
**Risk**: Adding layers reduces 300k+ msg/sec performance
**Mitigation**: 
- Benchmark each layer independently
- Use zero-copy deserialization (rkyv, bincode)
- Implement connection pooling for PostgreSQL

### Risk 2: Protocol Complexity
**Risk**: A2A protocol extensions become unwieldy
**Mitigation**:
- Clear extension points in protocol design
- Versioned schemas with backward compatibility
- Comprehensive validation

### Risk 3: Security Integration
**Risk**: MCP crypto integration introduces vulnerabilities
**Mitigation**:
- Reuse battle-tested MCP implementation
- Security audit before production
- Penetration testing

### Trade-off: Consistency vs Performance
**Choice**: Eventual consistency for fast path, strong for persistent
**Rationale**: LLM agents can tolerate minor message loss, business-critical messages need guarantees

## Open Questions

1. **Message ordering**: How to guarantee ordering across fast and persistent paths?
2. **Schema evolution**: Best approach for A2A protocol schema changes?
3. **Monitoring**: How to monitor hybrid system effectively?
4. **Backpressure**: Handling when fast path queues are full?

## Next Steps

1. Implement `PersistenceBackend` trait and `PostgresBackend`
2. Add `delivery_mode` to `Message` struct
3. Create `A2AProtocolLayer` with basic validation
4. Benchmark performance impact of each layer
5. Create migration script for existing messages