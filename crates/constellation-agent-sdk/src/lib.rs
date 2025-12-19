//! Constellation Agent SDK
//!
//! High-level SDK for building agents that communicate using A2A patterns.
//!
//! ## Features
//! - **Request-Response**: Send requests and await responses with timeouts
//! - **Publish-Subscribe**: Subscribe to topics and receive published messages
//! - **Fire-and-Forget**: Send notifications without waiting for response
//! - **Delivery Guarantees**: Configurable delivery semantics
//! - **Connection Management**: Automatic reconnection and session management
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use constellation_agent_sdk::{AgentClient, AgentConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create agent client
//!     let config = AgentConfig::default()
//!         .with_agent_id("my_agent")
//!         .with_broker_url("127.0.0.1:8090");
//!
//!     let mut client = AgentClient::connect(config).await?;
//!
//!     // Send a request
//!     let response = client
//!         .request("other_agent", "ping")
//!         .await?;
//!
//!     println!("Received response: {:?}", response);
//!
//!     // Subscribe to a topic
//!     client.subscribe("system.alerts").await?;
//!
//!     // Process incoming messages
//!     let messages = client.receive(10).await?;
//!     for message in messages {
//!         println!("Received: {:?}", message);
//!     }
//!
//!     Ok(())
//! }
//! ```

mod client;
mod config;
mod connection_pool;
mod error;
mod handler;

#[cfg(feature = "monitoring")]
mod monitoring;

pub use client::AgentClient;
pub use config::AgentConfig;
pub use connection_pool::{ConnectionPoolConfig, PoolStats};
pub use error::{AgentError, AgentResult};
pub use handler::{DefaultMessageHandler, DefaultRequestHandler, MessageHandler, RequestHandler};

#[cfg(feature = "monitoring")]
pub use monitoring::{MonitoringConfig, MonitoringServer};

// Re-export commonly used types from constellation-core
pub use constellation_core::models::communication::{
    DeliveryGuarantee, NotificationMessage, PublishMessage, RequestConfig, RequestMessage,
    ResponseConfig, ResponseMessage, ResponseStatus, Subscription, TopicPattern,
};
pub use constellation_core::models::message_broker::{
    AgentSession, Message, MessageAcknowledgment, MessagePriority,
};

/// Pattern-specific metrics
#[derive(Debug, Clone, serde::Serialize)]
pub enum PatternMetrics {
    /// Request-response pattern metrics
    RequestResponse(constellation_core::communication::RequestResponseMetricsSnapshot),
    /// Publish-subscribe pattern metrics
    PublishSubscribe(constellation_core::communication::PublishSubscribeMetricsSnapshot),
    /// Fire-and-forget pattern metrics
    FireAndForget(constellation_core::communication::FireAndForgetMetricsSnapshot),
}
