//! Open-world research environment for emergent collaboration and discovery.
//!
//! Provides a sandboxed environment where agents can:
//! - Generate and test hypotheses
//! - Design and execute experiments
//! - Make discoveries and have them validated
//! - Collaborate emergently
//! - Balance exploration vs exploitation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::autonomy::CapabilityAxis;

/// Represents a research hypothesis in the open-world environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchHypothesis {
    pub id: String,
    pub description: String,
    pub domain: String,
    pub complexity: f64,
    pub novelty: f64,
    pub testability: f64,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub status: HypothesisStatus,
    pub supporting_evidence: Vec<Evidence>,
    pub counter_evidence: Vec<Evidence>,
    pub confidence: f64,
}

/// Status of a research hypothesis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HypothesisStatus {
    Proposed,
    UnderInvestigation,
    PartiallySupported,
    StronglySupported,
    Refuted,
    Inconclusive,
    Archived,
}

/// Evidence supporting or refuting a hypothesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub source: EvidenceSource,
    pub content: String,
    pub strength: f64,
    pub reliability: f64,
    pub collected_at: DateTime<Utc>,
    pub collected_by: String,
}

/// Source of evidence in the research environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceSource {
    ExperimentalResult,
    Simulation,
    LiteratureReview,
    ExpertOpinion,
    DataAnalysis,
    PeerReview,
    Replication,
}

/// Represents a research experiment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchExperiment {
    pub id: String,
    pub hypothesis_id: String,
    pub design: ExperimentDesign,
    pub methodology: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub expected_outcomes: Vec<String>,
    pub conducted_by: Vec<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub results: Option<ExperimentResults>,
    pub status: ExperimentStatus,
}

/// Design of a research experiment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentDesign {
    pub design_type: ExperimentDesignType,
    pub sample_size: Option<usize>,
    pub control_group: bool,
    pub randomization: bool,
    pub blinding: Option<BlindingType>,
    pub duration_hours: f64,
    pub resources_required: Vec<String>,
}

/// Type of experiment design.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentDesignType {
    Observational,
    Experimental,
    QuasiExperimental,
    Simulation,
    MetaAnalysis,
    SystematicReview,
}

/// Type of blinding in experiments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlindingType {
    SingleBlind,
    DoubleBlind,
    TripleBlind,
}

/// Results of a research experiment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResults {
    pub raw_data: serde_json::Value,
    pub analysis_method: String,
    pub statistical_significance: Option<f64>,
    pub effect_size: Option<f64>,
    pub confidence_interval: Option<(f64, f64)>,
    pub interpretation: String,
    pub limitations: Vec<String>,
    pub implications: Vec<String>,
}

/// Status of an experiment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExperimentStatus {
    Designed,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Represents a research discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchDiscovery {
    pub id: String,
    pub title: String,
    pub description: String,
    pub domain: String,
    pub significance: DiscoverySignificance,
    pub novelty: f64,
    pub reproducibility: f64,
    pub supporting_hypotheses: Vec<String>,
    pub supporting_experiments: Vec<String>,
    pub evidence: Vec<Evidence>,
    pub discovered_by: Vec<String>,
    pub discovered_at: DateTime<Utc>,
    pub validation_status: ValidationStatus,
    pub peer_reviews: Vec<PeerReview>,
    pub impact_score: f64,
}

/// Significance level of a discovery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiscoverySignificance {
    Minor,
    Moderate,
    Major,
    Breakthrough,
    ParadigmShift,
}

/// Validation status of a discovery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationStatus {
    Unvalidated,
    UnderReview,
    PartiallyValidated,
    FullyValidated,
    Contested,
    Retracted,
}

/// Peer review of a discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerReview {
    pub reviewer_id: String,
    pub review_date: DateTime<Utc>,
    pub rating: f64,
    pub comments: String,
    pub suggested_improvements: Vec<String>,
    pub validation_attempts: Vec<ValidationAttempt>,
}

/// Attempt to validate a discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationAttempt {
    pub attempt_id: String,
    pub validator_id: String,
    pub method: ValidationMethod,
    pub success: bool,
    pub notes: String,
    pub timestamp: DateTime<Utc>,
}

/// Method used for validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationMethod {
    Replication,
    IndependentVerification,
    StatisticalAnalysis,
    ExpertConsensus,
    CrossValidation,
}

/// Configuration for the open-world research environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenWorldConfig {
    pub max_concurrent_experiments: usize,
    pub resource_limits: HashMap<String, f64>,
    pub collaboration_enabled: bool,
    pub peer_review_enabled: bool,
    pub discovery_validation_required: bool,
    pub exploration_exploitation_ratio: f64,
    pub minimum_evidence_strength: f64,
    pub minimum_reproducibility: f64,
}

/// Metrics for the open-world environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenWorldMetrics {
    pub total_hypotheses: usize,
    pub total_experiments: usize,
    pub total_discoveries: usize,
    pub hypothesis_success_rate: f64,
    pub experiment_success_rate: f64,
    pub discovery_validation_rate: f64,
    pub average_discovery_significance: f64,
    pub collaboration_efficiency: f64,
    pub exploration_exploitation_balance: f64,
    pub knowledge_growth_rate: f64,
}

/// Open-world research environment manager.
pub struct OpenWorldResearchEnvironment {
    config: OpenWorldConfig,
    hypotheses: Arc<RwLock<HashMap<String, ResearchHypothesis>>>,
    experiments: Arc<RwLock<HashMap<String, ResearchExperiment>>>,
    discoveries: Arc<RwLock<HashMap<String, ResearchDiscovery>>>,
    agent_capabilities: Arc<RwLock<HashMap<String, Vec<CapabilityAxis>>>>,
    metrics: Arc<RwLock<OpenWorldMetrics>>,
}

impl OpenWorldResearchEnvironment {
    /// Create a new open-world research environment.
    pub fn new(config: OpenWorldConfig) -> Self {
        Self {
            config,
            hypotheses: Arc::new(RwLock::new(HashMap::new())),
            experiments: Arc::new(RwLock::new(HashMap::new())),
            discoveries: Arc::new(RwLock::new(HashMap::new())),
            agent_capabilities: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(OpenWorldMetrics {
                total_hypotheses: 0,
                total_experiments: 0,
                total_discoveries: 0,
                hypothesis_success_rate: 0.0,
                experiment_success_rate: 0.0,
                discovery_validation_rate: 0.0,
                average_discovery_significance: 0.0,
                collaboration_efficiency: 0.0,
                exploration_exploitation_balance: 0.5,
                knowledge_growth_rate: 0.0,
            })),
        }
    }

    /// Propose a new research hypothesis.
    pub async fn propose_hypothesis(
        &self,
        hypothesis: ResearchHypothesis,
        _agent_id: &str,
    ) -> Result<String, String> {
        let mut hypotheses = self.hypotheses.write().await;

        if hypotheses.contains_key(&hypothesis.id) {
            return Err("Hypothesis ID already exists".to_string());
        }

        let hypothesis_id = hypothesis.id.clone();
        hypotheses.insert(hypothesis_id.clone(), hypothesis);

        let mut metrics = self.metrics.write().await;
        metrics.total_hypotheses += 1;

        Ok(hypothesis_id)
    }

    /// Design a new experiment for a hypothesis.
    pub async fn design_experiment(
        &self,
        experiment: ResearchExperiment,
        agent_ids: &[String],
    ) -> Result<String, String> {
        let mut experiments = self.experiments.write().await;

        if experiments.contains_key(&experiment.id) {
            return Err("Experiment ID already exists".to_string());
        }

        let experiment_id = experiment.id.clone();
        experiments.insert(experiment_id.clone(), experiment);

        let mut metrics = self.metrics.write().await;
        metrics.total_experiments += 1;

        Ok(experiment_id)
    }

    /// Execute an experiment and record results.
    pub async fn execute_experiment(
        &self,
        experiment_id: &str,
        results: ExperimentResults,
        agent_id: &str,
    ) -> Result<(), String> {
        let mut experiments = self.experiments.write().await;

        let experiment = experiments
            .get_mut(experiment_id)
            .ok_or_else(|| "Experiment not found".to_string())?;

        experiment.results = Some(results);
        experiment.end_time = Some(Utc::now());
        experiment.status = ExperimentStatus::Completed;

        Ok(())
    }

    /// Record a new discovery.
    pub async fn record_discovery(
        &self,
        discovery: ResearchDiscovery,
        agent_ids: &[String],
    ) -> Result<String, String> {
        let mut discoveries = self.discoveries.write().await;

        if discoveries.contains_key(&discovery.id) {
            return Err("Discovery ID already exists".to_string());
        }

        let discovery_id = discovery.id.clone();
        discoveries.insert(discovery_id.clone(), discovery);

        let mut metrics = self.metrics.write().await;
        metrics.total_discoveries += 1;

        Ok(discovery_id)
    }

    /// Validate a discovery through peer review.
    pub async fn validate_discovery(
        &self,
        discovery_id: &str,
        peer_review: PeerReview,
    ) -> Result<(), String> {
        let mut discoveries = self.discoveries.write().await;

        let discovery = discoveries
            .get_mut(discovery_id)
            .ok_or_else(|| "Discovery not found".to_string())?;

        discovery.peer_reviews.push(peer_review);

        let validation_count = discovery.peer_reviews.len();
        let positive_reviews = discovery
            .peer_reviews
            .iter()
            .filter(|r| r.rating >= 0.7)
            .count();

        if validation_count >= 3 && positive_reviews as f64 / validation_count as f64 >= 0.67 {
            discovery.validation_status = ValidationStatus::FullyValidated;
        } else if validation_count >= 1 {
            discovery.validation_status = ValidationStatus::PartiallyValidated;
        }

        let mut metrics = self.metrics.write().await;
        let validated_discoveries = discoveries
            .values()
            .filter(|d| d.validation_status == ValidationStatus::FullyValidated)
            .count();

        metrics.discovery_validation_rate = validated_discoveries as f64 / discoveries.len() as f64;

        Ok(())
    }

    /// Get hypotheses by domain.
    pub async fn get_hypotheses_by_domain(&self, domain: &str) -> Vec<ResearchHypothesis> {
        let hypotheses = self.hypotheses.read().await;
        hypotheses
            .values()
            .filter(|h| h.domain == domain)
            .cloned()
            .collect()
    }

    /// Get experiments by status.
    pub async fn get_experiments_by_status(
        &self,
        status: ExperimentStatus,
    ) -> Vec<ResearchExperiment> {
        let experiments = self.experiments.read().await;
        experiments
            .values()
            .filter(|e| e.status == status)
            .cloned()
            .collect()
    }

    /// Get discoveries by significance.
    pub async fn get_discoveries_by_significance(
        &self,
        significance: DiscoverySignificance,
    ) -> Vec<ResearchDiscovery> {
        let discoveries = self.discoveries.read().await;
        discoveries
            .values()
            .filter(|d| d.significance == significance)
            .cloned()
            .collect()
    }

    /// Update agent capabilities based on research participation.
    pub async fn update_agent_capabilities(
        &self,
        agent_id: &str,
        capabilities: Vec<CapabilityAxis>,
    ) {
        let mut agent_capabilities = self.agent_capabilities.write().await;
        agent_capabilities.insert(agent_id.to_string(), capabilities);
    }

    /// Calculate exploration vs exploitation balance.
    pub async fn calculate_exploration_balance(&self) -> f64 {
        let hypotheses = self.hypotheses.read().await;
        let experiments = self.experiments.read().await;

        let exploration_hypotheses = hypotheses.values().filter(|h| h.novelty > 0.7).count();

        let exploitation_hypotheses = hypotheses.values().filter(|h| h.novelty <= 0.3).count();

        let total = exploration_hypotheses + exploitation_hypotheses;
        if total > 0 {
            exploration_hypotheses as f64 / total as f64
        } else {
            0.5
        }
    }

    /// Calculate collaboration efficiency.
    pub async fn calculate_collaboration_efficiency(&self) -> f64 {
        let experiments = self.experiments.read().await;
        let discoveries = self.discoveries.read().await;

        let collaborative_experiments = experiments
            .values()
            .filter(|e| e.conducted_by.len() > 1)
            .count();

        let collaborative_discoveries = discoveries
            .values()
            .filter(|d| d.discovered_by.len() > 1)
            .count();

        let total_experiments = experiments.len();
        let total_discoveries = discoveries.len();

        let exp_efficiency = if total_experiments > 0 {
            collaborative_experiments as f64 / total_experiments as f64
        } else {
            0.0
        };

        let disc_efficiency = if total_discoveries > 0 {
            collaborative_discoveries as f64 / total_discoveries as f64
        } else {
            0.0
        };

        (exp_efficiency + disc_efficiency) / 2.0
    }

    /// Get environment metrics.
    pub async fn get_metrics(&self) -> OpenWorldMetrics {
        let mut metrics = self.metrics.write().await;

        metrics.exploration_exploitation_balance = self.calculate_exploration_balance().await;
        metrics.collaboration_efficiency = self.calculate_collaboration_efficiency().await;

        metrics.clone()
    }

    /// Generate research recommendations for an agent.
    pub async fn generate_recommendations(
        &self,
        agent_id: &str,
        agent_capabilities: &[CapabilityAxis],
    ) -> Vec<ResearchRecommendation> {
        let hypotheses = self.hypotheses.read().await;
        let experiments = self.experiments.read().await;
        let discoveries = self.discoveries.read().await;

        let mut recommendations = Vec::new();

        for hypothesis in hypotheses.values() {
            if hypothesis.status == HypothesisStatus::Proposed {
                let match_score =
                    self.calculate_hypothesis_match_score(hypothesis, agent_capabilities);
                if match_score > 0.6 {
                    recommendations.push(ResearchRecommendation {
                        recommendation_type: ResearchRecommendationType::InvestigateHypothesis,
                        target_id: hypothesis.id.clone(),
                        confidence: match_score,
                        rationale: format!(
                            "Matches agent capabilities in domain: {}",
                            hypothesis.domain
                        ),
                    });
                }
            }
        }

        for experiment in experiments.values() {
            if experiment.status == ExperimentStatus::Designed {
                recommendations.push(ResearchRecommendation {
                    recommendation_type: ResearchRecommendationType::ExecuteExperiment,
                    target_id: experiment.id.clone(),
                    confidence: 0.7,
                    rationale: "Experiment is designed and ready for execution".to_string(),
                });
            }
        }

        for discovery in discoveries.values() {
            if discovery.validation_status == ValidationStatus::Unvalidated {
                recommendations.push(ResearchRecommendation {
                    recommendation_type: ResearchRecommendationType::ValidateDiscovery,
                    target_id: discovery.id.clone(),
                    confidence: 0.8,
                    rationale: "Discovery requires validation".to_string(),
                });
            }
        }

        recommendations.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        recommendations.truncate(10);
        recommendations
    }

    /// Calculate how well an agent's capabilities match a hypothesis.
    fn calculate_hypothesis_match_score(
        &self,
        hypothesis: &ResearchHypothesis,
        agent_capabilities: &[CapabilityAxis],
    ) -> f64 {
        let domain_expertise = agent_capabilities
            .iter()
            .filter(|c| {
                c.name()
                    .to_lowercase()
                    .contains(&hypothesis.domain.to_lowercase())
            })
            .count() as f64;

        let research_capabilities = agent_capabilities
            .iter()
            .filter(|c| {
                let name = c.name();
                name == "Planning"
                    || name == "Reasoning"
                    || name == "Creativity"
                    || name == "MetaCognition"
            })
            .count() as f64;

        (domain_expertise * 0.6 + research_capabilities * 0.4) / 4.0
    }

    /// Integrate discovery into autonomy measurement.
    pub async fn integrate_discovery_into_autonomy(
        &self,
        discovery_id: &str,
        autonomy_engine: &crate::autonomy::measurement_engine::AutonomyMeasurementEngine,
    ) -> Result<(), String> {
        let discoveries = self.discoveries.read().await;
        let discovery = discoveries
            .get(discovery_id)
            .ok_or_else(|| "Discovery not found".to_string())?;

        if discovery.validation_status != ValidationStatus::FullyValidated {
            return Err("Discovery must be fully validated before integration".to_string());
        }

        for agent_id in &discovery.discovered_by {
            autonomy_engine.record_observation(
                agent_id,
                CapabilityAxis::Creativity,
                discovery.novelty * discovery.impact_score,
                1.0,
                None,
                1.0,
            );

            autonomy_engine.record_observation(
                agent_id,
                CapabilityAxis::Collaboration,
                if discovery.discovered_by.len() > 1 {
                    1.0
                } else {
                    0.5
                },
                1.0,
                None,
                1.0,
            );
        }

        Ok(())
    }
}

/// Research recommendation for agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRecommendation {
    pub recommendation_type: ResearchRecommendationType,
    pub target_id: String,
    pub confidence: f64,
    pub rationale: String,
}

/// Type of research recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResearchRecommendationType {
    InvestigateHypothesis,
    DesignExperiment,
    ExecuteExperiment,
    ValidateDiscovery,
    CollaborateWithAgent,
    ExploreNewDomain,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_open_world_environment() {
        let config = OpenWorldConfig {
            max_concurrent_experiments: 10,
            resource_limits: HashMap::new(),
            collaboration_enabled: true,
            peer_review_enabled: true,
            discovery_validation_required: true,
            exploration_exploitation_ratio: 0.3,
            minimum_evidence_strength: 0.7,
            minimum_reproducibility: 0.8,
        };

        let env = OpenWorldResearchEnvironment::new(config);

        let hypothesis = ResearchHypothesis {
            id: "hypothesis_1".to_string(),
            description: "Test hypothesis".to_string(),
            domain: "AI".to_string(),
            complexity: 0.7,
            novelty: 0.8,
            testability: 0.9,
            created_by: "agent_1".to_string(),
            created_at: Utc::now(),
            status: HypothesisStatus::Proposed,
            supporting_evidence: Vec::new(),
            counter_evidence: Vec::new(),
            confidence: 0.6,
        };

        let hypothesis_id = env.propose_hypothesis(hypothesis, "agent_1").await.unwrap();
        assert_eq!(hypothesis_id, "hypothesis_1");

        let metrics = env.get_metrics().await;
        assert_eq!(metrics.total_hypotheses, 1);
    }

    #[tokio::test]
    async fn test_experiment_design_and_execution() {
        let config = OpenWorldConfig {
            max_concurrent_experiments: 10,
            resource_limits: HashMap::new(),
            collaboration_enabled: true,
            peer_review_enabled: true,
            discovery_validation_required: true,
            exploration_exploitation_ratio: 0.3,
            minimum_evidence_strength: 0.7,
            minimum_reproducibility: 0.8,
        };

        let env = OpenWorldResearchEnvironment::new(config);

        let experiment = ResearchExperiment {
            id: "experiment_1".to_string(),
            hypothesis_id: "hypothesis_1".to_string(),
            design: ExperimentDesign {
                design_type: ExperimentDesignType::Experimental,
                sample_size: Some(100),
                control_group: true,
                randomization: true,
                blinding: Some(BlindingType::DoubleBlind),
                duration_hours: 24.0,
                resources_required: vec!["compute".to_string(), "data".to_string()],
            },
            methodology: "Randomized controlled trial".to_string(),
            parameters: HashMap::new(),
            expected_outcomes: vec!["Improved performance".to_string()],
            conducted_by: vec!["agent_1".to_string(), "agent_2".to_string()],
            start_time: Utc::now(),
            end_time: None,
            results: None,
            status: ExperimentStatus::Designed,
        };

        let experiment_id = env
            .design_experiment(experiment, &["agent_1".to_string(), "agent_2".to_string()])
            .await
            .unwrap();
        assert_eq!(experiment_id, "experiment_1");

        let results = ExperimentResults {
            raw_data: serde_json::json!({"result": "success"}),
            analysis_method: "t-test".to_string(),
            statistical_significance: Some(0.05),
            effect_size: Some(0.8),
            confidence_interval: Some((0.6, 0.9)),
            interpretation: "Hypothesis supported".to_string(),
            limitations: vec!["Small sample size".to_string()],
            implications: vec!["Further research needed".to_string()],
        };

        env.execute_experiment("experiment_1", results, "agent_1")
            .await
            .unwrap();

        let experiments = env
            .get_experiments_by_status(ExperimentStatus::Completed)
            .await;
        assert_eq!(experiments.len(), 1);
    }

    #[tokio::test]
    async fn test_discovery_and_validation() {
        let config = OpenWorldConfig {
            max_concurrent_experiments: 10,
            resource_limits: HashMap::new(),
            collaboration_enabled: true,
            peer_review_enabled: true,
            discovery_validation_required: true,
            exploration_exploitation_ratio: 0.3,
            minimum_evidence_strength: 0.7,
            minimum_reproducibility: 0.8,
        };

        let env = OpenWorldResearchEnvironment::new(config);

        let discovery = ResearchDiscovery {
            id: "discovery_1".to_string(),
            title: "Test Discovery".to_string(),
            description: "A significant finding".to_string(),
            domain: "AI".to_string(),
            significance: DiscoverySignificance::Major,
            novelty: 0.9,
            reproducibility: 0.95,
            supporting_hypotheses: vec!["hypothesis_1".to_string()],
            supporting_experiments: vec!["experiment_1".to_string()],
            evidence: Vec::new(),
            discovered_by: vec!["agent_1".to_string(), "agent_2".to_string()],
            discovered_at: Utc::now(),
            validation_status: ValidationStatus::Unvalidated,
            peer_reviews: Vec::new(),
            impact_score: 0.8,
        };

        let discovery_id = env
            .record_discovery(discovery, &["agent_1".to_string(), "agent_2".to_string()])
            .await
            .unwrap();
        assert_eq!(discovery_id, "discovery_1");

        let peer_review = PeerReview {
            reviewer_id: "agent_3".to_string(),
            review_date: Utc::now(),
            rating: 0.9,
            comments: "Well-documented and reproducible".to_string(),
            suggested_improvements: vec!["Add more statistical tests".to_string()],
            validation_attempts: vec![ValidationAttempt {
                attempt_id: "validation_1".to_string(),
                validator_id: "agent_3".to_string(),
                method: ValidationMethod::Replication,
                success: true,
                notes: "Successfully replicated".to_string(),
                timestamp: Utc::now(),
            }],
        };

        env.validate_discovery("discovery_1", peer_review)
            .await
            .unwrap();

        let metrics = env.get_metrics().await;
        assert_eq!(metrics.total_discoveries, 1);
    }
}
