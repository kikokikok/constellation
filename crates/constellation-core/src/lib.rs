//! Core types and utilities for the Constellation multi-agent platform.

pub mod models;

// Re-export common types for convenience.
pub use models::agent::{Agent, AgentCapabilities, AgentContact, AgentInterface, AgentProvider, AgentSkill, ProtocolBinding, SecuritySchemeType};
pub use models::dtg::{DataTransformationGraph, DtgNode, DtgDataRef, DtgNodeStatus, DtgMetrics, DtgEdge, DtgGraphStatus, DtgProvenance};
pub use models::mcp::{McpSecurityContext, SecurityLevel, McpAlgorithms, KeyManagement, AccessControl, AuditLogging, McpSecureEnvelope, McpEncryptedMessage, McpSignature};
pub use models::hybrid_agent::{HybridAgentConfig, StrategistConfig, ExecutorConfig, CoordinationStrategy, ResourceAllocation, PerformanceTargets};
