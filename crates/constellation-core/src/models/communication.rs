//! Communication patterns for agent-to-agent (A2A) messaging.
//!
//! This module provides structured communication patterns that build on top
//! of the basic A2A message broker, including:
//! - Request-response with timeouts and retries
//! - Publish-subscribe with topic-based routing
//! - Fire-and-forget notifications
//! - Delivery guarantees and idempotency

use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::models::message_broker::{A2AMessage, MessagePriority};

/// Communication pattern type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CommunicationPattern {
    /// Request-response: sender expects a response
    RequestResponse,
    /// Publish-subscribe: publisher sends to topic, subscribers receive
    PublishSubscribe,
    /// Fire-and-forget: sender doesn't expect a response
    FireAndForget,
    /// Broadcast: send to all connected agents
    Broadcast,
}

/// Delivery guarantee level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DeliveryGuarantee {
    /// Best effort - may be lost
    BestEffort,
    /// At least once - guaranteed delivery with possible duplicates
    AtLeastOnce,
    /// At most once - no duplicates but may be lost
    AtMostOnce,
    /// Exactly once - guaranteed exactly one delivery
    ExactlyOnce,
}

/// Request configuration for request-response pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestConfig {
    /// Timeout for response
    pub timeout: Duration,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Whether to use exponential backoff between retries
    pub use_exponential_backoff: bool,
    /// Base delay for retries (exponential backoff base)
    pub retry_base_delay: Duration,
    /// Whether response is required
    pub require_response: bool,
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_retries: 3,
            use_exponential_backoff: true,
            retry_base_delay: Duration::from_secs(1),
            require_response: true,
        }
    }
}

/// Response configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseConfig {
    /// Whether to include original request in response
    pub include_request: bool,
    /// Whether to validate response against request
    pub validate_response: bool,
    /// Maximum response size in bytes
    pub max_size_bytes: Option<usize>,
}

impl Default for ResponseConfig {
    fn default() -> Self {
        Self {
            include_request: true,
            validate_response: true,
            max_size_bytes: Some(10 * 1024 * 1024), // 10 MB default
        }
    }
}

/// Topic subscription pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TopicPattern {
    /// Exact topic match
    Exact(String),
    /// Wildcard pattern (e.g., "system.*")
    Wildcard(String),
    /// Regex pattern
    Regex(String),
}

impl TopicPattern {
    /// Check if a topic matches this pattern
    pub fn matches(&self, topic: &str) -> bool {
        match self {
            TopicPattern::Exact(pattern) => pattern == topic,
            TopicPattern::Wildcard(pattern) => {
                // Simple wildcard matching: * matches any sequence
                let pattern_parts: Vec<&str> = pattern.split('.').collect();
                let topic_parts: Vec<&str> = topic.split('.').collect();

                if pattern_parts.len() != topic_parts.len() {
                    return false;
                }

                pattern_parts
                    .iter()
                    .zip(topic_parts.iter())
                    .all(|(p, t)| *p == "*" || p == t)
            }
            TopicPattern::Regex(pattern) => regex::Regex::new(pattern)
                .map(|re| re.is_match(topic))
                .unwrap_or(false),
        }
    }
}

/// Subscription for publish-subscribe pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// Subscription ID
    pub id: String,
    /// Agent ID of subscriber
    pub agent_id: String,
    /// Topic pattern to subscribe to
    pub topic_pattern: TopicPattern,
    /// Whether subscription is active
    pub active: bool,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last message received timestamp
    pub last_message_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Subscription {
    /// Create a new subscription
    pub fn new(agent_id: String, topic_pattern: TopicPattern) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            agent_id,
            topic_pattern,
            active: true,
            created_at: chrono::Utc::now(),
            last_message_at: None,
        }
    }

    /// Update last message timestamp
    pub fn update_last_message(&mut self) {
        self.last_message_at = Some(chrono::Utc::now());
    }
}

/// Request message for request-response pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestMessage {
    /// Unique request ID
    pub request_id: String,
    /// Correlation ID (matches response)
    pub correlation_id: String,
    /// Sender agent ID
    pub sender_id: String,
    /// Recipient agent ID or topic
    pub recipient: String,
    /// Request payload (JSON string)
    pub payload: String,
    /// Request configuration
    pub config: RequestConfig,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Expiration timestamp
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Current retry count
    pub retry_count: u32,
    /// Priority level
    pub priority: MessagePriority,
}

impl RequestMessage {
    /// Create a new request message
    pub fn new(
        sender_id: String,
        recipient: String,
        payload: String,
        config: RequestConfig,
        priority: MessagePriority,
    ) -> Self {
        let request_id = Uuid::new_v4().to_string();
        let correlation_id = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now();
        let expires_at = created_at
            + chrono::Duration::from_std(config.timeout)
                .unwrap_or_else(|_| chrono::Duration::seconds(30));

        Self {
            request_id,
            correlation_id,
            sender_id,
            recipient,
            payload,
            config,
            created_at,
            expires_at,
            retry_count: 0,
            priority,
        }
    }

    /// Check if request has expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() >= self.expires_at
    }

    /// Check if request can be retried
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.config.max_retries && !self.is_expired()
    }

    /// Calculate next retry delay
    pub fn next_retry_delay(&self) -> Duration {
        if !self.config.use_exponential_backoff {
            return self.config.retry_base_delay;
        }

        // Exponential backoff: base_delay * 2^retry_count
        let multiplier = 2u32.pow(self.retry_count);
        self.config.retry_base_delay * multiplier
    }

    /// Convert to A2A message
    pub fn to_a2a_message(&self) -> A2AMessage {
        A2AMessage::new(
            self.request_id.clone(),
            self.sender_id.clone(),
            self.recipient.clone(),
            "request".to_string(),
            self.payload.clone(),
        )
        .with_correlation_id(Some(self.correlation_id.clone()))
        .with_priority(self.priority)
        .with_ttl(Some(self.config.timeout.as_secs() as i32))
    }
}

/// Response message for request-response pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    /// Response ID
    pub response_id: String,
    /// Correlation ID (matches request)
    pub correlation_id: String,
    /// Sender agent ID (responder)
    pub sender_id: String,
    /// Recipient agent ID (original requester)
    pub recipient_id: String,
    /// Response payload (JSON string)
    pub payload: String,
    /// Response status
    pub status: ResponseStatus,
    /// Response configuration
    pub config: ResponseConfig,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Original request (if included)
    pub original_request: Option<Box<RequestMessage>>,
    /// Priority level
    pub priority: MessagePriority,
    /// Time-to-live in seconds
    pub ttl_seconds: Option<u32>,
}

/// Response status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResponseStatus {
    /// Request was successful
    Success,
    /// Request failed
    Error(String),
    /// Request timed out
    Timeout,
    /// Request was rejected
    Rejected(String),
}

impl ResponseMessage {
    /// Create a new success response
    pub fn success(
        correlation_id: String,
        sender_id: String,
        recipient_id: String,
        payload: String,
        original_request: Option<RequestMessage>,
    ) -> Self {
        Self {
            response_id: Uuid::new_v4().to_string(),
            correlation_id,
            sender_id,
            recipient_id,
            payload,
            status: ResponseStatus::Success,
            config: ResponseConfig::default(),
            created_at: chrono::Utc::now(),
            original_request: original_request.map(Box::new),
            priority: MessagePriority::Normal,
            ttl_seconds: None,
        }
    }

    /// Create a new error response
    pub fn error(
        correlation_id: String,
        sender_id: String,
        recipient_id: String,
        error_message: String,
        original_request: Option<RequestMessage>,
    ) -> Self {
        Self {
            response_id: Uuid::new_v4().to_string(),
            correlation_id,
            sender_id,
            recipient_id,
            payload: error_message.clone(),
            status: ResponseStatus::Error(error_message),
            config: ResponseConfig::default(),
            created_at: chrono::Utc::now(),
            original_request: original_request.map(Box::new),
            priority: MessagePriority::Normal,
            ttl_seconds: None,
        }
    }

    /// Convert to A2A message
    pub fn to_a2a_message(&self) -> A2AMessage {
        A2AMessage::new(
            self.response_id.clone(),
            self.sender_id.clone(),
            self.recipient_id.clone(),
            "response".to_string(),
            self.payload.clone(),
        )
        .with_correlation_id(Some(self.correlation_id.clone()))
        .with_priority(self.priority)
        .with_ttl(Some(self.ttl_seconds.map(|t| t as i32).unwrap_or(0)))
    }
}

/// Notification message for fire-and-forget pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    /// Message ID
    pub message_id: String,
    /// Sender agent ID
    pub sender_id: String,
    /// Recipient agent ID or topic
    pub recipient: String,
    /// Message payload (JSON string)
    pub payload: String,
    /// Delivery guarantee
    pub delivery_guarantee: DeliveryGuarantee,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Priority level
    pub priority: MessagePriority,
}

impl NotificationMessage {
    /// Create a new notification message
    pub fn new(
        sender_id: String,
        recipient: String,
        payload: String,
        delivery_guarantee: DeliveryGuarantee,
        priority: MessagePriority,
    ) -> Self {
        Self {
            message_id: Uuid::new_v4().to_string(),
            sender_id,
            recipient,
            payload,
            delivery_guarantee,
            created_at: chrono::Utc::now(),
            priority,
        }
    }

    /// Convert to A2A message
    pub fn to_a2a_message(&self) -> A2AMessage {
        A2AMessage::new(
            self.message_id.clone(),
            self.sender_id.clone(),
            self.recipient.clone(),
            "notification".to_string(),
            self.payload.clone(),
        )
        .with_priority(self.priority)
    }
}

/// Publish message for publish-subscribe pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishMessage {
    /// Message ID
    pub message_id: String,
    /// Publisher agent ID
    pub publisher_id: String,
    /// Topic to publish to
    pub topic: String,
    /// Message payload (JSON string)
    pub payload: String,
    /// Delivery guarantee
    pub delivery_guarantee: DeliveryGuarantee,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Priority level
    pub priority: MessagePriority,
    /// Time-to-live in seconds
    pub ttl_seconds: Option<u32>,
}

impl PublishMessage {
    /// Create a new publish message
    pub fn new(
        publisher_id: String,
        topic: String,
        payload: String,
        delivery_guarantee: DeliveryGuarantee,
        priority: MessagePriority,
        ttl_seconds: Option<u32>,
    ) -> Self {
        Self {
            message_id: Uuid::new_v4().to_string(),
            publisher_id,
            topic,
            payload,
            delivery_guarantee,
            created_at: chrono::Utc::now(),
            priority,
            ttl_seconds,
        }
    }

    /// Convert to A2A message
    pub fn to_a2a_message(&self) -> A2AMessage {
        A2AMessage::new(
            self.message_id.clone(),
            self.publisher_id.clone(),
            self.topic.clone(), // Topic as recipient for routing
            "publish".to_string(),
            self.payload.clone(),
        )
        .with_priority(self.priority)
        .with_ttl(self.ttl_seconds.map(|t| t as i32))
    }
}

/// Communication pattern error
#[derive(Debug, thiserror::Error)]
pub enum CommunicationError {
    /// Request timeout
    #[error("Request timeout: {0}")]
    Timeout(String),

    /// Maximum retries exceeded
    #[error("Maximum retries exceeded: {0}")]
    MaxRetriesExceeded(String),

    /// Invalid response
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// Subscription error
    #[error("Subscription error: {0}")]
    SubscriptionError(String),

    /// Pattern not supported
    #[error("Pattern not supported: {0}")]
    PatternNotSupported(String),

    /// Message broker error
    #[error("Message broker error: {0}")]
    MessageBrokerError(#[from] crate::models::message_broker::MessageBrokerError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Result type for communication pattern operations
pub type CommunicationResult<T> = Result<T, CommunicationError>;
