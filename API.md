# Constellation A2A Message Broker API Documentation

## Overview

The Constellation A2A Message Broker API provides a complete interface for agent-to-agent communication in multi-agent systems. This API implements the A2A (Agent-to-Agent) protocol with support for versions 1.0, 1.1, and 2.0.

## Quick Start

### 1. Local Development Setup

```bash
# Clone and build
git clone <repository>
cd constellation
cargo build --release

# Start with Docker Compose
docker-compose up -d

# Or run directly
cargo run --release --bin constellation-server
```

### 2. Authentication

#### JWT Token Authentication (Agents)
```bash
# Generate token
curl -X POST http://localhost:8080/v1/auth/token \
  -H "Content-Type: application/json" \
  -d '{
    "agentId": "550e8400-e29b-41d4-a716-446655440000",
    "signature": "base64-signed-timestamp",
    "timestamp": "2025-01-15T10:30:00Z"
  }'

# Response
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expiresAt": "2025-01-15T11:30:00Z",
  "agentId": "550e8400-e29b-41d4-a716-446655440000"
}
```

#### API Key Authentication (Services)
```bash
curl -X GET http://localhost:8080/v1/agents \
  -H "X-API-Key: your-api-key-here"
```

### 3. Register an Agent

```bash
curl -X POST http://localhost:8080/v1/agents \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "name": "data-processor-agent",
    "publicKey": "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE...",
    "capabilities": ["data-processing", "analysis", "reporting"],
    "metadata": {
      "version": "1.0.0",
      "environment": "production"
    }
  }'
```

## API Endpoints

### System Endpoints

#### Health Check
```http
GET /v1/health
```

**Response:**
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime": 3600,
  "components": {
    "message_broker": "healthy",
    "database": "healthy",
    "authentication": "healthy"
  }
}
```

#### Metrics
```http
GET /v1/metrics
```

Returns Prometheus metrics in text format for monitoring.

### Agent Management

#### List Agents
```http
GET /v1/agents?status=online&limit=50
```

**Query Parameters:**
- `status`: Filter by status (online, offline, error)
- `limit`: Maximum agents to return (1-1000, default: 100)

**Response:**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "data-processor-agent",
    "status": "online",
    "capabilities": ["data-processing", "analysis"],
    "metadata": {"version": "1.0.0"},
    "registeredAt": "2025-01-15T09:00:00Z",
    "lastSeen": "2025-01-15T10:30:00Z"
  }
]
```

#### Get Agent Details
```http
GET /v1/agents/{agentId}
```

#### Deregister Agent
```http
DELETE /v1/agents/{agentId}
```

### Message Operations

#### Send Message
```http
POST /v1/agents/{agentId}/messages
```

**Request Body:**
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "sender": "agent-alpha",
  "recipient": "agent-beta",
  "type": "command",
  "payload": {
    "action": "process_data",
    "parameters": {
      "dataset": "sales-2024",
      "operation": "aggregate"
    }
  },
  "timestamp": "2025-01-15T10:30:00Z",
  "priority": 8,
  "correlationId": "req-12345",
  "a2aVersion": "1.1",
  "headers": {
    "content-type": "application/json",
    "retry-count": "0"
  },
  "ttl": 3600
}
```

**Response (202 Accepted):**
```json
{
  "messageId": "123e4567-e89b-12d3-a456-426614174000",
  "status": "queued",
  "timestamp": "2025-01-15T10:30:01Z",
  "queuePosition": 5,
  "estimatedDelivery": "2025-01-15T10:30:05Z"
}
```

#### Retrieve Messages
```http
GET /v1/agents/{agentId}/messages?limit=10&since=2025-01-15T10:00:00Z&priority=high
```

**Query Parameters:**
- `limit`: Max messages to return (1-1000, default: 100)
- `since`: Only messages after this timestamp
- `priority`: Filter by priority (critical, high, normal, low)

**Response:**
```json
[
  {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "sender": "agent-alpha",
    "recipient": "agent-beta",
    "type": "command",
    "payload": {"action": "process_data"},
    "timestamp": "2025-01-15T10:30:00Z",
    "priority": 8,
    "correlationId": "req-12345",
    "a2aVersion": "1.1",
    "retryCount": 0
  }
]
```

#### Get Message Details
```http
GET /v1/agents/{agentId}/messages/{messageId}
```

#### Delete Message
```http
DELETE /v1/agents/{agentId}/messages/{messageId}
```

### Broadcast Messages

#### Send Broadcast
```http
POST /v1/broadcast
```

**Request Body:**
```json
{
  "sender": "system-admin",
  "type": "announcement",
  "payload": {
    "message": "System maintenance scheduled",
    "startTime": "2025-01-15T23:00:00Z",
    "duration": "2h"
  },
  "priority": 9,
  "excludeAgents": ["agent-in-testing"]
}
```

**Response:**
```json
{
  "broadcastId": "789e0123-f45g-67h8-i901-234567890123",
  "recipients": 42,
  "timestamp": "2025-01-15T10:30:00Z"
}
```

### Agent Status

#### Get Agent Status
```http
GET /v1/agents/{agentId}/status
```

**Response:**
```json
{
  "agentId": "550e8400-e29b-41d4-a716-446655440000",
  "status": "online",
  "lastActivity": "2025-01-15T10:30:00Z",
  "queueStats": {
    "total": 15,
    "byPriority": {
      "critical": 2,
      "high": 5,
      "normal": 8,
      "low": 0
    },
    "oldestMessage": "2025-01-15T10:15:00Z"
  },
  "sessionId": "session-abc123"
}
```

## Message Protocol

### A2A Protocol Versions

| Version | Features |
|---------|----------|
| 1.0 | Basic message exchange, no headers |
| 1.1 | Added headers, improved validation |
| 2.0 | Streaming support, chunked messages |

### Message Types

1. **Command**: Request for action (expects response)
2. **Query**: Request for information (expects response)
3. **Event**: Notification (no response expected)
4. **Response**: Response to command/query
5. **Error**: Error response

### Priority Levels

| Priority | Value | Description |
|----------|-------|-------------|
| Critical | 9-10 | System-critical, immediate delivery |
| High | 7-8 | Important, expedited delivery |
| Normal | 4-6 | Standard priority |
| Low | 1-3 | Background tasks |

## Error Handling

### Common Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| 400 | Bad Request | Check request format and parameters |
| 401 | Unauthorized | Provide valid authentication |
| 403 | Forbidden | Check agent permissions |
| 404 | Not Found | Resource doesn't exist |
| 409 | Conflict | Resource already exists |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Error | Server issue, retry later |

### Error Response Format
```json
{
  "code": "AGENT_NOT_FOUND",
  "message": "Agent with ID 550e8400-e29b-41d4-a716-446655440000 not found",
  "details": {
    "agentId": "550e8400-e29b-41d4-a716-446655440000"
  },
  "requestId": "req-abc123"
}
```

## Rate Limiting

- **Agents**: 100 requests/minute per agent
- **Services**: 1000 requests/minute per API key
- **Burst**: 20% above limit allowed for 10 seconds

Headers included in responses:
- `X-RateLimit-Limit`: Requests per minute
- `X-RateLimit-Remaining`: Remaining requests
- `X-RateLimit-Reset`: Reset timestamp

## WebSocket Interface

For real-time communication, use the WebSocket endpoint:

```javascript
const ws = new WebSocket('ws://localhost:8080/v1/ws');

ws.onopen = () => {
  // Authenticate
  ws.send(JSON.stringify({
    type: 'auth',
    token: 'your-jwt-token'
  }));
  
  // Subscribe to messages
  ws.send(JSON.stringify({
    type: 'subscribe',
    agentId: 'your-agent-id'
  }));
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Received:', message);
};
```

### WebSocket Message Types

1. **auth**: Authenticate connection
2. **subscribe**: Subscribe to agent messages
3. **message**: Send/receive messages
4. **ack**: Message acknowledgment
5. **error**: Error notification

## Examples

### Complete Agent Lifecycle

```python
import requests
import json
import time

class ConstellationAgent:
    def __init__(self, base_url, agent_id, private_key):
        self.base_url = base_url
        self.agent_id = agent_id
        self.private_key = private_key
        self.token = None
    
    def authenticate(self):
        """Get authentication token"""
        timestamp = time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())
        signature = self._sign_timestamp(timestamp)
        
        response = requests.post(
            f"{self.base_url}/auth/token",
            json={
                "agentId": self.agent_id,
                "signature": signature,
                "timestamp": timestamp
            }
        )
        
        if response.status_code == 200:
            data = response.json()
            self.token = data['token']
            return True
        return False
    
    def register(self, name, capabilities):
        """Register agent with broker"""
        response = requests.post(
            f"{self.base_url}/agents",
            headers={"Authorization": f"Bearer {self.token}"},
            json={
                "name": name,
                "publicKey": self._get_public_key(),
                "capabilities": capabilities
            }
        )
        return response.status_code == 201
    
    def send_message(self, recipient, payload, message_type="command"):
        """Send message to another agent"""
        response = requests.post(
            f"{self.base_url}/agents/{recipient}/messages",
            headers={"Authorization": f"Bearer {self.token}"},
            json={
                "sender": self.agent_id,
                "recipient": recipient,
                "type": message_type,
                "payload": payload,
                "timestamp": time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
                "priority": 5
            }
        )
        return response.json() if response.status_code == 202 else None
    
    def poll_messages(self):
        """Poll for new messages"""
        response = requests.get(
            f"{self.base_url}/agents/{self.agent_id}/messages",
            headers={"Authorization": f"Bearer {self.token}"},
            params={"limit": 10}
        )
        return response.json() if response.status_code == 200 else []
```

### Request/Response Pattern

```javascript
// Send request with correlation ID
const correlationId = `req-${Date.now()}`;

const request = {
  id: uuidv4(),
  sender: 'agent-a',
  recipient: 'agent-b',
  type: 'query',
  payload: { question: 'What is the current status?' },
  correlationId: correlationId,
  timestamp: new Date().toISOString()
};

// Send request
await fetch(`/v1/agents/agent-b/messages`, {
  method: 'POST',
  headers: { 'Authorization': `Bearer ${token}` },
  body: JSON.stringify(request)
});

// Poll for response
const response = await pollForResponse(correlationId);

async function pollForResponse(correlationId) {
  while (true) {
    const messages = await fetch(`/v1/agents/agent-a/messages?since=${new Date().toISOString()}`)
      .then(r => r.json());
    
    const response = messages.find(m => 
      m.correlationId === correlationId && m.type === 'response'
    );
    
    if (response) return response;
    
    await new Promise(resolve => setTimeout(resolve, 1000));
  }
}
```

## Monitoring and Observability

### Metrics

Key metrics available at `/v1/metrics`:

```
# Message throughput
constellation_messages_total{type="sent"} 1234
constellation_messages_total{type="received"} 5678
constellation_messages_total{type="delivered"} 5432
constellation_messages_total{type="failed"} 246

# Queue sizes
constellation_queue_size{agent="agent-a",priority="high"} 5
constellation_queue_size{agent="agent-a",priority="normal"} 12

# Agent status
constellation_agents_total{status="online"} 8
constellation_agents_total{status="offline"} 2

# Performance
constellation_message_delivery_latency_seconds{quantile="0.95"} 0.123
constellation_message_processing_duration_seconds_sum 456.7
```

### Tracing

Distributed tracing is enabled by default. Include these headers in requests:

- `X-Trace-Id`: Trace identifier
- `X-Span-Id`: Span identifier
- `X-Parent-Span-Id`: Parent span identifier

## Security

### Authentication Methods

1. **JWT Tokens**: For agent authentication (Ed25519 signatures)
2. **API Keys**: For service-to-service communication
3. **Mutual TLS**: For production deployments (recommended)

### Security Headers

All responses include security headers:
- `Strict-Transport-Security`: Enforce HTTPS
- `X-Content-Type-Options`: Prevent MIME sniffing
- `X-Frame-Options`: Prevent clickjacking
- `Content-Security-Policy`: Restrict resource loading

### Data Protection

- All messages encrypted in transit (TLS 1.3)
- Sensitive data encrypted at rest
- Automatic key rotation
- Audit logging for all operations

## Troubleshooting

### Common Issues

1. **Authentication Failures**
   - Check token expiration
   - Verify signature algorithm
   - Ensure timestamp is within 5 minutes

2. **Message Delivery Delays**
   - Check agent status (online/offline)
   - Review queue statistics
   - Verify priority settings

3. **Rate Limiting**
   - Monitor rate limit headers
   - Implement exponential backoff
   - Consider batching requests

### Debugging

Enable debug logging:
```bash
RUST_LOG=debug cargo run --bin constellation-server
```

Check server logs for detailed error information.

## Support

- **Documentation**: [https://docs.constellation.example.com](https://docs.constellation.example.com)
- **API Reference**: Interactive Swagger UI at `/v1/docs`
- **Community**: [GitHub Discussions](https://github.com/constellation/discussions)
- **Issues**: [GitHub Issues](https://github.com/constellation/issues)

---

*Last Updated: 2025-01-15*  
*API Version: 1.0.0*