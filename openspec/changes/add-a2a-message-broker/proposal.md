# Change: Add Hybrid A2A Message Broker Infrastructure

## Why
The Constellation platform needs a high-performance, A2A-compatible message broker that balances speed with reliability. Based on implementation learnings, we need a hybrid approach: fast in-memory path for LLM agent communication (300k+ msg/sec) with optional PostgreSQL persistence for durability. This maintains A2A protocol compatibility while optimizing for LLM use cases.

## What Changes
- **ADD** Hybrid message broker with dual-path architecture:
  - **Fast path**: In-memory queues for LLM agents (300k+ msg/sec)
  - **Persistence path**: PostgreSQL for durable messages and audit trails
- **ADD** HTTP/WebSocket gateway with A2A protocol compliance
- **ADD** JWT-based authentication integrated with MCP security
- **ADD** Protocol version negotiation and backward compatibility
- **MODIFY** Agent A2A protocol to support hybrid delivery modes
- **MODIFY** Existing LLM broker implementation to add persistence and interfaces
- **BREAKING** All agent communication must use A2A-compatible interfaces

## Impact
- Affected specs: agent-a2a-protocol, message-broker (new), system-integration
- Affected code: Enhance existing `LlmMessageBroker`, add HTTP/WebSocket interfaces
- Infrastructure: Optional PostgreSQL, HTTP servers, Redis caching
- Performance: Maintain 300k+ msg/sec for fast path, add persistence options
- Agents: All agents gain A2A-compatible communication with performance options