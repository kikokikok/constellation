"""
Constellation Client Exceptions
"""

from typing import Optional


class ConstellationError(Exception):
    """Base exception for Constellation client errors."""

    pass


class AuthenticationError(ConstellationError):
    """Authentication failed."""

    pass


class APIError(ConstellationError):
    """API request failed."""

    def __init__(
        self,
        message: str,
        status_code: Optional[int] = None,
        error_code: Optional[str] = None,
    ):
        super().__init__(message)
        self.status_code = status_code
        self.error_code = error_code


class ValidationError(ConstellationError):
    """Input validation failed."""

    pass


class ConnectionError(ConstellationError):
    """Connection to Constellation server failed."""

    pass


class TimeoutError(ConstellationError):
    """Request timeout."""

    pass


class RateLimitError(APIError):
    """Rate limit exceeded."""

    pass


class NotFoundError(APIError):
    """Resource not found."""

    pass


class ConflictError(APIError):
    """Resource conflict."""

    pass
