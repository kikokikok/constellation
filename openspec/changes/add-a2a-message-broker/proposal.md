# Change: Add A2A Message Broker Infrastructure

## Why
The Constellation platform needs a robust message broker to enable reliable agent-to-agent communication using the A2A protocol. Currently, agents lack a standardized way to exchange messages with delivery guarantees, authentication, and protocol compliance.

## What Changes
- **ADD** Message broker service with PostgreSQL persistence
- **ADD** HTTP/WebSocket gateway for agent communication
- **ADD** JWT-based authentication service
- **ADD** Protocol version negotiation
- **MODIFY** Agent A2A protocol to include message delivery requirements
- **BREAKING** All agent communication must use the new message broker

## Impact
- Affected specs: agent-a2a-protocol, message-broker (new)
- Affected code: New Rust services, updates to agent models
- Infrastructure: PostgreSQL, Redis, HTTP servers
- Agents: All agents must update to use new communication patterns