use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

/// Message priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Default)]
#[sqlx(type_name = "message_priority", rename_all = "lowercase")]
pub enum MessagePriority {
    #[default]
    Normal,
    Low,
    High,
    Critical,
}

/// Message delivery status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Default)]
#[sqlx(type_name = "delivery_status", rename_all = "lowercase")]
pub enum DeliveryStatus {
    #[default]
    Pending,
    Queued,
    Delivering,
    Delivered,
    Failed,
    DeadLetter,
}

/// Agent session status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Default)]
#[sqlx(type_name = "session_status", rename_all = "lowercase")]
pub enum SessionStatus {
    #[default]
    Connected,
    Disconnected,
    Reconnecting,
}

/// A2A message for storage in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    /// Primary identifier
    pub id: Uuid,

    /// Message metadata
    pub message_id: String,
    pub correlation_id: Option<String>,
    pub conversation_id: Option<String>,

    /// Sender and recipient
    pub sender_id: String,
    pub recipient_id: String,

    /// Message content
    pub message_type: String,
    pub protocol_version: String,
    pub content_type: String,
    pub payload: String,
    pub metadata: Option<serde_json::Value>,

    /// Delivery properties
    pub priority: MessagePriority,
    pub delivery_guarantee: String,
    pub ttl_seconds: Option<i32>,
    pub max_retries: i32,

    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Message {
    /// Create a new message with default values
    pub fn new(
        message_id: String,
        sender_id: String,
        recipient_id: String,
        message_type: String,
        payload: String,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            message_id,
            correlation_id: None,
            conversation_id: None,
            sender_id,
            recipient_id,
            message_type,
            protocol_version: "1.0".to_string(),
            content_type: "application/json".to_string(),
            payload,
            metadata: None,
            priority: MessagePriority::Normal,
            delivery_guarantee: "at-least-once".to_string(),
            ttl_seconds: None,
            max_retries: 3,
            created_at: now,
            scheduled_for: None,
            expires_at: None,
        }
    }

    /// Set correlation ID for request-response pattern
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Set conversation ID for multi-message conversations
    pub fn with_conversation_id(mut self, conversation_id: String) -> Self {
        self.conversation_id = Some(conversation_id);
        self
    }

    /// Set message priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set time-to-live in seconds
    pub fn with_ttl(mut self, ttl_seconds: i32) -> Self {
        self.ttl_seconds = Some(ttl_seconds);
        self.expires_at = Some(self.created_at + chrono::Duration::seconds(ttl_seconds as i64));
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Check if message has expired
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires_at) => Utc::now() > expires_at,
            None => false,
        }
    }
}

/// Queue entry for priority-based message queuing
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QueueEntry {
    pub id: Uuid,
    pub message_id: Uuid,
    pub queue_name: String,
    pub priority: MessagePriority,
    pub sequence_number: i64,
    pub enqueued_at: DateTime<Utc>,
    pub dequeued_at: Option<DateTime<Utc>>,
}

impl QueueEntry {
    /// Create a new queue entry
    pub fn new(message_id: Uuid, queue_name: String, priority: MessagePriority) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_id,
            queue_name,
            priority,
            sequence_number: 0, // Will be set by database sequence
            enqueued_at: Utc::now(),
            dequeued_at: None,
        }
    }

    /// Check if entry is still in queue (not dequeued)
    pub fn is_in_queue(&self) -> bool {
        self.dequeued_at.is_none()
    }
}

/// Delivery status tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeliveryStatusEntry {
    pub id: Uuid,
    pub message_id: Uuid,
    pub status: DeliveryStatus,
    pub current_retry: i32,
    pub last_delivery_attempt: Option<DateTime<Utc>>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledgment_payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DeliveryStatusEntry {
    /// Create a new delivery status entry
    pub fn new(message_id: Uuid) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            message_id,
            status: DeliveryStatus::Pending,
            current_retry: 0,
            last_delivery_attempt: None,
            next_retry_at: None,
            delivered_at: None,
            failed_at: None,
            failure_reason: None,
            acknowledged_at: None,
            acknowledgment_payload: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Mark as delivering
    pub fn mark_delivering(&mut self) {
        self.status = DeliveryStatus::Delivering;
        self.last_delivery_attempt = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark as delivered
    pub fn mark_delivered(&mut self, acknowledgment: Option<serde_json::Value>) {
        self.status = DeliveryStatus::Delivered;
        self.delivered_at = Some(Utc::now());
        self.acknowledged_at = Some(Utc::now());
        self.acknowledgment_payload = acknowledgment;
        self.updated_at = Utc::now();
    }

    /// Mark as failed with retry logic
    pub fn mark_failed(&mut self, reason: String, max_retries: i32) -> bool {
        self.current_retry += 1;
        self.status = DeliveryStatus::Failed;
        self.failed_at = Some(Utc::now());
        self.failure_reason = Some(reason);
        self.updated_at = Utc::now();

        if self.current_retry >= max_retries {
            self.status = DeliveryStatus::DeadLetter;
            false // No more retries
        } else {
            // Exponential backoff: 2^retry * base_delay (seconds)
            let base_delay = 5; // 5 seconds base delay
            let delay_seconds = 2_i32.pow(self.current_retry as u32) * base_delay;
            self.next_retry_at = Some(Utc::now() + chrono::Duration::seconds(delay_seconds as i64));
            true // Will retry
        }
    }

    /// Check if ready for retry
    pub fn is_ready_for_retry(&self) -> bool {
        match (self.status, self.next_retry_at) {
            (DeliveryStatus::Failed, Some(next_retry_at)) => Utc::now() >= next_retry_at,
            _ => false,
        }
    }
}

/// Agent session for connection management
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentSession {
    pub id: Uuid,
    pub agent_id: String,
    pub session_token: String,
    pub connection_id: Option<String>,
    pub protocol_binding: String,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub status: SessionStatus,
    pub capabilities: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl AgentSession {
    /// Create a new agent session
    pub fn new(
        agent_id: String,
        session_token: String,
        protocol_binding: String,
        capabilities: Option<serde_json::Value>,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            agent_id,
            session_token,
            connection_id: None,
            protocol_binding,
            client_ip: None,
            user_agent: None,
            status: SessionStatus::Connected,
            capabilities,
            created_at: now,
            last_activity_at: now,
            expires_at: now + chrono::Duration::hours(1),
        }
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity_at = Utc::now();
        // Extend expiration on activity
        self.expires_at = self.last_activity_at + chrono::Duration::hours(1);
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if session is active (not expired and connected)
    pub fn is_active(&self) -> bool {
        !self.is_expired() && self.status == SessionStatus::Connected
    }
}

/// Dead letter queue entry for failed messages
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeadLetterEntry {
    pub id: Uuid,
    pub message_id: Uuid,
    pub original_queue: String,
    pub failure_reason: String,
    pub failure_details: Option<serde_json::Value>,
    pub failed_at: DateTime<Utc>,
    pub retry_count: i32,
    pub last_retry_attempt: Option<DateTime<Utc>>,
}

/// Routing rule for advanced message routing
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RoutingRule {
    pub id: Uuid,
    pub rule_name: String,
    pub match_pattern: serde_json::Value,
    pub priority: i32,
    pub target_queue: String,
    pub transform_script: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Message for sending via A2A protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AMessage {
    /// Message ID (must be unique)
    pub message_id: String,

    /// Correlation ID for request-response
    pub correlation_id: Option<String>,

    /// Conversation ID for multi-message conversations
    pub conversation_id: Option<String>,

    /// Sender agent ID
    pub sender_id: String,

    /// Recipient agent ID (or broadcast address)
    pub recipient_id: String,

    /// Message type: request, response, notification, etc.
    pub message_type: String,

    /// A2A protocol version
    pub protocol_version: String,

    /// Content type (e.g., application/json)
    pub content_type: String,

    /// Message payload (JSON string)
    pub payload: String,

    /// Additional metadata
    pub metadata: Option<serde_json::Value>,

    /// Message priority
    pub priority: MessagePriority,

    /// Delivery guarantee
    pub delivery_guarantee: String,

    /// Time-to-live in seconds
    pub ttl_seconds: Option<i32>,
}

impl A2AMessage {
    /// Create a new A2A message
    pub fn new(
        message_id: String,
        sender_id: String,
        recipient_id: String,
        message_type: String,
        payload: String,
    ) -> Self {
        Self {
            message_id,
            correlation_id: None,
            conversation_id: None,
            sender_id,
            recipient_id,
            message_type,
            protocol_version: "1.0".to_string(),
            content_type: "application/json".to_string(),
            payload,
            metadata: None,
            priority: MessagePriority::Normal,
            delivery_guarantee: "at-least-once".to_string(),
            ttl_seconds: None,
        }
    }

    /// Convert to database Message
    pub fn to_message(&self) -> Message {
        Message::new(
            self.message_id.clone(),
            self.sender_id.clone(),
            self.recipient_id.clone(),
            self.message_type.clone(),
            self.payload.clone(),
        )
        .with_correlation_id(self.correlation_id.clone().unwrap_or_default())
        .with_conversation_id(self.conversation_id.clone().unwrap_or_default())
        .with_priority(self.priority)
        .with_ttl(self.ttl_seconds.unwrap_or(0))
        .with_metadata(self.metadata.clone().unwrap_or(serde_json::json!({})))
    }
}

/// Message acknowledgment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAcknowledgment {
    pub message_id: String,
    pub acknowledged_at: DateTime<Utc>,
    pub status: String, // "delivered", "processed", "failed"
    pub details: Option<serde_json::Value>,
}

/// Queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub queue_name: String,
    pub total_messages: i64,
    pub pending_messages: i64,
    pub delivered_messages: i64,
    pub failed_messages: i64,
    pub avg_delivery_time_ms: Option<f64>,
    pub last_updated: DateTime<Utc>,
}

/// Agent connection request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConnectionRequest {
    pub agent_id: String,
    pub protocol_binding: String,
    pub capabilities: Option<serde_json::Value>,
    pub auth_token: Option<String>,
}

/// Agent connection response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConnectionResponse {
    pub session_token: String,
    pub expires_at: DateTime<Utc>,
    pub message_broker_url: String,
    pub supported_protocols: Vec<String>,
}

/// Error types for message broker operations
#[derive(Debug, thiserror::Error)]
pub enum MessageBrokerError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Message not found: {0}")]
    MessageNotFound(String),

    #[error("Queue not found: {0}")]
    QueueNotFound(String),

    #[error("Queue full: {0}")]
    QueueFull(String),

    #[error("Agent not connected: {0}")]
    AgentNotConnected(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    #[error("Message expired: {0}")]
    MessageExpired(String),

    #[error("Delivery failed: {0}")]
    DeliveryFailed(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
}

/// Result type for message broker operations
pub type MessageBrokerResult<T> = Result<T, MessageBrokerError>;
