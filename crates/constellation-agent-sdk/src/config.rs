//! Configuration for Agent SDK

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::AgentResult;

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent ID
    pub agent_id: String,

    /// Message broker URL (e.g., "127.0.0.1:8090" for Iggy)
    pub broker_url: String,

    /// Broker username
    pub broker_username: String,

    /// Broker password
    pub broker_password: String,

    /// Stream name for Iggy
    pub stream_name: String,

    /// Topic name for agent messages
    pub topic_name: String,

    /// Default request timeout
    pub default_request_timeout: Duration,

    /// Default max retries for requests
    pub default_max_retries: u32,

    /// Default retry base delay
    pub default_retry_base_delay: Duration,

    /// Enable automatic reconnection
    pub auto_reconnect: bool,

    /// Reconnection delay
    pub reconnect_delay: Duration,

    /// Max reconnection attempts (0 = infinite)
    pub max_reconnect_attempts: u32,

    /// Enable message persistence
    pub enable_persistence: bool,

    /// Message retention period in seconds
    pub message_retention_period: u32,

    /// Session timeout in seconds
    pub session_timeout_seconds: u64,

    /// Enable metrics collection
    pub enable_metrics: bool,

    /// Enable tracing
    pub enable_tracing: bool,

    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_id: "unnamed_agent".to_string(),
            broker_url: "127.0.0.1:8090".to_string(),
            broker_username: "guest".to_string(),
            broker_password: "guest".to_string(),
            stream_name: "constellation".to_string(),
            topic_name: "agent_messages".to_string(),
            default_request_timeout: Duration::from_secs(30),
            default_max_retries: 3,
            default_retry_base_delay: Duration::from_secs(1),
            auto_reconnect: true,
            reconnect_delay: Duration::from_secs(5),
            max_reconnect_attempts: 10,
            enable_persistence: true,
            message_retention_period: 3600, // 1 hour
            session_timeout_seconds: 300,   // 5 minutes
            enable_metrics: true,
            enable_tracing: false,
            log_level: "info".to_string(),
        }
    }
}

impl AgentConfig {
    /// Create a new configuration with the given agent ID
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            ..Default::default()
        }
    }

    /// Set the agent ID
    pub fn with_agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = agent_id.into();
        self
    }

    /// Set the broker URL
    pub fn with_broker_url(mut self, url: impl Into<String>) -> Self {
        self.broker_url = url.into();
        self
    }

    /// Set broker credentials
    pub fn with_broker_credentials(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.broker_username = username.into();
        self.broker_password = password.into();
        self
    }

    /// Set stream name
    pub fn with_stream_name(mut self, name: impl Into<String>) -> Self {
        self.stream_name = name.into();
        self
    }

    /// Set topic name
    pub fn with_topic_name(mut self, name: impl Into<String>) -> Self {
        self.topic_name = name.into();
        self
    }

    /// Set default request timeout
    pub fn with_default_request_timeout(mut self, timeout: Duration) -> Self {
        self.default_request_timeout = timeout;
        self
    }

    /// Set default max retries
    pub fn with_default_max_retries(mut self, retries: u32) -> Self {
        self.default_max_retries = retries;
        self
    }

    /// Set default retry base delay
    pub fn with_default_retry_base_delay(mut self, delay: Duration) -> Self {
        self.default_retry_base_delay = delay;
        self
    }

    /// Enable or disable auto reconnection
    pub fn with_auto_reconnect(mut self, enable: bool) -> Self {
        self.auto_reconnect = enable;
        self
    }

    /// Set reconnection delay
    pub fn with_reconnect_delay(mut self, delay: Duration) -> Self {
        self.reconnect_delay = delay;
        self
    }

    /// Set max reconnection attempts
    pub fn with_max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.max_reconnect_attempts = attempts;
        self
    }

    /// Enable or disable message persistence
    pub fn with_persistence(mut self, enable: bool) -> Self {
        self.enable_persistence = enable;
        self
    }

    /// Set message retention period
    pub fn with_message_retention_period(mut self, seconds: u32) -> Self {
        self.message_retention_period = seconds;
        self
    }

    /// Set session timeout
    pub fn with_session_timeout(mut self, seconds: u64) -> Self {
        self.session_timeout_seconds = seconds;
        self
    }

    /// Enable or disable metrics
    pub fn with_metrics(mut self, enable: bool) -> Self {
        self.enable_metrics = enable;
        self
    }

    /// Enable or disable tracing
    pub fn with_tracing(mut self, enable: bool) -> Self {
        self.enable_tracing = enable;
        self
    }

    /// Set log level
    pub fn with_log_level(mut self, level: impl Into<String>) -> Self {
        self.log_level = level.into();
        self
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(agent_id) = std::env::var("CONSTELLATION_AGENT_ID") {
            config.agent_id = agent_id;
        }

        if let Ok(broker_url) = std::env::var("CONSTELLATION_BROKER_URL") {
            config.broker_url = broker_url;
        }

        if let Ok(username) = std::env::var("CONSTELLATION_BROKER_USERNAME") {
            config.broker_username = username;
        }

        if let Ok(password) = std::env::var("CONSTELLATION_BROKER_PASSWORD") {
            config.broker_password = password;
        }

        if let Ok(stream_name) = std::env::var("CONSTELLATION_STREAM_NAME") {
            config.stream_name = stream_name;
        }

        if let Ok(topic_name) = std::env::var("CONSTELLATION_TOPIC_NAME") {
            config.topic_name = topic_name;
        }

        config
    }

    /// Load configuration from a file
    pub fn from_file(path: &str) -> AgentResult<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a file
    pub fn save_to_file(&self, path: &str) -> AgentResult<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
