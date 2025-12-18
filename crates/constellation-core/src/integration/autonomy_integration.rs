//! Integration of autonomy measurement with all agent operations.
//!
//! This module connects the autonomy measurement system (Kardashev-style
//! scale, 10 capability axes, κ scoring) with agent operations, tracking
//! and optimizing agent autonomy across all activities.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::autonomy::{
    AutonomyMeasurementEngine, CollaborationPatternDetector, OpenWorldResearchEnvironment,
    SelfAssessmentEngine,
};
use crate::hybrid::coordinator::{Task, TaskResult};
use crate::integration::{DtgAgentIntegrationEngine, HybridA2AIntegration, McpSecurityIntegration};
use crate::models::autonomy::{AutonomyLevel, CapabilityAxis, KappaScore, SelfAssessment};

/// Integration engine for autonomy measurement across all operations.
pub struct AutonomyIntegrationEngine {
    /// Autonomy measurement engine.
    autonomy_engine: Arc<RwLock<AutonomyMeasurementEngine>>,

    /// Self-assessment engine.
    self_assessment: Arc<RwLock<SelfAssessmentEngine>>,

    /// Collaboration pattern detector.
    collaboration_detector: Arc<RwLock<CollaborationPatternDetector>>,

    /// Open-world research environment.
    open_world: Arc<RwLock<OpenWorldResearchEnvironment>>,

    /// Agent autonomy levels cache.
    agent_autonomy: Arc<RwLock<HashMap<String, AutonomyLevel>>>,

    /// Agent κ scores cache.
    agent_kappa_scores: Arc<RwLock<HashMap<String, KappaScore>>>,

    /// Integration with other systems.
    dtg_integration: Option<Arc<RwLock<DtgAgentIntegrationEngine>>>,
    hybrid_a2a_integration: Option<Arc<RwLock<HybridA2AIntegration>>>,
    security_integration: Option<Arc<RwLock<McpSecurityIntegration>>>,
}

impl AutonomyIntegrationEngine {
    /// Create a new autonomy integration engine.
    pub fn new(
        autonomy_engine: AutonomyMeasurementEngine,
        self_assessment: SelfAssessmentEngine,
        collaboration_detector: CollaborationPatternDetector,
        open_world: OpenWorldResearchEnvironment,
    ) -> Self {
        Self {
            autonomy_engine: Arc::new(RwLock::new(autonomy_engine)),
            self_assessment: Arc::new(RwLock::new(self_assessment)),
            collaboration_detector: Arc::new(RwLock::new(collaboration_detector)),
            open_world: Arc::new(RwLock::new(open_world)),
            agent_autonomy: Arc::new(RwLock::new(HashMap::new())),
            agent_kappa_scores: Arc::new(RwLock::new(HashMap::new())),
            dtg_integration: None,
            hybrid_a2a_integration: None,
            security_integration: None,
        }
    }

    /// Connect with DTG integration.
    pub fn with_dtg_integration(mut self, integration: DtgAgentIntegrationEngine) -> Self {
        self.dtg_integration = Some(Arc::new(RwLock::new(integration)));
        self
    }

    /// Connect with hybrid A2A integration.
    pub fn with_hybrid_a2a_integration(mut self, integration: HybridA2AIntegration) -> Self {
        self.hybrid_a2a_integration = Some(Arc::new(RwLock::new(integration)));
        self
    }

    /// Connect with security integration.
    pub fn with_security_integration(mut self, integration: McpSecurityIntegration) -> Self {
        self.security_integration = Some(Arc::new(RwLock::new(integration)));
        self
    }

    /// Record agent task execution for autonomy measurement.
    pub async fn record_task_execution(
        &self,
        agent_id: &str,
        task: &Task,
        task_result: &TaskResult,
    ) -> Result<AutonomyUpdate, String> {
        let autonomy_engine = self.autonomy_engine.write().await;
        let self_assessment = self.self_assessment.write().await;

        // Determine capability axes involved in this task
        let axes = self.determine_capability_axes(task, task_result).await;

        // Record observations for each axis
        for axis in &axes {
            let score = self.calculate_axis_score(task, task_result, *axis).await;
            let confidence = self.calculate_confidence(task_result).await;

            autonomy_engine.record_observation(
                agent_id,
                *axis,
                score,
                confidence,
                Some(task.id),
                1.0, // Environment complexity
            );
        }

        // Perform self-assessment
        let assessment = self_assessment
            .assess_task_automated(agent_id, task_result)
            .unwrap_or_else(|| SelfAssessment {
                id: Uuid::new_v4(),
                agent_id: agent_id.to_string(),
                task_id: None,
                score: 0.5,
                confidence: 0.5,
                strengths: Vec::new(),
                improvements: Vec::new(),
                errors_identified: Vec::new(),
                corrective_actions: Vec::new(),
                timestamp: SystemTime::now(),
                assessment_duration: std::time::Duration::from_secs(0),
                externally_validated: false,
                external_validation_score: None,
            });

        // Calculate κ scores
        let kappa_scores_map = autonomy_engine.calculate_kappa_scores(agent_id);

        // Calculate average κ score (simplified composite)
        let composite_kappa = if kappa_scores_map.is_empty() {
            0.0
        } else {
            kappa_scores_map
                .values()
                .map(|score| score.score)
                .sum::<f64>()
                / kappa_scores_map.len() as f64
        };

        // Create a KappaScore using Planning axis (arbitrary choice for composite)
        let kappa_score = KappaScore {
            axis: CapabilityAxis::Planning,
            score: composite_kappa,
            confidence: 0.8, // Default confidence
            timestamp: SystemTime::now(),
            observation_count: 1,
        };

        // Update autonomy level (using default environment complexity of 0.5)
        let autonomy_level = autonomy_engine.determine_autonomy_level(&kappa_scores_map, 0.5);

        // Get axes involved
        let axes: Vec<CapabilityAxis> = kappa_scores_map.keys().cloned().collect();

        // Update caches
        let mut agent_autonomy = self.agent_autonomy.write().await;
        let mut agent_kappa_scores = self.agent_kappa_scores.write().await;

        agent_autonomy.insert(agent_id.to_string(), autonomy_level);
        agent_kappa_scores.insert(agent_id.to_string(), kappa_score);

        // Generate improvement recommendations
        let recommendations = self
            .generate_improvement_recommendations(
                agent_id,
                &autonomy_level,
                &kappa_score,
                &assessment,
            )
            .await;

        Ok(AutonomyUpdate {
            agent_id: agent_id.to_string(),
            autonomy_level,
            kappa_score,
            assessment,
            axes_involved: axes,
            recommendations,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Determine which capability axes are involved in a task.
    async fn determine_capability_axes(
        &self,
        task: &Task,
        task_result: &TaskResult,
    ) -> Vec<CapabilityAxis> {
        let mut axes = Vec::new();

        // Analyze task type to determine axes
        match task.task_type.as_str() {
            "planning" | "strategy" => {
                axes.push(CapabilityAxis::Planning);
                axes.push(CapabilityAxis::Reasoning);
            }
            "execution" | "implementation" => {
                axes.push(CapabilityAxis::Execution);
                axes.push(CapabilityAxis::Adaptation);
            }
            "learning" | "training" => {
                axes.push(CapabilityAxis::Learning);
                axes.push(CapabilityAxis::MetaCognition);
            }
            "creative" | "generation" => {
                axes.push(CapabilityAxis::Creativity);
                axes.push(CapabilityAxis::Reasoning);
            }
            "collaboration" | "coordination" => {
                axes.push(CapabilityAxis::Collaboration);
                axes.push(CapabilityAxis::SelfAssessment);
            }
            "resource_management" | "optimization" => {
                axes.push(CapabilityAxis::ResourceManagement);
                axes.push(CapabilityAxis::Planning);
            }
            _ => {
                // Default axes for unknown task types
                axes.push(CapabilityAxis::Execution);
                axes.push(CapabilityAxis::Adaptation);
            }
        }

        // Add axes based on task complexity
        if self.is_complex_task(task).await {
            axes.push(CapabilityAxis::Reasoning);
            axes.push(CapabilityAxis::MetaCognition);
        }

        // Add axes based on task success
        if task_result.success {
            axes.push(CapabilityAxis::SelfAssessment);
        }

        axes.dedup();
        axes
    }

    /// Calculate score for a specific capability axis.
    async fn calculate_axis_score(
        &self,
        task: &Task,
        task_result: &TaskResult,
        axis: CapabilityAxis,
    ) -> f64 {
        match axis {
            CapabilityAxis::Planning => {
                // Planning score based on task complexity and success
                let complexity = self.estimate_task_complexity(task).await;
                let success_bonus = if task_result.success { 0.3 } else { 0.0 };
                (complexity * 0.7 + success_bonus).min(1.0)
            }
            CapabilityAxis::Execution => {
                // Execution score based on efficiency and quality
                let efficiency = self.calculate_efficiency(task_result).await;
                let quality = task_result.quality_score;
                (efficiency * 0.4 + quality * 0.6).min(1.0)
            }
            CapabilityAxis::Learning => {
                // Learning score based on improvement over time
                // For now, base on task novelty

                self.estimate_task_novelty(task).await
            }
            CapabilityAxis::Adaptation => {
                // Adaptation score based on handling variations
                let variation = self.estimate_task_variation(task).await;
                let success_bonus = if task_result.success { 0.2 } else { 0.0 };
                (variation * 0.8 + success_bonus).min(1.0)
            }
            CapabilityAxis::Reasoning => {
                // Reasoning score based on task complexity

                self.estimate_task_complexity(task).await
            }
            CapabilityAxis::Creativity => {
                // Creativity score based on task novelty and output uniqueness
                let novelty = self.estimate_task_novelty(task).await;
                let uniqueness = self.estimate_output_uniqueness(task_result).await;
                (novelty * 0.6 + uniqueness * 0.4).min(1.0)
            }
            CapabilityAxis::Collaboration => {
                // Collaboration score (if this was a collaborative task)
                // For now, base on whether multiple agents were involved
                let collaborative = self.is_collaborative_task(task).await;
                if collaborative { 0.8 } else { 0.3 }
            }
            CapabilityAxis::SelfAssessment => {
                // Self-assessment score based on accuracy of self-evaluation
                // For now, base on whether the agent correctly assessed its performance
                let accurate = self.is_accurate_self_assessment(task_result).await;
                if accurate { 0.9 } else { 0.4 }
            }
            CapabilityAxis::ResourceManagement => {
                // Resource management score based on efficiency

                self.calculate_resource_efficiency(task_result).await
            }
            CapabilityAxis::MetaCognition => {
                // Meta-cognition score based on learning from experience

                self.estimates_learning_from_task(task).await
            }
        }
    }

    /// Calculate confidence in the observation.
    async fn calculate_confidence(&self, task_result: &TaskResult) -> f64 {
        // Confidence based on task result quality and completeness
        let quality_confidence = task_result.quality_score;
        let completeness_confidence = if task_result.success { 0.9 } else { 0.5 };

        (quality_confidence * 0.6 + completeness_confidence * 0.4).min(1.0)
    }

    /// Estimate task complexity.
    async fn estimate_task_complexity(&self, task: &Task) -> f64 {
        // Simple heuristic based on task metadata
        if let Some(complexity) = task.metadata.get("complexity") {
            complexity.as_f64().unwrap_or(0.5)
        } else {
            // Estimate based on input size and task type
            let input_complexity = match serde_json::to_string(&task.input).unwrap().len() {
                0..=100 => 0.3,
                101..=1000 => 0.5,
                1001..=10000 => 0.7,
                _ => 0.9,
            };

            let type_complexity = match task.task_type.as_str() {
                "simple" | "basic" => 0.3,
                "standard" | "normal" => 0.5,
                "complex" | "advanced" => 0.7,
                "expert" | "challenging" => 0.9,
                _ => 0.5,
            };

            f64::min(input_complexity * 0.4 + type_complexity * 0.6, 1.0)
        }
    }

    /// Calculate task execution efficiency.
    async fn calculate_efficiency(&self, task_result: &TaskResult) -> f64 {
        // Efficiency based on execution time relative to some baseline
        // For now, use a simple heuristic
        let time_ms = task_result.execution_time_ms as f64;

        if time_ms == 0.0 {
            return 0.5;
        }

        // Assume optimal time is 1000ms, efficiency decreases with longer times
        let optimal_time = 1000.0;
        let efficiency = optimal_time / time_ms.max(optimal_time);

        efficiency.min(1.0)
    }

    /// Estimate task novelty.
    async fn estimate_task_novelty(&self, task: &Task) -> f64 {
        // Novelty based on task type and input uniqueness
        // For now, use a simple heuristic
        match task.task_type.as_str() {
            "creative" | "generation" | "innovation" => 0.8,
            "standard" | "routine" | "repetitive" => 0.2,
            _ => 0.5,
        }
    }

    /// Check if task is complex.
    async fn is_complex_task(&self, task: &Task) -> bool {
        self.estimate_task_complexity(task).await > 0.7
    }

    /// Check if task is collaborative.
    async fn is_collaborative_task(&self, task: &Task) -> bool {
        task.metadata
            .get("collaborative")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// Check if self-assessment was accurate.
    async fn is_accurate_self_assessment(&self, task_result: &TaskResult) -> bool {
        // For now, assume accurate if quality score is high
        task_result.quality_score > 0.7
    }

    /// Calculate resource efficiency.
    async fn calculate_resource_efficiency(&self, task_result: &TaskResult) -> f64 {
        // Resource efficiency based on cost and resource usage
        // For now, use a simple heuristic

        1.0 / (task_result.cost.max(0.01) * 10.0).min(1.0)
    }

    /// Estimate learning from task.
    async fn estimates_learning_from_task(&self, task: &Task) -> f64 {
        // Learning estimation based on task type
        match task.task_type.as_str() {
            "learning" | "training" | "experiment" => 0.8,
            "application" | "implementation" => 0.5,
            "repetitive" | "routine" => 0.2,
            _ => 0.4,
        }
    }

    /// Estimate task variation.
    async fn estimate_task_variation(&self, task: &Task) -> f64 {
        // Variation based on how different this task is from previous ones
        // For now, use a simple heuristic
        0.5
    }

    /// Estimate output uniqueness.
    async fn estimate_output_uniqueness(&self, task_result: &TaskResult) -> f64 {
        // Uniqueness based on output characteristics
        // For now, use a simple heuristic
        0.5
    }

    /// Generate improvement recommendations for an agent.
    async fn generate_improvement_recommendations(
        &self,
        agent_id: &str,
        autonomy_level: &AutonomyLevel,
        kappa_score: &KappaScore,
        assessment: &crate::models::autonomy::SelfAssessment,
    ) -> Vec<ImprovementRecommendation> {
        let mut recommendations = Vec::new();

        // Check if agent is ready for next autonomy level
        let current_level_value = autonomy_level.value();
        let next_level = autonomy_level.next_level();

        if let Some(next) = next_level {
            let progress = kappa_score.score;
            let threshold = 0.8; // Need 80% κ score to advance

            if progress >= threshold {
                recommendations.push(ImprovementRecommendation {
                    recommendation_type: ImprovementType::AdvanceAutonomyLevel,
                    description: format!(
                        "Ready to advance from {} to {}",
                        autonomy_level.description(),
                        next.description()
                    ),
                    priority: Priority::High,
                    estimated_effort: 0.7,
                    expected_impact: 0.9,
                });
            } else {
                let needed = threshold - progress;
                recommendations.push(ImprovementRecommendation {
                    recommendation_type: ImprovementType::ImproveKappaScore,
                    description: format!(
                        "Need to improve κ score by {:.1}% to advance autonomy level",
                        needed * 100.0
                    ),
                    priority: Priority::Medium,
                    estimated_effort: needed,
                    expected_impact: 0.8,
                });
            }
        }

        // Check weak capability axes
        let weak_axes = self.identify_weak_axes(kappa_score).await;
        for axis in weak_axes {
            recommendations.push(ImprovementRecommendation {
                recommendation_type: ImprovementType::StrengthenCapabilityAxis(axis),
                description: format!("Strengthen {} capability", axis.name()),
                priority: Priority::Medium,
                estimated_effort: 0.5,
                expected_impact: 0.6,
            });
        }

        // Check self-assessment accuracy
        if assessment.score < 0.7 {
            recommendations.push(ImprovementRecommendation {
                recommendation_type: ImprovementType::ImproveSelfAssessment,
                description: "Improve self-assessment accuracy".to_string(),
                priority: Priority::Low,
                estimated_effort: 0.3,
                expected_impact: 0.4,
            });
        }

        // Check for collaboration opportunities
        if autonomy_level.value() >= 3 {
            // Level 3 or higher
            recommendations.push(ImprovementRecommendation {
                recommendation_type: ImprovementType::IncreaseCollaboration,
                description: "Engage in more collaborative tasks".to_string(),
                priority: Priority::Low,
                estimated_effort: 0.4,
                expected_impact: 0.5,
            });
        }

        recommendations
    }

    /// Identify weak capability axes based on κ scores.
    async fn identify_weak_axes(&self, kappa_score: &KappaScore) -> Vec<CapabilityAxis> {
        let mut weak_axes = Vec::new();
        let threshold = 0.6; // Axes below this are considered weak

        // Check each axis (simplified - in real implementation would check individual axis scores)
        if kappa_score.score < threshold {
            // If overall score is low, recommend working on core axes
            weak_axes.push(CapabilityAxis::Planning);
            weak_axes.push(CapabilityAxis::Execution);
            weak_axes.push(CapabilityAxis::Reasoning);
        }

        weak_axes
    }

    /// Get agent autonomy level.
    pub async fn get_agent_autonomy_level(&self, agent_id: &str) -> Option<AutonomyLevel> {
        let agent_autonomy = self.agent_autonomy.read().await;
        agent_autonomy.get(agent_id).cloned()
    }

    /// Get agent κ score.
    pub async fn get_agent_kappa_score(&self, agent_id: &str) -> Option<KappaScore> {
        let agent_kappa_scores = self.agent_kappa_scores.read().await;
        agent_kappa_scores.get(agent_id).cloned()
    }

    /// Get autonomy report for an agent.
    pub async fn get_autonomy_report(&self, agent_id: &str) -> Option<AutonomyReport> {
        let autonomy_level = self.get_agent_autonomy_level(agent_id).await?;
        let kappa_score = self.get_agent_kappa_score(agent_id).await?;

        // Calculate progress based on kappa score (simplified)
        // Progress is the kappa score normalized to the next level threshold
        let progress = kappa_score.score / 0.8; // Assuming 0.8 is the threshold for next level

        Some(AutonomyReport {
            agent_id: agent_id.to_string(),
            autonomy_level,
            kappa_score,
            progress,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Record collaboration event for pattern detection.
    pub async fn record_collaboration_event(
        &self,
        agent_ids: Vec<String>,
        task_id: Option<Uuid>,
        success: bool,
        efficiency: f64,
        duration: std::time::Duration,
    ) -> Result<CollaborationAnalysis, String> {
        let detector = self.collaboration_detector.write().await;

        // Record the collaboration
        detector.record_collaboration(agent_ids.clone(), task_id, success, efficiency, duration);

        // Get collaboration patterns for the first agent (simplified)
        let patterns = if let Some(first_agent) = agent_ids.first() {
            detector.get_agent_patterns(first_agent)
        } else {
            Vec::new()
        };

        // Get efficiency for the first agent (simplified)
        let agent_efficiency = if let Some(first_agent) = agent_ids.first() {
            detector.get_agent_efficiency(first_agent).unwrap_or(0.0)
        } else {
            0.0
        };

        Ok(CollaborationAnalysis {
            event_id: Uuid::new_v4(),
            agent_ids,
            patterns,
            efficiency: agent_efficiency,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Integrate with open-world research environment.
    pub async fn integrate_with_open_world(&self, discovery_id: &str) -> Result<(), String> {
        let open_world = self.open_world.read().await;
        let autonomy_engine = self.autonomy_engine.read().await;

        open_world
            .integrate_discovery_into_autonomy(discovery_id, &autonomy_engine)
            .await
    }

    /// Get a reference to the autonomy engine (for testing).
    pub fn get_autonomy_engine(&self) -> &Arc<RwLock<AutonomyMeasurementEngine>> {
        &self.autonomy_engine
    }

    /// Get a reference to the self-assessment engine (for testing).
    pub fn get_self_assessment(&self) -> &Arc<RwLock<SelfAssessmentEngine>> {
        &self.self_assessment
    }

    /// Get a reference to the collaboration detector (for testing).
    pub fn get_collaboration_detector(&self) -> &Arc<RwLock<CollaborationPatternDetector>> {
        &self.collaboration_detector
    }

    /// Get a reference to the open-world research environment (for testing).
    pub fn get_open_world(&self) -> &Arc<RwLock<OpenWorldResearchEnvironment>> {
        &self.open_world
    }
}

/// Autonomy update after task execution.
#[derive(Debug, Clone)]
pub struct AutonomyUpdate {
    /// Agent ID.
    pub agent_id: String,

    /// Current autonomy level.
    pub autonomy_level: AutonomyLevel,

    /// Current κ score.
    pub kappa_score: KappaScore,

    /// Self-assessment results.
    pub assessment: crate::models::autonomy::SelfAssessment,

    /// Capability axes involved in the task.
    pub axes_involved: Vec<CapabilityAxis>,

    /// Improvement recommendations.
    pub recommendations: Vec<ImprovementRecommendation>,

    /// Update timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Improvement recommendation for an agent.
#[derive(Debug, Clone)]
pub struct ImprovementRecommendation {
    /// Type of improvement needed.
    pub recommendation_type: ImprovementType,

    /// Description of the recommendation.
    pub description: String,

    /// Priority level.
    pub priority: Priority,

    /// Estimated effort (0.0 to 1.0).
    pub estimated_effort: f64,

    /// Expected impact (0.0 to 1.0).
    pub expected_impact: f64,
}

/// Type of improvement recommendation.
#[derive(Debug, Clone)]
pub enum ImprovementType {
    /// Advance to next autonomy level.
    AdvanceAutonomyLevel,

    /// Improve κ score.
    ImproveKappaScore,

    /// Strengthen a specific capability axis.
    StrengthenCapabilityAxis(CapabilityAxis),

    /// Improve self-assessment accuracy.
    ImproveSelfAssessment,

    /// Increase collaboration.
    IncreaseCollaboration,

    /// Improve resource management.
    ImproveResourceManagement,

    /// Enhance creativity.
    EnhanceCreativity,
}

/// Priority level for recommendations.
#[derive(Debug, Clone)]
pub enum Priority {
    /// High priority - should be addressed immediately.
    High,

    /// Medium priority - should be addressed soon.
    Medium,

    /// Low priority - can be addressed when convenient.
    Low,
}

/// Autonomy report for an agent.
#[derive(Debug, Clone)]
pub struct AutonomyReport {
    /// Agent ID.
    pub agent_id: String,

    /// Current autonomy level.
    pub autonomy_level: AutonomyLevel,

    /// Current κ score.
    pub kappa_score: KappaScore,

    /// Progress towards next level (0.0 to 1.0).
    pub progress: f64,

    /// Report timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Collaboration event for pattern detection.
pub type CollaborationEvent = crate::autonomy::collaboration::CollaborationEvent;

/// Collaboration analysis result.
#[derive(Debug, Clone)]
pub struct CollaborationAnalysis {
    /// Event ID.
    pub event_id: Uuid,

    /// Agent IDs involved in the collaboration.
    pub agent_ids: Vec<String>,

    /// Detected collaboration patterns.
    pub patterns: Vec<crate::models::autonomy::CollaborationPattern>,

    /// Collaboration efficiency score.
    pub efficiency: f64,

    /// Analysis timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autonomy::collaboration::CollaborationConfig;
    use crate::autonomy::measurement_engine::MeasurementConfig;
    use crate::autonomy::self_assessment::SelfAssessmentConfig;
    use crate::autonomy::{OpenWorldConfig, OpenWorldResearchEnvironment};
    use crate::models::autonomy::{AutonomyMeasurement, SelfAssessment};

    #[tokio::test]
    async fn test_autonomy_integration() {
        // Create autonomy components with test-friendly config
        let mut measurement_config = MeasurementConfig::default();
        measurement_config.min_tasks_per_axis = 1; // Allow calculation with single observation

        let autonomy_engine = AutonomyMeasurementEngine::new(measurement_config);
        let self_assessment = SelfAssessmentEngine::new(SelfAssessmentConfig::default());
        let collaboration_detector =
            CollaborationPatternDetector::new(CollaborationConfig::default());

        let open_world_config = OpenWorldConfig {
            max_concurrent_experiments: 5,
            resource_limits: HashMap::new(),
            collaboration_enabled: true,
            peer_review_enabled: true,
            discovery_validation_required: true,
            exploration_exploitation_ratio: 0.3,
            minimum_evidence_strength: 0.7,
            minimum_reproducibility: 0.8,
        };

        let open_world = OpenWorldResearchEnvironment::new(open_world_config);

        // Create integration engine
        let integration = AutonomyIntegrationEngine::new(
            autonomy_engine,
            self_assessment,
            collaboration_detector,
            open_world,
        );

        // Create test task and result
        let task = Task {
            id: Uuid::new_v4(),
            task_type: "planning".to_string(),
            input: serde_json::json!({"goal": "Test planning"}),
            expected_output: Some(serde_json::json!({"plan": "Test plan"})),
            assigned_to: Some("agent_1".to_string()),
            priority: 5,
            timeout_ms: 30000,
            created_at: chrono::Utc::now(),
            status: crate::hybrid::coordinator::TaskStatus::Completed,
            metadata: HashMap::new(),
            deadline: None,
            quality_requirement: 0.8,
            budget_allocation: 1.0,
            resource_requirements: crate::hybrid::coordinator::ResourceRequirements {
                min_cpu_cores: 1,
                min_memory_mb: 1024,
                gpu_memory_mb: None,
                network_mbps: 10,
            },
        };

        let task_result = TaskResult {
            task_id: task.id,
            executor_id: "agent_1".to_string(),
            completed_at: chrono::Utc::now(),
            result: serde_json::json!({"plan": "Test plan completed"}),
            success: true,
            error: None,
            quality_score: 0.85,
            execution_time_ms: 1500,
            resource_usage: crate::hybrid::coordinator::ResourceUsage {
                cpu_core_seconds: 0.0,
                memory_mb_seconds: 0.0,
                gpu_memory_mb_seconds: None,
                network_mb: 0.0,
            },
            cost: 0.1,
        };

        // Record task execution
        let update = integration
            .record_task_execution("agent_1", &task, &task_result)
            .await
            .unwrap();

        assert_eq!(update.agent_id, "agent_1");
        assert!(!update.axes_involved.is_empty());

        // Get autonomy report
        let report = integration.get_autonomy_report("agent_1").await;
        assert!(report.is_some());
    }
}
