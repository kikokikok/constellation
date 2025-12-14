//! Core types and utilities for the Constellation multi-agent platform.

pub mod models;

// Re-export common types for convenience.
pub use models::agent::{
    Agent, AgentCapabilities, AgentContact, AgentInterface, AgentProvider, AgentSkill,
    ProtocolBinding, SecuritySchemeType,
};
pub use models::dtg::{
    DataTransformationGraph, DtgDataRef, DtgEdge, DtgGraphStatus, DtgMetrics, DtgNode,
    DtgNodeStatus, DtgProvenance,
};
pub use models::hybrid_agent::{
    CoordinationStrategy, ExecutorConfig, HybridAgentConfig, PerformanceTargets,
    ResourceAllocation, StrategistConfig,
};
pub use models::mcp::{
    AccessControl, AuditLogging, KeyManagement, McpAlgorithms, McpEncryptedMessage,
    McpSecureEnvelope, McpSecurityContext, McpSignature, SecurityLevel,
};
