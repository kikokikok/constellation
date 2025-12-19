//! Plugin-based agent harness for autonomous software development
//!
//! This crate provides a framework for long-running agents that can:
//! - Work across multiple context windows with progress preservation
//! - Use plugin-based architecture for language/framework independence
//! - Integrate with neuroscience-inspired memory systems
//! - Support skill-based progressive disclosure
//! - Coordinate multi-agent research and development
//!
//! Key features:
//! 1. Plugin system for language/framework adapters
//! 2. Memory-integrated progress tracking
//! 3. Skill execution with progressive disclosure
//! 4. Multi-agent orchestration
//! 5. Abstract interfaces, not hard-coded implementations

pub mod adapter;
pub mod business;
pub mod error;
pub mod harness;
pub mod orchestrator;
pub mod plugin;
pub mod progress;
pub mod session;
pub mod skill;

pub use adapter::{FrameworkAdapter, LanguageAdapter, TestingAdapter};
pub use business::integration::BusinessOrchestrator;
pub use business::{
    BusinessAgent, BusinessContext, BusinessGoal, BusinessMetrics, BusinessTask, BusinessTaskType,
};
pub use error::{Error, Result};
pub use harness::AgentHarness;
pub use orchestrator::{AgentRole, Orchestrator, TaskAssignment};
pub use plugin::{Plugin, PluginConfig, PluginRegistry};
pub use progress::{Feature, FeatureStatus, ProgressFile, ProgressTracker};
pub use session::{Session, SessionManager, SessionState};
pub use skill::{Skill, SkillExecutor, SkillRegistry};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        AgentHarness, BusinessAgent, BusinessContext, BusinessGoal, BusinessMetrics,
        BusinessOrchestrator, BusinessTask, BusinessTaskType, Error, FrameworkAdapter,
        LanguageAdapter, Orchestrator, ProgressTracker, Result, Session, SessionManager, Skill,
        SkillExecutor, TestingAdapter,
    };
    pub use constellation_core::communication::CommunicationFramework;
    pub use constellation_core::memory::prelude::*;
}
