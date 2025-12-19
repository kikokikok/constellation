//! Connection pooling for agent SDK

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::time;
use tracing::{debug, error, info, warn};

use crate::config::AgentConfig;
use crate::error::{AgentError, AgentResult};

use constellation_core::communication::CommunicationFramework;
use constellation_core::message_broker::{
    AgentSession, IggyBrokerConfig, IggyMessageBroker, IggyMessageBrokerBuilder,
};

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections in the pool
    pub max_connections: usize,
    /// Minimum number of connections to keep alive
    pub min_connections: usize,
    /// Connection idle timeout (seconds)
    pub idle_timeout_seconds: u64,
    /// Connection validation interval (seconds)
    pub validation_interval_seconds: u64,
    /// Maximum connection lifetime (seconds)
    pub max_lifetime_seconds: u64,
    /// Connection acquisition timeout (seconds)
    pub acquisition_timeout_seconds: u64,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            idle_timeout_seconds: 300, // 5 minutes
            validation_interval_seconds: 30,
            max_lifetime_seconds: 3600, // 1 hour
            acquisition_timeout_seconds: 30,
        }
    }
}

/// Pooled connection
struct PooledConnection {
    /// Connection ID
    id: String,
    /// Message broker instance
    broker: Arc<IggyMessageBroker>,
    /// Communication framework
    framework: Arc<CommunicationFramework<IggyMessageBroker>>,
    /// Created timestamp
    created_at: Instant,
    /// Last used timestamp
    last_used_at: Instant,
    /// Is connection valid
    is_valid: bool,
}

impl PooledConnection {
    /// Create a new pooled connection
    async fn new(config: &AgentConfig) -> AgentResult<Self> {
        let connection_id = uuid::Uuid::new_v4().to_string();
        info!("Creating new connection: {}", connection_id);

        // Create Iggy broker configuration
        let _broker_config = IggyBrokerConfig {
            iggy_server_address: config.broker_url.clone(),
            iggy_username: config.broker_username.clone(),
            iggy_password: config.broker_password.clone(),
            stream_name: config.stream_name.clone(),
            topic_name: config.topic_name.clone(),
            partitions_count: 4,
            message_retention_period: config.message_retention_period,
            max_batch_size: 1000,
            session_timeout_seconds: config.session_timeout_seconds,
        };

        // Create Iggy message broker
        let broker = IggyMessageBrokerBuilder::new()
            .server_address(config.broker_url.clone())
            .credentials(
                config.broker_username.clone(),
                config.broker_password.clone(),
            )
            .stream_name(config.stream_name.clone())
            .topic_name(config.topic_name.clone())
            .partitions_count(4)
            .message_retention(config.message_retention_period)
            .max_batch_size(1000)
            .session_timeout(config.session_timeout_seconds)
            .build()
            .await
            .map_err(|e| AgentError::Connection(format!("Failed to create broker: {}", e)))?;

        let broker = Arc::new(broker);

        // Create communication framework
        let framework = Arc::new(CommunicationFramework::new(broker.clone()));

        // Register agent session
        let session = AgentSession::new(
            config.agent_id.clone(),
            "token".to_string(), // TODO: Use proper authentication
            "sdk".to_string(),
            None,
        );

        broker
            .register_session(session)
            .await
            .map_err(|e| AgentError::Connection(format!("Failed to register session: {}", e)))?;

        Ok(Self {
            id: connection_id,
            broker,
            framework,
            created_at: Instant::now(),
            last_used_at: Instant::now(),
            is_valid: true,
        })
    }

    /// Validate the connection
    async fn validate(&mut self) -> bool {
        debug!("Validating connection: {}", self.id);

        // Check if connection is still alive by trying to get session
        match self.broker.get_session("test").await {
            Ok(_) => {
                self.is_valid = true;
                true
            }
            Err(e) => {
                warn!("Connection {} validation failed: {}", self.id, e);
                self.is_valid = false;
                false
            }
        }
    }

    /// Mark connection as used
    fn mark_used(&mut self) {
        self.last_used_at = Instant::now();
    }

    /// Check if connection is idle
    fn is_idle(&self, idle_timeout: Duration) -> bool {
        self.last_used_at.elapsed() > idle_timeout
    }

    /// Check if connection has expired
    fn is_expired(&self, max_lifetime: Duration) -> bool {
        self.created_at.elapsed() > max_lifetime
    }
}

/// Connection pool
pub struct ConnectionPool {
    /// Pool configuration
    config: ConnectionPoolConfig,
    /// Agent configuration
    agent_config: AgentConfig,
    /// Available connections
    available: Arc<Mutex<Vec<PooledConnection>>>,
    /// In-use connections
    in_use: Arc<Mutex<Vec<PooledConnection>>>,
    /// Semaphore for limiting concurrent connections
    semaphore: Arc<Semaphore>,
    /// Background task handle
    background_task: Option<tokio::task::JoinHandle<()>>,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub async fn new(
        agent_config: AgentConfig,
        pool_config: Option<ConnectionPoolConfig>,
    ) -> AgentResult<Self> {
        let config = pool_config.unwrap_or_default();

        info!(
            "Creating connection pool for agent '{}' with {} max connections",
            agent_config.agent_id, config.max_connections
        );

        let pool = Self {
            config: config.clone(),
            agent_config,
            available: Arc::new(Mutex::new(Vec::new())),
            in_use: Arc::new(Mutex::new(Vec::new())),
            semaphore: Arc::new(Semaphore::new(config.max_connections)),
            background_task: None,
        };

        // Initialize minimum connections
        pool.initialize_min_connections().await?;

        Ok(pool)
    }

    /// Initialize minimum connections
    async fn initialize_min_connections(&self) -> AgentResult<()> {
        let mut available = self.available.lock().await;

        for i in 0..self.config.min_connections {
            match PooledConnection::new(&self.agent_config).await {
                Ok(connection) => {
                    info!(
                        "Initialized connection {}/{}",
                        i + 1,
                        self.config.min_connections
                    );
                    available.push(connection);
                }
                Err(e) => {
                    error!("Failed to initialize connection {}: {}", i + 1, e);
                    // Continue trying to create other connections
                }
            }
        }

        Ok(())
    }

    /// Get a connection from the pool
    pub async fn get_connection(&self) -> AgentResult<PooledConnectionHandle> {
        // Try to acquire semaphore with timeout
        let _permit = time::timeout(
            Duration::from_secs(self.config.acquisition_timeout_seconds),
            self.semaphore.acquire(),
        )
        .await
        .map_err(|_| AgentError::Connection("Connection acquisition timeout".to_string()))?
        .map_err(|e| AgentError::Connection(format!("Semaphore error: {}", e)))?;

        let mut available = self.available.lock().await;
        let mut in_use = self.in_use.lock().await;

        // Try to find a valid connection
        while let Some(mut connection) = available.pop() {
            // Check if connection is still valid
            if connection.is_valid {
                connection.mark_used();
                in_use.push(connection);

                return Ok(PooledConnectionHandle {
                    connection_id: in_use.last().unwrap().id.clone(),
                    broker: in_use.last().unwrap().broker.clone(),
                    framework: in_use.last().unwrap().framework.clone(),
                    pool: self.clone(),
                });
            }
        }

        // No valid connection available, create a new one
        drop(available);
        drop(in_use);

        let mut connection = PooledConnection::new(&self.agent_config).await?;
        connection.mark_used();

        let mut in_use = self.in_use.lock().await;
        in_use.push(connection);

        Ok(PooledConnectionHandle {
            connection_id: in_use.last().unwrap().id.clone(),
            broker: in_use.last().unwrap().broker.clone(),
            framework: in_use.last().unwrap().framework.clone(),
            pool: self.clone(),
        })
    }

    /// Return a connection to the pool
    async fn return_connection(&self, connection_id: String) {
        let mut in_use = self.in_use.lock().await;
        let mut available = self.available.lock().await;

        // Find the connection in the in-use list
        if let Some(index) = in_use.iter().position(|c| c.id == connection_id) {
            let mut connection = in_use.remove(index);
            connection.mark_used();
            available.push(connection);
        }
    }

    /// Start background maintenance tasks
    pub async fn start_maintenance(&mut self) {
        let available = self.available.clone();
        let in_use = self.in_use.clone();
        let config = self.config.clone();
        let agent_config = self.agent_config.clone();

        let task = tokio::spawn(async move {
            let validation_interval = Duration::from_secs(config.validation_interval_seconds);
            let idle_timeout = Duration::from_secs(config.idle_timeout_seconds);
            let max_lifetime = Duration::from_secs(config.max_lifetime_seconds);

            loop {
                tokio::time::sleep(validation_interval).await;

                // Validate and clean up connections
                let mut available_lock = available.lock().await;
                let mut in_use_lock = in_use.lock().await;

                // Validate available connections
                let mut i = 0;
                while i < available_lock.len() {
                    let current_len = available_lock.len();
                    let should_remove = {
                        let connection = &mut available_lock[i];

                        // Check if connection has expired
                        if connection.is_expired(max_lifetime) {
                            debug!("Removing expired connection: {}", connection.id);
                            true
                        }
                        // Check if connection is idle and should be removed
                        else if connection.is_idle(idle_timeout)
                            && current_len > config.min_connections
                        {
                            debug!("Removing idle connection: {}", connection.id);
                            true
                        }
                        // Validate connection health
                        else if !connection.validate().await {
                            debug!("Removing invalid connection: {}", connection.id);
                            true
                        } else {
                            false
                        }
                    };

                    if should_remove {
                        available_lock.remove(i);
                    } else {
                        i += 1;
                    }
                }

                // Validate in-use connections
                let mut i = 0;
                while i < in_use_lock.len() {
                    let connection = &mut in_use_lock[i];

                    // Check if connection has expired
                    if connection.is_expired(max_lifetime) {
                        warn!("In-use connection expired: {}", connection.id);
                        in_use_lock.remove(i);
                        continue;
                    }

                    i += 1;
                }

                // Ensure minimum connections
                while available_lock.len() < config.min_connections {
                    match PooledConnection::new(&agent_config).await {
                        Ok(connection) => {
                            info!("Added new connection to maintain minimum pool size");
                            available_lock.push(connection);
                        }
                        Err(e) => {
                            error!("Failed to add connection to pool: {}", e);
                            break;
                        }
                    }
                }
            }
        });

        self.background_task = Some(task);
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> PoolStats {
        let available = self.available.lock().await;
        let in_use = self.in_use.lock().await;

        PoolStats {
            total_connections: available.len() + in_use.len(),
            available_connections: available.len(),
            in_use_connections: in_use.len(),
            max_connections: self.config.max_connections,
            min_connections: self.config.min_connections,
        }
    }
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            agent_config: self.agent_config.clone(),
            available: self.available.clone(),
            in_use: self.in_use.clone(),
            semaphore: self.semaphore.clone(),
            background_task: None, // Don't clone background task
        }
    }
}

/// Handle for a pooled connection
pub struct PooledConnectionHandle {
    /// Connection ID
    connection_id: String,
    /// Message broker instance
    pub broker: Arc<IggyMessageBroker>,
    /// Communication framework
    pub framework: Arc<CommunicationFramework<IggyMessageBroker>>,
    /// Reference to the pool
    pool: ConnectionPool,
}

impl PooledConnectionHandle {
    /// Get the connection ID
    pub fn connection_id(&self) -> &str {
        &self.connection_id
    }
}

impl Drop for PooledConnectionHandle {
    fn drop(&mut self) {
        let pool = self.pool.clone();
        let connection_id = self.connection_id.clone();

        // Return connection to pool asynchronously
        tokio::spawn(async move {
            pool.return_connection(connection_id).await;
        });
    }
}

/// Connection pool statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct PoolStats {
    /// Total number of connections
    pub total_connections: usize,
    /// Number of available connections
    pub available_connections: usize,
    /// Number of in-use connections
    pub in_use_connections: usize,
    /// Maximum connections allowed
    pub max_connections: usize,
    /// Minimum connections to maintain
    pub min_connections: usize,
}
