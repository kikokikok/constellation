//! Error types for the Agent SDK

use thiserror::Error;

/// Agent SDK error type
#[derive(Error, Debug)]
pub enum AgentError {
    /// Connection-related errors
    #[error("Connection error: {0}")]
    Connection(String),

    /// Authentication errors
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Communication errors
    #[error("Communication error: {0}")]
    Communication(String),

    /// Timeout errors
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Message processing errors
    #[error("Message processing error: {0}")]
    MessageProcessing(String),

    /// Subscription errors
    #[error("Subscription error: {0}")]
    Subscription(String),

    /// Request errors
    #[error("Request error: {0}")]
    Request(String),

    /// Internal errors
    #[error("Internal error: {0}")]
    Internal(String),

    /// Underlying communication framework error
    #[error("Communication framework error: {0}")]
    Framework(#[from] constellation_core::models::communication::CommunicationError),

    /// Underlying message broker error
    #[error("Message broker error: {0}")]
    Broker(#[from] constellation_core::models::message_broker::MessageBrokerError),

    /// Metrics-related errors
    #[error("Metrics error: {0}")]
    Metrics(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for Agent SDK operations
pub type AgentResult<T> = Result<T, AgentError>;
