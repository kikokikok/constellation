# Change: Add Agent Communication Framework

## Why
Agents need flexible communication patterns beyond basic message delivery to support complex workflows and interactions. The framework should provide request-response, publish-subscribe, and fire-and-forget patterns with reliability features.

## What Changes
- **ADD** Request-response pattern with timeout and retry
- **ADD** Publish-subscribe system with topic-based routing
- **ADD** Fire-and-forget pattern for one-way notifications
- **ADD** Delivery guarantees with idempotency
- **ADD** Priority-based message queuing
- **MODIFY** Message broker to support multiple communication patterns

## Impact
- Affected specs: agent-a2a-protocol
- Affected code: Message broker extensions, agent SDK
- Infrastructure: Redis pub/sub, additional queue types
- Agents: All agents gain access to advanced communication patterns