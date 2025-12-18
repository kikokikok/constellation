use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use tracing::{debug, error, info, warn};

use crate::models::message_broker::{
    A2AMessage, AgentSession, DeadLetterEntry, DeliveryStatusEntry, Message, MessageBrokerError,
    MessageBrokerResult, MessagePriority, QueueEntry, QueueStats, RoutingRule, SessionStatus,
};

/// Database layer for the message broker.
///
/// Handles all PostgreSQL operations for:
/// - Message storage and retrieval
/// - Queue management
/// - Delivery status tracking
/// - Agent session management
/// - Dead letter queue
#[derive(Clone)]
pub struct MessageBrokerDatabase {
    pool: PgPool,
}

impl MessageBrokerDatabase {
    /// Create a new database instance with connection pool.
    pub async fn new(database_url: &str) -> MessageBrokerResult<Self> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(MessageBrokerError::DatabaseError)?;
        
        info!("Connected to message broker database");
        Ok(Self { pool })
    }
    
    /// Get a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
    
    /// Begin a database transaction.
    pub async fn begin_transaction(&self) -> MessageBrokerResult<Transaction<'_, Postgres>> {
        self.pool
            .begin()
            .await
            .map_err(MessageBrokerError::DatabaseError)
    }
}

#[async_trait]
pub trait MessageStore: Send + Sync {
    /// Store a new message in the database.
    async fn store_message(&self, message: &Message) -> MessageBrokerResult<()>;
    
    /// Retrieve a message by ID.
    async fn get_message(&self, message_id: &str) -> MessageBrokerResult<Option<Message>>;
    
    /// Retrieve messages by correlation ID.
    async fn get_messages_by_correlation_id(
        &self,
        correlation_id: &str,
    ) -> MessageBrokerResult<Vec<Message>>;
    
    /// Retrieve messages by conversation ID.
    async fn get_messages_by_conversation_id(
        &self,
        conversation_id: &str,
    ) -> MessageBrokerResult<Vec<Message>>;
    
    /// Retrieve undelivered messages for a recipient.
    async fn get_undelivered_messages(
        &self,
        recipient_id: &str,
        limit: i64,
    ) -> MessageBrokerResult<Vec<Message>>;
    
    /// Delete expired messages.
    async fn delete_expired_messages(&self) -> MessageBrokerResult<u64>;
}

#[async_trait]
pub trait QueueManager: Send + Sync {
    /// Enqueue a message with priority.
    async fn enqueue_message(
        &self,
        message_id: uuid::Uuid,
        queue_name: &str,
        priority: MessagePriority,
    ) -> MessageBrokerResult<QueueEntry>;
    
    /// Dequeue the next message from a queue.
    async fn dequeue_message(&self, queue_name: &str) -> MessageBrokerResult<Option<(Message, QueueEntry)>>;
    
    /// Get queue statistics.
    async fn get_queue_stats(&self, queue_name: &str) -> MessageBrokerResult<QueueStats>;
    
    /// Get all queue names.
    async fn get_queues(&self) -> MessageBrokerResult<Vec<String>>;
    
    /// Purge a queue (remove all messages).
    async fn purge_queue(&self, queue_name: &str) -> MessageBrokerResult<u64>;
}

#[async_trait]
pub trait DeliveryTracker: Send + Sync {
    /// Create delivery status for a message.
    async fn create_delivery_status(
        &self,
        message_id: uuid::Uuid,
    ) -> MessageBrokerResult<DeliveryStatusEntry>;
    
    /// Update delivery status.
    async fn update_delivery_status(
        &self,
        delivery_status: &DeliveryStatusEntry,
    ) -> MessageBrokerResult<()>;
    
    /// Get delivery status by message ID.
    async fn get_delivery_status(
        &self,
        message_id: uuid::Uuid,
    ) -> MessageBrokerResult<Option<DeliveryStatusEntry>>;
    
    /// Get messages ready for retry.
    async fn get_messages_for_retry(&self, limit: i64) -> MessageBrokerResult<Vec<Message>>;
    
    /// Acknowledge message delivery.
    async fn acknowledge_delivery(
        &self,
        message_id: uuid::Uuid,
        acknowledgment: serde_json::Value,
    ) -> MessageBrokerResult<()>;
}

#[async_trait]
pub trait SessionManager: Send + Sync {
    /// Create or update agent session.
    async fn create_session(&self, session: &AgentSession) -> MessageBrokerResult<()>;
    
    /// Get session by token.
    async fn get_session(&self, session_token: &str) -> MessageBrokerResult<Option<AgentSession>>;
    
    /// Get active sessions for an agent.
    async fn get_agent_sessions(&self, agent_id: &str) -> MessageBrokerResult<Vec<AgentSession>>;
    
    /// Update session activity.
    async fn update_session_activity(&self, session_token: &str) -> MessageBrokerResult<()>;
    
    /// Update session status.
    async fn update_session_status(
        &self,
        session_token: &str,
        status: SessionStatus,
    ) -> MessageBrokerResult<()>;
    
    /// Delete expired sessions.
    async fn delete_expired_sessions(&self) -> MessageBrokerResult<u64>;
    
    /// Disconnect all sessions for an agent.
    async fn disconnect_agent(&self, agent_id: &str) -> MessageBrokerResult<u64>;
}

#[async_trait]
pub trait DeadLetterManager: Send + Sync {
    /// Move message to dead letter queue.
    async fn move_to_dead_letter(
        &self,
        message_id: uuid::Uuid,
        queue_name: &str,
        reason: &str,
        details: Option<serde_json::Value>,
    ) -> MessageBrokerResult<DeadLetterEntry>;
    
    /// Get dead letter entries.
    async fn get_dead_letter_entries(
        &self,
        limit: i64,
        offset: i64,
    ) -> MessageBrokerResult<Vec<DeadLetterEntry>>;
    
    /// Retry dead letter entry.
    async fn retry_dead_letter_entry(&self, entry_id: uuid::Uuid) -> MessageBrokerResult<()>;
    
    /// Delete dead letter entry.
    async fn delete_dead_letter_entry(&self, entry_id: uuid::Uuid) -> MessageBrokerResult<()>;
}

#[async_trait]
pub trait RoutingManager: Send + Sync {
    /// Create routing rule.
    async fn create_routing_rule(&self, rule: &RoutingRule) -> MessageBrokerResult<()>;
    
    /// Get routing rules.
    async fn get_routing_rules(&self) -> MessageBrokerResult<Vec<RoutingRule>>;
    
    /// Update routing rule.
    async fn update_routing_rule(&self, rule: &RoutingRule) -> MessageBrokerResult<()>;
    
    /// Delete routing rule.
    async fn delete_routing_rule(&self, rule_id: uuid::Uuid) -> MessageBrokerResult<()>;
    
    /// Apply routing rules to a message.
    async fn route_message(&self, message: &A2AMessage) -> MessageBrokerResult<Vec<String>>;
}

impl MessageBrokerDatabase {
    /// Store a complete message with all related data in a transaction.
    pub async fn store_complete_message(
        &self,
        a2a_message: &A2AMessage,
        queue_name: &str,
    ) -> MessageBrokerResult<()> {
        let mut transaction = self.begin_transaction().await?;
        
        // Convert A2A message to database message
        let message = a2a_message.to_message();
        
        // Store message
        sqlx::query(
            r#"
            INSERT INTO messages (
                id, message_id, correlation_id, conversation_id,
                sender_id, recipient_id, message_type, protocol_version,
                content_type, payload, metadata, priority, delivery_guarantee,
                ttl_seconds, max_retries, created_at, scheduled_for, expires_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
            )
            "#,
            message.id,
            message.message_id,
            message.correlation_id,
            message.conversation_id,
            message.sender_id,
            message.recipient_id,
            message.message_type,
            message.protocol_version,
            message.content_type,
            message.payload,
            message.metadata,
            message.priority as MessagePriority,
            message.delivery_guarantee,
            message.ttl_seconds,
            message.max_retries,
            message.created_at,
            message.scheduled_for,
            message.expires_at,
        )
        .execute(&mut *transaction)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        // Enqueue message
        sqlx::query(
            r#"
            INSERT INTO queues (message_id, queue_name, priority)
            VALUES ($1, $2, $3)
            "#,
            message.id,
            queue_name,
            message.priority as MessagePriority,
        )
        .execute(&mut *transaction)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        // Create delivery status
        let delivery_status = DeliveryStatusEntry::new(message.id);
        sqlx::query(
            r#"
            INSERT INTO delivery_status (
                id, message_id, status, current_retry, last_delivery_attempt,
                next_retry_at, delivered_at, failed_at, failure_reason,
                acknowledged_at, acknowledgment_payload, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13
            )
            "#,
            delivery_status.id,
            delivery_status.message_id,
            delivery_status.status as DeliveryStatus,
            delivery_status.current_retry,
            delivery_status.last_delivery_attempt,
            delivery_status.next_retry_at,
            delivery_status.delivered_at,
            delivery_status.failed_at,
            delivery_status.failure_reason,
            delivery_status.acknowledged_at,
            delivery_status.acknowledgment_payload,
            delivery_status.created_at,
            delivery_status.updated_at,
        )
        .execute(&mut *transaction)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        transaction
            .commit()
            .await
            .map_err(MessageBrokerError::DatabaseError)?;
        
        debug!("Stored complete message: {}", a2a_message.message_id);
        Ok(())
    }
    
    /// Get next message from queue using database function.
    pub async fn get_next_queue_message(
        &self,
        queue_name: &str,
    ) -> MessageBrokerResult<Option<(Message, QueueEntry)>> {
        let result = sqlx::query(
            r#"
            SELECT * FROM get_next_queue_message($1, 1)
            "#,
            queue_name,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        match result {
            Some(row) => {
                // Get the full message
                let message = sqlx::query_as::<_, 
                    Message,
                    r#"
                    SELECT * FROM messages WHERE id = $1
                    "#,
                    row.message_id,
                )
                .fetch_one(&self.pool)
                .await
                .map_err(MessageBrokerError::DatabaseError)?;
                
                // Get the queue entry
                let queue_entry = sqlx::query_as::<_, 
                    QueueEntry,
                    r#"
                    SELECT * FROM queues WHERE id = $1
                    "#,
                    row.queue_id,
                )
                .fetch_one(&self.pool)
                .await
                .map_err(MessageBrokerError::DatabaseError)?;
                
                Ok(Some((message, queue_entry)))
            }
            None => Ok(None),
        }
    }
    
    /// Get dashboard statistics.
    pub async fn get_dashboard_stats(&self) -> MessageBrokerResult<serde_json::Value> {
        // Total messages
        let total_messages: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM messages")
            .fetch_one(&self.pool)
            .await
            .map_err(MessageBrokerError::DatabaseError)?;
        
        // Pending messages
        let pending_messages: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM delivery_status WHERE status IN ('pending', 'queued', 'delivering')
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        // Delivered messages
        let delivered_messages: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM delivery_status WHERE status = 'delivered'"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        // Failed messages
        let failed_messages: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM delivery_status WHERE status = 'failed'"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        // Active sessions
        let active_sessions: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM agent_sessions 
            WHERE expires_at > CURRENT_TIMESTAMP AND status = 'connected'
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        // Queue sizes
        let queue_sizes = sqlx::query(
            r#"
            SELECT queue_name, COUNT(*) as count
            FROM queues 
            WHERE dequeued_at IS NULL
            GROUP BY queue_name
            ORDER BY queue_name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(serde_json::json!({
            "total_messages": total_messages,
            "pending_messages": pending_messages,
            "delivered_messages": delivered_messages,
            "failed_messages": failed_messages,
            "active_sessions": active_sessions,
            "queues": queue_sizes.into_iter().map(|q| {
                serde_json::json!({
                    "name": q.queue_name,
                    "size": q.count.unwrap_or(0)
                })
            }).collect::<Vec<_>>(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
    }
    
    /// Run maintenance tasks (expire messages, clean sessions, etc.)
    pub async fn run_maintenance(&self) -> MessageBrokerResult<()> {
        info!("Running message broker maintenance tasks");
        
        // Expire old messages
        let expired = sqlx::query("SELECT expire_old_messages()")
            .execute(&self.pool)
            .await
            .map_err(MessageBrokerError::DatabaseError)?
            .rows_affected();
        
        if expired > 0 {
            debug!("Expired {} messages", expired);
        }
        
        // Delete expired sessions
        let deleted_sessions = sqlx::query(
            r#"
            DELETE FROM agent_sessions 
            WHERE expires_at < CURRENT_TIMESTAMP
            RETURNING id
            "#
        )
        .fetch_all(&self.pool)
        .await
            .map_err(MessageBrokerError::DatabaseError)?
            .len() as u64;
        
        if deleted_sessions > 0 {
            debug!("Deleted {} expired sessions", deleted_sessions);
        }
        
        // Update idle sessions
        let idle_threshold = chrono::Utc::now() - chrono::Duration::minutes(5);
        let idle_sessions = sqlx::query(
            r#"
            UPDATE agent_sessions 
            SET status = 'idle'::session_status
            WHERE status = 'connected'::session_status 
                AND last_activity_at < $1
            RETURNING id
            "#,
            idle_threshold,
        )
        .fetch_all(&self.pool)
        .await
            .map_err(MessageBrokerError::DatabaseError)?
            .len() as u64;
        
        if idle_sessions > 0 {
            debug!("Marked {} sessions as idle", idle_sessions);
        }
        
        Ok(())
    }
}

// Implement traits for MessageBrokerDatabase
#[async_trait]
impl MessageStore for MessageBrokerDatabase {
    async fn store_message(&self, message: &Message) -> MessageBrokerResult<()> {
        sqlx::query(
            r#"
            INSERT INTO messages (
                id, message_id, correlation_id, conversation_id,
                sender_id, recipient_id, message_type, protocol_version,
                content_type, payload, metadata, priority, delivery_guarantee,
                ttl_seconds, max_retries, created_at, scheduled_for, expires_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
            )
            "#,
            message.id,
            message.message_id,
            message.correlation_id,
            message.conversation_id,
            message.sender_id,
            message.recipient_id,
            message.message_type,
            message.protocol_version,
            message.content_type,
            message.payload,
            message.metadata,
            message.priority as MessagePriority,
            message.delivery_guarantee,
            message.ttl_seconds,
            message.max_retries,
            message.created_at,
            message.scheduled_for,
            message.expires_at,
        )
        .execute(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(())
    }
    
    async fn get_message(&self, message_id: &str) -> MessageBrokerResult<Option<Message>> {
        let message = sqlx::query_as::<_, 
            Message,
            r#"
            SELECT * FROM messages WHERE message_id = $1
            "#,
            message_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(message)
    }
    
    async fn get_messages_by_correlation_id(
        &self,
        correlation_id: &str,
    ) -> MessageBrokerResult<Vec<Message>> {
        let messages = sqlx::query_as::<_, 
            Message,
            r#"
            SELECT * FROM messages WHERE correlation_id = $1 ORDER BY created_at
            "#,
            correlation_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(messages)
    }
    
    async fn get_messages_by_conversation_id(
        &self,
        conversation_id: &str,
    ) -> MessageBrokerResult<Vec<Message>> {
        let messages = sqlx::query_as::<_, 
            Message,
            r#"
            SELECT * FROM messages WHERE conversation_id = $1 ORDER BY created_at
            "#,
            conversation_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(messages)
    }
    
    async fn get_undelivered_messages(
        &self,
        recipient_id: &str,
        limit: i64,
    ) -> MessageBrokerResult<Vec<Message>> {
        let messages = sqlx::query_as::<_, 
            Message,
            r#"
            SELECT m.* 
            FROM messages m
            JOIN delivery_status ds ON m.id = ds.message_id
            WHERE m.recipient_id = $1 
                AND ds.status IN ('pending', 'queued', 'delivering', 'failed')
                AND (m.expires_at IS NULL OR m.expires_at > CURRENT_TIMESTAMP)
            ORDER BY m.priority, m.created_at
            LIMIT $2
            "#,
            recipient_id,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(messages)
    }
    
    async fn delete_expired_messages(&self) -> MessageBrokerResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM messages 
            WHERE expires_at IS NOT NULL AND expires_at < CURRENT_TIMESTAMP
            RETURNING id
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(result.len() as u64)
    }
}

#[async_trait]
impl QueueManager for MessageBrokerDatabase {
    async fn enqueue_message(
        &self,
        message_id: uuid::Uuid,
        queue_name: &str,
        priority: MessagePriority,
    ) -> MessageBrokerResult<QueueEntry> {
        let entry = sqlx::query_as::<_, 
            QueueEntry,
            r#"
            INSERT INTO queues (message_id, queue_name, priority)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            message_id,
            queue_name,
            priority as MessagePriority,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(entry)
    }
    
    async fn dequeue_message(&self, queue_name: &str) -> MessageBrokerResult<Option<(Message, QueueEntry)>> {
        self.get_next_queue_message(queue_name).await
    }
    
    async fn get_queue_stats(&self, queue_name: &str) -> MessageBrokerResult<QueueStats> {
        let stats = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_messages,
                COUNT(CASE WHEN q.dequeued_at IS NULL THEN 1 END) as pending_messages,
                COUNT(CASE WHEN ds.status = 'delivered' THEN 1 END) as delivered_messages,
                COUNT(CASE WHEN ds.status = 'failed' THEN 1 END) as failed_messages,
                AVG(EXTRACT(EPOCH FROM (ds.delivered_at - m.created_at)) * 1000) as avg_delivery_time_ms
            FROM queues q
            JOIN messages m ON q.message_id = m.id
            LEFT JOIN delivery_status ds ON m.id = ds.message_id
            WHERE q.queue_name = $1
            GROUP BY q.queue_name
            "#,
            queue_name,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        match stats {
            Some(row) => Ok(QueueStats {
                queue_name: queue_name.to_string(),
                total_messages: row.total_messages.unwrap_or(0) as i64,
                pending_messages: row.pending_messages.unwrap_or(0) as i64,
                delivered_messages: row.delivered_messages.unwrap_or(0) as i64,
                failed_messages: row.failed_messages.unwrap_or(0) as i64,
                avg_delivery_time_ms: row.avg_delivery_time_ms,
                last_updated: chrono::Utc::now(),
            }),
            None => Ok(QueueStats {
                queue_name: queue_name.to_string(),
                total_messages: 0,
                pending_messages: 0,
                delivered_messages: 0,
                failed_messages: 0,
                avg_delivery_time_ms: None,
                last_updated: chrono::Utc::now(),
            }),
        }
    }
    
    async fn get_queues(&self) -> MessageBrokerResult<Vec<String>> {
        let queues = sqlx::query(
            r#"
            SELECT DISTINCT queue_name FROM queues ORDER BY queue_name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(queues.into_iter().map(|q| q.queue_name).collect())
    }
    
    async fn purge_queue(&self, queue_name: &str) -> MessageBrokerResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM queues WHERE queue_name = $1 RETURNING id
            "#,
            queue_name,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(result.len() as u64)
    }
}

#[async_trait]
impl DeliveryTracker for MessageBrokerDatabase {
    async fn create_delivery_status(
        &self,
        message_id: uuid::Uuid,
    ) -> MessageBrokerResult<DeliveryStatusEntry> {
        let status = DeliveryStatusEntry::new(message_id);
        
        sqlx::query(
            r#"
            INSERT INTO delivery_status (
                id, message_id, status, current_retry, last_delivery_attempt,
                next_retry_at, delivered_at, failed_at, failure_reason,
                acknowledged_at, acknowledgment_payload, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13
            )
            "#,
            status.id,
            status.message_id,
            status.status as DeliveryStatus,
            status.current_retry,
            status.last_delivery_attempt,
            status.next_retry_at,
            status.delivered_at,
            status.failed_at,
            status.failure_reason,
            status.acknowledged_at,
            status.acknowledgment_payload,
            status.created_at,
            status.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(status)
    }
    
    async fn update_delivery_status(
        &self,
        delivery_status: &DeliveryStatusEntry,
    ) -> MessageBrokerResult<()> {
        sqlx::query(
            r#"
            UPDATE delivery_status SET
                status = $1,
                current_retry = $2,
                last_delivery_attempt = $3,
                next_retry_at = $4,
                delivered_at = $5,
                failed_at = $6,
                failure_reason = $7,
                acknowledged_at = $8,
                acknowledgment_payload = $9,
                updated_at = $10
            WHERE id = $11
            "#,
            delivery_status.status as DeliveryStatus,
            delivery_status.current_retry,
            delivery_status.last_delivery_attempt,
            delivery_status.next_retry_at,
            delivery_status.delivered_at,
            delivery_status.failed_at,
            delivery_status.failure_reason,
            delivery_status.acknowledged_at,
            delivery_status.acknowledgment_payload,
            delivery_status.updated_at,
            delivery_status.id,
        )
        .execute(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(())
    }
    
    async fn get_delivery_status(
        &self,
        message_id: uuid::Uuid,
    ) -> MessageBrokerResult<Option<DeliveryStatusEntry>> {
        let status = sqlx::query_as::<_, 
            DeliveryStatusEntry,
            r#"
            SELECT * FROM delivery_status WHERE message_id = $1
            "#,
            message_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(status)
    }
    
    async fn get_messages_for_retry(&self, limit: i64) -> MessageBrokerResult<Vec<Message>> {
        let messages = sqlx::query_as::<_, 
            Message,
            r#"
            SELECT m.*
            FROM messages m
            JOIN delivery_status ds ON m.id = ds.message_id
            WHERE ds.status = 'failed'::delivery_status
                AND ds.next_retry_at <= CURRENT_TIMESTAMP
                AND (m.expires_at IS NULL OR m.expires_at > CURRENT_TIMESTAMP)
                AND ds.current_retry < m.max_retries
            ORDER BY ds.next_retry_at
            LIMIT $1
            "#,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(messages)
    }
    
    async fn acknowledge_delivery(
        &self,
        message_id: uuid::Uuid,
        acknowledgment: serde_json::Value,
    ) -> MessageBrokerResult<()> {
        sqlx::query(
            r#"
            UPDATE delivery_status SET
                status = 'delivered'::delivery_status,
                acknowledged_at = CURRENT_TIMESTAMP,
                acknowledgment_payload = $1,
                updated_at = CURRENT_TIMESTAMP
            WHERE message_id = $2
            "#,
            acknowledgment,
            message_id,
        )
        .execute(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(())
    }
}

#[async_trait]
impl SessionManager for MessageBrokerDatabase {
    async fn create_session(&self, session: &AgentSession) -> MessageBrokerResult<()> {
        sqlx::query(
            r#"
            INSERT INTO agent_sessions (
                id, agent_id, session_token, connection_id, protocol_binding,
                client_ip, user_agent, status, capabilities, created_at,
                last_activity_at, expires_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12
            )
            "#,
            session.id,
            session.agent_id,
            session.session_token,
            session.connection_id,
            session.protocol_binding,
            session.client_ip,
            session.user_agent,
            session.status as SessionStatus,
            session.capabilities,
            session.created_at,
            session.last_activity_at,
            session.expires_at,
        )
        .execute(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(())
    }
    
    async fn get_session(&self, session_token: &str) -> MessageBrokerResult<Option<AgentSession>> {
        let session = sqlx::query_as::<_, 
            AgentSession,
            r#"
            SELECT * FROM agent_sessions WHERE session_token = $1
            "#,
            session_token,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(session)
    }
    
    async fn get_agent_sessions(&self, agent_id: &str) -> MessageBrokerResult<Vec<AgentSession>> {
        let sessions = sqlx::query_as::<_, 
            AgentSession,
            r#"
            SELECT * FROM agent_sessions 
            WHERE agent_id = $1 AND expires_at > CURRENT_TIMESTAMP
            ORDER BY last_activity_at DESC
            "#,
            agent_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(sessions)
    }
    
    async fn update_session_activity(&self, session_token: &str) -> MessageBrokerResult<()> {
        sqlx::query(
            r#"
            UPDATE agent_sessions SET
                last_activity_at = CURRENT_TIMESTAMP,
                expires_at = CURRENT_TIMESTAMP + INTERVAL '1 hour'
            WHERE session_token = $1
            "#,
            session_token,
        )
        .execute(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(())
    }
    
    async fn update_session_status(
        &self,
        session_token: &str,
        status: SessionStatus,
    ) -> MessageBrokerResult<()> {
        sqlx::query(
            r#"
            UPDATE agent_sessions SET
                status = $1,
                last_activity_at = CURRENT_TIMESTAMP
            WHERE session_token = $2
            "#,
            status as SessionStatus,
            session_token,
        )
        .execute(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(())
    }
    
    async fn delete_expired_sessions(&self) -> MessageBrokerResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM agent_sessions 
            WHERE expires_at < CURRENT_TIMESTAMP
            RETURNING id
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(result.len() as u64)
    }
    
    async fn disconnect_agent(&self, agent_id: &str) -> MessageBrokerResult<u64> {
        let result = sqlx::query(
            r#"
            UPDATE agent_sessions SET
                status = 'disconnected'::session_status,
                expires_at = CURRENT_TIMESTAMP + INTERVAL '5 minutes'
            WHERE agent_id = $1 AND status != 'disconnected'::session_status
            RETURNING id
            "#,
            agent_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(result.len() as u64)
    }
}

#[async_trait]
impl DeadLetterManager for MessageBrokerDatabase {
    async fn move_to_dead_letter(
        &self,
        message_id: uuid::Uuid,
        queue_name: &str,
        reason: &str,
        details: Option<serde_json::Value>,
    ) -> MessageBrokerResult<DeadLetterEntry> {
        let entry = sqlx::query_as::<_, 
            DeadLetterEntry,
            r#"
            INSERT INTO dead_letter_queue (
                message_id, original_queue, failure_reason, failure_details, failed_at
            ) VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
            RETURNING *
            "#,
            message_id,
            queue_name,
            reason,
            details,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        // Update delivery status to dead_letter
        sqlx::query(
            r#"
            UPDATE delivery_status SET
                status = 'dead_letter'::delivery_status,
                updated_at = CURRENT_TIMESTAMP,
                failure_reason = $1
            WHERE message_id = $2
            "#,
            reason,
            message_id,
        )
        .execute(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(entry)
    }
    
    async fn get_dead_letter_entries(
        &self,
        limit: i64,
        offset: i64,
    ) -> MessageBrokerResult<Vec<DeadLetterEntry>> {
        let entries = sqlx::query_as::<_, 
            DeadLetterEntry,
            r#"
            SELECT * FROM dead_letter_queue
            ORDER BY failed_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(entries)
    }
    
    async fn retry_dead_letter_entry(&self, entry_id: uuid::Uuid) -> MessageBrokerResult<()> {
        let mut transaction = self.begin_transaction().await?;
        
        // Get the entry
        let entry = sqlx::query_as::<_, 
            DeadLetterEntry,
            r#"
            SELECT * FROM dead_letter_queue WHERE id = $1
            "#,
            entry_id,
        )
        .fetch_optional(&mut *transaction)
        .await
        .map_err(MessageBrokerError::DatabaseError)?
        .ok_or_else(|| MessageBrokerError::MessageNotFound("Dead letter entry not found".to_string()))?;
        
        // Update retry count
        sqlx::query(
            r#"
            UPDATE dead_letter_queue SET
                retry_count = retry_count + 1,
                last_retry_attempt = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
            entry_id,
        )
        .execute(&mut *transaction)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        // Reset delivery status
        sqlx::query(
            r#"
            UPDATE delivery_status SET
                status = 'pending'::delivery_status,
                current_retry = 0,
                next_retry_at = NULL,
                failure_reason = NULL,
                updated_at = CURRENT_TIMESTAMP
            WHERE message_id = $1
            "#,
            entry.message_id,
        )
        .execute(&mut *transaction)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        // Re-enqueue message
        sqlx::query(
            r#"
            UPDATE queues SET
                dequeued_at = NULL,
                enqueued_at = CURRENT_TIMESTAMP
            WHERE message_id = $1
            "#,
            entry.message_id,
        )
        .execute(&mut *transaction)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        transaction.commit().await.map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(())
    }
    
    async fn delete_dead_letter_entry(&self, entry_id: uuid::Uuid) -> MessageBrokerResult<()> {
        sqlx::query(
            r#"
            DELETE FROM dead_letter_queue WHERE id = $1
            "#,
            entry_id,
        )
        .execute(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(())
    }
}

#[async_trait]
impl RoutingManager for MessageBrokerDatabase {
    async fn create_routing_rule(&self, rule: &RoutingRule) -> MessageBrokerResult<()> {
        sqlx::query(
            r#"
            INSERT INTO routing_rules (
                id, rule_name, match_pattern, priority, target_queue,
                transform_script, enabled, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            rule.id,
            rule.rule_name,
            rule.match_pattern,
            rule.priority,
            rule.target_queue,
            rule.transform_script,
            rule.enabled,
            rule.created_at,
            rule.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(())
    }
    
    async fn get_routing_rules(&self) -> MessageBrokerResult<Vec<RoutingRule>> {
        let rules = sqlx::query_as::<_, 
            RoutingRule,
            r#"
            SELECT * FROM routing_rules ORDER BY priority, rule_name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(rules)
    }
    
    async fn update_routing_rule(&self, rule: &RoutingRule) -> MessageBrokerResult<()> {
        sqlx::query(
            r#"
            UPDATE routing_rules SET
                rule_name = $1,
                match_pattern = $2,
                priority = $3,
                target_queue = $4,
                transform_script = $5,
                enabled = $6,
                updated_at = $7
            WHERE id = $8
            "#,
            rule.rule_name,
            rule.match_pattern,
            rule.priority,
            rule.target_queue,
            rule.transform_script,
            rule.enabled,
            rule.updated_at,
            rule.id,
        )
        .execute(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(())
    }
    
    async fn delete_routing_rule(&self, rule_id: uuid::Uuid) -> MessageBrokerResult<()> {
        sqlx::query(
            r#"
            DELETE FROM routing_rules WHERE id = $1
            "#,
            rule_id,
        )
        .execute(&self.pool)
        .await
        .map_err(MessageBrokerError::DatabaseError)?;
        
        Ok(())
    }
    
    async fn route_message(&self, message: &A2AMessage) -> MessageBrokerResult<Vec<String>> {
        let rules = self.get_routing_rules().await?;
        let mut target_queues = Vec::new();
        
        for rule in rules {
            if !rule.enabled {
                continue;
            }
            
            // Simple pattern matching (could be enhanced with JSONPath or similar)
            if self.matches_pattern(&rule.match_pattern, message) {
                target_queues.push(rule.target_queue.clone());
                
                // Apply transformation if specified
                if let Some(script) = &rule.transform_script {
                    // TODO: Implement message transformation
                    debug!("Would transform message with script: {}", script);
                }
            }
        }
        
        // Default to default queue if no rules match
        if target_queues.is_empty() {
            target_queues.push("default".to_string());
        }
        
        Ok(target_queues)
    }
}

impl MessageBrokerDatabase {
    /// Check if message matches routing pattern.
    fn matches_pattern(&self, pattern: &serde_json::Value, message: &A2AMessage) -> bool {
        // Simple exact match implementation
        // In production, this would use JSONPath or similar for complex patterns
        
        if let Some(obj) = pattern.as_object() {
            for (key, value) in obj {
                match key.as_str() {
                    "sender_id" => {
                        if let Some(pattern_value) = value.as_str() {
                            if pattern_value != message.sender_id {
                                return false;
                            }
                        }
                    }
                    "recipient_id" => {
                        if let Some(pattern_value) = value.as_str() {
                            if pattern_value != message.recipient_id {
                                return false;
                            }
                        }
                    }
                    "message_type" => {
                        if let Some(pattern_value) = value.as_str() {
                            if pattern_value != message.message_type {
                                return false;
                            }
                        }
                    }
                    "priority" => {
                        if let Some(pattern_value) = value.as_str() {
                            let msg_priority = match message.priority {
                                MessagePriority::Low => "low",
                                MessagePriority::Normal => "normal",
                                MessagePriority::High => "high",
                                MessagePriority::Critical => "critical",
                            };
                            if pattern_value != msg_priority {
                                return false;
                            }
                        }
                    }
                    _ => {
                        // Check metadata
                        if let Some(metadata) = &message.metadata {
                            if let Some(pattern_value) = value.as_str() {
                                if let Some(metadata_value) = metadata.get(key).and_then(|v| v.as_str()) {
                                    if pattern_value != metadata_value {
                                        return false;
                                    }
                                } else {
                                    return false;
                                }
                            }
                        } else {
                            return false;
                        }
                    }
                }
            }
            true
        } else {
            // Empty pattern matches everything
            true
        }
    }
}