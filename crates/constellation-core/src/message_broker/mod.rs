//! LLM-optimized message broker for Constellation A2A protocol.
//!
//! Features:
//! - Async, in-memory message queues (no database required!)
//! - Priority-based scheduling (Critical, High, Normal, Low)
//! - Conversation context tracking
//! - Streaming message support
//! - Dead letter queue with retry logic
//! - WebSocket and HTTP interfaces

mod llm_broker;
mod websocket;

pub use llm_broker::{BrokerConfig, LlmMessageBroker, LlmMessageBrokerBuilder, LlmQueueStats};

/// Re-export commonly used types
pub use crate::models::message_broker::{
    A2AMessage, AgentConnectionRequest, AgentConnectionResponse, AgentSession, DeliveryStatus,
    DeliveryStatusEntry, Message, MessageAcknowledgment, MessageBrokerError, MessageBrokerResult,
    MessagePriority, RoutingRule, SessionStatus,
};
