//! Data models for the Constellation platform.

pub mod agent;
pub mod autonomy;
pub mod communication;
pub mod dtg;
pub mod hybrid_agent;
pub mod mcp;
pub mod message_broker;

#[cfg(test)]
mod communication_tests;

// Re-export the agent types.
pub use agent::{
    Agent, AgentCapabilities, AgentContact, AgentInterface, AgentProvider, AgentSkill,
    ProtocolBinding, SecuritySchemeType,
};
pub use autonomy::{
    AutonomyBenchmark, AutonomyLevel, AutonomyMeasurement, AutonomyProgress, AxisValidation,
    BenchmarkValidationResult, CapabilityAxis, CollaborationPattern, CollaborationPatternType,
    KappaScore, SelfAssessment,
};
pub use communication::{
    CommunicationError, CommunicationPattern, CommunicationResult, DeliveryGuarantee,
    NotificationMessage, PublishMessage, RequestConfig, RequestMessage, ResponseConfig,
    ResponseMessage, ResponseStatus, Subscription, TopicPattern,
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
pub use message_broker::{
    A2AMessage, AgentConnectionRequest, AgentConnectionResponse, AgentSession, DeadLetterEntry,
    DeliveryStatus, DeliveryStatusEntry, Message, MessageAcknowledgment, MessageBrokerError,
    MessageBrokerResult, MessagePriority, QueueEntry, QueueStats, RoutingRule, SessionStatus,
};
