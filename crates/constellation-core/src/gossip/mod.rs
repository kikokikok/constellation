//! Gossip protocol implementation for decentralized service discovery and state synchronization.
//!
//! This module implements the SWIM (Scalable Weakly-consistent Infection-style Process Group Membership)
//! gossip protocol for peer discovery, health checking, and state dissemination in Constellation.
//!
//! ## Features
//! - Decentralized peer discovery without single points of failure
//! - Efficient health checking with configurable failure detection
//! - State synchronization with configurable dissemination strategies
//! - Protocol negotiation for backward compatibility
//! - Integration with A2A protocol for agent communication
//! - Multi-format serialization support (JSON, TOON, MessagePack, Protobuf)
//!
//! ## Serialization Formats
//!
//! The gossip protocol supports multiple serialization formats with automatic negotiation:
//!
//! 1. **JSON** - Default format, human-readable, universal compatibility
//! 2. **TOON** - Token-efficient format (30-60% smaller for arrays), optimized for LLM communication
//! 3. **MessagePack** - Compact binary format (75% smaller than JSON), efficient for network transport
//! 4. **Protobuf** - Type-safe binary format (planned), ideal for high-performance scenarios
//!
//! ## Protocol Negotiation
//!
//! Nodes automatically negotiate the best serialization format:
//! - Each node advertises its supported formats
//! - Preferred format is tried first
//! - Falls back through configured fallback formats
//! - JSON is always available as universal fallback
//! - Negotiation can be disabled for specific use cases
//!
//! ## Performance
//!
//! - **MessagePack**: 75% size reduction vs JSON
//! - **TOON**: 30-60% size reduction for array data
//! - **Automatic negotiation**: Zero configuration required
//! - **Backward compatibility**: JSON fallback ensures communication

mod discovery;
mod models;
mod protocol;
mod state;
// mod toon; // Using external toon crate instead
// mod libp2p_gossip; // Temporarily disabled due to compilation errors

pub use discovery::*;
pub use models::*;
pub use protocol::*;
pub use state::{CachedQuery, GossipState, GossipStats, ServiceEntry, ServiceState};
// pub use toon::*; // Using external toon crate instead
// pub use libp2p_gossip::*;
