"""
Constellation Python Client SDK

A Python client for interacting with the Constellation A2A Message Broker API.
"""

__version__ = "0.1.0"
__author__ = "Constellation Team"
__email__ = "team@constellation.example.com"

from .client import ConstellationClient
from .exceptions import ConstellationError, AuthenticationError, APIError
from .models import Agent, Message, HealthResponse, TokenResponse

__all__ = [
    "ConstellationClient",
    "ConstellationError",
    "AuthenticationError",
    "APIError",
    "Agent",
    "Message",
    "HealthResponse",
    "TokenResponse",
]
