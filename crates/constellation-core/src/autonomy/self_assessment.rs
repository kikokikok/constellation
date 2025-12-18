//! Self-assessment engine for autonomous error correction and improvement.

use crate::hybrid::coordinator::TaskResult;
use crate::models::autonomy::{KappaScore, SelfAssessment};
use crate::models::dtg::{DtgNode, DtgNodeStatus};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Configuration for self-assessment.
#[derive(Debug, Clone)]
pub struct SelfAssessmentConfig {
    /// Minimum confidence required for self-assessment.
    pub min_confidence: f64,

    /// Maximum age of task results to consider for assessment.
    pub max_task_age: Duration,

    /// Weight for recent assessments in accuracy calculation.
    pub recency_weight: f64,

    /// Tolerance for self-assessment accuracy validation.
    pub accuracy_tolerance: f64,

    /// Minimum tasks to assess before calculating accuracy.
    pub min_tasks_for_accuracy: u32,

    /// Whether to require external validation for critical assessments.
    pub require_external_validation: bool,

    /// Threshold for critical assessments (requires external validation).
    pub critical_assessment_threshold: f64,
}

impl Default for SelfAssessmentConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.6,
            max_task_age: Duration::from_secs(86400), // 24 hours
            recency_weight: 0.7,
            accuracy_tolerance: 0.2,
            min_tasks_for_accuracy: 5,
            require_external_validation: true,
            critical_assessment_threshold: 0.8,
        }
    }
}

/// Task context for self-assessment.
#[derive(Debug, Clone)]
struct TaskContext {
    task_id: Uuid,
    task_type: String,
    input: serde_json::Value,
    expected_output: Option<serde_json::Value>,
    actual_output: Option<serde_json::Value>,
    execution_time_ms: u64,
    success: bool,
    quality_score: f64,
    timestamp: SystemTime,
}

impl TaskContext {
    fn from_task_result(task_result: &TaskResult) -> Self {
        Self {
            task_id: task_result.task_id,
            task_type: "unknown".to_string(), // Task type not available in TaskResult
            input: serde_json::Value::Null,   // Input not available in TaskResult
            expected_output: None,            // Expected output not available in TaskResult
            actual_output: Some(task_result.result.clone()), // Use result as actual output
            execution_time_ms: task_result.execution_time_ms,
            success: task_result.success,
            quality_score: task_result.quality_score,
            timestamp: SystemTime::now(),
        }
    }
}

/// Self-assessment engine for autonomous error correction.
#[derive(Debug)]
pub struct SelfAssessmentEngine {
    config: SelfAssessmentConfig,
    assessments: Arc<RwLock<HashMap<String, VecDeque<SelfAssessment>>>>,
    task_contexts: Arc<RwLock<HashMap<Uuid, TaskContext>>>,
    accuracy_history: Arc<RwLock<HashMap<String, Vec<f64>>>>,
}

impl SelfAssessmentEngine {
    /// Create a new self-assessment engine.
    pub fn new(config: SelfAssessmentConfig) -> Self {
        Self {
            config,
            assessments: Arc::new(RwLock::new(HashMap::new())),
            task_contexts: Arc::new(RwLock::new(HashMap::new())),
            accuracy_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record task context for future assessment.
    pub fn record_task_context(&self, task_result: &TaskResult) {
        let task_context = TaskContext::from_task_result(task_result);
        let mut task_contexts = self.task_contexts.write().unwrap();
        task_contexts.insert(task_result.task_id, task_context);

        // Clean up old task contexts (inlined to avoid reentrant lock)
        let now = SystemTime::now();
        task_contexts.retain(|_, context| {
            now.duration_since(context.timestamp)
                .map(|age| age <= self.config.max_task_age)
                .unwrap_or(false)
        });
    }

    /// Perform self-assessment for a completed task.
    pub fn assess_task(
        &self,
        agent_id: &str,
        task_id: Uuid,
        self_score: f64,
        confidence: f64,
        strengths: Vec<String>,
        improvements: Vec<String>,
        errors_identified: Vec<String>,
        corrective_actions: Vec<String>,
        assessment_duration: Duration,
    ) -> Option<SelfAssessment> {
        if confidence < self.config.min_confidence {
            return None;
        }

        // Get task context
        let task_contexts = self.task_contexts.read().unwrap();
        let task_context = task_contexts.get(&task_id)?;

        // Create self-assessment
        let assessment = SelfAssessment::new(
            agent_id.to_string(),
            Some(task_id),
            self_score,
            confidence,
            strengths,
            improvements,
            errors_identified,
            corrective_actions,
            assessment_duration,
        );

        // Store assessment
        let mut assessments = self.assessments.write().unwrap();
        let agent_assessments = assessments.entry(agent_id.to_string()).or_default();

        agent_assessments.push_back(assessment.clone());

        // Clean up old assessments (inlined to avoid reentrant lock)
        // Keep only recent assessments
        let max_assessments = self.config.min_tasks_for_accuracy * 2;
        if agent_assessments.len() > max_assessments as usize {
            let to_remove = agent_assessments.len() - max_assessments as usize;
            for _ in 0..to_remove {
                agent_assessments.pop_front();
            }
        }

        // Check if external validation is required
        if self.config.require_external_validation
            && self_score >= self.config.critical_assessment_threshold
        {
            // In a real implementation, this would trigger external validation
            // For now, we'll simulate it with the task success
            let external_score = if task_context.success {
                task_context.quality_score
            } else {
                0.3
            };

            // We would normally wait for actual external validation
            // For simulation, we'll apply it immediately
            let mut assessment = assessment;
            assessment.validate_externally(external_score);

            // Update accuracy history (inlined to avoid reentrant lock)
            if let Some(discrepancy) = assessment.validation_discrepancy() {
                let mut accuracy_history = self.accuracy_history.write().unwrap();
                let agent_history = accuracy_history.entry(agent_id.to_string()).or_default();

                // Accuracy is inverse of discrepancy
                let accuracy = 1.0 - discrepancy.min(1.0);
                agent_history.push(accuracy);

                // Keep only recent history
                let max_history = 100;
                if agent_history.len() > max_history {
                    agent_history.drain(0..agent_history.len() - max_history);
                }
            }

            Some(assessment)
        } else {
            Some(assessment)
        }
    }

    /// Perform automated self-assessment based on task result.
    pub fn assess_task_automated(
        &self,
        agent_id: &str,
        task_result: &TaskResult,
    ) -> Option<SelfAssessment> {
        // Record task context first
        self.record_task_context(task_result);

        // Automated assessment logic
        let self_score = if task_result.success {
            // Successful task: assess based on quality and efficiency
            let quality_factor = task_result.quality_score;
            let efficiency_factor = if task_result.execution_time_ms > 0 {
                let expected_time = 5000.0; // 5 seconds expected
                (expected_time / task_result.execution_time_ms as f64).min(2.0) / 2.0
            } else {
                0.5
            };

            (quality_factor * 0.7 + efficiency_factor * 0.3).clamp(0.0, 1.0)
        } else {
            // Failed task: lower score
            0.3
        };

        let confidence = if task_result.success { 0.7 } else { 0.8 };

        // Generate strengths and improvements
        let strengths = self.identify_strengths(task_result);
        let improvements = self.identify_improvements(task_result);
        let errors_identified = self.identify_errors(task_result);
        let corrective_actions = self.suggest_corrective_actions(task_result);

        self.assess_task(
            agent_id,
            task_result.task_id,
            self_score,
            confidence,
            strengths,
            improvements,
            errors_identified,
            corrective_actions,
            Duration::from_millis(100), // Simulated assessment duration
        )
    }

    /// Perform self-assessment based on DTG node.
    pub fn assess_dtg_node(&self, agent_id: &str, dtg_node: &DtgNode) -> Option<SelfAssessment> {
        // Convert DTG node to assessment
        let self_score = match dtg_node.status {
            DtgNodeStatus::Completed => dtg_node.metrics.quality_score,
            DtgNodeStatus::Failed => 0.3,
            _ => 0.5,
        };

        let confidence = 0.6;

        let strengths = if matches!(dtg_node.status, DtgNodeStatus::Completed) {
            vec!["Successfully completed transformation".to_string()]
        } else {
            Vec::new()
        };

        let improvements = if matches!(dtg_node.status, DtgNodeStatus::Failed) {
            vec!["Improve transformation reliability".to_string()]
        } else {
            Vec::new()
        };

        let errors_identified = Vec::new(); // Would analyze error logs in real implementation
        let corrective_actions = Vec::new();

        self.assess_task(
            agent_id,
            dtg_node.id,
            self_score,
            confidence,
            strengths,
            improvements,
            errors_identified,
            corrective_actions,
            Duration::from_millis(50),
        )
    }

    /// Identify strengths from task result.
    fn identify_strengths(&self, task_result: &TaskResult) -> Vec<String> {
        let mut strengths = Vec::new();

        if task_result.success {
            strengths.push("Task completion".to_string());

            if task_result.quality_score >= 0.8 {
                strengths.push("High quality output".to_string());
            }

            if task_result.execution_time_ms < 2000 {
                strengths.push("Fast execution".to_string());
            }
        }

        strengths
    }

    /// Identify improvements from task result.
    fn identify_improvements(&self, task_result: &TaskResult) -> Vec<String> {
        let mut improvements = Vec::new();

        if !task_result.success {
            improvements.push("Task completion reliability".to_string());
        }

        if task_result.quality_score < 0.7 {
            improvements.push("Output quality".to_string());
        }

        if task_result.execution_time_ms > 10000 {
            improvements.push("Execution efficiency".to_string());
        }

        improvements
    }

    /// Identify errors from task result.
    fn identify_errors(&self, task_result: &TaskResult) -> Vec<String> {
        let mut errors = Vec::new();

        if !task_result.success {
            errors.push("Task execution failed".to_string());

            if let Some(error) = &task_result.error {
                errors.push(format!("Error: {error}"));
            }
        }

        errors
    }

    /// Suggest corrective actions from task result.
    fn suggest_corrective_actions(&self, task_result: &TaskResult) -> Vec<String> {
        let mut actions = Vec::new();

        if !task_result.success {
            actions.push("Review and fix implementation errors".to_string());
            actions.push("Add better error handling".to_string());
            actions.push("Test with simpler inputs first".to_string());
        }

        if task_result.quality_score < 0.7 {
            actions.push("Improve output validation".to_string());
            actions.push("Add quality checks during execution".to_string());
        }

        if task_result.execution_time_ms > 10000 {
            actions.push("Optimize algorithm efficiency".to_string());
            actions.push("Consider parallel execution".to_string());
        }

        actions
    }

    /// Validate a self-assessment externally.
    pub fn validate_externally(
        &self,
        agent_id: &str,
        assessment_id: Uuid,
        external_score: f64,
    ) -> bool {
        let mut assessments = self.assessments.write().unwrap();

        if let Some(agent_assessments) = assessments.get_mut(agent_id) {
            for assessment in agent_assessments.iter_mut() {
                if assessment.id == assessment_id {
                    assessment.validate_externally(external_score);

                    // Update accuracy history (inlined to avoid reentrant lock)
                    if let Some(discrepancy) = assessment.validation_discrepancy() {
                        let mut accuracy_history = self.accuracy_history.write().unwrap();
                        let agent_history =
                            accuracy_history.entry(agent_id.to_string()).or_default();

                        // Accuracy is inverse of discrepancy
                        let accuracy = 1.0 - discrepancy.min(1.0);
                        agent_history.push(accuracy);

                        // Keep only recent history
                        let max_history = 100;
                        if agent_history.len() > max_history {
                            agent_history.drain(0..agent_history.len() - max_history);
                        }
                    }

                    return true;
                }
            }
        }

        false
    }

    /// Get self-assessment accuracy for an agent.
    pub fn get_accuracy(&self, agent_id: &str) -> Option<f64> {
        let accuracy_history = self.accuracy_history.read().unwrap();
        let agent_history = accuracy_history.get(agent_id)?;

        if agent_history.is_empty() {
            return None;
        }

        // Calculate weighted average with recency bias
        let mut total_weighted = 0.0;
        let mut total_weight = 0.0;

        for (i, accuracy) in agent_history.iter().enumerate() {
            let recency = i as f64 / agent_history.len() as f64;
            let weight =
                self.config.recency_weight * (1.0 - recency) + (1.0 - self.config.recency_weight);

            total_weighted += accuracy * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            Some(total_weighted / total_weight)
        } else {
            None
        }
    }

    /// Check if agent's self-assessments are accurate.
    pub fn is_accurate_assessor(&self, agent_id: &str) -> bool {
        self.get_accuracy(agent_id)
            .map(|accuracy| accuracy >= 1.0 - self.config.accuracy_tolerance)
            .unwrap_or(false)
    }

    /// Get recent self-assessments for an agent.
    pub fn get_recent_assessments(&self, agent_id: &str, limit: usize) -> Vec<SelfAssessment> {
        let assessments = self.assessments.read().unwrap();

        assessments
            .get(agent_id)
            .map(|agent_assessments| {
                agent_assessments
                    .iter()
                    .rev()
                    .take(limit)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get improvement trends from self-assessments.
    pub fn get_improvement_trends(&self, agent_id: &str) -> HashMap<String, f64> {
        let assessments = self.get_recent_assessments(agent_id, 20);
        let mut trends = HashMap::new();

        if assessments.len() < 2 {
            return trends;
        }

        // Calculate improvement in self-assessment scores
        let first_score = assessments.first().unwrap().score;
        let last_score = assessments.last().unwrap().score;
        let score_improvement = last_score - first_score;

        trends.insert("self_assessment_score".to_string(), score_improvement);

        // Calculate improvement in accuracy if available
        if let Some(accuracy) = self.get_accuracy(agent_id) {
            trends.insert("assessment_accuracy".to_string(), accuracy);
        }

        // Count frequency of improvement areas
        let mut improvement_counts = HashMap::new();
        for assessment in &assessments {
            for improvement in &assessment.improvements {
                *improvement_counts.entry(improvement.clone()).or_insert(0) += 1;
            }
        }

        // Convert counts to frequencies
        for (improvement, count) in improvement_counts {
            let frequency = count as f64 / assessments.len() as f64;
            trends.insert(format!("improvement_frequency:{improvement}"), frequency);
        }

        trends
    }

    /// Generate self-assessment report for an agent.
    pub fn generate_report(&self, agent_id: &str) -> Option<SelfAssessmentReport> {
        let assessments = self.get_recent_assessments(agent_id, 50);

        if assessments.is_empty() {
            return None;
        }

        let accuracy = self.get_accuracy(agent_id).unwrap_or(0.0);
        let is_accurate = self.is_accurate_assessor(agent_id);

        // Calculate average scores
        let avg_self_score: f64 =
            assessments.iter().map(|a| a.score).sum::<f64>() / assessments.len() as f64;
        let avg_confidence: f64 =
            assessments.iter().map(|a| a.confidence).sum::<f64>() / assessments.len() as f64;

        // Identify common strengths and improvements
        let mut strength_counts = HashMap::new();
        let mut improvement_counts = HashMap::new();

        for assessment in &assessments {
            for strength in &assessment.strengths {
                *strength_counts.entry(strength.clone()).or_insert(0) += 1;
            }
            for improvement in &assessment.improvements {
                *improvement_counts.entry(improvement.clone()).or_insert(0) += 1;
            }
        }

        let common_strengths: Vec<_> = strength_counts
            .into_iter()
            .filter(|(_, count)| *count >= assessments.len() / 3)
            .map(|(strength, _)| strength)
            .collect();

        let common_improvements: Vec<_> = improvement_counts
            .into_iter()
            .filter(|(_, count)| *count >= assessments.len() / 3)
            .map(|(improvement, _)| improvement)
            .collect();

        // Calculate trend
        let trend = if assessments.len() >= 3 {
            let first_third: f64 = assessments[..assessments.len() / 3]
                .iter()
                .map(|a| a.score)
                .sum::<f64>()
                / (assessments.len() / 3) as f64;

            let last_third: f64 = assessments[assessments.len() * 2 / 3..]
                .iter()
                .map(|a| a.score)
                .sum::<f64>()
                / (assessments.len() / 3) as f64;

            last_third - first_third
        } else {
            0.0
        };

        Some(SelfAssessmentReport {
            agent_id: agent_id.to_string(),
            total_assessments: assessments.len(),
            accuracy,
            is_accurate,
            avg_self_score,
            avg_confidence,
            common_strengths,
            common_improvements,
            trend,
            recommendations: self.generate_recommendations(accuracy, is_accurate, trend),
        })
    }

    /// Generate recommendations based on assessment data.
    fn generate_recommendations(
        &self,
        accuracy: f64,
        is_accurate: bool,
        trend: f64,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !is_accurate {
            recommendations
                .push("Improve self-assessment accuracy through external validation".to_string());
            recommendations.push("Practice objective self-evaluation".to_string());
        }

        if accuracy < 0.7 {
            recommendations.push("Increase frequency of external validation".to_string());
            recommendations.push("Calibrate self-assessment against objective metrics".to_string());
        }

        if trend < -0.1 {
            recommendations.push("Address declining self-assessment scores".to_string());
            recommendations.push("Review recent task failures".to_string());
        } else if trend > 0.1 {
            recommendations.push("Continue current improvement strategies".to_string());
        }

        recommendations
    }

    /// Estimate self-assessment capability κ score.
    pub fn estimate_self_assessment_kappa(&self, agent_id: &str) -> Option<KappaScore> {
        let accuracy = self.get_accuracy(agent_id)?;
        let assessments = self.get_recent_assessments(agent_id, 20);

        if assessments.is_empty() {
            return None;
        }

        // Calculate score based on accuracy and consistency
        let avg_confidence: f64 =
            assessments.iter().map(|a| a.confidence).sum::<f64>() / assessments.len() as f64;

        // Score is weighted combination of accuracy and confidence calibration
        let accuracy_weight = 0.7;
        let confidence_weight = 0.3;

        let score = accuracy * accuracy_weight + avg_confidence * confidence_weight;
        let confidence = (assessments.len() as f64 / 20.0).min(1.0) * 0.8;

        Some(KappaScore::new(
            crate::models::autonomy::CapabilityAxis::SelfAssessment,
            score,
            confidence,
            assessments.len() as u32,
        ))
    }
}

/// Self-assessment report.
#[derive(Debug, Clone)]
pub struct SelfAssessmentReport {
    pub agent_id: String,
    pub total_assessments: usize,
    pub accuracy: f64,
    pub is_accurate: bool,
    pub avg_self_score: f64,
    pub avg_confidence: f64,
    pub common_strengths: Vec<String>,
    pub common_improvements: Vec<String>,
    pub trend: f64,
    pub recommendations: Vec<String>,
}

impl Default for SelfAssessmentEngine {
    fn default() -> Self {
        Self::new(SelfAssessmentConfig::default())
    }
}
