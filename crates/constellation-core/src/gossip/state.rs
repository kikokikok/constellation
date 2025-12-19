//! Gossip protocol state management

use crate::gossip::models::{GossipConfig, Membership, Node};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Shared state for gossip protocol
#[derive(Debug, Clone)]
pub struct GossipState {
    /// Local node information
    pub local_node: Node,
    /// Cluster membership information
    pub membership: Arc<RwLock<Membership>>,
    /// Protocol configuration
    pub config: GossipConfig,
    /// Protocol statistics
    pub stats: Arc<RwLock<GossipStats>>,
    /// Service discovery state
    pub service_state: Arc<RwLock<ServiceState>>,
}

/// Protocol statistics
#[derive(Debug, Clone)]
pub struct GossipStats {
    /// Number of ping messages sent
    pub pings_sent: u64,
    /// Number of ping messages received
    pub pings_received: u64,
    /// Number of ack messages sent
    pub acks_sent: u64,
    /// Number of ack messages received
    pub acks_received: u64,
    /// Number of ping-req messages sent
    pub ping_reqs_sent: u64,
    /// Number of ping-req messages received
    pub ping_reqs_received: u64,
    /// Number of suspicion messages sent
    pub suspicions_sent: u64,
    /// Number of suspicion messages received
    pub suspicions_received: u64,
    /// Number of membership updates sent
    pub membership_updates_sent: u64,
    /// Number of membership updates received
    pub membership_updates_received: u64,
    /// Number of service updates sent
    pub service_updates_sent: u64,
    /// Number of service updates received
    pub service_updates_received: u64,
    /// Number of failed serializations
    pub serialization_failures: u64,
    /// Number of failed deserializations
    pub deserialization_failures: u64,
    /// Number of protocol negotiation failures
    pub negotiation_failures: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Last statistics reset time
    pub last_reset: SystemTime,
}

/// Service discovery state
#[derive(Debug, Default, Clone)]
pub struct ServiceState {
    /// Registered services by service ID
    pub services: HashMap<Uuid, ServiceEntry>,
    /// Service queries cache
    pub query_cache: HashMap<String, CachedQuery>,
    /// Service load metrics
    pub load_metrics: HashMap<Uuid, LoadMetrics>,
}

/// Service entry in the state
#[derive(Debug, Clone)]
pub struct ServiceEntry {
    /// Service information
    pub service: crate::gossip::models::ServiceInfo,
    /// Last health check time
    pub last_health_check: SystemTime,
    /// Health status
    pub healthy: bool,
    /// Number of consecutive failures
    pub consecutive_failures: u32,
}

/// Cached query result
#[derive(Debug, Clone)]
pub struct CachedQuery {
    /// Query result
    pub result: Vec<Uuid>,
    /// Time when the query was cached
    pub cached_at: SystemTime,
    /// Cache TTL
    pub ttl: Duration,
}

/// Load metrics for a service (re-export from models)
pub type LoadMetrics = super::models::LoadMetrics;

impl GossipState {
    /// Create a new gossip state
    pub fn new(local_node: Node, config: GossipConfig) -> Self {
        let membership = Membership::new(local_node.clone());

        Self {
            local_node,
            membership: Arc::new(RwLock::new(membership)),
            config,
            stats: Arc::new(RwLock::new(GossipStats::default())),
            service_state: Arc::new(RwLock::new(ServiceState::default())),
        }
    }

    /// Update protocol statistics
    pub fn update_stats<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut GossipStats),
    {
        if let Ok(mut stats) = self.stats.write() {
            update_fn(&mut stats);
        }
    }

    /// Get a copy of current statistics
    pub fn get_stats(&self) -> GossipStats {
        self.stats.read().unwrap().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.write() {
            *stats = GossipStats::default();
            stats.last_reset = SystemTime::now();
        }
    }

    /// Add a node to membership
    pub fn add_node(&self, node: Node) {
        if let Ok(mut membership) = self.membership.write() {
            membership.add_node(node);
        }
    }

    /// Remove a node from membership
    pub fn remove_node(&self, node_id: Uuid) {
        if let Ok(mut membership) = self.membership.write() {
            membership.remove_node(&node_id);
        }
    }

    /// Update node state
    pub fn update_node_state(&self, node_id: Uuid, state: crate::gossip::models::NodeState) {
        if let Ok(mut membership) = self.membership.write() {
            membership.update_node_state(&node_id, state);
        }
    }

    /// Get current membership
    pub fn get_membership(&self) -> Membership {
        self.membership.read().unwrap().clone()
    }

    /// Register a service
    pub fn register_service(&self, service: crate::gossip::models::ServiceInfo) {
        if let Ok(mut state) = self.service_state.write() {
            let entry = ServiceEntry {
                service: service.clone(),
                last_health_check: SystemTime::now(),
                healthy: true,
                consecutive_failures: 0,
            };
            state.services.insert(service.id, entry);
        }
    }

    /// Unregister a service
    pub fn unregister_service(&self, service_id: Uuid) {
        if let Ok(mut state) = self.service_state.write() {
            state.services.remove(&service_id);
            state.load_metrics.remove(&service_id);
        }
    }

    /// Update service health
    pub fn update_service_health(&self, service_id: Uuid, healthy: bool) {
        if let Ok(mut state) = self.service_state.write()
            && let Some(entry) = state.services.get_mut(&service_id)
        {
            entry.last_health_check = SystemTime::now();
            if healthy {
                entry.healthy = true;
                entry.consecutive_failures = 0;
            } else {
                entry.consecutive_failures += 1;
                if entry.consecutive_failures >= 3 {
                    entry.healthy = false;
                }
            }
        }
    }

    /// Update service load metrics
    pub fn update_load_metrics(&self, service_id: Uuid, metrics: LoadMetrics) {
        if let Ok(mut state) = self.service_state.write() {
            state.load_metrics.insert(service_id, metrics);
        }
    }

    /// Cache a query result
    pub fn cache_query(&self, query: String, result: Vec<Uuid>, ttl: Duration) {
        if let Ok(mut state) = self.service_state.write() {
            let cached = CachedQuery {
                result,
                cached_at: SystemTime::now(),
                ttl,
            };
            state.query_cache.insert(query, cached);
        }
    }

    /// Get cached query result
    pub fn get_cached_query(&self, query: &str) -> Option<Vec<Uuid>> {
        if let Ok(state) = self.service_state.read()
            && let Some(cached) = state.query_cache.get(query)
            && SystemTime::now()
                .duration_since(cached.cached_at)
                .unwrap_or(Duration::from_secs(0))
                < cached.ttl
        {
            return Some(cached.result.clone());
        }
        None
    }

    /// Clean up expired cache entries
    pub fn cleanup_expired_cache(&self) {
        if let Ok(mut state) = self.service_state.write() {
            let now = SystemTime::now();
            state.query_cache.retain(|_, cached| {
                now.duration_since(cached.cached_at)
                    .unwrap_or(Duration::from_secs(0))
                    < cached.ttl
            });
        }
    }
}

impl Default for GossipStats {
    fn default() -> Self {
        Self {
            pings_sent: 0,
            pings_received: 0,
            acks_sent: 0,
            acks_received: 0,
            ping_reqs_sent: 0,
            ping_reqs_received: 0,
            suspicions_sent: 0,
            suspicions_received: 0,
            membership_updates_sent: 0,
            membership_updates_received: 0,
            service_updates_sent: 0,
            service_updates_received: 0,
            serialization_failures: 0,
            deserialization_failures: 0,
            negotiation_failures: 0,
            bytes_sent: 0,
            bytes_received: 0,
            last_reset: SystemTime::now(),
        }
    }
}
