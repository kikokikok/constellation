# Constellation Python Client

A Python client for the Constellation A2A Message Broker API.

## Installation

```bash
pip install constellation-client
```

Or from source:

```bash
pip install -e .
```

## Quick Start

```python
from constellation import ConstellationClient
from datetime import datetime

# Create client
client = ConstellationClient(base_url="http://localhost:8080/v1")

# Check health
health = client.health()
print(f"Status: {health.status}")

# Authenticate (if you have agent credentials)
token_response = client.authenticate(
    agent_id="your-agent-id",
    signature="your-signature",
    timestamp=datetime.utcnow().isoformat()
)

# Set token for subsequent requests
client.set_token(token_response.token)

# Register a new agent
agent = client.register_agent(
    name="data-processor",
    public_key="your-public-key",
    capabilities=["data-processing", "analysis"],
    metadata={"version": "1.0.0"}
)

# Send a message
ack = client.send_message(
    recipient="agent-beta",
    sender="agent-alpha",
    message_type="command",
    payload={"action": "process_data", "dataset": "sales-2024"},
    priority=8,
    correlation_id="req-12345"
)

print(f"Message queued: {ack['messageId']}")

# Retrieve messages
messages = client.get_messages(
    agent_id="agent-beta",
    limit=10,
    priority="high"
)

for message in messages:
    print(f"Message from {message.sender}: {message.payload}")

# Close client
client.close()
```

## API Reference

### Client Initialization

```python
client = ConstellationClient(
    base_url="http://localhost:8080/v1",
    api_key="your-api-key",  # Optional
    token="your-jwt-token",  # Optional
    timeout=30
)
```

### Authentication

```python
# Get JWT token
token_response = client.authenticate(
    agent_id="your-agent-id",
    signature="signature-of-timestamp",
    timestamp="2025-01-15T10:30:00Z"
)

# Set token for subsequent requests
client.set_token(token_response.token)
```

### Agent Management

```python
# List agents
agents = client.list_agents(status="online", limit=50)

# Register agent
agent = client.register_agent(
    name="data-processor",
    public_key="MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE...",
    capabilities=["data-processing", "analysis"],
    metadata={"environment": "production"}
)

# Get agent details
agent = client.get_agent("agent-id")

# Deregister agent
client.deregister_agent("agent-id")

# Get agent status
status = client.get_agent_status("agent-id")
print(f"Queue size: {status.queue_stats.total}")
```

### Message Operations

```python
# Send message
ack = client.send_message(
    recipient="agent-beta",
    sender="agent-alpha",
    message_type="command",
    payload={"action": "process_data"},
    priority=8,
    correlation_id="req-12345",
    a2a_version="1.1",
    headers={"content-type": "application/json"},
    ttl=3600
)

# Retrieve messages
messages = client.get_messages(
    agent_id="agent-beta",
    limit=10,
    since="2025-01-15T10:00:00Z",
    priority="high"
)

# Get specific message
message = client.get_message("agent-id", "message-id")

# Delete message
client.delete_message("agent-id", "message-id")

# Broadcast message
broadcast_ack = client.broadcast_message(
    sender="system-admin",
    message_type="announcement",
    payload={"message": "System maintenance scheduled"},
    exclude_agents=["agent-in-testing"]
)
```

### System Operations

```python
# Health check
health = client.health()
print(f"Components: {health.components}")

# Get metrics
metrics = client.metrics()
print(metrics)

# Poll for response
response = client.poll_for_response(
    correlation_id="req-12345",
    timeout=30,
    poll_interval=1
)
```

## Error Handling

```python
from constellation import ConstellationError, AuthenticationError, APIError

try:
    client = ConstellationClient(base_url="http://localhost:8080/v1")
    health = client.health()
except AuthenticationError as e:
    print(f"Authentication failed: {e}")
except APIError as e:
    print(f"API error ({e.status_code}): {e}")
except ConstellationError as e:
    print(f"Client error: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

## Using with Context Manager

```python
with ConstellationClient(base_url="http://localhost:8080/v1") as client:
    health = client.health()
    print(f"Status: {health.status}")
    # Client automatically closed when exiting context
```

## Advanced Usage

### Custom Headers

```python
client.session.headers.update({
    "X-Custom-Header": "value",
    "X-Request-ID": "unique-id"
})
```

### Retry Logic

```python
import time
from constellation import APIError

def send_with_retry(client, max_retries=3):
    for attempt in range(max_retries):
        try:
            return client.send_message(...)
        except APIError as e:
            if e.status_code >= 500:  # Server error
                if attempt < max_retries - 1:
                    time.sleep(2 ** attempt)  # Exponential backoff
                    continue
            raise
    raise APIError("Max retries exceeded")
```

### Async Support (Example with asyncio)

```python
import asyncio
import aiohttp
import json

async def async_health_check(base_url):
    async with aiohttp.ClientSession() as session:
        async with session.get(f"{base_url}/health") as response:
            data = await response.json()
            return data
```

## Development

### Setup Development Environment

```bash
# Clone repository
git clone https://github.com/constellation/constellation.git
cd constellation/clients/python

# Install in development mode
pip install -e ".[dev]"

# Run tests
pytest

# Run type checking
mypy constellation/

# Format code
black constellation/
isort constellation/
```

### Running Tests

```bash
# Run all tests
pytest

# Run with coverage
pytest --cov=constellation --cov-report=html

# Run specific test file
pytest tests/test_client.py -v
```

## License

MIT License - see LICENSE file for details.

## Support

- Documentation: [https://docs.constellation.example.com](https://docs.constellation.example.com)
- Issues: [https://github.com/constellation/constellation/issues](https://github.com/constellation/constellation/issues)
- API Reference: [https://api.constellation.example.com/docs](https://api.constellation.example.com/docs)