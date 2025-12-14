//! Data Transformation Graph (DTG) module.
//!
//! Provides models and execution engine for tracking agent skill execution
//! as data transformations with full provenance tracking.

pub mod engine;
pub mod metrics;
pub mod provenance;

pub use engine::{DtgExecutionEngine, ExecutionStats};
pub use metrics::{
    DtgMetricsCollector, PerformanceAnalysis, PerformanceStatus, QualityReport,
    QualityScoringConfig,
};
pub use provenance::{
    AuditOperation, AuditTrailIntegrityResult, IntegrityCheckResult, ProvenanceError,
    ProvenanceManager, SignatureVerificationResult,
};