use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] yaml_rust::ScanError),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Invalid progress file format: {0}")]
    InvalidProgressFile(String),

    #[error("Repository not initialized: {0}")]
    RepositoryNotInitialized(String),

    #[error("Feature not found: {0}")]
    FeatureNotFound(String),

    #[error("Session recovery failed: {0}")]
    SessionRecoveryFailed(String),

    #[error("Context window limit exceeded")]
    ContextWindowLimitExceeded,

    #[error("Testing failed: {0}")]
    TestingFailed(String),

    #[error("Memory compression failed: {0}")]
    MemoryCompressionFailed(String),

    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("External tool error: {0}")]
    ExternalToolError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),

    // Orchestrator errors
    #[error("Agent busy: {0}")]
    AgentBusy(String),

    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Invalid agent state: {0}")]
    InvalidAgentState(String),

    #[error("Task queue full: {0}")]
    TaskQueueFull(usize),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    // Plugin errors
    #[error("Plugin error: {0}")]
    PluginError(String),

    // Skill errors
    #[error("Skill error: {0}")]
    SkillError(String),

    // Adapter errors
    #[error("Adapter error: {0}")]
    AdapterError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
