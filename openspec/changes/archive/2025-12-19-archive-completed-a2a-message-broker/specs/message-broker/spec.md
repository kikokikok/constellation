## ADDED Requirements

### Requirement: Hybrid Message Broker Service
The system SHALL provide a hybrid message broker service with dual-path architecture for optimal performance and reliability. The broker SHALL route messages between agents with A2A protocol compliance and configurable delivery guarantees.

#### Scenario: Fast path message routing
- **WHEN** an agent sends a high-priority message with `delivery_mode: fast`
- **THEN** the broker SHALL route the message through in-memory queues
- **AND** achieve throughput > 100,000 messages per second
- **AND** maintain sub-millisecond latency

#### Scenario: Persistent message routing
- **WHEN** an agent sends a critical message with `delivery_mode: persistent`
- **THEN** the broker SHALL store the message in PostgreSQL
- **AND** guarantee delivery even after broker restart
- **AND** provide message audit trail

#### Scenario: Agent registration with A2A compatibility
- **WHEN** an agent starts up with A2A protocol version 1.0 or higher
- **AND** connects to the message broker
- **THEN** the broker SHALL register the agent with protocol version negotiation
- **AND** assign a unique session identifier compatible with A2A spec

#### Scenario: Message routing with A2A headers
- **WHEN** an agent sends a message with A2A protocol headers
- **THEN** the broker SHALL preserve all A2A headers during routing
- **AND** maintain message ordering for each sender-recipient pair
- **AND** validate message format against A2A schema

#### Scenario: Broadcast messages with A2A compliance
- **WHEN** an agent sends an A2A-compliant broadcast message
- **THEN** the broker SHALL deliver the message to all connected agents
- **AND** exclude the sender from the recipient list per A2A spec
- **AND** maintain broadcast delivery tracking

### Requirement: HTTP/WebSocket Gateway
The system SHALL provide HTTP and WebSocket interfaces for agent communication.

#### Scenario: REST API message sending
- **WHEN** an agent sends a message via HTTP POST
- **THEN** the broker SHALL accept the message
- **AND** return a message ID and delivery status

#### Scenario: WebSocket real-time communication
- **WHEN** an agent establishes a WebSocket connection
- **AND** subscribes to messages
- **THEN** the broker SHALL push incoming messages in real-time
- **AND** maintain connection state

#### Scenario: Connection health monitoring
- **WHEN** a WebSocket connection becomes unresponsive
- **THEN** the broker SHALL detect the failure
- **AND** attempt reconnection with exponential backoff

### Requirement: Authentication and Authorization
The system SHALL authenticate agents and authorize message operations.

#### Scenario: JWT token validation
- **WHEN** an agent presents a JWT token
- **THEN** the broker SHALL validate the token signature and claims
- **AND** extract agent identity and permissions

#### Scenario: API key authentication
- **WHEN** an agent uses an API key for authentication
- **THEN** the broker SHALL validate the key against stored credentials
- **AND** enforce rate limits based on key tier

#### Scenario: Permission-based authorization
- **WHEN** an agent attempts to send a message to a restricted recipient
- **THEN** the broker SHALL check authorization permissions
- **AND** reject unauthorized messages

### Requirement: Monitoring and Metrics
The system SHALL provide comprehensive monitoring and metrics for operational visibility.

#### Scenario: Message throughput monitoring
- **WHEN** messages are processed by the broker
- **THEN** the system SHALL track messages per second
- **AND** expose metrics via Prometheus endpoint

#### Scenario: Latency measurement
- **WHEN** a message is sent and delivered
- **THEN** the system SHALL measure end-to-end latency
- **AND** track latency percentiles (p50, p95, p99)

#### Scenario: Error tracking
- **WHEN** message delivery fails
- **THEN** the system SHALL log detailed error information
- **AND** increment error counters for alerting