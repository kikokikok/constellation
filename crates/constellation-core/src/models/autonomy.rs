//! Autonomy measurement models for Kardashev-style AI capability scaling.
//!
//! Implements a 10-axis capability measurement system with κ (kappa) scoring
//! for tracking autonomous AI progress toward AGI-scale capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Kardashev-style autonomy level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AutonomyLevel {
    /// Level 0: Scripted/Reactive
    /// Basic rule-based or scripted behavior with no adaptation.
    Level0Scripted,

    /// Level 1: Goal-Oriented
    /// Can pursue simple goals with limited environmental understanding.
    Level1GoalOriented,

    /// Level 2: Adaptive
    /// Learns from experience and adapts to changing environments.
    Level2Adaptive,

    /// Level 3: Strategic
    /// Long-term planning, resource management, and strategic thinking.
    Level3Strategic,

    /// Level 4: Self-Improving
    /// Modifies own architecture and algorithms for improvement.
    Level4SelfImproving,

    /// Level 5: Collaborative
    /// Forms effective teams and collaborates with other agents/humans.
    Level5Collaborative,

    /// Level 6: Creative
    /// Generates novel solutions and creative approaches to problems.
    Level6Creative,

    /// Level 7: Meta-Cognitive
    /// Reflects on own thinking processes and biases.
    Level7MetaCognitive,

    /// Level 8: Self-Sustaining
    /// Maintains and improves itself without external intervention.
    Level8SelfSustaining,

    /// Level 9: Transcendent
    /// Creates new knowledge domains and fundamentally new capabilities.
    Level9Transcendent,
}

impl AutonomyLevel {
    /// Get the numeric value of the autonomy level.
    pub fn value(&self) -> u8 {
        match self {
            AutonomyLevel::Level0Scripted => 0,
            AutonomyLevel::Level1GoalOriented => 1,
            AutonomyLevel::Level2Adaptive => 2,
            AutonomyLevel::Level3Strategic => 3,
            AutonomyLevel::Level4SelfImproving => 4,
            AutonomyLevel::Level5Collaborative => 5,
            AutonomyLevel::Level6Creative => 6,
            AutonomyLevel::Level7MetaCognitive => 7,
            AutonomyLevel::Level8SelfSustaining => 8,
            AutonomyLevel::Level9Transcendent => 9,
        }
    }

    /// Get the description of the autonomy level.
    pub fn description(&self) -> &'static str {
        match self {
            AutonomyLevel::Level0Scripted => "Scripted/Reactive: Basic rule-based behavior",
            AutonomyLevel::Level1GoalOriented => "Goal-Oriented: Pursues simple goals",
            AutonomyLevel::Level2Adaptive => "Adaptive: Learns from experience",
            AutonomyLevel::Level3Strategic => "Strategic: Long-term planning",
            AutonomyLevel::Level4SelfImproving => "Self-Improving: Modifies own architecture",
            AutonomyLevel::Level5Collaborative => "Collaborative: Forms effective teams",
            AutonomyLevel::Level6Creative => "Creative: Generates novel solutions",
            AutonomyLevel::Level7MetaCognitive => "Meta-Cognitive: Reflects on own thinking",
            AutonomyLevel::Level8SelfSustaining => "Self-Sustaining: Maintains itself autonomously",
            AutonomyLevel::Level9Transcendent => "Transcendent: Creates new knowledge domains",
        }
    }

    /// Get the next level in the progression.
    pub fn next_level(&self) -> Option<Self> {
        match self {
            AutonomyLevel::Level0Scripted => Some(AutonomyLevel::Level1GoalOriented),
            AutonomyLevel::Level1GoalOriented => Some(AutonomyLevel::Level2Adaptive),
            AutonomyLevel::Level2Adaptive => Some(AutonomyLevel::Level3Strategic),
            AutonomyLevel::Level3Strategic => Some(AutonomyLevel::Level4SelfImproving),
            AutonomyLevel::Level4SelfImproving => Some(AutonomyLevel::Level5Collaborative),
            AutonomyLevel::Level5Collaborative => Some(AutonomyLevel::Level6Creative),
            AutonomyLevel::Level6Creative => Some(AutonomyLevel::Level7MetaCognitive),
            AutonomyLevel::Level7MetaCognitive => Some(AutonomyLevel::Level8SelfSustaining),
            AutonomyLevel::Level8SelfSustaining => Some(AutonomyLevel::Level9Transcendent),
            AutonomyLevel::Level9Transcendent => None,
        }
    }

    /// Get the previous level in the progression.
    pub fn previous_level(&self) -> Option<Self> {
        match self {
            AutonomyLevel::Level0Scripted => None,
            AutonomyLevel::Level1GoalOriented => Some(AutonomyLevel::Level0Scripted),
            AutonomyLevel::Level2Adaptive => Some(AutonomyLevel::Level1GoalOriented),
            AutonomyLevel::Level3Strategic => Some(AutonomyLevel::Level2Adaptive),
            AutonomyLevel::Level4SelfImproving => Some(AutonomyLevel::Level3Strategic),
            AutonomyLevel::Level5Collaborative => Some(AutonomyLevel::Level4SelfImproving),
            AutonomyLevel::Level6Creative => Some(AutonomyLevel::Level5Collaborative),
            AutonomyLevel::Level7MetaCognitive => Some(AutonomyLevel::Level6Creative),
            AutonomyLevel::Level8SelfSustaining => Some(AutonomyLevel::Level7MetaCognitive),
            AutonomyLevel::Level9Transcendent => Some(AutonomyLevel::Level8SelfSustaining),
        }
    }
}

/// Capability axis for autonomy measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityAxis {
    /// Planning: Ability to create and execute complex plans.
    Planning,

    /// Execution: Ability to reliably execute tasks and skills.
    Execution,

    /// Learning: Ability to learn from experience and data.
    Learning,

    /// Adaptation: Ability to adapt to new environments and constraints.
    Adaptation,

    /// Reasoning: Logical and abstract reasoning capabilities.
    Reasoning,

    /// Creativity: Ability to generate novel solutions and ideas.
    Creativity,

    /// Collaboration: Ability to work effectively with other agents/humans.
    Collaboration,

    /// SelfAssessment: Ability to evaluate own performance and limitations.
    SelfAssessment,

    /// ResourceManagement: Ability to manage computational and physical resources.
    ResourceManagement,

    /// MetaCognition: Ability to reflect on and improve own cognitive processes.
    MetaCognition,
}

impl CapabilityAxis {
    /// Get all capability axes.
    pub fn all() -> Vec<Self> {
        vec![
            CapabilityAxis::Planning,
            CapabilityAxis::Execution,
            CapabilityAxis::Learning,
            CapabilityAxis::Adaptation,
            CapabilityAxis::Reasoning,
            CapabilityAxis::Creativity,
            CapabilityAxis::Collaboration,
            CapabilityAxis::SelfAssessment,
            CapabilityAxis::ResourceManagement,
            CapabilityAxis::MetaCognition,
        ]
    }

    /// Get the name of the capability axis.
    pub fn name(&self) -> &'static str {
        match self {
            CapabilityAxis::Planning => "Planning",
            CapabilityAxis::Execution => "Execution",
            CapabilityAxis::Learning => "Learning",
            CapabilityAxis::Adaptation => "Adaptation",
            CapabilityAxis::Reasoning => "Reasoning",
            CapabilityAxis::Creativity => "Creativity",
            CapabilityAxis::Collaboration => "Collaboration",
            CapabilityAxis::SelfAssessment => "Self-Assessment",
            CapabilityAxis::ResourceManagement => "Resource Management",
            CapabilityAxis::MetaCognition => "Meta-Cognition",
        }
    }

    /// Get the description of the capability axis.
    pub fn description(&self) -> &'static str {
        match self {
            CapabilityAxis::Planning => "Ability to create and execute complex plans",
            CapabilityAxis::Execution => "Ability to reliably execute tasks and skills",
            CapabilityAxis::Learning => "Ability to learn from experience and data",
            CapabilityAxis::Adaptation => "Ability to adapt to new environments and constraints",
            CapabilityAxis::Reasoning => "Logical and abstract reasoning capabilities",
            CapabilityAxis::Creativity => "Ability to generate novel solutions and ideas",
            CapabilityAxis::Collaboration => "Ability to work effectively with other agents/humans",
            CapabilityAxis::SelfAssessment => "Ability to evaluate own performance and limitations",
            CapabilityAxis::ResourceManagement => {
                "Ability to manage computational and physical resources"
            }
            CapabilityAxis::MetaCognition => {
                "Ability to reflect on and improve own cognitive processes"
            }
        }
    }
}

/// κ (kappa) score for a capability axis (0.0 to 1.0).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KappaScore {
    /// The capability axis being measured.
    pub axis: CapabilityAxis,

    /// κ score (0.0 to 1.0).
    pub score: f64,

    /// Confidence in the measurement (0.0 to 1.0).
    pub confidence: f64,

    /// Timestamp of the measurement.
    pub timestamp: SystemTime,

    /// Number of observations contributing to this score.
    pub observation_count: u32,
}

impl KappaScore {
    /// Create a new κ score.
    pub fn new(axis: CapabilityAxis, score: f64, confidence: f64, observation_count: u32) -> Self {
        Self {
            axis,
            score: score.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            timestamp: SystemTime::now(),
            observation_count,
        }
    }

    /// Check if the score is valid (within bounds).
    pub fn is_valid(&self) -> bool {
        (0.0..=1.0).contains(&self.score) && (0.0..=1.0).contains(&self.confidence)
    }

    /// Get the weighted score (score * confidence).
    pub fn weighted_score(&self) -> f64 {
        self.score * self.confidence
    }
}

/// Autonomy measurement for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomyMeasurement {
    /// Unique identifier for this measurement.
    pub id: Uuid,

    /// Agent identifier.
    pub agent_id: String,

    /// Overall autonomy level.
    pub autonomy_level: AutonomyLevel,

    /// κ scores for each capability axis.
    pub kappa_scores: HashMap<CapabilityAxis, KappaScore>,

    /// Composite κ score (weighted average of all axes).
    pub composite_kappa: f64,

    /// Measurement timestamp.
    pub timestamp: SystemTime,

    /// Measurement duration.
    pub measurement_duration: Duration,

    /// Number of tasks observed during measurement.
    pub tasks_observed: u32,

    /// Environment complexity score (0.0 to 1.0).
    pub environment_complexity: f64,

    /// Self-assessment score (0.0 to 1.0).
    pub self_assessment_score: f64,

    /// External validation score (0.0 to 1.0).
    pub external_validation_score: f64,

    /// Metadata about the measurement context.
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AutonomyMeasurement {
    /// Create a new autonomy measurement.
    pub fn new(
        agent_id: String,
        autonomy_level: AutonomyLevel,
        kappa_scores: HashMap<CapabilityAxis, KappaScore>,
        tasks_observed: u32,
        environment_complexity: f64,
    ) -> Self {
        let composite_kappa = Self::calculate_composite_kappa(&kappa_scores);

        Self {
            id: Uuid::new_v4(),
            agent_id,
            autonomy_level,
            kappa_scores,
            composite_kappa,
            timestamp: SystemTime::now(),
            measurement_duration: Duration::default(),
            tasks_observed,
            environment_complexity: environment_complexity.clamp(0.0, 1.0),
            self_assessment_score: 0.0,
            external_validation_score: 0.0,
            metadata: HashMap::new(),
        }
    }

    /// Calculate composite κ score from individual axis scores.
    fn calculate_composite_kappa(kappa_scores: &HashMap<CapabilityAxis, KappaScore>) -> f64 {
        if kappa_scores.is_empty() {
            return 0.0;
        }

        let mut total_weighted = 0.0;
        let mut total_confidence = 0.0;

        for score in kappa_scores.values() {
            total_weighted += score.weighted_score();
            total_confidence += score.confidence;
        }

        if total_confidence > 0.0 {
            total_weighted / total_confidence
        } else {
            0.0
        }
    }

    /// Update composite κ score based on current scores.
    pub fn update_composite_kappa(&mut self) {
        self.composite_kappa = Self::calculate_composite_kappa(&self.kappa_scores);
    }

    /// Add or update a κ score for a capability axis.
    pub fn update_kappa_score(&mut self, score: KappaScore) {
        self.kappa_scores.insert(score.axis, score);
        self.update_composite_kappa();
    }

    /// Get the κ score for a specific capability axis.
    pub fn get_kappa_score(&self, axis: CapabilityAxis) -> Option<&KappaScore> {
        self.kappa_scores.get(&axis)
    }

    /// Calculate progress toward next autonomy level.
    pub fn progress_to_next_level(&self) -> f64 {
        let current_level_value = self.autonomy_level.value() as f64;
        let max_level_value = AutonomyLevel::Level9Transcendent.value() as f64;

        if current_level_value >= max_level_value {
            1.0
        } else {
            current_level_value / max_level_value
        }
    }

    /// Get the weakest capability axis (lowest κ score).
    pub fn weakest_axis(&self) -> Option<(&CapabilityAxis, &KappaScore)> {
        self.kappa_scores
            .iter()
            .min_by(|(_, a), (_, b)| a.score.partial_cmp(&b.score).unwrap())
    }

    /// Get the strongest capability axis (highest κ score).
    pub fn strongest_axis(&self) -> Option<(&CapabilityAxis, &KappaScore)> {
        self.kappa_scores
            .iter()
            .max_by(|(_, a), (_, b)| a.score.partial_cmp(&b.score).unwrap())
    }

    /// Check if the agent should level up based on κ scores.
    pub fn should_level_up(&self, threshold: f64) -> bool {
        self.composite_kappa >= threshold && self.progress_to_next_level() >= 0.8
    }
}

/// Self-assessment result from an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfAssessment {
    /// Unique identifier for this assessment.
    pub id: Uuid,

    /// Agent identifier.
    pub agent_id: String,

    /// Task identifier that triggered the assessment.
    pub task_id: Option<Uuid>,

    /// Self-assessment score (0.0 to 1.0).
    pub score: f64,

    /// Confidence in the self-assessment (0.0 to 1.0).
    pub confidence: f64,

    /// Areas of strength identified.
    pub strengths: Vec<String>,

    /// Areas for improvement identified.
    pub improvements: Vec<String>,

    /// Specific errors or issues identified.
    pub errors_identified: Vec<String>,

    /// Proposed corrective actions.
    pub corrective_actions: Vec<String>,

    /// Timestamp of the assessment.
    pub timestamp: SystemTime,

    /// Duration of the assessment.
    pub assessment_duration: Duration,

    /// Whether this assessment was validated externally.
    pub externally_validated: bool,

    /// External validation score if available.
    pub external_validation_score: Option<f64>,
}

impl SelfAssessment {
    /// Create a new self-assessment.
    pub fn new(
        agent_id: String,
        task_id: Option<Uuid>,
        score: f64,
        confidence: f64,
        strengths: Vec<String>,
        improvements: Vec<String>,
        errors_identified: Vec<String>,
        corrective_actions: Vec<String>,
        assessment_duration: Duration,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_id,
            task_id,
            score: score.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            strengths,
            improvements,
            errors_identified,
            corrective_actions,
            timestamp: SystemTime::now(),
            assessment_duration,
            externally_validated: false,
            external_validation_score: None,
        }
    }

    /// Validate the self-assessment with an external score.
    pub fn validate_externally(&mut self, external_score: f64) {
        self.externally_validated = true;
        self.external_validation_score = Some(external_score.clamp(0.0, 1.0));
    }

    /// Calculate the validation discrepancy.
    pub fn validation_discrepancy(&self) -> Option<f64> {
        self.external_validation_score
            .map(|external| (self.score - external).abs())
    }

    /// Check if the self-assessment is accurate (within tolerance).
    pub fn is_accurate(&self, tolerance: f64) -> bool {
        self.validation_discrepancy()
            .map(|discrepancy| discrepancy <= tolerance)
            .unwrap_or(false)
    }
}

/// Emergent collaboration pattern detected between agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationPattern {
    /// Unique identifier for this pattern.
    pub id: Uuid,

    /// Agent identifiers involved in the collaboration.
    pub agent_ids: Vec<String>,

    /// Pattern type.
    pub pattern_type: CollaborationPatternType,

    /// Strength of the pattern (0.0 to 1.0).
    pub strength: f64,

    /// Efficiency of the collaboration (0.0 to 1.0).
    pub efficiency: f64,

    /// Number of successful collaborations observed.
    pub success_count: u32,

    /// Number of failed collaborations observed.
    pub failure_count: u32,

    /// First observation timestamp.
    pub first_observed: SystemTime,

    /// Last observation timestamp.
    pub last_observed: SystemTime,

    /// Total observation duration.
    pub total_observation_duration: Duration,

    /// Metadata about the collaboration context.
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Type of emergent collaboration pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CollaborationPatternType {
    /// Hierarchical: Clear leader-follower structure.
    Hierarchical,

    /// Distributed: Equal peers with distributed decision-making.
    Distributed,

    /// Specialized: Agents with complementary specialized skills.
    Specialized,

    /// Adaptive: Structure changes based on task requirements.
    Adaptive,

    /// Swarm: Many simple agents achieving complex behavior.
    Swarm,

    /// MarketBased: Resource allocation through bidding/auction.
    MarketBased,

    /// Consensus: Decisions made through consensus mechanisms.
    Consensus,

    /// MentorApprentice: Knowledge transfer between agents.
    MentorApprentice,

    /// Competitive: Agents compete to improve performance.
    Competitive,

    /// Hybrid: Combination of multiple patterns.
    Hybrid,
}

impl CollaborationPatternType {
    /// Get all collaboration pattern types.
    pub fn all() -> Vec<Self> {
        vec![
            CollaborationPatternType::Hierarchical,
            CollaborationPatternType::Distributed,
            CollaborationPatternType::Specialized,
            CollaborationPatternType::Adaptive,
            CollaborationPatternType::Swarm,
            CollaborationPatternType::MarketBased,
            CollaborationPatternType::Consensus,
            CollaborationPatternType::MentorApprentice,
            CollaborationPatternType::Competitive,
            CollaborationPatternType::Hybrid,
        ]
    }

    /// Get the name of the pattern type.
    pub fn name(&self) -> &'static str {
        match self {
            CollaborationPatternType::Hierarchical => "Hierarchical",
            CollaborationPatternType::Distributed => "Distributed",
            CollaborationPatternType::Specialized => "Specialized",
            CollaborationPatternType::Adaptive => "Adaptive",
            CollaborationPatternType::Swarm => "Swarm",
            CollaborationPatternType::MarketBased => "Market-Based",
            CollaborationPatternType::Consensus => "Consensus",
            CollaborationPatternType::MentorApprentice => "Mentor-Apprentice",
            CollaborationPatternType::Competitive => "Competitive",
            CollaborationPatternType::Hybrid => "Hybrid",
        }
    }

    /// Get the description of the pattern type.
    pub fn description(&self) -> &'static str {
        match self {
            CollaborationPatternType::Hierarchical => {
                "Clear leader-follower structure with centralized decision-making"
            }
            CollaborationPatternType::Distributed => {
                "Equal peers with distributed decision-making and coordination"
            }
            CollaborationPatternType::Specialized => {
                "Agents with complementary specialized skills working together"
            }
            CollaborationPatternType::Adaptive => {
                "Structure changes dynamically based on task requirements"
            }
            CollaborationPatternType::Swarm => {
                "Many simple agents achieving complex emergent behavior"
            }
            CollaborationPatternType::MarketBased => {
                "Resource allocation and task assignment through bidding/auction mechanisms"
            }
            CollaborationPatternType::Consensus => {
                "Decisions made through consensus mechanisms among agents"
            }
            CollaborationPatternType::MentorApprentice => {
                "Knowledge transfer between experienced and learning agents"
            }
            CollaborationPatternType::Competitive => {
                "Agents compete to improve overall system performance"
            }
            CollaborationPatternType::Hybrid => "Combination of multiple collaboration patterns",
        }
    }
}

impl CollaborationPattern {
    /// Create a new collaboration pattern.
    pub fn new(
        agent_ids: Vec<String>,
        pattern_type: CollaborationPatternType,
        strength: f64,
        efficiency: f64,
        success_count: u32,
        failure_count: u32,
    ) -> Self {
        let now = SystemTime::now();

        Self {
            id: Uuid::new_v4(),
            agent_ids,
            pattern_type,
            strength: strength.clamp(0.0, 1.0),
            efficiency: efficiency.clamp(0.0, 1.0),
            success_count,
            failure_count,
            first_observed: now,
            last_observed: now,
            total_observation_duration: Duration::default(),
            metadata: HashMap::new(),
        }
    }

    /// Calculate success rate.
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.failure_count;
        if total > 0 {
            self.success_count as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Update pattern with new observation.
    pub fn update_with_observation(&mut self, success: bool, efficiency: f64, duration: Duration) {
        if success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }

        // Update efficiency with moving average
        let total_observations = self.success_count + self.failure_count;
        self.efficiency = (self.efficiency * (total_observations - 1) as f64 + efficiency)
            / total_observations as f64;

        self.last_observed = SystemTime::now();
        self.total_observation_duration += duration;
    }

    /// Check if pattern is stable (consistent over time).
    pub fn is_stable(&self, min_observations: u32, min_strength: f64) -> bool {
        let total_observations = self.success_count + self.failure_count;
        total_observations >= min_observations && self.strength >= min_strength
    }
}

/// Benchmark for autonomy measurement comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomyBenchmark {
    /// Unique identifier for this benchmark.
    pub id: Uuid,

    /// Benchmark name.
    pub name: String,

    /// Benchmark description.
    pub description: String,

    /// Benchmark version.
    pub version: String,

    /// Task categories included in the benchmark.
    pub task_categories: Vec<String>,

    /// Environment complexity levels.
    pub environment_complexities: Vec<f64>,

    /// Expected κ scores for each capability axis at different autonomy levels.
    pub expected_kappa_scores: HashMap<AutonomyLevel, HashMap<CapabilityAxis, f64>>,

    /// Minimum tasks required for valid measurement.
    pub min_tasks: u32,

    /// Maximum measurement duration.
    pub max_duration: Duration,

    /// Validation criteria.
    pub validation_criteria: HashMap<String, serde_json::Value>,

    /// Reference implementations.
    pub reference_implementations: Vec<String>,

    /// Creation timestamp.
    pub created_at: SystemTime,

    /// Last updated timestamp.
    pub updated_at: SystemTime,
}

impl AutonomyBenchmark {
    /// Create a new autonomy benchmark.
    pub fn new(
        name: String,
        description: String,
        version: String,
        task_categories: Vec<String>,
        environment_complexities: Vec<f64>,
        expected_kappa_scores: HashMap<AutonomyLevel, HashMap<CapabilityAxis, f64>>,
        min_tasks: u32,
        max_duration: Duration,
    ) -> Self {
        let now = SystemTime::now();

        Self {
            id: Uuid::new_v4(),
            name,
            description,
            version,
            task_categories,
            environment_complexities,
            expected_kappa_scores,
            min_tasks,
            max_duration,
            validation_criteria: HashMap::new(),
            reference_implementations: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Validate an autonomy measurement against this benchmark.
    pub fn validate_measurement(
        &self,
        measurement: &AutonomyMeasurement,
    ) -> BenchmarkValidationResult {
        let mut axis_validations = HashMap::new();

        // Validate each capability axis
        for axis in CapabilityAxis::all() {
            if let Some(expected_scores) =
                self.expected_kappa_scores.get(&measurement.autonomy_level)
                && let Some(expected_score) = expected_scores.get(&axis)
                && let Some(actual_score) = measurement.get_kappa_score(axis)
            {
                let discrepancy = (actual_score.score - expected_score).abs();
                let is_within_tolerance = discrepancy <= 0.1; // 10% tolerance

                axis_validations.insert(
                    axis,
                    AxisValidation {
                        expected_score: *expected_score,
                        actual_score: actual_score.score,
                        discrepancy,
                        is_within_tolerance,
                        confidence: actual_score.confidence,
                    },
                );
            }
        }

        // Check if minimum tasks requirement is met
        let tasks_requirement_met = measurement.tasks_observed >= self.min_tasks;

        // Calculate overall validation score
        let validation_score = if axis_validations.is_empty() {
            0.0
        } else {
            let valid_count = axis_validations
                .values()
                .filter(|v| v.is_within_tolerance)
                .count();
            valid_count as f64 / axis_validations.len() as f64
        };

        BenchmarkValidationResult {
            benchmark_id: self.id,
            measurement_id: measurement.id,
            validation_score,
            axis_validations,
            tasks_requirement_met,
            environment_complexity_match: self
                .environment_complexities
                .contains(&measurement.environment_complexity),
            is_valid: validation_score >= 0.7 && tasks_requirement_met, // 70% threshold
        }
    }
}

/// Validation result for a benchmark comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkValidationResult {
    /// Benchmark identifier.
    pub benchmark_id: Uuid,

    /// Measurement identifier.
    pub measurement_id: Uuid,

    /// Overall validation score (0.0 to 1.0).
    pub validation_score: f64,

    /// Validation results for each capability axis.
    pub axis_validations: HashMap<CapabilityAxis, AxisValidation>,

    /// Whether minimum tasks requirement was met.
    pub tasks_requirement_met: bool,

    /// Whether environment complexity matches benchmark.
    pub environment_complexity_match: bool,

    /// Whether the measurement is valid according to benchmark.
    pub is_valid: bool,
}

/// Validation result for a single capability axis.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AxisValidation {
    /// Expected κ score.
    pub expected_score: f64,

    /// Actual κ score.
    pub actual_score: f64,

    /// Discrepancy between expected and actual.
    pub discrepancy: f64,

    /// Whether the score is within tolerance.
    pub is_within_tolerance: bool,

    /// Confidence in the actual score.
    pub confidence: f64,
}

/// Progress tracking for autonomy development.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomyProgress {
    /// Agent identifier.
    pub agent_id: String,

    /// Starting autonomy level.
    pub starting_level: AutonomyLevel,

    /// Current autonomy level.
    pub current_level: AutonomyLevel,

    /// Target autonomy level.
    pub target_level: AutonomyLevel,

    /// Progress measurements over time.
    pub measurements: Vec<AutonomyMeasurement>,

    /// Self-assessments over time.
    pub self_assessments: Vec<SelfAssessment>,

    /// Collaboration patterns discovered.
    pub collaboration_patterns: Vec<CollaborationPattern>,

    /// Benchmark comparisons.
    pub benchmark_comparisons: Vec<BenchmarkValidationResult>,

    /// Progress rate (κ score improvement per day).
    pub progress_rate: f64,

    /// Estimated time to reach target level.
    pub estimated_time_to_target: Option<Duration>,

    /// Areas needing improvement.
    pub improvement_areas: Vec<CapabilityAxis>,

    /// Strengths to leverage.
    pub strengths: Vec<CapabilityAxis>,

    /// Creation timestamp.
    pub created_at: SystemTime,

    /// Last updated timestamp.
    pub updated_at: SystemTime,
}

impl AutonomyProgress {
    /// Create new autonomy progress tracking.
    pub fn new(
        agent_id: String,
        starting_level: AutonomyLevel,
        target_level: AutonomyLevel,
    ) -> Self {
        let now = SystemTime::now();

        Self {
            agent_id,
            starting_level,
            current_level: starting_level,
            target_level,
            measurements: Vec::new(),
            self_assessments: Vec::new(),
            collaboration_patterns: Vec::new(),
            benchmark_comparisons: Vec::new(),
            progress_rate: 0.0,
            estimated_time_to_target: None,
            improvement_areas: Vec::new(),
            strengths: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a new autonomy measurement.
    pub fn add_measurement(&mut self, measurement: AutonomyMeasurement) {
        self.measurements.push(measurement);
        self.update_progress_metrics();
        self.updated_at = SystemTime::now();
    }

    /// Add a new self-assessment.
    pub fn add_self_assessment(&mut self, assessment: SelfAssessment) {
        self.self_assessments.push(assessment);
        self.updated_at = SystemTime::now();
    }

    /// Update progress metrics based on current measurements.
    fn update_progress_metrics(&mut self) {
        if self.measurements.len() < 2 {
            return;
        }

        // Sort measurements by timestamp
        let mut sorted_measurements = self.measurements.clone();
        sorted_measurements.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Calculate progress rate (κ score improvement per day)
        let first = sorted_measurements.first().unwrap();
        let last = sorted_measurements.last().unwrap();

        if let Ok(duration) = last.timestamp.duration_since(first.timestamp) {
            let days = duration.as_secs_f64() / 86400.0; // seconds in a day
            if days > 0.0 {
                let kappa_improvement = last.composite_kappa - first.composite_kappa;
                self.progress_rate = kappa_improvement / days;

                // Update current level based on latest measurement
                self.current_level = last.autonomy_level;

                // Estimate time to target
                if self.progress_rate > 0.0 {
                    let kappa_to_target = 1.0 - last.composite_kappa; // Assuming target is κ=1.0
                    let days_to_target = kappa_to_target / self.progress_rate;
                    self.estimated_time_to_target =
                        Some(Duration::from_secs((days_to_target * 86400.0) as u64));
                }

                // Identify improvement areas and strengths
                self.identify_areas_for_improvement(last);
                self.identify_strengths(last);
            }
        }
    }

    /// Identify areas needing improvement based on latest measurement.
    fn identify_areas_for_improvement(&mut self, measurement: &AutonomyMeasurement) {
        self.improvement_areas.clear();

        for axis in CapabilityAxis::all() {
            if let Some(score) = measurement.get_kappa_score(axis)
                && score.score < 0.5
            {
                self.improvement_areas.push(axis);
            }
        }
    }

    /// Identify strengths based on latest measurement.
    fn identify_strengths(&mut self, measurement: &AutonomyMeasurement) {
        self.strengths.clear();

        for axis in CapabilityAxis::all() {
            if let Some(score) = measurement.get_kappa_score(axis)
                && score.score >= 0.8
            {
                self.strengths.push(axis);
            }
        }
    }

    /// Get progress percentage toward target level.
    pub fn progress_percentage(&self) -> f64 {
        let start_value = self.starting_level.value() as f64;
        let current_value = self.current_level.value() as f64;
        let target_value = self.target_level.value() as f64;

        if target_value <= start_value {
            100.0
        } else {
            ((current_value - start_value) / (target_value - start_value) * 100.0).clamp(0.0, 100.0)
        }
    }

    /// Check if target level has been reached.
    pub fn target_reached(&self) -> bool {
        self.current_level.value() >= self.target_level.value()
    }
}
