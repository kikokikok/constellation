//! Message broker implementations for Constellation A2A protocol.
//!
//! Features:
//! - Multiple implementations: Iggy-based (high-performance) and in-memory (simple)
//! - Priority-based scheduling (Critical, High, Normal, Low)
//! - Conversation context tracking
//! - Streaming message support
//! - Dead letter queue with retry logic
//! - WebSocket and HTTP interfaces with A2A protocol compliance

mod a2a_validation;
#[cfg(test)]
mod a2a_validation_test;
mod auth;
#[cfg(test)]
mod auth_test;
mod iggy_broker;
mod llm_broker;
mod websocket;

pub use a2a_validation::{
    A2AExtensionPoint, A2AFeature, A2AHeaders, A2AProtocolVersion, A2AValidator,
    ExtensionPointManager,
};
pub use auth::{AgentRegistration, AgentRegistrationService, AuthService, JwtClaims};
pub use iggy_broker::{IggyBrokerConfig, IggyMessageBroker, IggyMessageBrokerBuilder};
pub use llm_broker::{BrokerConfig, LlmMessageBroker, LlmMessageBrokerBuilder, LlmQueueStats};

/// Re-export commonly used types
pub use crate::models::message_broker::{
    A2AMessage, AgentConnectionRequest, AgentConnectionResponse, AgentSession, DeliveryStatus,
    DeliveryStatusEntry, Message, MessageAcknowledgment, MessageBrokerError, MessageBrokerResult,
    MessagePriority, QueueStats, RoutingRule, SessionStatus,
};
