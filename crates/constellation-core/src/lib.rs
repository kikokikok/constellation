//! Core types and utilities for the Constellation multi-agent platform.

pub mod dtg;
pub mod hybrid;
pub mod mcp;
pub mod models;

// Re-export common types for convenience.
pub use models::agent::{
    Agent, AgentCapabilities, AgentContact, AgentInterface, AgentProvider, AgentSkill,
    ProtocolBinding, SecuritySchemeType,
};
pub use dtg::engine::{DtgExecutionEngine, ExecutionStats};
pub use dtg::metrics::{
    DtgMetricsCollector, PerformanceAnalysis, PerformanceStatus, QualityReport,
    QualityScoringConfig,
};
pub use models::dtg::{
    DataTransformationGraph, DtgDataRef, DtgEdge, DtgGraphStatus, DtgMetrics, DtgNode,
    DtgNodeStatus, DtgProvenance,
};
pub use hybrid::{
    ExecutorStats, LlmStrategistCoordinator, PerformanceMetrics, QueueStats,
    ResourceRequirements, ResourceUsage, Task, TaskAssignment, TaskResult, TaskStatus,
};
pub use models::hybrid_agent::{
    CoordinationStrategy, ExecutorConfig, HybridAgentConfig, PerformanceTargets,
    ResourceAllocation as HybridResourceAllocation, StrategistConfig,
};
pub use mcp::crypto::{
    AlgorithmInfo, AlgorithmType, CryptoError, KeyMetadata, KeyStore, KeyUsage, McpCrypto,
    PrivateKey, PublicKey,
};
pub use models::mcp::{
    AccessControl, AuditLogging, KeyManagement, McpAlgorithms, McpEncryptedMessage,
    McpSecureEnvelope, McpSecurityContext, McpSignature, SecurityLevel,
};
