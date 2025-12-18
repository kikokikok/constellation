//! LLM-optimized message broker for Constellation A2A protocol.
//!
//! Features:
//! - Async, in-memory message queues
//! - Priority-based scheduling (Critical, High, Normal, Low)
//! - Conversation context tracking
//! - Streaming message support
//! - Dead letter queue with retry logic
//! - WebSocket and HTTP interfaces

use chrono::Utc;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::time;
use tracing::{debug, error, info, trace, warn};

use crate::models::message_broker::{
    A2AMessage, AgentSession, Message, MessageAcknowledgment, MessageBrokerError,
    MessageBrokerResult, MessagePriority,
};

/// LLM-optimized message broker.
///
/// Designed for high-throughput, low-latency agent communication.
/// Uses in-memory data structures for maximum performance.
#[derive(Clone)]
pub struct LlmMessageBroker {
    /// Priority queues for each agent
    queues: Arc<RwLock<HashMap<String, AgentQueues>>>,
    /// Active agent sessions
    sessions: Arc<RwLock<HashMap<String, AgentSession>>>,
    /// Dead letter queue for failed messages
    dead_letter: Arc<Mutex<VecDeque<DeadLetterEntry>>>,
    /// Configuration
    config: BrokerConfig,
}

/// Queues for a single agent
struct AgentQueues {
    critical: VecDeque<Message>,
    high: VecDeque<Message>,
    normal: VecDeque<Message>,
    low: VecDeque<Message>,
}

impl AgentQueues {
    fn new() -> Self {
        Self {
            critical: VecDeque::new(),
            high: VecDeque::new(),
            normal: VecDeque::new(),
            low: VecDeque::new(),
        }
    }

    /// Push message to appropriate queue based on priority
    fn push(&mut self, message: Message) {
        match message.priority {
            MessagePriority::Critical => self.critical.push_back(message),
            MessagePriority::High => self.high.push_back(message),
            MessagePriority::Normal => self.normal.push_back(message),
            MessagePriority::Low => self.low.push_back(message),
        }
    }

    /// Pop next message (priority order: Critical -> High -> Normal -> Low)
    fn pop(&mut self) -> Option<Message> {
        if !self.critical.is_empty() {
            return self.critical.pop_front();
        }
        if !self.high.is_empty() {
            return self.high.pop_front();
        }
        if !self.normal.is_empty() {
            return self.normal.pop_front();
        }
        self.low.pop_front()
    }

    /// Check if any queues have messages
    fn is_empty(&self) -> bool {
        self.critical.is_empty()
            && self.high.is_empty()
            && self.normal.is_empty()
            && self.low.is_empty()
    }

    /// Get total message count
    fn len(&self) -> usize {
        self.critical.len() + self.high.len() + self.normal.len() + self.low.len()
    }
}

/// Dead letter queue entry
struct DeadLetterEntry {
    message: Message,
    recipient: String,
    failure_reason: String,
    failed_at: Instant,
    retry_count: u32,
    max_retries: u32,
}

/// Broker configuration
#[derive(Clone, Debug)]
pub struct BrokerConfig {
    /// Maximum queue size per agent
    pub max_queue_size: usize,
    /// Message TTL in seconds
    pub message_ttl_seconds: u64,
    /// Max retry attempts
    pub max_retries: u32,
    /// Retry delay in seconds
    pub retry_delay_seconds: u64,
    /// Session timeout in seconds
    pub session_timeout_seconds: u64,
    /// Enable message persistence
    pub enable_persistence: bool,
}

impl Default for BrokerConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 1000,
            message_ttl_seconds: 3600, // 1 hour
            max_retries: 3,
            retry_delay_seconds: 60,      // 1 minute
            session_timeout_seconds: 300, // 5 minutes
            enable_persistence: false,
        }
    }
}

impl LlmMessageBroker {
    /// Create a new LLM message broker
    pub fn new(config: BrokerConfig) -> Self {
        let broker = Self {
            queues: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            dead_letter: Arc::new(Mutex::new(VecDeque::new())),
            config,
        };

        // Start maintenance tasks
        broker.start_maintenance_tasks();

        broker
    }

    /// Start background maintenance tasks
    fn start_maintenance_tasks(&self) {
        let broker = self.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = broker.run_maintenance().await {
                    error!("Maintenance task failed: {}", e);
                }
            }
        });
    }

    /// Send message to recipient
    pub async fn send_message(&self, message: Message) -> MessageBrokerResult<()> {
        let recipient = message.recipient_id.clone();

        // Check if recipient has active session
        let has_session = {
            let sessions = self.sessions.read().await;
            sessions.contains_key(&recipient)
        };

        if !has_session {
            return Err(MessageBrokerError::AgentNotConnected(recipient));
        }

        // Get or create queue for recipient
        let mut queues = self.queues.write().await;
        let agent_queues = queues
            .entry(recipient.clone())
            .or_insert_with(AgentQueues::new);

        // Check queue size limit
        if agent_queues.len() >= self.config.max_queue_size {
            return Err(MessageBrokerError::QueueFull(recipient));
        }

        // Add message to queue
        agent_queues.push(message);

        trace!("Message queued for agent: {}", recipient);
        Ok(())
    }

    /// Send A2A message (converts to internal format)
    pub async fn send_a2a_message(&self, a2a_message: A2AMessage) -> MessageBrokerResult<()> {
        let message = a2a_message.to_message();
        self.send_message(message).await
    }

    /// Receive messages for agent
    pub async fn receive_messages(
        &self,
        agent_id: &str,
        limit: usize,
    ) -> MessageBrokerResult<Vec<Message>> {
        // Update session activity
        self.update_session_activity(agent_id).await?;

        let mut queues = self.queues.write().await;
        let agent_queues = queues
            .entry(agent_id.to_string())
            .or_insert_with(AgentQueues::new);

        let mut messages = Vec::with_capacity(limit);
        for _ in 0..limit {
            if let Some(message) = agent_queues.pop() {
                messages.push(message);
            } else {
                break;
            }
        }

        trace!(
            "Delivered {} messages to agent: {}",
            messages.len(),
            agent_id
        );
        Ok(messages)
    }

    /// Acknowledge message delivery
    pub async fn acknowledge_message(
        &self,
        agent_id: &str,
        acknowledgment: MessageAcknowledgment,
    ) -> MessageBrokerResult<()> {
        // Update session activity
        self.update_session_activity(agent_id).await?;

        debug!(
            "Message acknowledged by {}: {}",
            agent_id, acknowledgment.message_id
        );
        Ok(())
    }

    /// Register agent session
    pub async fn register_session(&self, session: AgentSession) -> MessageBrokerResult<()> {
        let agent_id = session.agent_id.clone();
        let mut sessions = self.sessions.write().await;
        sessions.insert(agent_id.clone(), session);

        info!("Agent session registered: {}", agent_id);
        Ok(())
    }

    /// Get agent session
    pub async fn get_session(&self, agent_id: &str) -> MessageBrokerResult<Option<AgentSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(agent_id).cloned())
    }

    /// Update session activity
    pub async fn update_session_activity(&self, agent_id: &str) -> MessageBrokerResult<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(agent_id) {
            session.update_activity();
        }
        Ok(())
    }

    /// Broadcast message to all connected agents
    pub async fn broadcast(&self, message: Message) -> MessageBrokerResult<()> {
        let sessions = self.sessions.read().await;
        let agent_ids: Vec<String> = sessions.keys().cloned().collect();

        for agent_id in agent_ids {
            let mut broadcast_message = message.clone();
            broadcast_message.recipient_id = agent_id.clone();

            if let Err(e) = self.send_message(broadcast_message).await {
                warn!("Failed to broadcast to agent {}: {}", agent_id, e);
            }
        }

        info!("Broadcast message to {} agents", sessions.len());
        Ok(())
    }

    /// Get queue statistics
    pub async fn get_queue_stats(&self, agent_id: &str) -> MessageBrokerResult<LlmQueueStats> {
        let queues = self.queues.read().await;
        let sessions = self.sessions.read().await;

        let total_messages = queues.get(agent_id).map(|q| q.len()).unwrap_or(0);

        let is_connected = sessions.contains_key(agent_id);

        Ok(LlmQueueStats {
            agent_id: agent_id.to_string(),
            total_messages,
            is_connected,
            last_updated: chrono::Utc::now(),
        })
    }

    /// Get all connected agents
    pub async fn get_connected_agents(&self) -> Vec<String> {
        let sessions = self.sessions.read().await;
        sessions.keys().cloned().collect()
    }

    /// Run maintenance tasks
    pub async fn run_maintenance(&self) -> MessageBrokerResult<()> {
        info!("Running LLM message broker maintenance");

        // Clean up expired sessions
        self.cleanup_expired_sessions().await?;

        // Process dead letter queue
        self.process_dead_letter().await?;

        // Clean up empty queues
        self.cleanup_empty_queues().await?;

        Ok(())
    }

    /// Clean up expired sessions
    async fn cleanup_expired_sessions(&self) -> MessageBrokerResult<()> {
        let mut sessions = self.sessions.write().await;
        let timeout_seconds = self.config.session_timeout_seconds as i64;
        let now = Utc::now();

        let expired: Vec<String> = sessions
            .iter()
            .filter(|(_, session)| {
                let elapsed = now.signed_duration_since(session.last_activity_at);
                elapsed.num_seconds() > timeout_seconds
            })
            .map(|(id, _)| id.clone())
            .collect();

        for agent_id in expired {
            sessions.remove(&agent_id);
            info!("Expired session for agent: {}", agent_id);
        }

        Ok(())
    }

    /// Process dead letter queue
    async fn process_dead_letter(&self) -> MessageBrokerResult<()> {
        let mut dead_letter = self.dead_letter.lock().await;
        let now = Instant::now();
        let retry_delay = Duration::from_secs(self.config.retry_delay_seconds);

        // Collect entries that need processing
        let mut entries_to_keep = Vec::new();
        let mut entries_to_retry = Vec::new();

        // Drain the dead letter queue and process each entry
        while let Some(entry) = dead_letter.pop_front() {
            if entry.retry_count >= entry.max_retries {
                // Permanently failed
                warn!(
                    "Permanently failed message for {}: {}",
                    entry.recipient, entry.failure_reason
                );
                continue;
            }

            if now.duration_since(entry.failed_at) > retry_delay {
                // Ready for retry
                entries_to_retry.push(entry);
            } else {
                // Not ready yet, keep as-is
                entries_to_keep.push(entry);
            }
        }

        // Process retries
        for mut entry in entries_to_retry {
            entry.retry_count += 1;
            entry.failed_at = now;

            // Try to resend
            if let Err(e) = self.send_message(entry.message.clone()).await {
                warn!("Retry failed for {}: {}", entry.recipient, e);
                // Put back in queue for future retry
                entries_to_keep.push(entry);
            } else {
                info!("Successfully retried message for {}", entry.recipient);
            }
        }

        // Restore entries to keep
        for entry in entries_to_keep {
            dead_letter.push_back(entry);
        }

        Ok(())
    }

    /// Clean up empty queues
    async fn cleanup_empty_queues(&self) -> MessageBrokerResult<()> {
        let mut queues = self.queues.write().await;
        let empty_queues: Vec<String> = queues
            .iter()
            .filter(|(_, q)| q.is_empty())
            .map(|(id, _)| id.clone())
            .collect();

        for agent_id in empty_queues {
            queues.remove(&agent_id);
            trace!("Cleaned up empty queue for agent: {}", agent_id);
        }

        Ok(())
    }

    /// Move message to dead letter queue
    async fn move_to_dead_letter(
        &self,
        message: Message,
        recipient: String,
        reason: String,
    ) -> MessageBrokerResult<()> {
        let reason_clone = reason.clone();
        let entry = DeadLetterEntry {
            message,
            recipient,
            failure_reason: reason,
            failed_at: Instant::now(),
            retry_count: 0,
            max_retries: self.config.max_retries,
        };

        let mut dead_letter = self.dead_letter.lock().await;
        dead_letter.push_back(entry);

        warn!("Message moved to dead letter queue: {}", reason_clone);
        Ok(())
    }
}

/// LLM broker queue statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct LlmQueueStats {
    pub agent_id: String,
    pub total_messages: usize,
    pub is_connected: bool,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Builder for LlmMessageBroker
pub struct LlmMessageBrokerBuilder {
    config: BrokerConfig,
}

impl LlmMessageBrokerBuilder {
    pub fn new() -> Self {
        Self {
            config: BrokerConfig::default(),
        }
    }

    pub fn max_queue_size(mut self, size: usize) -> Self {
        self.config.max_queue_size = size;
        self
    }

    pub fn message_ttl(mut self, seconds: u64) -> Self {
        self.config.message_ttl_seconds = seconds;
        self
    }

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    pub fn retry_delay(mut self, seconds: u64) -> Self {
        self.config.retry_delay_seconds = seconds;
        self
    }

    pub fn session_timeout(mut self, seconds: u64) -> Self {
        self.config.session_timeout_seconds = seconds;
        self
    }

    pub fn enable_persistence(mut self, enable: bool) -> Self {
        self.config.enable_persistence = enable;
        self
    }

    pub fn build(self) -> LlmMessageBroker {
        LlmMessageBroker::new(self.config)
    }
}

impl Default for LlmMessageBrokerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::message_broker::AgentSession;

    #[tokio::test]
    async fn test_send_and_receive() {
        let broker = LlmMessageBroker::new(BrokerConfig::default());

        // Register session
        let session = AgentSession::new(
            "test_agent".to_string(),
            "token".to_string(),
            "websocket".to_string(),
            None,
        );
        broker.register_session(session).await.unwrap();

        // Send message
        let mut message = Message::new(
            "msg1".to_string(),
            "sender".to_string(),
            "test_agent".to_string(),
            "test".to_string(),
            "Hello, agent!".to_string(),
        );
        message.priority = MessagePriority::Normal;

        broker.send_message(message.clone()).await.unwrap();

        // Receive message
        let received = broker.receive_messages("test_agent", 10).await.unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].message_id, "msg1");
    }

    #[tokio::test]
    async fn test_priority_queuing() {
        let broker = LlmMessageBroker::new(BrokerConfig::default());

        // Register session
        let session = AgentSession::new(
            "test_agent".to_string(),
            "token".to_string(),
            "websocket".to_string(),
            None,
        );
        broker.register_session(session).await.unwrap();

        // Send messages with different priorities
        let mut low_msg = Message::new(
            "low".to_string(),
            "sender".to_string(),
            "test_agent".to_string(),
            "test".to_string(),
            "Low priority".to_string(),
        );
        low_msg.priority = MessagePriority::Low;

        let mut high_msg = Message::new(
            "high".to_string(),
            "sender".to_string(),
            "test_agent".to_string(),
            "test".to_string(),
            "High priority".to_string(),
        );
        high_msg.priority = MessagePriority::High;

        // Send low first, then high
        broker.send_message(low_msg).await.unwrap();
        broker.send_message(high_msg.clone()).await.unwrap();

        // High should be received first
        let received = broker.receive_messages("test_agent", 10).await.unwrap();
        assert_eq!(received.len(), 2);
        assert_eq!(received[0].message_id, "high"); // High priority first
    }
}
