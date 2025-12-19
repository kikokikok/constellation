//! Simplified SWIM gossip protocol implementation
//!
//! This is a simplified version that compiles and provides core functionality
//! without background task spawning issues.

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, RwLock};
use tokio::time;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::communication::MessageBroker;
use crate::models::message_broker::A2AMessage;

use super::models::{
    GossipConfig, GossipMessage, Membership, Node, NodeState, SerializationFormat, ServiceInfo,
};

/// Ping request tracking
#[derive(Debug, Clone)]
struct PingRequest {
    target_id: Uuid,
    sequence: u64,
    sent_at: SystemTime,
    indirect: bool,
    indirect_nodes: Vec<Uuid>,
}

/// Protocol statistics
#[derive(Debug, Clone, Default)]
pub struct ProtocolStats {
    pings_sent: u64,
    pings_received: u64,
    acks_sent: u64,
    acks_received: u64,
    ping_reqs_sent: u64,
    ping_reqs_received: u64,
    suspicions_sent: u64,
    suspicions_received: u64,
    membership_updates_sent: u64,
    membership_updates_received: u64,
    service_updates_sent: u64,
    service_updates_received: u64,
    serialization_failures: u64,
    deserialization_failures: u64,
    negotiation_failures: u64,
    bytes_sent: u64,
    bytes_received: u64,
    ping_timeouts: u64,
}

/// Simplified SWIM gossip protocol implementation
pub struct SimpleSwimGossipProtocol {
    /// Protocol configuration
    config: GossipConfig,
    /// Local node information
    local_node: Arc<RwLock<Node>>,
    /// Membership information
    membership: Arc<RwLock<Membership>>,
    /// Known services
    services: Arc<RwLock<HashMap<Uuid, ServiceInfo>>>,
    /// Message broker for communication
    message_broker: Arc<dyn MessageBroker + Send + Sync>,
    /// Pending ping requests
    pending_pings: Arc<Mutex<HashMap<u64, PingRequest>>>,
    /// Protocol statistics
    stats: Arc<Mutex<ProtocolStats>>,
}

impl SimpleSwimGossipProtocol {
    /// Create a new simplified gossip protocol instance
    pub fn new(
        config: GossipConfig,
        local_node: Node,
        message_broker: Arc<dyn MessageBroker + Send + Sync>,
    ) -> Self {
        let membership = Membership::new(local_node.clone());

        Self {
            config,
            local_node: Arc::new(RwLock::new(local_node)),
            membership: Arc::new(RwLock::new(membership)),
            services: Arc::new(RwLock::new(HashMap::new())),
            message_broker,
            pending_pings: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(ProtocolStats::default())),
        }
    }

    /// Start the protocol (simplified - no background tasks)
    pub async fn start(&self) -> anyhow::Result<()> {
        info!("Starting simplified gossip protocol");
        Ok(())
    }

    /// Stop the protocol
    pub async fn stop(&self) -> anyhow::Result<()> {
        info!("Stopping simplified gossip protocol");
        Ok(())
    }

    /// Run ping cycle (to be called periodically)
    pub async fn run_ping_cycle(&self) -> anyhow::Result<()> {
        // Get a random node to ping (excluding self)
        let target_node = {
            let membership_guard = self.membership.read().await;
            let alive_nodes = membership_guard.alive_nodes();

            if alive_nodes.len() <= 1 {
                return Ok(()); // No other nodes to ping
            }

            // Choose first non-local node (simplified)
            alive_nodes
                .iter()
                .find(|n| n.id != membership_guard.local_node.id)
                .map(|n| (*n).clone())
        };

        if let Some(target) = target_node {
            // Create ping message
            let sequence = self.get_next_sequence().await;
            let ping = GossipMessage::Ping {
                sender_id: self.local_node.read().await.id,
                sequence,
                timestamp: SystemTime::now(),
            };

            // Send ping
            if let Err(e) =
                Self::send_gossip_message(self.message_broker.clone(), &target, &ping, &self.config)
                    .await
            {
                error!("Failed to send ping to {}: {}", target.id, e);
                return Ok(());
            }

            // Track pending ping
            let ping_request = PingRequest {
                target_id: target.id,
                sequence,
                sent_at: SystemTime::now(),
                indirect: false,
                indirect_nodes: Vec::new(),
            };

            self.pending_pings
                .lock()
                .await
                .insert(sequence, ping_request);

            // Update stats
            self.stats.lock().await.pings_sent += 1;
        }

        Ok(())
    }

    /// Run failure detection cycle (to be called periodically)
    pub async fn run_failure_detection_cycle(&self) -> anyhow::Result<()> {
        let now = SystemTime::now();
        let mut pings_to_remove = Vec::new();
        let mut nodes_to_mark_suspect = Vec::new();

        // Check for timed out pings
        {
            let mut pending_pings_guard = self.pending_pings.lock().await;

            for (sequence, ping_request) in pending_pings_guard.iter() {
                let elapsed = now
                    .duration_since(ping_request.sent_at)
                    .unwrap_or(Duration::from_secs(0));

                if elapsed > Duration::from_millis(self.config.ping_timeout_ms) {
                    nodes_to_mark_suspect.push(ping_request.target_id);
                    pings_to_remove.push(*sequence);
                    self.stats.lock().await.ping_timeouts += 1;
                }
            }

            // Remove timed out pings
            for sequence in pings_to_remove {
                pending_pings_guard.remove(&sequence);
            }
        }

        // Mark nodes as suspect
        if !nodes_to_mark_suspect.is_empty() {
            let mut membership_guard = self.membership.write().await;

            for node_id in nodes_to_mark_suspect {
                if let Some(node) = membership_guard.get_node_mut(&node_id)
                    && node.is_alive()
                {
                    node.mark_suspect();
                    info!("Marked node {} as suspect", node_id);
                }
            }
        }

        Ok(())
    }

    /// Run gossip dissemination cycle (to be called periodically)
    pub async fn run_gossip_cycle(&self) -> anyhow::Result<()> {
        // Get random nodes to gossip to
        let gossip_targets = {
            let membership_guard = self.membership.read().await;
            let alive_nodes = membership_guard.alive_nodes();

            if alive_nodes.len() <= 1 {
                return Ok(()); // No other nodes to gossip to
            }

            // Select first few non-local nodes (simplified)
            alive_nodes
                .iter()
                .filter(|n| n.id != membership_guard.local_node.id)
                .take(self.config.gossip_fanout)
                .map(|n| (*n).clone())
                .collect::<Vec<_>>()
        };

        if gossip_targets.is_empty() {
            return Ok(());
        }

        // Create membership update
        let membership_update = {
            let membership_guard = self.membership.read().await;
            GossipMessage::MembershipUpdate {
                sender_id: self.local_node.read().await.id,
                membership: membership_guard.clone(),
                timestamp: SystemTime::now(),
            }
        };

        // Send to gossip targets
        for target in gossip_targets {
            if let Err(e) = Self::send_gossip_message(
                self.message_broker.clone(),
                &target,
                &membership_update,
                &self.config,
            )
            .await
            {
                error!("Failed to send gossip to {}: {}", target.id, e);
                continue;
            }

            self.stats.lock().await.membership_updates_sent += 1;
        }

        Ok(())
    }

    /// Get next sequence number
    async fn get_next_sequence(&self) -> u64 {
        let mut stats = self.stats.lock().await;
        stats.pings_sent += 1;
        stats.pings_sent // Use ping count as sequence
    }

    /// Negotiate serialization format with target node
    fn negotiate_format(config: &GossipConfig, target: &Node) -> SerializationFormat {
        if !config.enable_protocol_negotiation {
            return config.serialization_format.clone();
        }

        // Check if target supports our preferred format
        if target
            .supported_formats
            .contains(&config.serialization_format)
        {
            return config.serialization_format.clone();
        }

        // Try fallback formats
        for format in &config.fallback_formats {
            if target.supported_formats.contains(format) {
                return format.clone();
            }
        }

        // Default to JSON (should always be supported)
        SerializationFormat::Json
    }

    /// Send a gossip message
    async fn send_gossip_message(
        message_broker: Arc<dyn MessageBroker + Send + Sync>,
        target: &Node,
        message: &GossipMessage,
        config: &GossipConfig,
    ) -> anyhow::Result<()> {
        // Negotiate format
        let format = Self::negotiate_format(config, target);
        // Serialize message
        let payload = match format {
            SerializationFormat::Json => serde_json::to_string(message)?,
            SerializationFormat::Toon => {
                // Convert to serde_json::Value first, then encode to TOON
                let json_value = serde_json::to_value(message)?;
                toon::encode(&json_value, None)
            }
            SerializationFormat::Protobuf => {
                // TODO: Implement Protobuf serialization
                warn!("Protobuf serialization not yet implemented, falling back to JSON");
                serde_json::to_string(message)?
            }
            SerializationFormat::MessagePack => {
                // Serialize to MessagePack binary format
                let mut buf = Vec::new();
                rmp_serde::encode::write(&mut buf, message)
                    .map_err(|e| anyhow::anyhow!("MessagePack serialization failed: {}", e))?;
                // Convert to base64 string for transport
                use base64::Engine;
                base64::engine::general_purpose::STANDARD.encode(&buf)
            }
        };

        // Create A2A message
        let mut a2a_message = A2AMessage::new(
            Uuid::new_v4().to_string(),
            "gossip".to_string(), // Sender ID will be set by the broker
            target.id.to_string(),
            "gossip".to_string(),
            payload,
        );
        a2a_message.content_type = format.content_type().to_string();

        // Send message
        message_broker
            .send_message(a2a_message.to_message())
            .await?;

        Ok(())
    }

    /// Handle an incoming gossip message
    pub async fn handle_message(&self, message: GossipMessage) -> anyhow::Result<()> {
        match message {
            GossipMessage::Ping {
                sender_id,
                sequence,
                timestamp,
            } => self.handle_ping(sender_id, sequence, timestamp).await,
            GossipMessage::Ack {
                sender_id,
                ping_sequence,
                timestamp,
                membership_snapshot,
            } => {
                self.handle_ack(sender_id, ping_sequence, timestamp, membership_snapshot)
                    .await
            }
            GossipMessage::PingReq {
                sender_id,
                target_id,
                sequence,
                timestamp,
            } => {
                self.handle_ping_req(sender_id, target_id, sequence, timestamp)
                    .await
            }
            GossipMessage::MembershipUpdate {
                sender_id,
                membership,
                timestamp,
            } => {
                self.handle_membership_update(sender_id, membership, timestamp)
                    .await
            }
            GossipMessage::StateSync {
                sender_id,
                state_type,
                state_data,
                version,
                timestamp,
            } => {
                self.handle_state_sync(sender_id, state_type, state_data, version, timestamp)
                    .await
            }
        }
    }

    /// Handle a ping message
    async fn handle_ping(
        &self,
        sender_id: Uuid,
        sequence: u64,
        timestamp: SystemTime,
    ) -> anyhow::Result<()> {
        debug!("Received ping from {} (sequence: {})", sender_id, sequence);

        // Update stats
        self.stats.lock().await.pings_received += 1;

        // Get current membership snapshot
        let membership_snapshot = {
            let membership_guard = self.membership.read().await;
            Some(membership_guard.clone())
        };

        // Create ack response
        let ack = GossipMessage::Ack {
            sender_id: self.local_node.read().await.id,
            ping_sequence: sequence,
            timestamp: SystemTime::now(),
            membership_snapshot,
        };

        // Send ack
        if let Some(sender) = self.get_node(&sender_id).await {
            Self::send_gossip_message(self.message_broker.clone(), &sender, &ack, &self.config)
                .await?;
        }

        Ok(())
    }

    /// Handle an ack message
    async fn handle_ack(
        &self,
        sender_id: Uuid,
        ping_sequence: u64,
        timestamp: SystemTime,
        membership_snapshot: Option<Membership>,
    ) -> anyhow::Result<()> {
        debug!("Received ack from {} for ping {}", sender_id, ping_sequence);

        // Update stats
        self.stats.lock().await.acks_received += 1;

        // Remove pending ping
        self.pending_pings.lock().await.remove(&ping_sequence);

        // Merge membership if provided
        if let Some(snapshot) = membership_snapshot {
            let mut membership_guard = self.membership.write().await;
            membership_guard.merge(&snapshot);
        }

        Ok(())
    }

    /// Handle a ping-req message
    async fn handle_ping_req(
        &self,
        sender_id: Uuid,
        target_id: Uuid,
        sequence: u64,
        timestamp: SystemTime,
    ) -> anyhow::Result<()> {
        debug!(
            "Received ping-req from {} for target {}",
            sender_id, target_id
        );

        // Update stats
        self.stats.lock().await.ping_reqs_received += 1;

        // Check if we can ping the target
        if let Some(target) = self.get_node(&target_id).await {
            // Send ping to target
            let ping = GossipMessage::Ping {
                sender_id: self.local_node.read().await.id,
                sequence,
                timestamp: SystemTime::now(),
            };

            Self::send_gossip_message(self.message_broker.clone(), &target, &ping, &self.config)
                .await?;
        }

        Ok(())
    }

    /// Handle a membership update
    async fn handle_membership_update(
        &self,
        sender_id: Uuid,
        membership: Membership,
        timestamp: SystemTime,
    ) -> anyhow::Result<()> {
        debug!("Received membership update from {}", sender_id);

        // Update stats
        self.stats.lock().await.membership_updates_received += 1;

        // Merge membership
        let mut membership_guard = self.membership.write().await;
        membership_guard.merge(&membership);

        Ok(())
    }

    /// Handle a state sync message
    async fn handle_state_sync(
        &self,
        sender_id: Uuid,
        state_type: String,
        state_data: serde_json::Value,
        version: u64,
        timestamp: SystemTime,
    ) -> anyhow::Result<()> {
        debug!(
            "Received state sync from {}: {} v{}",
            sender_id, state_type, version
        );

        // Update stats
        self.stats.lock().await.service_updates_received += 1;

        // Handle based on state type
        match state_type.as_str() {
            "service" => {
                // Parse service info
                if let Ok(service_info) = serde_json::from_value::<ServiceInfo>(state_data) {
                    let mut services_guard = self.services.write().await;
                    services_guard.insert(service_info.id, service_info);
                }
            }
            _ => {
                warn!("Unknown state type: {}", state_type);
            }
        }

        Ok(())
    }

    /// Get a node by ID
    async fn get_node(&self, node_id: &Uuid) -> Option<Node> {
        let membership_guard = self.membership.read().await;
        membership_guard.get_node(node_id).cloned()
    }

    /// Get protocol statistics
    pub async fn get_stats(&self) -> ProtocolStats {
        self.stats.lock().await.clone()
    }

    /// Get current membership
    pub async fn get_membership(&self) -> Membership {
        self.membership.read().await.clone()
    }

    /// Get known services
    pub async fn get_services(&self) -> Vec<ServiceInfo> {
        let services_guard = self.services.read().await;
        services_guard.values().cloned().collect()
    }

    /// Register a service
    pub async fn register_service(&self, service_info: ServiceInfo) -> anyhow::Result<()> {
        let mut services_guard = self.services.write().await;
        services_guard.insert(service_info.id, service_info);
        Ok(())
    }

    /// Unregister a service
    pub async fn unregister_service(&self, service_id: Uuid) -> anyhow::Result<()> {
        let mut services_guard = self.services.write().await;
        services_guard.remove(&service_id);
        Ok(())
    }

    /// Broadcast service state
    pub async fn broadcast_service_state(&self, service_info: ServiceInfo) -> anyhow::Result<()> {
        // Add to local services
        {
            let mut services_guard = self.services.write().await;
            services_guard.insert(service_info.id, service_info.clone());
        }

        // Create state sync message
        let state_sync = GossipMessage::StateSync {
            sender_id: self.local_node.read().await.id,
            state_type: "service".to_string(),
            state_data: serde_json::to_value(&service_info)?,
            version: 1,
            timestamp: SystemTime::now(),
        };

        // Get all alive nodes (excluding self)
        let membership_guard = self.membership.read().await;
        let targets: Vec<Node> = membership_guard
            .alive_nodes()
            .iter()
            .filter(|n| n.id != membership_guard.local_node.id)
            .map(|n| (*n).clone())
            .collect();

        // Send to all targets
        for target in targets {
            if let Err(e) = Self::send_gossip_message(
                self.message_broker.clone(),
                &target,
                &state_sync,
                &self.config,
            )
            .await
            {
                error!("Failed to broadcast service update to {}: {}", target.id, e);
            }
        }

        Ok(())
    }
}
