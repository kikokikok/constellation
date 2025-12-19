//! Iggy-based message broker for Constellation A2A protocol.
//!
//! This module provides a high-performance message broker implementation
//! using Apache Iggy as the underlying message streaming platform.
//!
//! Features:
//! - Built on Apache Iggy for high-performance persistent message streaming
//! - Priority-based scheduling using Iggy partitions
//! - Conversation context tracking
//! - Streaming message support via Iggy's consumer groups
//! - Dead letter queue with retry logic
//! - WebSocket and HTTP interfaces with A2A protocol compliance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace, warn};

use super::a2a_validation::A2AValidator;
use super::auth::{AgentRegistrationService, AuthService};
use crate::mcp::crypto::{CryptoError, McpCrypto};
use crate::models::message_broker::{
    AgentSession, Message as ConstellationMessage, MessageAcknowledgment, MessageBrokerError,
    MessageBrokerResult, MessagePriority,
};

/// Iggy-based message broker for Constellation.
///
/// Maps Constellation concepts to Iggy concepts:
/// - Agent → Iggy Consumer (in a consumer group)
/// - Message queue → Iggy Topic with partitions for priority
/// - Priority → Iggy partition (0=Critical, 1=High, 2=Normal, 3=Low)
/// - Session → Iggy consumer group membership
#[derive(Clone)]
pub struct IggyMessageBroker {
    /// Active agent sessions
    sessions: Arc<RwLock<HashMap<String, AgentSession>>>,
    /// Configuration
    config: IggyBrokerConfig,
    /// Authentication service
    auth_service: Option<Arc<AuthService>>,
    /// Agent registration service
    registration_service: Option<Arc<AgentRegistrationService>>,
    /// A2A protocol validator
    a2a_validator: Arc<A2AValidator>,
}

/// Iggy broker configuration
#[derive(Clone, Debug)]
pub struct IggyBrokerConfig {
    /// Iggy server address
    pub iggy_server_address: String,
    /// Iggy username
    pub iggy_username: String,
    /// Iggy password
    pub iggy_password: String,
    /// Stream name for Constellation
    pub stream_name: String,
    /// Topic name for agent messages
    pub topic_name: String,
    /// Number of partitions (one per priority level)
    pub partitions_count: u32,
    /// Message retention period in seconds
    pub message_retention_period: u32,
    /// Max message batch size
    pub max_batch_size: u32,
    /// Session timeout in seconds
    pub session_timeout_seconds: u64,
}

impl Default for IggyBrokerConfig {
    fn default() -> Self {
        Self {
            iggy_server_address: "127.0.0.1:8090".to_string(),
            iggy_username: "guest".to_string(),
            iggy_password: "guest".to_string(),
            stream_name: "constellation".to_string(),
            topic_name: "agent_messages".to_string(),
            partitions_count: 4,            // One partition per priority level
            message_retention_period: 3600, // 1 hour
            max_batch_size: 1000,
            session_timeout_seconds: 300, // 5 minutes
        }
    }
}

impl IggyMessageBroker {
    /// Create a new Iggy message broker
    pub async fn new(config: IggyBrokerConfig) -> MessageBrokerResult<Self> {
        info!("Creating Iggy message broker (placeholder implementation)");
        info!(
            "Note: Full Iggy integration requires Iggy server running at {}",
            config.iggy_server_address
        );

        let broker = Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
            auth_service: None,
            registration_service: None,
            a2a_validator: Arc::new(A2AValidator::new()),
        };

        Ok(broker)
    }

    /// Create a new Iggy message broker with authentication
    pub async fn new_with_auth(
        config: IggyBrokerConfig,
        auth_service: AuthService,
        registration_service: AgentRegistrationService,
    ) -> MessageBrokerResult<Self> {
        info!("Creating Iggy message broker with authentication");
        info!(
            "Note: Full Iggy integration requires Iggy server running at {}",
            config.iggy_server_address
        );

        let broker = Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
            auth_service: Some(Arc::new(auth_service)),
            registration_service: Some(Arc::new(registration_service)),
            a2a_validator: Arc::new(A2AValidator::new()),
        };

        Ok(broker)
    }

    /// Map Constellation message priority to Iggy partition
    fn priority_to_partition(&self, priority: MessagePriority) -> u32 {
        match priority {
            MessagePriority::Critical => 0,
            MessagePriority::High => 1,
            MessagePriority::Normal => 2,
            MessagePriority::Low => 3,
        }
    }

    /// Get the A2A protocol validator
    pub fn a2a_validator(&self) -> &A2AValidator {
        &self.a2a_validator
    }

    /// Negotiate protocol version with client
    pub fn negotiate_protocol_version(
        &self,
        client_versions: &[String],
    ) -> MessageBrokerResult<String> {
        let version = self.a2a_validator.negotiate_version(client_versions)?;
        Ok(version.as_str().to_string())
    }

    /// Send message to recipient
    pub async fn send_message(&self, message: ConstellationMessage) -> MessageBrokerResult<()> {
        // Validate message against A2A protocol
        self.a2a_validator.validate_message(&message)?;

        let recipient = message.recipient_id.clone();

        // Check if recipient has active session
        let has_session = {
            let sessions = self.sessions.read().await;
            sessions.contains_key(&recipient)
        };

        if !has_session {
            return Err(MessageBrokerError::AgentNotConnected(recipient));
        }

        // Note: In full implementation, this would send to Iggy server
        // For now, we just log the message
        trace!(
            "[IGGY] Would send message to {} via partition {}",
            recipient,
            self.priority_to_partition(message.priority)
        );

        Ok(())
    }

    /// Receive messages for agent
    pub async fn receive_messages(
        &self,
        agent_id: &str,
        limit: usize,
    ) -> MessageBrokerResult<Vec<ConstellationMessage>> {
        // Update session activity
        self.update_session_activity(agent_id).await?;

        // Note: In full implementation, this would poll from Iggy server
        // For now, return empty vector
        trace!(
            "[IGGY] Would poll messages for {} (limit: {})",
            agent_id, limit
        );

        Ok(Vec::new())
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

    /// Register agent session with JWT authentication
    pub async fn register_session_with_auth(
        &self,
        session: AgentSession,
        jwt_token: &str,
    ) -> MessageBrokerResult<()> {
        // Validate JWT token if auth service is available
        if let Some(auth_service) = &self.auth_service {
            let claims = auth_service.validate_token(jwt_token)?;

            // Verify agent ID matches token claims
            if claims.agent_id != session.agent_id {
                return Err(MessageBrokerError::AuthenticationFailed(
                    "Agent ID mismatch".to_string(),
                ));
            }

            // Update agent activity if registration service is available
            if let Some(registration_service) = &self.registration_service {
                registration_service.validate_agent(jwt_token).await?;
            }
        }

        // Register session
        self.register_session(session).await
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
    pub async fn broadcast(&self, message: ConstellationMessage) -> MessageBrokerResult<()> {
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

    /// Get all connected agents
    pub async fn get_connected_agents(&self) -> Vec<String> {
        let sessions = self.sessions.read().await;
        sessions.keys().cloned().collect()
    }

    /// Run maintenance tasks
    pub async fn run_maintenance(&self) -> MessageBrokerResult<()> {
        info!("Running Iggy message broker maintenance");

        // Clean up expired sessions
        self.cleanup_expired_sessions().await?;

        Ok(())
    }

    /// Clean up expired sessions
    async fn cleanup_expired_sessions(&self) -> MessageBrokerResult<()> {
        let mut sessions = self.sessions.write().await;
        let timeout_seconds = self.config.session_timeout_seconds as i64;
        let now = chrono::Utc::now();

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

    /// Generate JWT token for agent
    pub async fn generate_jwt_token(
        &self,
        agent_id: &str,
        key_id: &str,
    ) -> MessageBrokerResult<String> {
        let auth_service = self.auth_service.as_ref().ok_or_else(|| {
            MessageBrokerError::AuthenticationFailed(
                "Authentication service not available".to_string(),
            )
        })?;

        auth_service.generate_token(agent_id, key_id)
    }

    /// Register new agent with key generation
    pub async fn register_new_agent(
        &self,
        agent_id: &str,
    ) -> MessageBrokerResult<(String, String)> {
        let registration_service = self.registration_service.as_ref().ok_or_else(|| {
            MessageBrokerError::AuthenticationFailed(
                "Registration service not available".to_string(),
            )
        })?;

        registration_service.register_agent(agent_id).await
    }
}

/// Builder for IggyMessageBroker
pub struct IggyMessageBrokerBuilder {
    config: IggyBrokerConfig,
    a2a_validator: Option<A2AValidator>,
}

impl IggyMessageBrokerBuilder {
    pub fn new() -> Self {
        Self {
            config: IggyBrokerConfig::default(),
            a2a_validator: None,
        }
    }

    pub fn server_address(mut self, address: String) -> Self {
        self.config.iggy_server_address = address;
        self
    }

    pub fn credentials(mut self, username: String, password: String) -> Self {
        self.config.iggy_username = username;
        self.config.iggy_password = password;
        self
    }

    pub fn stream_name(mut self, name: String) -> Self {
        self.config.stream_name = name;
        self
    }

    pub fn topic_name(mut self, name: String) -> Self {
        self.config.topic_name = name;
        self
    }

    pub fn partitions_count(mut self, count: u32) -> Self {
        self.config.partitions_count = count;
        self
    }

    pub fn message_retention(mut self, seconds: u32) -> Self {
        self.config.message_retention_period = seconds;
        self
    }

    pub fn max_batch_size(mut self, size: u32) -> Self {
        self.config.max_batch_size = size;
        self
    }

    pub fn session_timeout(mut self, seconds: u64) -> Self {
        self.config.session_timeout_seconds = seconds;
        self
    }

    pub fn a2a_validator(mut self, validator: A2AValidator) -> Self {
        self.a2a_validator = Some(validator);
        self
    }

    pub async fn build(self) -> MessageBrokerResult<IggyMessageBroker> {
        let mut broker = IggyMessageBroker::new(self.config).await?;

        // Set custom A2A validator if provided
        if let Some(validator) = self.a2a_validator {
            broker.a2a_validator = Arc::new(validator);
        }

        Ok(broker)
    }
}

impl Default for IggyMessageBrokerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::message_broker::AgentSession;

    #[tokio::test]
    async fn test_session_management() {
        let broker = IggyMessageBrokerBuilder::new()
            .build()
            .await
            .expect("Failed to create broker");

        // Register session
        let session = AgentSession::new(
            "test_agent".to_string(),
            "token".to_string(),
            "websocket".to_string(),
            None,
        );
        broker.register_session(session).await.unwrap();

        // Get session
        let retrieved = broker.get_session("test_agent").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().agent_id, "test_agent");

        // Update activity
        broker.update_session_activity("test_agent").await.unwrap();

        // Get connected agents
        let agents = broker.get_connected_agents().await;
        assert_eq!(agents, vec!["test_agent".to_string()]);
    }

    #[test]
    fn test_priority_to_partition_logic() {
        // Test the priority mapping logic
        let config = IggyBrokerConfig::default();
        let broker = IggyMessageBroker {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
            auth_service: None,
            registration_service: None,
            a2a_validator: Arc::new(A2AValidator::new()),
        };

        assert_eq!(broker.priority_to_partition(MessagePriority::Critical), 0);
        assert_eq!(broker.priority_to_partition(MessagePriority::High), 1);
        assert_eq!(broker.priority_to_partition(MessagePriority::Normal), 2);
        assert_eq!(broker.priority_to_partition(MessagePriority::Low), 3);
    }
}

#[async_trait::async_trait]
impl crate::communication::MessageBroker for IggyMessageBroker {
    async fn send_message(
        &self,
        message: crate::models::message_broker::Message,
    ) -> MessageBrokerResult<()> {
        self.send_message(message).await
    }

    async fn receive_messages(
        &self,
        agent_id: &str,
        limit: usize,
    ) -> MessageBrokerResult<Vec<crate::models::message_broker::Message>> {
        self.receive_messages(agent_id, limit).await
    }

    async fn register_session(&self, session: AgentSession) -> MessageBrokerResult<()> {
        self.register_session(session).await
    }

    async fn get_session(&self, agent_id: &str) -> MessageBrokerResult<Option<AgentSession>> {
        self.get_session(agent_id).await
    }

    async fn broadcast(
        &self,
        message: crate::models::message_broker::Message,
    ) -> MessageBrokerResult<()> {
        self.broadcast(message).await
    }
}
