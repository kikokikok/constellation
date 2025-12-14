//! Data models for the Constellation platform.

pub mod agent;
pub mod dtg;
pub mod hybrid_agent;
pub mod mcp;

// Re-export the agent types.
pub use agent::{
    Agent, AgentCapabilities, AgentContact, AgentInterface, AgentProvider, AgentSkill,
    ProtocolBinding, SecuritySchemeType,
};
pub use dtg::{
    DataTransformationGraph, DtgDataRef, DtgEdge, DtgGraphStatus, DtgMetrics, DtgNode,
    DtgNodeStatus, DtgProvenance,
};
pub use hybrid_agent::{
    CoordinationStrategy, ExecutorConfig, HybridAgentConfig, PerformanceTargets,
    ResourceAllocation, StrategistConfig,
};
pub use mcp::{
    AccessControl, AuditLogging, KeyManagement, McpAlgorithms, McpEncryptedMessage,
    McpSecureEnvelope, McpSecurityContext, McpSignature, SecurityLevel,
};
