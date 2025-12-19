"""
Constellation Data Models
"""

from typing import Dict, List, Optional, Any
from dataclasses import dataclass, asdict
from datetime import datetime


@dataclass
class Agent:
    """Agent model."""

    id: str
    name: str
    status: str
    registered_at: str
    last_seen: Optional[str] = None
    capabilities: Optional[List[str]] = None
    metadata: Optional[Dict[str, Any]] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Agent":
        """Create from dictionary."""
        return cls(**data)


@dataclass
class Message:
    """Message model."""

    id: str
    sender: str
    recipient: str
    type: str
    payload: Dict[str, Any]
    timestamp: str
    priority: Optional[int] = None
    correlation_id: Optional[str] = None
    a2a_version: Optional[str] = None
    headers: Optional[Dict[str, str]] = None
    ttl: Optional[int] = None
    retry_count: Optional[int] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Message":
        """Create from dictionary."""
        return cls(**data)


@dataclass
class HealthResponse:
    """Health check response."""

    status: str
    version: str
    uptime: int
    components: Dict[str, str]

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "HealthResponse":
        """Create from dictionary."""
        return cls(**data)


@dataclass
class TokenResponse:
    """Token response."""

    token: str
    expires_at: str
    agent_id: str

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "TokenResponse":
        """Create from dictionary."""
        return cls(**data)


@dataclass
class MessageAck:
    """Message acknowledgment."""

    message_id: str
    status: str
    timestamp: str
    queue_position: Optional[int] = None
    estimated_delivery: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "MessageAck":
        """Create from dictionary."""
        return cls(**data)


@dataclass
class BroadcastAck:
    """Broadcast acknowledgment."""

    broadcast_id: str
    recipients: int
    timestamp: str

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "BroadcastAck":
        """Create from dictionary."""
        return cls(**data)


@dataclass
class QueueStatistics:
    """Queue statistics."""

    total: int
    by_priority: Dict[str, int]
    oldest_message: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "QueueStatistics":
        """Create from dictionary."""
        return cls(**data)


@dataclass
class AgentStatus:
    """Agent status."""

    agent_id: str
    status: str
    queue_stats: QueueStatistics
    last_activity: Optional[str] = None
    session_id: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "AgentStatus":
        """Create from dictionary."""
        queue_stats = QueueStatistics.from_dict(data["queueStats"])
        return cls(
            agent_id=data["agentId"],
            status=data["status"],
            queue_stats=queue_stats,
            last_activity=data.get("lastActivity"),
            session_id=data.get("sessionId"),
        )
