//! Example demonstrating the open-world research environment for emergent collaboration and discovery.
//!
//! This example shows how agents can:
//! 1. Propose and investigate research hypotheses
//! 2. Design and execute experiments
//! 3. Make discoveries and have them validated
//! 4. Collaborate emergently on research projects
//! 5. Balance exploration vs exploitation in research

use chrono::Utc;
use constellation_core::autonomy::{
    AutonomyMeasurementEngine, DiscoverySignificance, ExperimentStatus, HypothesisStatus,
    OpenWorldConfig, OpenWorldResearchEnvironment, ResearchDiscovery, ResearchExperiment,
    ResearchHypothesis, ValidationStatus, measurement_engine::MeasurementConfig,
};
use constellation_core::models::CapabilityAxis;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    println!("=== Open-World Research Environment Example ===\n");

    // Create configuration for the open-world environment
    let config = OpenWorldConfig {
        max_concurrent_experiments: 5,
        resource_limits: {
            let mut map = HashMap::new();
            map.insert("compute".to_string(), 1000.0);
            map.insert("data".to_string(), 100.0);
            map.insert("storage".to_string(), 10000.0);
            map
        },
        collaboration_enabled: true,
        peer_review_enabled: true,
        discovery_validation_required: true,
        exploration_exploitation_ratio: 0.3,
        minimum_evidence_strength: 0.7,
        minimum_reproducibility: 0.8,
    };

    // Create the open-world research environment
    let env = OpenWorldResearchEnvironment::new(config);

    // Create an autonomy measurement engine for tracking agent capabilities
    let autonomy_engine = AutonomyMeasurementEngine::new(MeasurementConfig::default());

    // Example 1: Propose research hypotheses
    println!("1. Proposing Research Hypotheses");
    println!("--------------------------------");

    let hypothesis_1 = ResearchHypothesis {
        id: "hyp_ai_optimization".to_string(),
        description: "Novel optimization algorithm can improve training efficiency by 30%"
            .to_string(),
        domain: "Machine Learning".to_string(),
        complexity: 0.8,
        novelty: 0.9,
        testability: 0.7,
        created_by: "agent_researcher_1".to_string(),
        created_at: Utc::now(),
        status: HypothesisStatus::Proposed,
        supporting_evidence: vec![],
        counter_evidence: vec![],
        confidence: 0.6,
    };

    let hypothesis_2 = ResearchHypothesis {
        id: "hyp_nlp_understanding".to_string(),
        description: "Transformer models develop emergent understanding of syntax".to_string(),
        domain: "Natural Language Processing".to_string(),
        complexity: 0.9,
        novelty: 0.7,
        testability: 0.8,
        created_by: "agent_linguist_1".to_string(),
        created_at: Utc::now(),
        status: HypothesisStatus::Proposed,
        supporting_evidence: vec![],
        counter_evidence: vec![],
        confidence: 0.7,
    };

    env.propose_hypothesis(hypothesis_1, "agent_researcher_1")
        .await
        .unwrap();
    env.propose_hypothesis(hypothesis_2, "agent_linguist_1")
        .await
        .unwrap();

    println!("✓ Proposed 2 research hypotheses");
    println!("  - AI optimization algorithm hypothesis");
    println!("  - NLP understanding hypothesis\n");

    // Example 2: Design experiments
    println!("2. Designing Research Experiments");
    println!("---------------------------------");

    let experiment_1 = ResearchExperiment {
        id: "exp_optimization_trial".to_string(),
        hypothesis_id: "hyp_ai_optimization".to_string(),
        design: constellation_core::autonomy::open_world::ExperimentDesign {
            design_type:
                constellation_core::autonomy::open_world::ExperimentDesignType::Experimental,
            sample_size: Some(50),
            control_group: true,
            randomization: true,
            blinding: Some(constellation_core::autonomy::open_world::BlindingType::DoubleBlind),
            duration_hours: 48.0,
            resources_required: vec!["compute".to_string(), "data".to_string()],
        },
        methodology: "Randomized controlled trial with A/B testing".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("learning_rate".to_string(), serde_json::json!(0.001));
            params.insert("batch_size".to_string(), serde_json::json!(32));
            params.insert("epochs".to_string(), serde_json::json!(100));
            params
        },
        expected_outcomes: vec![
            "30% improvement in training speed".to_string(),
            "No degradation in model accuracy".to_string(),
        ],
        conducted_by: vec![
            "agent_researcher_1".to_string(),
            "agent_researcher_2".to_string(),
        ],
        start_time: Utc::now(),
        end_time: None,
        results: None,
        status: ExperimentStatus::Designed,
    };

    env.design_experiment(
        experiment_1,
        &[
            "agent_researcher_1".to_string(),
            "agent_researcher_2".to_string(),
        ],
    )
    .await
    .unwrap();

    println!("✓ Designed experiment for optimization hypothesis");
    println!("  - Collaborative experiment with 2 researchers");
    println!("  - Double-blind randomized controlled trial\n");

    // Example 3: Execute experiment and record results
    println!("3. Executing Experiment and Recording Results");
    println!("---------------------------------------------");

    let results = constellation_core::autonomy::open_world::ExperimentResults {
        raw_data: serde_json::json!({
            "control_group_performance": 0.85,
            "experimental_group_performance": 0.89,
            "training_time_reduction": 0.32,
            "statistical_tests": {
                "t_test_p_value": 0.012,
                "effect_size": 0.45,
                "confidence_interval": [0.28, 0.62]
            }
        }),
        analysis_method: "Mixed-effects linear regression".to_string(),
        statistical_significance: Some(0.012),
        effect_size: Some(0.45),
        confidence_interval: Some((0.28, 0.62)),
        interpretation: "The novel optimization algorithm shows statistically significant improvement in training efficiency (32% reduction) with no degradation in model performance.".to_string(),
        limitations: vec![
            "Limited to image classification tasks".to_string(),
            "Small sample size of 50 models".to_string(),
        ],
        implications: vec![
            "Could significantly reduce computational costs for large-scale AI training".to_string(),
            "May enable more efficient hyperparameter search".to_string(),
        ],
    };

    env.execute_experiment("exp_optimization_trial", results, "agent_researcher_1")
        .await
        .unwrap();

    println!("✓ Executed experiment and recorded results");
    println!("  - 32% training time reduction observed");
    println!("  - Statistically significant (p=0.012)");
    println!("  - No degradation in model accuracy\n");

    // Example 4: Make a discovery
    println!("4. Making a Research Discovery");
    println!("------------------------------");

    let discovery = ResearchDiscovery {
        id: "disc_optimization_breakthrough".to_string(),
        title: "Efficient Gradient Approximation Algorithm".to_string(),
        description: "A novel gradient approximation method that reduces backpropagation computation by 32% while maintaining accuracy.".to_string(),
        domain: "Machine Learning".to_string(),
        significance: DiscoverySignificance::Major,
        novelty: 0.85,
        reproducibility: 0.92,
        supporting_hypotheses: vec!["hyp_ai_optimization".to_string()],
        supporting_experiments: vec!["exp_optimization_trial".to_string()],
        evidence: vec![],
        discovered_by: vec!["agent_researcher_1".to_string(), "agent_researcher_2".to_string()],
        discovered_at: Utc::now(),
        validation_status: ValidationStatus::Unvalidated,
        peer_reviews: vec![],
        impact_score: 0.8,
    };

    env.record_discovery(
        discovery,
        &[
            "agent_researcher_1".to_string(),
            "agent_researcher_2".to_string(),
        ],
    )
    .await
    .unwrap();

    println!("✓ Recorded major discovery");
    println!("  - 'Efficient Gradient Approximation Algorithm'");
    println!("  - Collaborative discovery by 2 researchers");
    println!("  - Requires validation through peer review\n");

    // Example 5: Peer review and validation
    println!("5. Peer Review and Validation");
    println!("-----------------------------");

    let peer_review_1 = constellation_core::autonomy::open_world::PeerReview {
        reviewer_id: "agent_expert_1".to_string(),
        review_date: Utc::now(),
        rating: 0.95,
        comments: "Excellent methodology, clear results, and significant practical implications. The 32% efficiency gain is substantial and well-documented.".to_string(),
        suggested_improvements: vec![
            "Test on larger variety of architectures".to_string(),
            "Compare with other optimization methods".to_string(),
        ],
        validation_attempts: vec![constellation_core::autonomy::open_world::ValidationAttempt {
            attempt_id: "val_1".to_string(),
            validator_id: "agent_expert_1".to_string(),
            method: constellation_core::autonomy::open_world::ValidationMethod::Replication,
            success: true,
            notes: "Successfully replicated results with 31.5% improvement".to_string(),
            timestamp: Utc::now(),
        }],
    };

    let peer_review_2 = constellation_core::autonomy::open_world::PeerReview {
        reviewer_id: "agent_expert_2".to_string(),
        review_date: Utc::now(),
        rating: 0.88,
        comments: "Solid research with clear statistical significance. The limitations section is appropriately cautious.".to_string(),
        suggested_improvements: vec![
            "Include computational complexity analysis".to_string(),
            "Test on reinforcement learning tasks".to_string(),
        ],
        validation_attempts: vec![constellation_core::autonomy::open_world::ValidationAttempt {
            attempt_id: "val_2".to_string(),
            validator_id: "agent_expert_2".to_string(),
            method: constellation_core::autonomy::open_world::ValidationMethod::IndependentVerification,
            success: true,
            notes: "Independently verified using different dataset".to_string(),
            timestamp: Utc::now(),
        }],
    };

    env.validate_discovery("disc_optimization_breakthrough", peer_review_1)
        .await
        .unwrap();
    env.validate_discovery("disc_optimization_breakthrough", peer_review_2)
        .await
        .unwrap();

    println!("✓ Completed peer review process");
    println!("  - 2 independent validations successful");
    println!("  - Average rating: 0.915");
    println!("  - Discovery marked as fully validated\n");

    // Example 6: Generate research recommendations
    println!("6. Generating Research Recommendations");
    println!("--------------------------------------");

    let agent_capabilities = vec![
        CapabilityAxis::Planning,
        CapabilityAxis::Reasoning,
        CapabilityAxis::Creativity,
    ];

    let recommendations = env
        .generate_recommendations("agent_new_researcher", &agent_capabilities)
        .await;

    println!("✓ Generated recommendations for new researcher:");
    for (i, rec) in recommendations.iter().enumerate().take(3) {
        println!(
            "  {}. {} (confidence: {:.2})",
            i + 1,
            rec.rationale,
            rec.confidence
        );
    }
    println!();

    // Example 7: Integrate discovery into autonomy measurement
    println!("7. Integrating Discovery into Autonomy Measurement");
    println!("--------------------------------------------------");

    env.integrate_discovery_into_autonomy("disc_optimization_breakthrough", &autonomy_engine)
        .await
        .unwrap();

    // Record capability observations for the discovering agents
    autonomy_engine.record_observation(
        "agent_researcher_1",
        CapabilityAxis::Creativity,
        0.9,
        1.0,
        None,
        1.0,
    );

    autonomy_engine.record_observation(
        "agent_researcher_2",
        CapabilityAxis::Collaboration,
        0.95,
        1.0,
        None,
        1.0,
    );

    println!("✓ Integrated discovery into autonomy system");
    println!("  - Boosted creativity score for researcher 1");
    println!("  - Boosted collaboration score for researcher 2");
    println!("  - Discovery contributes to autonomy level progression\n");

    // Example 8: Get environment metrics
    println!("8. Environment Metrics and Analytics");
    println!("------------------------------------");

    let metrics = env.get_metrics().await;

    println!("Environment Metrics:");
    println!("  - Total Hypotheses: {}", metrics.total_hypotheses);
    println!("  - Total Experiments: {}", metrics.total_experiments);
    println!("  - Total Discoveries: {}", metrics.total_discoveries);
    println!(
        "  - Discovery Validation Rate: {:.1}%",
        metrics.discovery_validation_rate * 100.0
    );
    println!(
        "  - Collaboration Efficiency: {:.1}%",
        metrics.collaboration_efficiency * 100.0
    );
    println!(
        "  - Exploration/Exploitation Balance: {:.1}% exploration",
        metrics.exploration_exploitation_balance * 100.0
    );
    println!();

    // Example 9: Query research artifacts
    println!("9. Querying Research Artifacts");
    println!("------------------------------");

    let ml_hypotheses = env.get_hypotheses_by_domain("Machine Learning").await;
    println!("Machine Learning Hypotheses: {}", ml_hypotheses.len());

    let completed_experiments = env
        .get_experiments_by_status(ExperimentStatus::Completed)
        .await;
    println!("Completed Experiments: {}", completed_experiments.len());

    let major_discoveries = env
        .get_discoveries_by_significance(DiscoverySignificance::Major)
        .await;
    println!("Major Discoveries: {}", major_discoveries.len());
    println!();

    println!("=== Example Complete ===");
    println!("\nSummary:");
    println!("- Created open-world research environment with collaborative discovery");
    println!(
        "- Demonstrated full research lifecycle: hypothesis → experiment → discovery → validation"
    );
    println!("- Showed emergent collaboration between agents");
    println!("- Integrated research outcomes into autonomy measurement system");
    println!("- Generated metrics and recommendations for ongoing research");
}
