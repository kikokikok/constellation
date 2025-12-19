"""
Constellation Python Client

A comprehensive Python client for the Constellation A2A Message Broker API.
"""

import json
import time
import uuid
from typing import Dict, List, Optional, Any
from datetime import datetime

try:
    import requests
    from requests.exceptions import RequestException

    REQUESTS_AVAILABLE = True
except ImportError:
    REQUESTS_AVAILABLE = False
    requests = None
    RequestException = Exception

from .models import (
    Agent,
    Message,
    HealthResponse,
    TokenResponse,
    MessageAck,
    BroadcastAck,
    AgentStatus,
)
from .exceptions import ConstellationError, AuthenticationError, APIError


class ConstellationClient:
    """
    Constellation A2A Message Broker Client.

    This client provides a Python interface to the Constellation A2A Message Broker API.

    Example:
        ```python
        # Create client
        client = ConstellationClient(base_url="http://localhost:8080/v1")

        # Get health status
        health = client.health()
        print(f"Status: {health.status}")

        # Authenticate
        token = client.authenticate(
            agent_id="your-agent-id",
            signature="your-signature",
            timestamp=datetime.utcnow().isoformat()
        )

        # Register agent
        agent = client.register_agent(
            name="data-processor",
            public_key="your-public-key",
            capabilities=["data-processing", "analysis"]
        )

        # Send message
        ack = client.send_message(
            recipient="agent-beta",
            sender="agent-alpha",
            message_type="command",
            payload={"action": "process_data"}
        )
        ```
    """

    def __init__(
        self,
        base_url: str,
        api_key: Optional[str] = None,
        token: Optional[str] = None,
        timeout: int = 30,
    ):
        """
        Initialize the Constellation client.

        Args:
            base_url: Base URL of the Constellation API (e.g., "http://localhost:8080/v1")
            api_key: API key for service authentication (optional)
            token: JWT token for agent authentication (optional)
            timeout: Request timeout in seconds

        Raises:
            ImportError: If requests library is not installed
        """
        if not REQUESTS_AVAILABLE:
            raise ImportError(
                "The 'requests' library is required for ConstellationClient. "
                "Install it with: pip install requests"
            )

        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.token = token
        self.timeout = timeout
        self.session = requests.Session()

        # Set default headers
        self.session.headers.update(
            {
                "Content-Type": "application/json",
                "Accept": "application/json",
                "User-Agent": f"Constellation-Python-Client/0.1.0",
            }
        )

        # Add authentication headers if provided
        if self.api_key:
            self.session.headers["X-API-Key"] = self.api_key
        if self.token:
            self.session.headers["Authorization"] = f"Bearer {self.token}"

    def _request(self, method: str, endpoint: str, **kwargs) -> Dict[str, Any]:
        """
        Make an HTTP request to the Constellation API.

        Args:
            method: HTTP method (GET, POST, PUT, DELETE)
            endpoint: API endpoint (e.g., "/health")
            **kwargs: Additional arguments for requests.request()

        Returns:
            Response data as dictionary

        Raises:
            APIError: If the API request fails
            AuthenticationError: If authentication fails
            ConstellationError: For other errors
        """
        url = f"{self.base_url}{endpoint}"

        try:
            response = self.session.request(
                method=method, url=url, timeout=self.timeout, **kwargs
            )

            # Handle errors
            if response.status_code >= 400:
                try:
                    error_data = response.json()
                    error_msg = error_data.get("message", response.text)
                    error_code = error_data.get("code")

                    if response.status_code == 401:
                        raise AuthenticationError(f"Authentication failed: {error_msg}")
                    else:
                        raise APIError(
                            message=f"API error: {error_msg}",
                            status_code=response.status_code,
                            error_code=error_code,
                        )
                except ValueError:
                    raise APIError(
                        message=f"API error: {response.text}",
                        status_code=response.status_code,
                    )

            # Parse response
            if response.status_code == 204:  # No content
                return {}

            return response.json()

        except RequestException as e:
            raise ConstellationError(f"Request failed: {str(e)}")

    def health(self) -> HealthResponse:
        """
        Get health status of the message broker.

        Returns:
            HealthResponse object with status information
        """
        data = self._request("GET", "/health")
        return HealthResponse.from_dict(data)

    def metrics(self) -> str:
        """
        Get Prometheus metrics.

        Returns:
            Metrics in Prometheus text format
        """
        response = self.session.get(
            f"{self.base_url}/metrics",
            headers={"Accept": "text/plain"},
            timeout=self.timeout,
        )
        response.raise_for_status()
        return response.text

    def authenticate(
        self, agent_id: str, signature: str, timestamp: Optional[str] = None
    ) -> TokenResponse:
        """
        Authenticate and get JWT token.

        Args:
            agent_id: Agent identifier
            signature: Signature of timestamp using agent's private key
            timestamp: Timestamp used for signature (defaults to current time)

        Returns:
            TokenResponse with JWT token
        """
        if timestamp is None:
            timestamp = datetime.utcnow().isoformat() + "Z"

        data = {"agentId": agent_id, "signature": signature, "timestamp": timestamp}

        response_data = self._request("POST", "/auth/token", json=data)
        return TokenResponse.from_dict(response_data)

    def set_token(self, token: str):
        """
        Set authentication token for subsequent requests.

        Args:
            token: JWT authentication token
        """
        self.token = token
        self.session.headers["Authorization"] = f"Bearer {token}"

    def list_agents(
        self, status: Optional[str] = None, limit: Optional[int] = None
    ) -> List[Agent]:
        """
        List registered agents.

        Args:
            status: Filter by status (online, offline, error)
            limit: Maximum number of agents to return

        Returns:
            List of Agent objects
        """
        params = {}
        if status:
            params["status"] = status
        if limit:
            params["limit"] = limit

        data = self._request("GET", "/agents", params=params)
        return [Agent.from_dict(agent_data) for agent_data in data]

    def register_agent(
        self,
        name: str,
        public_key: str,
        capabilities: Optional[List[str]] = None,
        metadata: Optional[Dict[str, Any]] = None,
    ) -> Agent:
        """
        Register a new agent.

        Args:
            name: Human-readable agent name
            public_key: Agent's public key for authentication
            capabilities: List of agent capabilities
            metadata: Additional agent metadata

        Returns:
            Registered Agent object
        """
        data = {
            "name": name,
            "publicKey": public_key,
            "capabilities": capabilities or [],
            "metadata": metadata or {},
        }

        response_data = self._request("POST", "/agents", json=data)
        return Agent.from_dict(response_data)

    def get_agent(self, agent_id: str) -> Agent:
        """
        Get agent details.

        Args:
            agent_id: Agent identifier

        Returns:
            Agent object
        """
        data = self._request("GET", f"/agents/{agent_id}")
        return Agent.from_dict(data)

    def deregister_agent(self, agent_id: str):
        """
        Deregister an agent.

        Args:
            agent_id: Agent identifier
        """
        self._request("DELETE", f"/agents/{agent_id}")

    def get_agent_status(self, agent_id: str) -> AgentStatus:
        """
        Get agent status and queue statistics.

        Args:
            agent_id: Agent identifier

        Returns:
            AgentStatus object
        """
        data = self._request("GET", f"/agents/{agent_id}/status")
        return AgentStatus.from_dict(data)

    def send_message(
        self,
        recipient: str,
        sender: str,
        message_type: str,
        payload: Dict[str, Any],
        priority: Optional[int] = None,
        correlation_id: Optional[str] = None,
        a2a_version: Optional[str] = None,
        headers: Optional[Dict[str, str]] = None,
        ttl: Optional[int] = None,
    ) -> Dict[str, Any]:
        """
        Send a message to an agent.

        Args:
            recipient: Recipient agent identifier
            sender: Sender agent identifier
            message_type: Message type (command, query, event, response, error)
            payload: Message content
            priority: Message priority (1-10, default: 5)
            correlation_id: Correlation ID for request/response patterns
            a2a_version: A2A protocol version (1.0, 1.1, 2.0)
            headers: Additional message headers
            ttl: Time-to-live in seconds

        Returns:
            Message acknowledgment
        """
        message = {
            "id": str(uuid.uuid4()),
            "sender": sender,
            "recipient": recipient,
            "type": message_type,
            "payload": payload,
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "priority": priority or 5,
            "correlationId": correlation_id,
            "a2aVersion": a2a_version or "1.0",
            "headers": headers or {},
            "ttl": ttl,
        }

        # Remove None values
        message = {k: v for k, v in message.items() if v is not None}

        return self._request("POST", f"/agents/{recipient}/messages", json=message)

    def get_messages(
        self,
        agent_id: str,
        limit: Optional[int] = None,
        since: Optional[str] = None,
        priority: Optional[str] = None,
    ) -> List[Message]:
        """
        Retrieve messages for an agent.

        Args:
            agent_id: Agent identifier
            limit: Maximum number of messages to return
            since: Only return messages since this timestamp
            priority: Filter messages by priority (critical, high, normal, low)

        Returns:
            List of Message objects
        """
        params = {}
        if limit:
            params["limit"] = limit
        if since:
            params["since"] = since
        if priority:
            params["priority"] = priority

        data = self._request("GET", f"/agents/{agent_id}/messages", params=params)
        return [Message.from_dict(msg_data) for msg_data in data]

    def get_message(self, agent_id: str, message_id: str) -> Message:
        """
        Get message details.

        Args:
            agent_id: Agent identifier
            message_id: Message identifier

        Returns:
            Message object
        """
        data = self._request("GET", f"/agents/{agent_id}/messages/{message_id}")
        return Message.from_dict(data)

    def delete_message(self, agent_id: str, message_id: str):
        """
        Delete a message.

        Args:
            agent_id: Agent identifier
            message_id: Message identifier
        """
        self._request("DELETE", f"/agents/{agent_id}/messages/{message_id}")

    def broadcast_message(
        self,
        sender: str,
        message_type: str,
        payload: Dict[str, Any],
        priority: Optional[int] = None,
        exclude_agents: Optional[List[str]] = None,
    ) -> Dict[str, Any]:
        """
        Broadcast message to all agents.

        Args:
            sender: Sender agent identifier
            message_type: Broadcast type (announcement, system, emergency)
            payload: Message content
            priority: Message priority (1-10, default: 5)
            exclude_agents: Agents to exclude from broadcast

        Returns:
            Broadcast acknowledgment
        """
        data = {
            "sender": sender,
            "type": message_type,
            "payload": payload,
            "priority": priority or 5,
            "excludeAgents": exclude_agents or [],
        }

        return self._request("POST", "/broadcast", json=data)

    def poll_for_response(
        self, correlation_id: str, timeout: int = 30, poll_interval: int = 1
    ) -> Optional[Message]:
        """
        Poll for a response message with specific correlation ID.

        Args:
            correlation_id: Correlation ID to wait for
            timeout: Maximum time to wait in seconds
            poll_interval: Time between polls in seconds

        Returns:
            Response Message if found, None if timeout
        """
        start_time = time.time()

        while time.time() - start_time < timeout:
            messages = self.get_messages(
                agent_id=self._get_agent_id_from_token(),
                since=datetime.utcfromtimestamp(start_time).isoformat() + "Z",
            )

            for message in messages:
                if (
                    message.correlation_id == correlation_id
                    and message.type == "response"
                ):
                    return message

            time.sleep(poll_interval)

        return None

    def _get_agent_id_from_token(self) -> str:
        """
        Extract agent ID from JWT token.

        Returns:
            Agent ID from token

        Raises:
            AuthenticationError: If token is not set or invalid
        """
        if not self.token:
            raise AuthenticationError("No authentication token set")

        # Simple extraction - in production, use proper JWT decoding
        # This is a simplified version for demonstration
        try:
            # Parse token (assuming format: token-{agent_id}-{uuid})
            parts = self.token.split("-")
            if len(parts) >= 2:
                return parts[1]
        except:
            pass

        raise AuthenticationError("Could not extract agent ID from token")

    def close(self):
        """Close the client session."""
        self.session.close()

    def __enter__(self):
        """Context manager entry."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()
