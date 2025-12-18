//! Core types and utilities for the Constellation multi-agent platform.
#![allow(
    unused_variables,
    dead_code,
    unused_imports,
    clippy::too_many_arguments,
    clippy::type_complexity
)]

pub mod models;

pub mod autonomy;
pub mod dtg;
pub mod hybrid;
pub mod integration;
pub mod mcp;
pub mod message_broker;

// Re-export common types for convenience.
pub use models::agent::{
    Agent, AgentCapabilities, AgentContact, AgentInterface, AgentProvider, AgentSkill,
    ProtocolBinding, SecuritySchemeType,
};
pub use models::message_broker::{
    A2AMessage, AgentConnectionRequest, AgentConnectionResponse, AgentSession, DeliveryStatus,
    DeliveryStatusEntry, Message, MessageAcknowledgment, MessageBrokerError, MessageBrokerResult,
    MessagePriority, QueueEntry, QueueStats, RoutingRule, SessionStatus,
};

pub use message_broker::{BrokerConfig, LlmMessageBroker, LlmMessageBrokerBuilder, LlmQueueStats};

// Note: Other re-exports are temporarily disabled due to module compilation errors
