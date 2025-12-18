//! Autonomy measurement engine for tracking and optimizing AI capabilities.

use crate::hybrid::coordinator::TaskResult;
use crate::models::autonomy::{
    AutonomyLevel, AutonomyMeasurement, AutonomyProgress, CapabilityAxis, KappaScore,
};
use crate::models::dtg::{DtgNode, DtgNodeStatus};
use crate::models::hybrid_agent::HybridAgentConfig;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Configuration for autonomy measurement.
#[derive(Debug, Clone)]
pub struct MeasurementConfig {
    /// Minimum number of tasks to observe before calculating κ scores.
    pub min_tasks_per_axis: u32,

    /// Maximum age of observations to consider (in seconds).
    pub max_observation_age: Duration,

    /// Weight for recent observations (0.0 to 1.0).
    pub recency_weight: f64,

    /// Environment complexity thresholds for different autonomy levels.
    pub environment_thresholds: HashMap<AutonomyLevel, f64>,

    /// κ score thresholds for autonomy level progression.
    pub kappa_thresholds: HashMap<AutonomyLevel, f64>,

    /// Capability axis weights for composite κ calculation.
    pub axis_weights: HashMap<CapabilityAxis, f64>,
}

impl Default for MeasurementConfig {
    fn default() -> Self {
        let mut environment_thresholds = HashMap::new();
        environment_thresholds.insert(AutonomyLevel::Level0Scripted, 0.1);
        environment_thresholds.insert(AutonomyLevel::Level1GoalOriented, 0.2);
        environment_thresholds.insert(AutonomyLevel::Level2Adaptive, 0.3);
        environment_thresholds.insert(AutonomyLevel::Level3Strategic, 0.4);
        environment_thresholds.insert(AutonomyLevel::Level4SelfImproving, 0.5);
        environment_thresholds.insert(AutonomyLevel::Level5Collaborative, 0.6);
        environment_thresholds.insert(AutonomyLevel::Level6Creative, 0.7);
        environment_thresholds.insert(AutonomyLevel::Level7MetaCognitive, 0.8);
        environment_thresholds.insert(AutonomyLevel::Level8SelfSustaining, 0.9);
        environment_thresholds.insert(AutonomyLevel::Level9Transcendent, 1.0);

        let mut kappa_thresholds = HashMap::new();
        kappa_thresholds.insert(AutonomyLevel::Level0Scripted, 0.1);
        kappa_thresholds.insert(AutonomyLevel::Level1GoalOriented, 0.2);
        kappa_thresholds.insert(AutonomyLevel::Level2Adaptive, 0.3);
        kappa_thresholds.insert(AutonomyLevel::Level3Strategic, 0.4);
        kappa_thresholds.insert(AutonomyLevel::Level4SelfImproving, 0.5);
        kappa_thresholds.insert(AutonomyLevel::Level5Collaborative, 0.6);
        kappa_thresholds.insert(AutonomyLevel::Level6Creative, 0.7);
        kappa_thresholds.insert(AutonomyLevel::Level7MetaCognitive, 0.8);
        kappa_thresholds.insert(AutonomyLevel::Level8SelfSustaining, 0.9);
        kappa_thresholds.insert(AutonomyLevel::Level9Transcendent, 1.0);

        let mut axis_weights = HashMap::new();
        for axis in CapabilityAxis::all() {
            axis_weights.insert(axis, 1.0); // Equal weights by default
        }

        Self {
            min_tasks_per_axis: 10,
            max_observation_age: Duration::from_secs(86400), // 24 hours
            recency_weight: 0.7,
            environment_thresholds,
            kappa_thresholds,
            axis_weights,
        }
    }
}

/// Observation of agent capability for a specific axis.
#[derive(Debug, Clone)]
struct CapabilityObservation {
    axis: CapabilityAxis,
    score: f64,
    confidence: f64,
    timestamp: SystemTime,
    task_id: Option<Uuid>,
    environment_complexity: f64,
    metadata: HashMap<String, serde_json::Value>,
}

impl CapabilityObservation {
    fn new(
        axis: CapabilityAxis,
        score: f64,
        confidence: f64,
        task_id: Option<Uuid>,
        environment_complexity: f64,
    ) -> Self {
        Self {
            axis,
            score: score.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            timestamp: SystemTime::now(),
            task_id,
            environment_complexity: environment_complexity.clamp(0.0, 1.0),
            metadata: HashMap::new(),
        }
    }

    fn age(&self) -> Duration {
        self.timestamp.elapsed().unwrap_or(Duration::from_secs(0))
    }

    fn is_valid(&self, max_age: Duration) -> bool {
        self.age() <= max_age
    }

    fn weighted_score(&self, recency_weight: f64, max_age: Duration) -> f64 {
        let age_factor = 1.0 - (self.age().as_secs_f64() / max_age.as_secs_f64()).min(1.0);
        let recency_factor = recency_weight * age_factor + (1.0 - recency_weight);
        self.score * self.confidence * recency_factor
    }
}

/// Autonomy measurement engine for tracking and optimizing agent capabilities.
#[derive(Debug)]
pub struct AutonomyMeasurementEngine {
    config: MeasurementConfig,
    observations: Arc<RwLock<HashMap<String, VecDeque<CapabilityObservation>>>>,
    progress_tracking: Arc<RwLock<HashMap<String, AutonomyProgress>>>,
    last_measurements: Arc<RwLock<HashMap<String, AutonomyMeasurement>>>,
}

impl AutonomyMeasurementEngine {
    /// Create a new autonomy measurement engine.
    pub fn new(config: MeasurementConfig) -> Self {
        Self {
            config,
            observations: Arc::new(RwLock::new(HashMap::new())),
            progress_tracking: Arc::new(RwLock::new(HashMap::new())),
            last_measurements: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record an observation of agent capability.
    pub fn record_observation(
        &self,
        agent_id: &str,
        axis: CapabilityAxis,
        score: f64,
        confidence: f64,
        task_id: Option<Uuid>,
        environment_complexity: f64,
    ) {
        let observation =
            CapabilityObservation::new(axis, score, confidence, task_id, environment_complexity);

        let mut observations = self.observations.write().unwrap();
        let agent_observations = observations.entry(agent_id.to_string()).or_default();

        // Add new observation
        agent_observations.push_back(observation);

        // Clean up old observations (inlined to avoid reentrant lock)
        if let Some(agent_observations) = observations.get_mut(agent_id) {
            // Remove invalid observations (older than 30 days)
            let max_age = std::time::Duration::from_secs(30 * 24 * 60 * 60);
            agent_observations.retain(|obs| obs.is_valid(max_age));

            // Keep only the most recent 1000 observations
            const MAX_OBSERVATIONS_PER_AGENT: usize = 1000;
            if agent_observations.len() > MAX_OBSERVATIONS_PER_AGENT {
                let excess = agent_observations.len() - MAX_OBSERVATIONS_PER_AGENT;
                agent_observations.drain(0..excess);
            }
        }
    }

    /// Record observation from a task result.
    pub fn record_from_task_result(
        &self,
        agent_id: &str,
        task_result: &TaskResult,
        environment_complexity: f64,
    ) {
        // Extract capability observations from task result
        // This is a simplified implementation - in practice, we would analyze
        // the task execution in detail

        // Planning capability (based on execution time and quality)
        if task_result.execution_time_ms > 0 {
            let planning_score = if task_result.success {
                0.8 + (task_result.quality_score * 0.2)
            } else {
                0.3
            };
            let planning_confidence = 0.7;

            self.record_observation(
                agent_id,
                CapabilityAxis::Planning,
                planning_score,
                planning_confidence,
                Some(task_result.task_id),
                environment_complexity,
            );
        }

        // Execution capability (based on success and execution time)
        let execution_score = if task_result.success {
            0.9 * task_result.quality_score
        } else {
            0.2
        };
        let execution_confidence = 0.8;

        self.record_observation(
            agent_id,
            CapabilityAxis::Execution,
            execution_score,
            execution_confidence,
            Some(task_result.task_id),
            environment_complexity,
        );

        // Learning capability (based on improvement over similar tasks)
        // Simplified: assume some learning if task was successful
        if task_result.success {
            let learning_score = 0.6;
            let learning_confidence = 0.5;

            self.record_observation(
                agent_id,
                CapabilityAxis::Learning,
                learning_score,
                learning_confidence,
                Some(task_result.task_id),
                environment_complexity,
            );
        }
    }

    /// Record observation from a DTG node.
    pub fn record_from_dtg_node(
        &self,
        agent_id: &str,
        dtg_node: &DtgNode,
        environment_complexity: f64,
    ) {
        // Analyze DTG node for capability insights
        match dtg_node.status {
            DtgNodeStatus::Completed => {
                // Successful execution shows good execution capability
                let execution_score = 0.8;
                let execution_confidence = 0.7;

                self.record_observation(
                    agent_id,
                    CapabilityAxis::Execution,
                    execution_score,
                    execution_confidence,
                    Some(dtg_node.id),
                    environment_complexity,
                );

                // Quality score indicates reasoning capability
                let reasoning_score = dtg_node.metrics.quality_score;
                let reasoning_confidence = 0.6;

                self.record_observation(
                    agent_id,
                    CapabilityAxis::Reasoning,
                    reasoning_score,
                    reasoning_confidence,
                    Some(dtg_node.id),
                    environment_complexity,
                );
            }
            DtgNodeStatus::Failed => {
                // Failure shows areas for improvement
                let execution_score = 0.3;
                let execution_confidence = 0.8;

                self.record_observation(
                    agent_id,
                    CapabilityAxis::Execution,
                    execution_score,
                    execution_confidence,
                    Some(dtg_node.id),
                    environment_complexity,
                );
            }
            _ => {
                // Other statuses don't provide clear capability signals
            }
        }
    }

    /// Calculate κ scores for an agent based on observations.
    pub fn calculate_kappa_scores(&self, agent_id: &str) -> HashMap<CapabilityAxis, KappaScore> {
        let observations = self.observations.read().unwrap();
        let agent_observations = observations.get(agent_id);

        let mut kappa_scores = HashMap::new();

        for axis in CapabilityAxis::all() {
            if let Some(axis_observations) = agent_observations
                .map(|obs| obs.iter().filter(|o| o.axis == axis).collect::<Vec<_>>())
                && axis_observations.len() >= self.config.min_tasks_per_axis as usize
            {
                // Calculate weighted average score
                let mut total_weighted = 0.0;
                let mut total_weight = 0.0;

                for obs in &axis_observations {
                    let weight = obs.weighted_score(
                        self.config.recency_weight,
                        self.config.max_observation_age,
                    );
                    total_weighted += obs.score * weight;
                    total_weight += weight;
                }

                if total_weight > 0.0 {
                    let score = total_weighted / total_weight;

                    // Calculate confidence based on observation count and variance
                    let observation_count = axis_observations.len() as u32;
                    let confidence =
                        (observation_count as f64 / self.config.min_tasks_per_axis as f64).min(1.0)
                            * 0.8; // Base confidence factor

                    let kappa_score = KappaScore::new(axis, score, confidence, observation_count);
                    kappa_scores.insert(axis, kappa_score);
                }
            }
        }

        kappa_scores
    }

    /// Determine autonomy level based on κ scores and environment complexity.
    pub fn determine_autonomy_level(
        &self,
        kappa_scores: &HashMap<CapabilityAxis, KappaScore>,
        environment_complexity: f64,
    ) -> AutonomyLevel {
        // Calculate composite κ score
        let composite_kappa = self.calculate_composite_kappa(kappa_scores);

        // Find the highest autonomy level where both conditions are met:
        // 1. Composite κ score >= threshold for that level
        // 2. Environment complexity >= threshold for that level
        let mut highest_level = AutonomyLevel::Level0Scripted;

        for level in [
            AutonomyLevel::Level0Scripted,
            AutonomyLevel::Level1GoalOriented,
            AutonomyLevel::Level2Adaptive,
            AutonomyLevel::Level3Strategic,
            AutonomyLevel::Level4SelfImproving,
            AutonomyLevel::Level5Collaborative,
            AutonomyLevel::Level6Creative,
            AutonomyLevel::Level7MetaCognitive,
            AutonomyLevel::Level8SelfSustaining,
            AutonomyLevel::Level9Transcendent,
        ] {
            let kappa_threshold = self.config.kappa_thresholds.get(&level).unwrap_or(&0.0);
            let env_threshold = self
                .config
                .environment_thresholds
                .get(&level)
                .unwrap_or(&0.0);

            if composite_kappa >= *kappa_threshold && environment_complexity >= *env_threshold {
                highest_level = level;
            } else {
                break;
            }
        }

        highest_level
    }

    /// Calculate composite κ score from individual axis scores.
    fn calculate_composite_kappa(&self, kappa_scores: &HashMap<CapabilityAxis, KappaScore>) -> f64 {
        if kappa_scores.is_empty() {
            return 0.0;
        }

        let mut total_weighted = 0.0;
        let mut total_weight = 0.0;

        for (axis, score) in kappa_scores {
            let axis_weight = self.config.axis_weights.get(axis).unwrap_or(&1.0);
            let weight = score.confidence * axis_weight;

            total_weighted += score.score * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            total_weighted / total_weight
        } else {
            0.0
        }
    }

    /// Measure autonomy for an agent.
    pub fn measure_autonomy(
        &self,
        agent_id: &str,
        environment_complexity: f64,
    ) -> Option<AutonomyMeasurement> {
        // Calculate κ scores
        let kappa_scores = self.calculate_kappa_scores(agent_id);

        if kappa_scores.is_empty() {
            return None;
        }

        // Determine autonomy level
        let autonomy_level = self.determine_autonomy_level(&kappa_scores, environment_complexity);

        // Count total tasks observed
        let observations = self.observations.read().unwrap();
        let tasks_observed = observations
            .get(agent_id)
            .map(|obs| obs.len() as u32)
            .unwrap_or(0);

        // Create measurement
        let measurement = AutonomyMeasurement::new(
            agent_id.to_string(),
            autonomy_level,
            kappa_scores,
            tasks_observed,
            environment_complexity,
        );

        // Store last measurement
        let mut last_measurements = self.last_measurements.write().unwrap();
        last_measurements.insert(agent_id.to_string(), measurement.clone());

        // Update progress tracking
        self.update_progress_tracking(agent_id, &measurement);

        Some(measurement)
    }

    /// Update progress tracking for an agent.
    fn update_progress_tracking(&self, agent_id: &str, measurement: &AutonomyMeasurement) {
        let mut progress_tracking = self.progress_tracking.write().unwrap();

        if let Some(progress) = progress_tracking.get_mut(agent_id) {
            progress.add_measurement(measurement.clone());
        } else {
            // Create new progress tracking
            let target_level = AutonomyLevel::Level9Transcendent; // Default target
            let mut progress = AutonomyProgress::new(
                agent_id.to_string(),
                measurement.autonomy_level,
                target_level,
            );
            progress.add_measurement(measurement.clone());
            progress_tracking.insert(agent_id.to_string(), progress);
        }
    }

    /// Get the last autonomy measurement for an agent.
    pub fn get_last_measurement(&self, agent_id: &str) -> Option<AutonomyMeasurement> {
        let last_measurements = self.last_measurements.read().unwrap();
        last_measurements.get(agent_id).cloned()
    }

    /// Get progress tracking for an agent.
    pub fn get_progress_tracking(&self, agent_id: &str) -> Option<AutonomyProgress> {
        let progress_tracking = self.progress_tracking.read().unwrap();
        progress_tracking.get(agent_id).cloned()
    }

    /// Get recommendations for improving autonomy.
    pub fn get_improvement_recommendations(&self, agent_id: &str) -> Vec<(CapabilityAxis, String)> {
        let mut recommendations = Vec::new();

        if let Some(measurement) = self.get_last_measurement(agent_id) {
            // Identify weak axes
            for axis in CapabilityAxis::all() {
                if let Some(score) = measurement.get_kappa_score(axis)
                    && score.score < 0.5
                {
                    let recommendation = match axis {
                        CapabilityAxis::Planning => {
                            "Practice complex task decomposition and multi-step planning. Use DTG to visualize execution paths."
                        }
                        CapabilityAxis::Execution => {
                            "Focus on task completion reliability. Implement better error handling and retry mechanisms."
                        }
                        CapabilityAxis::Learning => {
                            "Incorporate more learning from experience. Use reinforcement learning for adaptive behavior."
                        }
                        CapabilityAxis::Adaptation => {
                            "Test in more diverse environments. Implement dynamic strategy switching."
                        }
                        CapabilityAxis::Reasoning => {
                            "Work on logical reasoning tasks. Use chain-of-thought prompting for complex problems."
                        }
                        CapabilityAxis::Creativity => {
                            "Practice generating novel solutions. Use brainstorming techniques and constraint relaxation."
                        }
                        CapabilityAxis::Collaboration => {
                            "Engage in more multi-agent tasks. Practice communication and coordination protocols."
                        }
                        CapabilityAxis::SelfAssessment => {
                            "Implement regular self-assessment routines. Compare self-evaluations with external feedback."
                        }
                        CapabilityAxis::ResourceManagement => {
                            "Optimize resource allocation. Practice budget-aware task execution."
                        }
                        CapabilityAxis::MetaCognition => {
                            "Reflect on thinking processes. Implement meta-reasoning about problem-solving strategies."
                        }
                    };

                    recommendations.push((axis, recommendation.to_string()));
                }
            }
        }

        recommendations
    }

    /// Analyze agent configuration for autonomy potential.
    pub fn analyze_configuration_potential(
        &self,
        config: &HybridAgentConfig,
    ) -> HashMap<CapabilityAxis, f64> {
        let mut potential_scores = HashMap::new();

        // Analyze strategist configuration
        let strategist_potential = config.strategist.capabilities.len() as f64 / 10.0;
        potential_scores.insert(CapabilityAxis::Planning, strategist_potential * 0.8);
        potential_scores.insert(CapabilityAxis::Reasoning, strategist_potential * 0.7);
        potential_scores.insert(CapabilityAxis::Creativity, strategist_potential * 0.6);

        // Analyze executor configurations
        let executor_count = config.executors.len() as f64;
        let max_executors = 10.0; // Reference maximum

        potential_scores.insert(
            CapabilityAxis::Execution,
            (executor_count / max_executors).min(1.0) * 0.9,
        );
        potential_scores.insert(
            CapabilityAxis::Adaptation,
            (executor_count / 3.0).min(1.0) * 0.7,
        );

        // Analyze coordination strategy
        let coordination_potential = match config.coordination.strategy_type {
            crate::models::hybrid_agent::CoordinationStrategyType::Hierarchical => 0.6,
            crate::models::hybrid_agent::CoordinationStrategyType::Collaborative => 0.8,
            crate::models::hybrid_agent::CoordinationStrategyType::Competitive => 0.5,
            crate::models::hybrid_agent::CoordinationStrategyType::MarketBased => 0.7,
            crate::models::hybrid_agent::CoordinationStrategyType::Federated => 0.9,
            crate::models::hybrid_agent::CoordinationStrategyType::Swarm => 0.4,
        };

        potential_scores.insert(CapabilityAxis::Collaboration, coordination_potential * 0.8);

        // Analyze resource allocation
        let resource_potential = match config.resource_allocation.strategy {
            crate::models::hybrid_agent::AllocationStrategy::Static => 0.3,
            crate::models::hybrid_agent::AllocationStrategy::Dynamic => 0.7,
            crate::models::hybrid_agent::AllocationStrategy::Predictive => 0.9,
            crate::models::hybrid_agent::AllocationStrategy::Reactive => 0.5,
            crate::models::hybrid_agent::AllocationStrategy::Optimistic => 0.6,
            crate::models::hybrid_agent::AllocationStrategy::Conservative => 0.4,
        };

        potential_scores.insert(CapabilityAxis::ResourceManagement, resource_potential * 0.8);

        // Set default scores for remaining axes
        for axis in CapabilityAxis::all() {
            potential_scores.entry(axis).or_insert(0.5);
        }

        potential_scores
    }

    /// Estimate time to reach target autonomy level.
    pub fn estimate_time_to_target(
        &self,
        agent_id: &str,
        target_level: AutonomyLevel,
    ) -> Option<Duration> {
        if let Some(progress) = self.get_progress_tracking(agent_id) {
            progress.estimated_time_to_target
        } else {
            None
        }
    }

    /// Check if agent is ready to level up.
    pub fn is_ready_to_level_up(&self, agent_id: &str) -> bool {
        if let Some(measurement) = self.get_last_measurement(agent_id)
            && let Some(next_level) = measurement.autonomy_level.next_level()
        {
            let kappa_threshold = self
                .config
                .kappa_thresholds
                .get(&next_level)
                .unwrap_or(&0.0);
            return measurement.composite_kappa >= *kappa_threshold;
        }
        false
    }

    /// Get all agents sorted by autonomy level.
    pub fn get_agents_by_autonomy(&self) -> Vec<(String, AutonomyLevel, f64)> {
        let last_measurements = self.last_measurements.read().unwrap();
        let mut agents: Vec<_> = last_measurements
            .iter()
            .map(|(id, measurement)| {
                (
                    id.clone(),
                    measurement.autonomy_level,
                    measurement.composite_kappa,
                )
            })
            .collect();

        agents.sort_by(|a, b| {
            b.1.value()
                .cmp(&a.1.value())
                .then(b.2.partial_cmp(&a.2).unwrap())
        });

        agents
    }
}

impl Default for AutonomyMeasurementEngine {
    fn default() -> Self {
        Self::new(MeasurementConfig::default())
    }
}
