//! Data models for the Constellation platform.

pub mod agent;
pub mod dtg;
pub mod mcp;
pub mod hybrid_agent;

// Re-export the agent types.
pub use agent::{Agent, AgentCapabilities, AgentContact, AgentInterface, AgentProvider, AgentSkill, ProtocolBinding, SecuritySchemeType};
pub use dtg::{DataTransformationGraph, DtgNode, DtgDataRef, DtgNodeStatus, DtgMetrics, DtgEdge, DtgGraphStatus, DtgProvenance};
pub use mcp::{McpSecurityContext, SecurityLevel, McpAlgorithms, KeyManagement, AccessControl, AuditLogging, McpSecureEnvelope, McpEncryptedMessage, McpSignature};
pub use hybrid_agent::{HybridAgentConfig, StrategistConfig, ExecutorConfig, CoordinationStrategy, ResourceAllocation, PerformanceTargets};
