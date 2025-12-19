//! Gossip protocol data models

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Node state in the gossip cluster
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeState {
    /// Node is healthy and responding
    Alive,
    /// Node is suspected to be dead (in failure detection phase)
    Suspect,
    /// Node is confirmed dead
    Dead,
    /// Node is temporarily unavailable (maintenance, etc.)
    Unavailable,
}

/// Node information in the gossip cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique node identifier
    pub id: Uuid,
    /// Node name (human-readable)
    pub name: String,
    /// Network address (IP:port)
    pub address: SocketAddr,
    /// Current node state
    pub state: NodeState,
    /// Node metadata (capabilities, version, etc.)
    pub metadata: serde_json::Value,
    /// Last heartbeat timestamp
    pub last_heartbeat: SystemTime,
    /// Time when node was marked as suspect (if applicable)
    pub suspect_since: Option<SystemTime>,
    /// Time when node was marked as dead (if applicable)
    pub dead_since: Option<SystemTime>,
    /// Protocol version supported by this node
    pub protocol_version: String,
    /// Serialization formats supported by this node
    pub supported_formats: Vec<SerializationFormat>,
}

impl Node {
    /// Create a new node
    pub fn new(
        name: String,
        address: SocketAddr,
        metadata: serde_json::Value,
        protocol_version: String,
        supported_formats: Vec<SerializationFormat>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            address,
            state: NodeState::Alive,
            metadata,
            last_heartbeat: SystemTime::now(),
            suspect_since: None,
            dead_since: None,
            protocol_version,
            supported_formats,
        }
    }

    /// Check if node is considered alive
    pub fn is_alive(&self) -> bool {
        matches!(self.state, NodeState::Alive)
    }

    /// Check if node is considered dead
    pub fn is_dead(&self) -> bool {
        matches!(self.state, NodeState::Dead)
    }

    /// Check if node is suspected
    pub fn is_suspect(&self) -> bool {
        matches!(self.state, NodeState::Suspect)
    }

    /// Update heartbeat timestamp
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = SystemTime::now();
    }

    /// Mark node as suspect
    pub fn mark_suspect(&mut self) {
        self.state = NodeState::Suspect;
        self.suspect_since = Some(SystemTime::now());
    }

    /// Mark node as alive
    pub fn mark_alive(&mut self) {
        self.state = NodeState::Alive;
        self.suspect_since = None;
        self.dead_since = None;
        self.update_heartbeat();
    }

    /// Mark node as dead
    pub fn mark_dead(&mut self) {
        self.state = NodeState::Dead;
        self.dead_since = Some(SystemTime::now());
    }
}

/// Membership list representing the cluster state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Membership {
    /// Local node information
    pub local_node: Node,
    /// Known nodes in the cluster
    pub known_nodes: Vec<Node>,
    /// Cluster generation number (incremented on major changes)
    pub generation: u64,
    /// Timestamp of last membership change
    pub last_change: SystemTime,
}

impl Membership {
    /// Create a new membership list for a node
    pub fn new(local_node: Node) -> Self {
        Self {
            local_node,
            known_nodes: Vec::new(),
            generation: 0,
            last_change: SystemTime::now(),
        }
    }

    /// Add a node to the membership list
    pub fn add_node(&mut self, node: Node) {
        if !self.known_nodes.iter().any(|n| n.id == node.id) {
            self.known_nodes.push(node);
            self.last_change = SystemTime::now();
        }
    }

    /// Remove a node from the membership list
    pub fn remove_node(&mut self, node_id: &Uuid) {
        let len_before = self.known_nodes.len();
        self.known_nodes.retain(|n| n.id != *node_id);
        if self.known_nodes.len() != len_before {
            self.last_change = SystemTime::now();
        }
    }

    /// Get a node by ID
    pub fn get_node(&self, node_id: &Uuid) -> Option<&Node> {
        if self.local_node.id == *node_id {
            Some(&self.local_node)
        } else {
            self.known_nodes.iter().find(|n| n.id == *node_id)
        }
    }

    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, node_id: &Uuid) -> Option<&mut Node> {
        if self.local_node.id == *node_id {
            Some(&mut self.local_node)
        } else {
            self.known_nodes.iter_mut().find(|n| n.id == *node_id)
        }
    }

    /// Get all alive nodes (including local node)
    pub fn alive_nodes(&self) -> Vec<&Node> {
        let mut nodes = Vec::new();

        if self.local_node.is_alive() {
            nodes.push(&self.local_node);
        }

        nodes.extend(self.known_nodes.iter().filter(|n| n.is_alive()));
        nodes
    }

    /// Update node state
    pub fn update_node_state(&mut self, node_id: &Uuid, state: NodeState) {
        if let Some(node) = self.get_node_mut(node_id) {
            node.state = state.clone();
            match state {
                NodeState::Alive => {
                    node.mark_alive();
                }
                NodeState::Suspect => {
                    node.mark_suspect();
                }
                NodeState::Dead => {
                    node.mark_dead();
                }
                NodeState::Unavailable => {
                    // For unavailable, we just set the state
                }
            }
        }
    }

    /// Get all nodes (including local node)
    pub fn all_nodes(&self) -> Vec<&Node> {
        let mut nodes = vec![&self.local_node];
        nodes.extend(self.known_nodes.iter());
        nodes
    }

    /// Merge another membership list into this one
    pub fn merge(&mut self, other: &Membership) -> bool {
        let mut changed = false;

        for node in &other.known_nodes {
            if let Some(existing) = self.get_node_mut(&node.id) {
                // Update if other node has more recent information
                if node.last_heartbeat > existing.last_heartbeat {
                    *existing = node.clone();
                    changed = true;
                }
            } else {
                // Add new node
                self.add_node(node.clone());
                changed = true;
            }
        }

        if changed {
            self.last_change = SystemTime::now();
        }

        changed
    }
}

/// Gossip message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipMessage {
    /// Ping message for health checking
    Ping {
        sender_id: Uuid,
        sequence: u64,
        timestamp: SystemTime,
    },
    /// Ack message in response to ping
    Ack {
        sender_id: Uuid,
        ping_sequence: u64,
        timestamp: SystemTime,
        membership_snapshot: Option<Membership>,
    },
    /// Ping request (indirect ping)
    PingReq {
        sender_id: Uuid,
        target_id: Uuid,
        sequence: u64,
        timestamp: SystemTime,
    },
    /// Membership update
    MembershipUpdate {
        sender_id: Uuid,
        membership: Membership,
        timestamp: SystemTime,
    },
    /// State synchronization
    StateSync {
        sender_id: Uuid,
        state_type: String,
        state_data: serde_json::Value,
        version: u64,
        timestamp: SystemTime,
    },
}

/// Serialization formats supported by the gossip protocol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SerializationFormat {
    /// JSON serialization (default, human-readable)
    Json,
    /// TOON serialization (type-safe, efficient binary)
    Toon,
    /// Protocol Buffers serialization (compact binary)
    Protobuf,
    /// MessagePack serialization (compact binary)
    MessagePack,
}

impl SerializationFormat {
    /// Get the content type string for this format
    pub fn content_type(&self) -> &'static str {
        match self {
            SerializationFormat::Json => "application/json",
            SerializationFormat::Toon => "application/x-toon",
            SerializationFormat::Protobuf => "application/x-protobuf",
            SerializationFormat::MessagePack => "application/x-msgpack",
        }
    }

    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            SerializationFormat::Json => "json",
            SerializationFormat::Toon => "toon",
            SerializationFormat::Protobuf => "proto",
            SerializationFormat::MessagePack => "msgpack",
        }
    }
}

/// Gossip protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipConfig {
    /// Protocol version to use
    pub protocol_version: String,
    /// Ping interval in milliseconds
    pub ping_interval_ms: u64,
    /// Ping timeout in milliseconds
    pub ping_timeout_ms: u64,
    /// Indirect ping count (number of nodes to ask)
    pub indirect_ping_count: usize,
    /// Suspicion timeout multiplier
    pub suspicion_multiplier: u32,
    /// Maximum gossip messages per interval
    pub max_gossip_messages: usize,
    /// Gossip fanout (number of nodes to gossip to)
    pub gossip_fanout: usize,
    /// Serialization format to use
    pub serialization_format: SerializationFormat,
    /// Fallback serialization formats (in order of preference)
    pub fallback_formats: Vec<SerializationFormat>,
    /// Enable protocol negotiation
    pub enable_protocol_negotiation: bool,
    /// Enable compression
    pub enable_compression: bool,
}

impl Default for GossipConfig {
    fn default() -> Self {
        Self {
            protocol_version: "1.0".to_string(),
            ping_interval_ms: 1000, // 1 second
            ping_timeout_ms: 500,   // 500ms
            indirect_ping_count: 3,
            suspicion_multiplier: 5,
            max_gossip_messages: 10,
            gossip_fanout: 3,
            serialization_format: SerializationFormat::Json,
            fallback_formats: vec![
                SerializationFormat::Toon,
                SerializationFormat::MessagePack,
                SerializationFormat::Protobuf,
            ],
            enable_protocol_negotiation: true,
            enable_compression: true,
        }
    }
}

/// Service information for discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service identifier
    pub id: Uuid,
    /// Service name
    pub name: String,
    /// Service type/category
    pub service_type: String,
    /// Node hosting this service
    pub node_id: Uuid,
    /// Service endpoint (URL, socket, etc.)
    pub endpoint: String,
    /// Service metadata (capabilities, version, etc.)
    pub metadata: serde_json::Value,
    /// Service health status
    pub health: ServiceHealth,
    /// Service load metrics
    pub load_metrics: LoadMetrics,
    /// Last health check timestamp
    pub last_health_check: SystemTime,
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceHealth {
    /// Service is healthy
    Healthy,
    /// Service is unhealthy
    Unhealthy,
    /// Service is starting up
    Starting,
    /// Service is shutting down
    ShuttingDown,
    /// Service health is unknown
    Unknown,
}

/// Load metrics for a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadMetrics {
    /// CPU usage percentage (0-100)
    pub cpu_usage: f32,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Active connections
    pub active_connections: u32,
    /// Request rate (requests per second)
    pub request_rate: f32,
    /// Error rate (errors per second)
    pub error_rate: f32,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f32,
}

impl Default for LoadMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0,
            active_connections: 0,
            request_rate: 0.0,
            error_rate: 0.0,
            avg_response_time_ms: 0.0,
        }
    }
}

/// Service discovery request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceDiscoveryRequest {
    /// Register a service
    RegisterService(ServiceInfo),
    /// Unregister a service
    UnregisterService(Uuid),
    /// Update service health
    UpdateServiceHealth {
        service_id: Uuid,
        health: ServiceHealth,
    },
    /// Update service load metrics
    UpdateServiceLoad {
        service_id: Uuid,
        load_metrics: LoadMetrics,
    },
    /// Query services by type
    QueryServices {
        service_type: String,
        healthy_only: bool,
        max_results: Option<usize>,
    },
    /// Get service by ID
    GetService(Uuid),
}

/// Service discovery response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceDiscoveryResponse {
    /// Service registration acknowledged
    ServiceRegistered(Uuid),
    /// Service unregistration acknowledged
    ServiceUnregistered(Uuid),
    /// Service health updated
    ServiceHealthUpdated(Uuid),
    /// Service load updated
    ServiceLoadUpdated(Uuid),
    /// List of matching services
    ServicesFound(Vec<ServiceInfo>),
    /// Specific service information
    ServiceInfo(ServiceInfo),
    /// Error response
    Error { request_type: String, error: String },
}
