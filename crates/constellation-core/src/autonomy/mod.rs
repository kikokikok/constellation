//! Autonomy measurement engine for Kardashev-style AI capability scaling.
//!
//! Implements measurement, tracking, and optimization of autonomous AI capabilities
//! across 10 capability axes with κ (kappa) scoring.

pub mod benchmarks;
pub mod collaboration;
pub mod measurement_engine;
pub mod open_world;
pub mod self_assessment;

pub use benchmarks::BenchmarkManager;
pub use collaboration::CollaborationPatternDetector;
pub use measurement_engine::AutonomyMeasurementEngine;
pub use open_world::{
    DiscoverySignificance, ExperimentStatus, HypothesisStatus, OpenWorldConfig, OpenWorldMetrics,
    OpenWorldResearchEnvironment, ResearchDiscovery, ResearchExperiment, ResearchHypothesis,
    ResearchRecommendation, ResearchRecommendationType, ValidationStatus,
};
pub use self_assessment::SelfAssessmentEngine;
