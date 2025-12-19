//! Service discovery layer for gossip protocol

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, RwLock};
use tokio::time;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use rand::seq::SliceRandom;

use crate::communication::MessageBroker;
use crate::models::message_broker::A2AMessage;

use super::models::{
    GossipConfig, LoadMetrics, SerializationFormat, ServiceDiscoveryRequest,
    ServiceDiscoveryResponse, ServiceHealth, ServiceInfo,
};
use super::protocol::SimpleSwimGossipProtocol;

/// Service discovery manager
pub struct ServiceDiscovery {
    /// Gossip protocol instance
    gossip: Arc<SimpleSwimGossipProtocol>,
    /// Message broker for direct communication
    message_broker: Arc<dyn MessageBroker + Send + Sync>,
    /// Local services registry
    local_services: Arc<RwLock<HashMap<Uuid, ServiceInfo>>>,
    /// Remote services cache
    remote_services: Arc<RwLock<HashMap<Uuid, ServiceInfo>>>,
    /// Service queries cache
    query_cache: Arc<Mutex<QueryCache>>,
    /// Load balancing strategy
    load_balancer: LoadBalancer,
    /// Background task handles
    background_tasks: Vec<tokio::task::JoinHandle<()>>,
}

/// Query cache for service discovery
struct QueryCache {
    /// Cache entries
    entries: HashMap<String, CacheEntry>,
    /// Maximum cache size
    max_size: usize,
    /// Cache TTL in seconds
    ttl_seconds: u64,
}

/// Cache entry
struct CacheEntry {
    /// Cached services
    services: Vec<ServiceInfo>,
    /// Timestamp when cached
    cached_at: SystemTime,
    /// Query parameters
    query: QueryParams,
}

/// Query parameters
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct QueryParams {
    /// Service type
    service_type: String,
    /// Healthy only flag
    healthy_only: bool,
    /// Maximum results
    max_results: Option<usize>,
}

/// Load balancing strategies
#[derive(Debug, Clone, Default)]
pub enum LoadBalancer {
    /// Round-robin load balancing
    #[default]
    RoundRobin,
    /// Least connections load balancing
    LeastConnections,
    /// Weighted round-robin based on load metrics
    WeightedRoundRobin,
    /// Random selection
    Random,
    /// Sticky sessions (by client ID)
    Sticky(String),
}

impl ServiceDiscovery {
    /// Create a new service discovery instance
    pub async fn new(
        gossip: Arc<SimpleSwimGossipProtocol>,
        message_broker: Arc<dyn MessageBroker + Send + Sync>,
        load_balancer: Option<LoadBalancer>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            gossip,
            message_broker,
            local_services: Arc::new(RwLock::new(HashMap::new())),
            remote_services: Arc::new(RwLock::new(HashMap::new())),
            query_cache: Arc::new(Mutex::new(QueryCache::new(1000, 30))), // 1000 entries, 30 seconds TTL
            load_balancer: load_balancer.unwrap_or_default(),
            background_tasks: Vec::new(),
        })
    }

    /// Start the service discovery
    pub async fn start(&mut self) -> anyhow::Result<()> {
        info!("Starting service discovery");

        // Start background tasks
        self.start_background_tasks().await?;

        info!("Service discovery started successfully");
        Ok(())
    }

    /// Stop the service discovery
    pub async fn stop(&mut self) -> anyhow::Result<()> {
        info!("Stopping service discovery");

        // Cancel all background tasks
        for task in self.background_tasks.drain(..) {
            task.abort();
            let _ = task.await;
        }

        info!("Service discovery stopped");
        Ok(())
    }

    /// Start background tasks
    async fn start_background_tasks(&mut self) -> anyhow::Result<()> {
        // Start health check task
        let health_check_task = self.start_health_check_task();
        self.background_tasks.push(health_check_task);

        // Start cache cleanup task
        let cache_cleanup_task = self.start_cache_cleanup_task();
        self.background_tasks.push(cache_cleanup_task);

        // Start service synchronization task
        let service_sync_task = self.start_service_sync_task();
        self.background_tasks.push(service_sync_task);

        Ok(())
    }

    /// Start health check task
    fn start_health_check_task(&self) -> tokio::task::JoinHandle<()> {
        let local_services = self.local_services.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(10)); // Check every 10 seconds

            loop {
                interval.tick().await;

                let mut services_guard = local_services.write().await;
                let now = SystemTime::now();

                for service in services_guard.values_mut() {
                    // Update last health check timestamp
                    service.last_health_check = now;

                    // Simple health check - in a real implementation,
                    // this would actually check the service health
                    // For now, we'll just mark all local services as healthy
                    service.health = ServiceHealth::Healthy;
                }
            }
        })
    }

    /// Start cache cleanup task
    fn start_cache_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        let query_cache = self.query_cache.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60)); // Cleanup every minute

            loop {
                interval.tick().await;
                query_cache.lock().await.cleanup();
            }
        })
    }

    /// Start service synchronization task
    fn start_service_sync_task(&self) -> tokio::task::JoinHandle<()> {
        let gossip = self.gossip.clone();
        let remote_services = self.remote_services.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(30)); // Sync every 30 seconds

            loop {
                interval.tick().await;

                // Get services from gossip protocol
                let services = gossip.get_services().await;

                // Update remote services cache
                let mut remote_services_guard = remote_services.write().await;
                remote_services_guard.clear();

                for service in services {
                    remote_services_guard.insert(service.id, service);
                }
            }
        })
    }

    /// Register a local service
    pub async fn register_service(
        &self,
        name: String,
        service_type: String,
        endpoint: String,
        metadata: serde_json::Value,
    ) -> anyhow::Result<Uuid> {
        let service_id = Uuid::new_v4();

        // Get local node ID from gossip
        let membership = self.gossip.get_membership().await;
        let node_id = membership.local_node.id;

        // Create service info
        let service = ServiceInfo {
            id: service_id,
            name: name.clone(),
            service_type: service_type.clone(),
            node_id,
            endpoint,
            metadata,
            health: ServiceHealth::Healthy,
            load_metrics: LoadMetrics::default(),
            last_health_check: SystemTime::now(),
        };

        // Add to local services
        {
            let mut services_guard = self.local_services.write().await;
            services_guard.insert(service_id, service.clone());
        }

        // Register with gossip protocol
        self.gossip.register_service(service).await?;

        info!("Registered service: {} ({})", name, service_id);
        Ok(service_id)
    }

    /// Unregister a local service
    pub async fn unregister_service(&self, service_id: Uuid) -> anyhow::Result<()> {
        // Remove from local services
        {
            let mut services_guard = self.local_services.write().await;
            services_guard.remove(&service_id);
        }

        // Unregister from gossip protocol
        self.gossip.unregister_service(service_id).await?;

        info!("Unregistered service: {}", service_id);
        Ok(())
    }

    /// Discover services by type
    pub async fn discover_services(
        &self,
        service_type: &str,
        healthy_only: bool,
        max_results: Option<usize>,
    ) -> anyhow::Result<Vec<ServiceInfo>> {
        // Check cache first
        let cache_key = QueryParams {
            service_type: service_type.to_string(),
            healthy_only,
            max_results,
        };

        if let Some(cached) = self.query_cache.lock().await.get(&cache_key) {
            return Ok(cached);
        }

        // Get services from cache
        let services = {
            let remote_services_guard = self.remote_services.read().await;
            let local_services_guard = self.local_services.read().await;

            let mut all_services = Vec::new();

            // Add remote services
            all_services.extend(remote_services_guard.values().cloned());

            // Add local services
            all_services.extend(local_services_guard.values().cloned());

            // Filter by type
            all_services.retain(|s| s.service_type == service_type);

            // Filter by health if requested
            if healthy_only {
                all_services.retain(|s| s.health == ServiceHealth::Healthy);
            }

            // Apply max results
            if let Some(max) = max_results {
                all_services.truncate(max);
            }

            all_services
        };

        // Cache the results
        self.query_cache
            .lock()
            .await
            .set(cache_key, services.clone());

        Ok(services)
    }

    /// Get a service by ID
    pub async fn get_service(&self, service_id: Uuid) -> anyhow::Result<Option<ServiceInfo>> {
        // Check local services first
        {
            let local_services_guard = self.local_services.read().await;
            if let Some(service) = local_services_guard.get(&service_id) {
                return Ok(Some(service.clone()));
            }
        }

        // Check remote services
        {
            let remote_services_guard = self.remote_services.read().await;
            Ok(remote_services_guard.get(&service_id).cloned())
        }
    }

    /// Select a service using load balancing strategy
    pub async fn select_service(
        &self,
        service_type: &str,
        client_id: Option<&str>,
    ) -> anyhow::Result<Option<ServiceInfo>> {
        let services = self.discover_services(service_type, true, None).await?;

        if services.is_empty() {
            return Ok(None);
        }

        let selected = match &self.load_balancer {
            LoadBalancer::RoundRobin => self.round_robin_select(&services).await,
            LoadBalancer::LeastConnections => self.least_connections_select(&services).await,
            LoadBalancer::WeightedRoundRobin => self.weighted_round_robin_select(&services).await,
            LoadBalancer::Random => self.random_select(&services).await,
            LoadBalancer::Sticky(sticky_key) => {
                if let Some(client_id) = client_id {
                    self.sticky_select(&services, client_id).await
                } else {
                    self.round_robin_select(&services).await
                }
            }
        };

        Ok(selected)
    }

    /// Round-robin service selection
    async fn round_robin_select(&self, services: &[ServiceInfo]) -> Option<ServiceInfo> {
        static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

        let index = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        services.get(index % services.len()).cloned()
    }

    /// Least connections service selection
    async fn least_connections_select(&self, services: &[ServiceInfo]) -> Option<ServiceInfo> {
        services
            .iter()
            .min_by_key(|s| s.load_metrics.active_connections)
            .cloned()
    }

    /// Weighted round-robin service selection
    async fn weighted_round_robin_select(&self, services: &[ServiceInfo]) -> Option<ServiceInfo> {
        // Simple implementation - weight based on inverse of CPU usage
        // Lower CPU usage = higher weight
        let total_weight: f32 = services
            .iter()
            .map(|s| 100.0 - s.load_metrics.cpu_usage)
            .sum();

        if total_weight <= 0.0 {
            return services.first().cloned();
        }

        let random = rand::random::<f32>() * total_weight;
        let mut cumulative = 0.0;

        for service in services {
            let weight = 100.0 - service.load_metrics.cpu_usage;
            cumulative += weight;

            if random <= cumulative {
                return Some(service.clone());
            }
        }

        services.first().cloned()
    }

    /// Random service selection
    async fn random_select(&self, services: &[ServiceInfo]) -> Option<ServiceInfo> {
        use rand::prelude::*;

        #[allow(deprecated)]
        let mut rng = rand::thread_rng();
        services.choose(&mut rng).cloned()
    }

    /// Sticky session service selection
    async fn sticky_select(
        &self,
        services: &[ServiceInfo],
        client_id: &str,
    ) -> Option<ServiceInfo> {
        // Simple hash-based sticky selection
        let hash = client_id.chars().map(|c| c as u32).sum::<u32>() as usize;
        services.get(hash % services.len()).cloned()
    }

    /// Update service load metrics
    pub async fn update_service_load(
        &self,
        service_id: Uuid,
        load_metrics: LoadMetrics,
    ) -> anyhow::Result<()> {
        // Update local service if it exists
        {
            let mut local_services_guard = self.local_services.write().await;
            if let Some(service) = local_services_guard.get_mut(&service_id) {
                service.load_metrics = load_metrics.clone();
            }
        }

        // TODO: Broadcast load metrics update via gossip
        // This would require extending the gossip protocol

        Ok(())
    }

    /// Handle service discovery request
    pub async fn handle_request(
        &self,
        request: ServiceDiscoveryRequest,
    ) -> anyhow::Result<ServiceDiscoveryResponse> {
        match request {
            ServiceDiscoveryRequest::RegisterService(service) => {
                match self
                    .register_service(
                        service.name,
                        service.service_type,
                        service.endpoint,
                        service.metadata,
                    )
                    .await
                {
                    Ok(service_id) => Ok(ServiceDiscoveryResponse::ServiceRegistered(service_id)),
                    Err(e) => Ok(ServiceDiscoveryResponse::Error {
                        request_type: "RegisterService".to_string(),
                        error: e.to_string(),
                    }),
                }
            }
            ServiceDiscoveryRequest::UnregisterService(service_id) => {
                match self.unregister_service(service_id).await {
                    Ok(_) => Ok(ServiceDiscoveryResponse::ServiceUnregistered(service_id)),
                    Err(e) => Ok(ServiceDiscoveryResponse::Error {
                        request_type: "UnregisterService".to_string(),
                        error: e.to_string(),
                    }),
                }
            }
            ServiceDiscoveryRequest::UpdateServiceHealth { service_id, health } => {
                // Update local service health
                {
                    let mut local_services_guard = self.local_services.write().await;
                    if let Some(service) = local_services_guard.get_mut(&service_id) {
                        service.health = health;
                    }
                }

                Ok(ServiceDiscoveryResponse::ServiceHealthUpdated(service_id))
            }
            ServiceDiscoveryRequest::UpdateServiceLoad {
                service_id,
                load_metrics,
            } => match self.update_service_load(service_id, load_metrics).await {
                Ok(_) => Ok(ServiceDiscoveryResponse::ServiceLoadUpdated(service_id)),
                Err(e) => Ok(ServiceDiscoveryResponse::Error {
                    request_type: "UpdateServiceLoad".to_string(),
                    error: e.to_string(),
                }),
            },
            ServiceDiscoveryRequest::QueryServices {
                service_type,
                healthy_only,
                max_results,
            } => {
                match self
                    .discover_services(&service_type, healthy_only, max_results)
                    .await
                {
                    Ok(services) => Ok(ServiceDiscoveryResponse::ServicesFound(services)),
                    Err(e) => Ok(ServiceDiscoveryResponse::Error {
                        request_type: "QueryServices".to_string(),
                        error: e.to_string(),
                    }),
                }
            }
            ServiceDiscoveryRequest::GetService(service_id) => {
                match self.get_service(service_id).await {
                    Ok(Some(service)) => Ok(ServiceDiscoveryResponse::ServiceInfo(service)),
                    Ok(None) => Ok(ServiceDiscoveryResponse::Error {
                        request_type: "GetService".to_string(),
                        error: format!("Service {} not found", service_id),
                    }),
                    Err(e) => Ok(ServiceDiscoveryResponse::Error {
                        request_type: "GetService".to_string(),
                        error: e.to_string(),
                    }),
                }
            }
        }
    }
}

impl QueryCache {
    /// Create a new query cache
    fn new(max_size: usize, ttl_seconds: u64) -> Self {
        Self {
            entries: HashMap::new(),
            max_size,
            ttl_seconds,
        }
    }

    /// Get cached services for a query
    fn get(&self, query: &QueryParams) -> Option<Vec<ServiceInfo>> {
        let key = Self::query_to_key(query);

        self.entries.get(&key).and_then(|entry| {
            let now = SystemTime::now();
            let elapsed = now
                .duration_since(entry.cached_at)
                .unwrap_or(Duration::from_secs(0));

            if elapsed < Duration::from_secs(self.ttl_seconds) {
                Some(entry.services.clone())
            } else {
                None // Entry expired
            }
        })
    }

    /// Cache services for a query
    fn set(&mut self, query: QueryParams, services: Vec<ServiceInfo>) {
        // Clean up if cache is full
        if self.entries.len() >= self.max_size {
            self.cleanup();
        }

        let key = Self::query_to_key(&query);
        let entry = CacheEntry {
            services,
            cached_at: SystemTime::now(),
            query,
        };

        self.entries.insert(key, entry);
    }

    /// Clean up expired cache entries
    fn cleanup(&mut self) {
        let now = SystemTime::now();
        let ttl = Duration::from_secs(self.ttl_seconds);

        self.entries.retain(|_, entry| {
            let elapsed = now
                .duration_since(entry.cached_at)
                .unwrap_or(Duration::from_secs(0));
            elapsed < ttl
        });

        // If still too large, remove oldest entries
        if self.entries.len() > self.max_size {
            let mut entries: Vec<_> = self.entries.drain().collect();
            entries.sort_by_key(|(_, entry)| entry.cached_at);
            entries.truncate(self.max_size);
            self.entries = entries.into_iter().collect();
        }
    }

    /// Convert query parameters to cache key
    fn query_to_key(query: &QueryParams) -> String {
        format!(
            "{}:{}:{}",
            query.service_type,
            query.healthy_only,
            query.max_results.unwrap_or(0)
        )
    }
}
