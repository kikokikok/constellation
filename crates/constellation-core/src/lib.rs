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
pub mod communication;
pub mod dtg;
pub mod gossip;
pub mod hybrid;
pub mod integration;
pub mod mcp;
pub mod memory;
pub mod message_broker;
pub mod tracing;

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

// Re-export tracing utilities
pub use tracing::{
    TracingConfig, generate_trace_id, init_tracing, log_with_trace_context, shutdown_tracing,
};

// Note: Other re-exports are temporarily disabled due to module compilation errors

#[cfg(test)]
mod tests {
    #[test]
    fn test_toon_serialization() {
        // This test will be run when testing the crate
        println!("TOON serialization tests are in the gossip module");
    }
}
