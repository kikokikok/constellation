//! Benchmark management for autonomy measurement comparison.

use crate::models::autonomy::{
    AutonomyBenchmark, AutonomyLevel, AutonomyMeasurement, BenchmarkValidationResult,
    CapabilityAxis,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Configuration for benchmark management.
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Minimum validation score for benchmark acceptance.
    pub min_validation_score: f64,

    /// Maximum age of benchmark results to consider.
    pub max_result_age: Duration,

    /// Whether to require periodic benchmark revalidation.
    pub require_revalidation: bool,

    /// Revalidation interval.
    pub revalidation_interval: Duration,

    /// Minimum measurements for benchmark stability.
    pub min_measurements_for_stability: u32,

    /// Whether to enable automatic benchmark creation.
    pub enable_auto_benchmark_creation: bool,

    /// Threshold for auto-benchmark creation.
    pub auto_benchmark_threshold: f64,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            min_validation_score: 0.7,
            max_result_age: Duration::from_secs(2592000), // 30 days
            require_revalidation: true,
            revalidation_interval: Duration::from_secs(604800), // 7 days
            min_measurements_for_stability: 10,
            enable_auto_benchmark_creation: true,
            auto_benchmark_threshold: 0.8,
        }
    }
}

/// Benchmark result storage.
#[derive(Debug, Clone)]
struct BenchmarkResult {
    benchmark_id: Uuid,
    measurement_id: Uuid,
    validation_result: BenchmarkValidationResult,
    timestamp: SystemTime,
    metadata: HashMap<String, serde_json::Value>,
}

impl BenchmarkResult {
    fn new(
        benchmark_id: Uuid,
        measurement_id: Uuid,
        validation_result: BenchmarkValidationResult,
    ) -> Self {
        Self {
            benchmark_id,
            measurement_id,
            validation_result,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }

    fn age(&self) -> Duration {
        self.timestamp.elapsed().unwrap_or(Duration::from_secs(0))
    }

    fn is_valid(&self, max_age: Duration) -> bool {
        self.age() <= max_age
    }
}

/// Benchmark manager for autonomy measurement comparison.
#[derive(Debug)]
pub struct BenchmarkManager {
    config: BenchmarkConfig,
    benchmarks: Arc<RwLock<HashMap<Uuid, AutonomyBenchmark>>>,
    results: Arc<RwLock<HashMap<String, VecDeque<BenchmarkResult>>>>,
    benchmark_categories: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
}

impl BenchmarkManager {
    /// Create a new benchmark manager.
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config,
            benchmarks: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            benchmark_categories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a benchmark to the manager.
    pub fn add_benchmark(&self, benchmark: AutonomyBenchmark) {
        let mut benchmarks = self.benchmarks.write().unwrap();
        benchmarks.insert(benchmark.id, benchmark.clone());

        // Update categories
        let mut categories = self.benchmark_categories.write().unwrap();
        for category in &benchmark.task_categories {
            let category_benchmarks = categories.entry(category.clone()).or_default();

            if !category_benchmarks.contains(&benchmark.id) {
                category_benchmarks.push(benchmark.id);
            }
        }
    }

    /// Create a standard benchmark for an autonomy level.
    pub fn create_standard_benchmark(
        &self,
        autonomy_level: AutonomyLevel,
        name_suffix: &str,
    ) -> AutonomyBenchmark {
        let name = format!("Standard {} Benchmark", autonomy_level.description());
        let description = format!(
            "Standard benchmark for {} autonomy level. {}",
            autonomy_level.description(),
            name_suffix
        );

        let task_categories = match autonomy_level {
            AutonomyLevel::Level0Scripted => vec!["scripted".to_string(), "reactive".to_string()],
            AutonomyLevel::Level1GoalOriented => {
                vec!["goal-oriented".to_string(), "simple-planning".to_string()]
            }
            AutonomyLevel::Level2Adaptive => vec!["adaptive".to_string(), "learning".to_string()],
            AutonomyLevel::Level3Strategic => {
                vec!["strategic".to_string(), "resource-management".to_string()]
            }
            AutonomyLevel::Level4SelfImproving => {
                vec!["self-improvement".to_string(), "architecture".to_string()]
            }
            AutonomyLevel::Level5Collaborative => {
                vec!["collaboration".to_string(), "multi-agent".to_string()]
            }
            AutonomyLevel::Level6Creative => {
                vec!["creative".to_string(), "novel-solutions".to_string()]
            }
            AutonomyLevel::Level7MetaCognitive => {
                vec!["meta-cognitive".to_string(), "self-reflection".to_string()]
            }
            AutonomyLevel::Level8SelfSustaining => vec![
                "self-sustaining".to_string(),
                "autonomous-maintenance".to_string(),
            ],
            AutonomyLevel::Level9Transcendent => {
                vec!["transcendent".to_string(), "knowledge-creation".to_string()]
            }
        };

        let environment_complexities = match autonomy_level {
            AutonomyLevel::Level0Scripted => vec![0.1, 0.2],
            AutonomyLevel::Level1GoalOriented => vec![0.2, 0.3],
            AutonomyLevel::Level2Adaptive => vec![0.3, 0.4],
            AutonomyLevel::Level3Strategic => vec![0.4, 0.5],
            AutonomyLevel::Level4SelfImproving => vec![0.5, 0.6],
            AutonomyLevel::Level5Collaborative => vec![0.6, 0.7],
            AutonomyLevel::Level6Creative => vec![0.7, 0.8],
            AutonomyLevel::Level7MetaCognitive => vec![0.8, 0.9],
            AutonomyLevel::Level8SelfSustaining => vec![0.9, 1.0],
            AutonomyLevel::Level9Transcendent => vec![1.0],
        };

        let mut expected_kappa_scores = HashMap::new();

        // Set expected scores for this autonomy level
        let mut level_scores = HashMap::new();
        let base_score = autonomy_level.value() as f64 / 10.0;

        for axis in CapabilityAxis::all() {
            // Adjust base score based on axis importance at this level
            let axis_score = match (autonomy_level, axis) {
                // Early levels focus on execution and planning
                (AutonomyLevel::Level0Scripted, CapabilityAxis::Execution) => base_score * 1.2,
                (AutonomyLevel::Level1GoalOriented, CapabilityAxis::Planning) => base_score * 1.2,
                (AutonomyLevel::Level1GoalOriented, CapabilityAxis::Execution) => base_score * 1.1,

                // Middle levels focus on learning and adaptation
                (AutonomyLevel::Level2Adaptive, CapabilityAxis::Learning) => base_score * 1.2,
                (AutonomyLevel::Level2Adaptive, CapabilityAxis::Adaptation) => base_score * 1.2,
                (AutonomyLevel::Level3Strategic, CapabilityAxis::ResourceManagement) => {
                    base_score * 1.2
                }

                // Advanced levels focus on creativity and collaboration
                (AutonomyLevel::Level5Collaborative, CapabilityAxis::Collaboration) => {
                    base_score * 1.3
                }
                (AutonomyLevel::Level6Creative, CapabilityAxis::Creativity) => base_score * 1.3,

                // Highest levels focus on meta-cognition and self-assessment
                (AutonomyLevel::Level7MetaCognitive, CapabilityAxis::MetaCognition) => {
                    base_score * 1.4
                }
                (AutonomyLevel::Level7MetaCognitive, CapabilityAxis::SelfAssessment) => {
                    base_score * 1.4
                }
                (AutonomyLevel::Level8SelfSustaining, CapabilityAxis::SelfAssessment) => {
                    base_score * 1.5
                }
                (AutonomyLevel::Level9Transcendent, CapabilityAxis::Creativity) => base_score * 1.5,
                (AutonomyLevel::Level9Transcendent, CapabilityAxis::MetaCognition) => {
                    base_score * 1.5
                }

                // Default
                _ => base_score,
            };

            level_scores.insert(axis, axis_score.min(1.0));
        }

        expected_kappa_scores.insert(autonomy_level, level_scores);

        // Also include scores for lower levels
        for lower_level in (0..autonomy_level.value()).rev() {
            let lower_level_enum = match lower_level {
                0 => AutonomyLevel::Level0Scripted,
                1 => AutonomyLevel::Level1GoalOriented,
                2 => AutonomyLevel::Level2Adaptive,
                3 => AutonomyLevel::Level3Strategic,
                4 => AutonomyLevel::Level4SelfImproving,
                5 => AutonomyLevel::Level5Collaborative,
                6 => AutonomyLevel::Level6Creative,
                7 => AutonomyLevel::Level7MetaCognitive,
                8 => AutonomyLevel::Level8SelfSustaining,
                _ => continue,
            };

            let mut lower_scores = HashMap::new();
            let lower_base = lower_level as f64 / 10.0;

            for axis in CapabilityAxis::all() {
                lower_scores.insert(axis, lower_base.min(1.0));
            }

            expected_kappa_scores.insert(lower_level_enum, lower_scores);
        }

        let min_tasks = match autonomy_level {
            AutonomyLevel::Level0Scripted => 5,
            AutonomyLevel::Level1GoalOriented => 10,
            AutonomyLevel::Level2Adaptive => 15,
            AutonomyLevel::Level3Strategic => 20,
            AutonomyLevel::Level4SelfImproving => 25,
            AutonomyLevel::Level5Collaborative => 30,
            AutonomyLevel::Level6Creative => 35,
            AutonomyLevel::Level7MetaCognitive => 40,
            AutonomyLevel::Level8SelfSustaining => 45,
            AutonomyLevel::Level9Transcendent => 50,
        };

        let max_duration = Duration::from_secs(3600); // 1 hour

        let benchmark = AutonomyBenchmark::new(
            name,
            description,
            "1.0".to_string(),
            task_categories,
            environment_complexities,
            expected_kappa_scores,
            min_tasks,
            max_duration,
        );

        // Add to manager
        self.add_benchmark(benchmark.clone());

        benchmark
    }

    /// Initialize with standard benchmarks.
    pub fn initialize_standard_benchmarks(&self) {
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
            self.create_standard_benchmark(level, "Standard measurement protocol");
        }
    }

    /// Validate a measurement against all applicable benchmarks.
    pub fn validate_measurement(
        &self,
        measurement: &AutonomyMeasurement,
    ) -> Vec<BenchmarkValidationResult> {
        let benchmarks = self.benchmarks.read().unwrap();
        let mut results = Vec::new();

        for benchmark in benchmarks.values() {
            // Check if benchmark is applicable to this measurement
            if self.is_benchmark_applicable(benchmark, measurement) {
                let validation_result = benchmark.validate_measurement(measurement);

                if validation_result.validation_score >= self.config.min_validation_score {
                    let result_clone = validation_result.clone();
                    results.push(result_clone.clone());

                    // Store result
                    self.store_validation_result(
                        &measurement.agent_id,
                        benchmark.id,
                        measurement.id,
                        result_clone,
                    );
                }
            }
        }

        // Check if we should create an auto-benchmark
        if self.config.enable_auto_benchmark_creation && results.is_empty() {
            self.consider_auto_benchmark_creation(measurement);
        }

        results
    }

    /// Check if a benchmark is applicable to a measurement.
    fn is_benchmark_applicable(
        &self,
        benchmark: &AutonomyBenchmark,
        measurement: &AutonomyMeasurement,
    ) -> bool {
        // Check environment complexity
        if !benchmark
            .environment_complexities
            .contains(&measurement.environment_complexity)
        {
            return false;
        }

        // Check if benchmark has expectations for this autonomy level
        if !benchmark
            .expected_kappa_scores
            .contains_key(&measurement.autonomy_level)
        {
            return false;
        }

        // Check task count requirement
        if measurement.tasks_observed < benchmark.min_tasks {
            return false;
        }

        true
    }

    /// Store validation result.
    fn store_validation_result(
        &self,
        agent_id: &str,
        benchmark_id: Uuid,
        measurement_id: Uuid,
        validation_result: BenchmarkValidationResult,
    ) {
        let benchmark_result =
            BenchmarkResult::new(benchmark_id, measurement_id, validation_result);

        let mut results = self.results.write().unwrap();
        let agent_results = results.entry(agent_id.to_string()).or_default();

        agent_results.push_back(benchmark_result);

        // Clean up old results
        self.cleanup_results(agent_id);
    }

    /// Clean up old results for an agent.
    fn cleanup_results(&self, agent_id: &str) {
        let mut results = self.results.write().unwrap();

        if let Some(agent_results) = results.get_mut(agent_id) {
            // Remove invalid results
            agent_results.retain(|result| result.is_valid(self.config.max_result_age));

            // Keep only recent results
            let max_results = 100;
            if agent_results.len() > max_results {
                let to_remove = agent_results.len() - max_results;
                for _ in 0..to_remove {
                    agent_results.pop_front();
                }
            }
        }
    }

    /// Consider automatic benchmark creation based on measurement.
    fn consider_auto_benchmark_creation(&self, measurement: &AutonomyMeasurement) {
        // Check if measurement meets auto-benchmark criteria
        if measurement.composite_kappa >= self.config.auto_benchmark_threshold
            && measurement.tasks_observed >= self.config.min_measurements_for_stability
        {
            // Check if we already have a similar benchmark
            let benchmarks = self.benchmarks.read().unwrap();
            let similar_benchmark_exists = benchmarks.values().any(|benchmark| {
                benchmark
                    .expected_kappa_scores
                    .contains_key(&measurement.autonomy_level)
                    && (benchmark
                        .environment_complexities
                        .contains(&measurement.environment_complexity)
                        || benchmark
                            .environment_complexities
                            .iter()
                            .any(|c| (*c - measurement.environment_complexity).abs() < 0.1))
            });

            if !similar_benchmark_exists {
                // Create auto-benchmark
                self.create_auto_benchmark(measurement);
            }
        }
    }

    /// Create an automatic benchmark based on measurement.
    fn create_auto_benchmark(&self, measurement: &AutonomyMeasurement) {
        let name = format!(
            "Auto-Benchmark: {} at Level {}",
            measurement.agent_id,
            measurement.autonomy_level.value()
        );

        let description = format!(
            "Automatically created benchmark based on {}'s performance at autonomy level {}. Composite κ: {:.2}",
            measurement.agent_id,
            measurement.autonomy_level.description(),
            measurement.composite_kappa
        );

        let task_categories = vec!["auto-generated".to_string(), "empirical".to_string()];

        // Environment complexities around the measurement value
        let env_complexity = measurement.environment_complexity;
        let environment_complexities = if env_complexity > 0.1 {
            vec![
                (env_complexity - 0.1).max(0.0),
                env_complexity,
                (env_complexity + 0.1).min(1.0),
            ]
        } else {
            vec![env_complexity, (env_complexity + 0.1).min(1.0)]
        };

        // Create expected κ scores based on measurement
        let mut expected_kappa_scores = HashMap::new();
        let mut level_scores = HashMap::new();

        for axis in CapabilityAxis::all() {
            if let Some(score) = measurement.get_kappa_score(axis) {
                level_scores.insert(axis, score.score);
            } else {
                level_scores.insert(axis, measurement.composite_kappa);
            }
        }

        expected_kappa_scores.insert(measurement.autonomy_level, level_scores);

        // Include lower levels with proportionally lower scores
        for lower_level in (0..measurement.autonomy_level.value()).rev() {
            let lower_level_enum = match lower_level {
                0 => AutonomyLevel::Level0Scripted,
                1 => AutonomyLevel::Level1GoalOriented,
                2 => AutonomyLevel::Level2Adaptive,
                3 => AutonomyLevel::Level3Strategic,
                4 => AutonomyLevel::Level4SelfImproving,
                5 => AutonomyLevel::Level5Collaborative,
                6 => AutonomyLevel::Level6Creative,
                7 => AutonomyLevel::Level7MetaCognitive,
                8 => AutonomyLevel::Level8SelfSustaining,
                _ => continue,
            };

            let mut lower_scores = HashMap::new();
            let proportion = lower_level as f64 / measurement.autonomy_level.value() as f64;

            for axis in CapabilityAxis::all() {
                if let Some(score) = measurement.get_kappa_score(axis) {
                    lower_scores.insert(axis, score.score * proportion);
                } else {
                    lower_scores.insert(axis, measurement.composite_kappa * proportion);
                }
            }

            expected_kappa_scores.insert(lower_level_enum, lower_scores);
        }

        let min_tasks = measurement.tasks_observed;
        let max_duration = Duration::from_secs(7200); // 2 hours

        let benchmark = AutonomyBenchmark::new(
            name,
            description,
            "1.0-auto".to_string(),
            task_categories,
            environment_complexities,
            expected_kappa_scores,
            min_tasks,
            max_duration,
        );

        // Add to manager
        self.add_benchmark(benchmark);
    }

    /// Get benchmark validation history for an agent.
    pub fn get_validation_history(&self, agent_id: &str) -> Vec<BenchmarkValidationResult> {
        let results = self.results.read().unwrap();

        results
            .get(agent_id)
            .map(|agent_results| {
                agent_results
                    .iter()
                    .map(|result| result.validation_result.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get benchmark validation score for an agent.
    pub fn get_validation_score(&self, agent_id: &str) -> Option<f64> {
        let history = self.get_validation_history(agent_id);

        if history.is_empty() {
            return None;
        }

        let total_score: f64 = history.iter().map(|result| result.validation_score).sum();
        Some(total_score / history.len() as f64)
    }

    /// Get benchmarks by category.
    pub fn get_benchmarks_by_category(&self, category: &str) -> Vec<AutonomyBenchmark> {
        let benchmarks = self.benchmarks.read().unwrap();
        let categories = self.benchmark_categories.read().unwrap();

        categories
            .get(category)
            .map(|benchmark_ids| {
                benchmark_ids
                    .iter()
                    .filter_map(|id| benchmarks.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get benchmarks for an autonomy level.
    pub fn get_benchmarks_for_level(
        &self,
        autonomy_level: AutonomyLevel,
    ) -> Vec<AutonomyBenchmark> {
        let benchmarks = self.benchmarks.read().unwrap();

        benchmarks
            .values()
            .filter(|benchmark| {
                benchmark
                    .expected_kappa_scores
                    .contains_key(&autonomy_level)
            })
            .cloned()
            .collect()
    }

    /// Get the most applicable benchmark for a measurement.
    pub fn get_most_applicable_benchmark(
        &self,
        measurement: &AutonomyMeasurement,
    ) -> Option<AutonomyBenchmark> {
        let applicable_benchmarks = self
            .get_benchmarks_for_level(measurement.autonomy_level)
            .into_iter()
            .filter(|benchmark| self.is_benchmark_applicable(benchmark, measurement))
            .collect::<Vec<_>>();

        // Find benchmark with closest environment complexity
        applicable_benchmarks.into_iter().min_by(|a, b| {
            let a_diff = a
                .environment_complexities
                .iter()
                .map(|c| (c - measurement.environment_complexity).abs())
                .fold(f64::INFINITY, f64::min);

            let b_diff = b
                .environment_complexities
                .iter()
                .map(|c| (c - measurement.environment_complexity).abs())
                .fold(f64::INFINITY, f64::min);

            a_diff.partial_cmp(&b_diff).unwrap()
        })
    }

    /// Check if agent needs benchmark revalidation.
    pub fn needs_revalidation(&self, agent_id: &str) -> bool {
        if !self.config.require_revalidation {
            return false;
        }

        let results = self.results.read().unwrap();

        if let Some(agent_results) = results.get(agent_id)
            && let Some(latest_result) = agent_results.back()
        {
            return latest_result.age() >= self.config.revalidation_interval;
        }

        // No results yet, needs validation
        true
    }

    /// Generate benchmark report for an agent.
    pub fn generate_benchmark_report(&self, agent_id: &str) -> Option<BenchmarkReport> {
        let validation_history = self.get_validation_history(agent_id);

        if validation_history.is_empty() {
            return None;
        }

        let validation_score = self.get_validation_score(agent_id).unwrap_or(0.0);
        let needs_reval = self.needs_revalidation(agent_id);

        // Calculate benchmark coverage
        let benchmarks = self.benchmarks.read().unwrap();
        let total_benchmarks = benchmarks.len();
        let validated_benchmarks: HashSet<_> = validation_history
            .iter()
            .map(|result| result.benchmark_id)
            .collect();

        let coverage = if total_benchmarks > 0 {
            validated_benchmarks.len() as f64 / total_benchmarks as f64
        } else {
            0.0
        };

        // Calculate performance by autonomy level
        let mut performance_by_level = HashMap::new();
        for result in &validation_history {
            // Find benchmark to get autonomy level
            if let Some(benchmark) = benchmarks.get(&result.benchmark_id) {
                for level in benchmark.expected_kappa_scores.keys() {
                    let entry = performance_by_level.entry(*level).or_insert(Vec::new());
                    entry.push(result.validation_score);
                }
            }
        }

        let avg_by_level: HashMap<_, _> = performance_by_level
            .into_iter()
            .map(|(level, scores)| {
                let avg = scores.iter().sum::<f64>() / scores.len() as f64;
                (level, avg)
            })
            .collect();

        // Identify strongest and weakest areas
        let mut axis_performance = HashMap::new();
        for result in &validation_history {
            for (axis, validation) in &result.axis_validations {
                let entry = axis_performance.entry(*axis).or_insert(Vec::new());
                entry.push(validation.is_within_tolerance as u8 as f64);
            }
        }

        let avg_by_axis: HashMap<_, _> = axis_performance
            .into_iter()
            .map(|(axis, scores)| {
                let avg = scores.iter().sum::<f64>() / scores.len() as f64;
                (axis, avg)
            })
            .collect();

        let strongest_axis = avg_by_axis
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(axis, _)| *axis);

        let weakest_axis = avg_by_axis
            .iter()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(axis, _)| *axis);

        Some(BenchmarkReport {
            agent_id: agent_id.to_string(),
            total_validations: validation_history.len(),
            validation_score,
            benchmark_coverage: coverage,
            needs_revalidation: needs_reval,
            performance_by_level: avg_by_level,
            strongest_axis,
            weakest_axis,
            recommendations: self.generate_benchmark_recommendations(
                validation_score,
                coverage,
                needs_reval,
                strongest_axis,
                weakest_axis,
            ),
        })
    }

    /// Generate benchmark recommendations.
    fn generate_benchmark_recommendations(
        &self,
        validation_score: f64,
        coverage: f64,
        needs_revalidation: bool,
        strongest_axis: Option<CapabilityAxis>,
        weakest_axis: Option<CapabilityAxis>,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if validation_score < self.config.min_validation_score {
            recommendations.push(
                "Improve benchmark validation scores through targeted capability development"
                    .to_string(),
            );
        }

        if coverage < 0.5 {
            recommendations
                .push("Increase benchmark coverage by testing against more benchmarks".to_string());
        }

        if needs_revalidation {
            recommendations.push(
                "Perform benchmark revalidation to ensure current performance levels".to_string(),
            );
        }

        if let Some(weak_axis) = weakest_axis {
            recommendations.push(format!(
                "Focus on improving {} capability",
                weak_axis.name()
            ));
        }

        if let Some(strong_axis) = strongest_axis {
            recommendations.push(format!(
                "Leverage {} strength in complex tasks",
                strong_axis.name()
            ));
        }

        if validation_score >= 0.9 && coverage >= 0.8 {
            recommendations
                .push("Consider creating new benchmarks based on current performance".to_string());
        }

        recommendations
    }

    /// Estimate benchmark validation κ score.
    pub fn estimate_benchmark_kappa(
        &self,
        agent_id: &str,
    ) -> Option<crate::models::autonomy::KappaScore> {
        let validation_score = self.get_validation_score(agent_id)?;
        let history = self.get_validation_history(agent_id);

        if history.is_empty() {
            return None;
        }

        // Calculate score based on validation performance and coverage
        let coverage = self.get_benchmark_coverage(agent_id);

        // Score is weighted combination of validation score and coverage
        let validation_weight = 0.7;
        let coverage_weight = 0.3;

        let score = validation_score * validation_weight + coverage * coverage_weight;
        let confidence = (history.len() as f64 / 10.0).min(1.0) * 0.8;

        Some(crate::models::autonomy::KappaScore::new(
            CapabilityAxis::Execution, // Benchmark validation relates to execution capability
            score,
            confidence,
            history.len() as u32,
        ))
    }

    /// Get benchmark coverage for an agent.
    fn get_benchmark_coverage(&self, agent_id: &str) -> f64 {
        let benchmarks = self.benchmarks.read().unwrap();
        let history = self.get_validation_history(agent_id);

        if benchmarks.is_empty() || history.is_empty() {
            return 0.0;
        }

        let validated_benchmarks: HashSet<_> =
            history.iter().map(|result| result.benchmark_id).collect();

        validated_benchmarks.len() as f64 / benchmarks.len() as f64
    }
}

/// Benchmark report for an agent.
#[derive(Debug, Clone)]
pub struct BenchmarkReport {
    pub agent_id: String,
    pub total_validations: usize,
    pub validation_score: f64,
    pub benchmark_coverage: f64,
    pub needs_revalidation: bool,
    pub performance_by_level: HashMap<AutonomyLevel, f64>,
    pub strongest_axis: Option<CapabilityAxis>,
    pub weakest_axis: Option<CapabilityAxis>,
    pub recommendations: Vec<String>,
}

impl Default for BenchmarkManager {
    fn default() -> Self {
        Self::new(BenchmarkConfig::default())
    }
}
