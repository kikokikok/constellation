//! Collaboration pattern detection for emergent multi-agent behaviors.

use crate::hybrid::coordinator::TaskResult;
use crate::models::autonomy::{CollaborationPattern, CollaborationPatternType};
use crate::models::dtg::{DataTransformationGraph, DtgEdge};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Configuration for collaboration pattern detection.
#[derive(Debug, Clone)]
pub struct CollaborationConfig {
    /// Minimum number of collaborations to detect a pattern.
    pub min_collaborations: u32,

    /// Minimum strength threshold for pattern detection.
    pub min_strength_threshold: f64,

    /// Minimum efficiency threshold for pattern detection.
    pub min_efficiency_threshold: f64,

    /// Time window for pattern analysis (in seconds).
    pub analysis_time_window: Duration,

    /// Minimum agents required for swarm pattern detection.
    pub min_agents_for_swarm: u32,

    /// Maximum age of collaborations to consider.
    pub max_collaboration_age: Duration,

    /// Whether to detect adaptive patterns.
    pub detect_adaptive_patterns: bool,

    /// Whether to detect hybrid patterns.
    pub detect_hybrid_patterns: bool,
}

impl Default for CollaborationConfig {
    fn default() -> Self {
        Self {
            min_collaborations: 5,
            min_strength_threshold: 0.6,
            min_efficiency_threshold: 0.5,
            analysis_time_window: Duration::from_secs(3600), // 1 hour
            min_agents_for_swarm: 5,
            max_collaboration_age: Duration::from_secs(86400), // 24 hours
            detect_adaptive_patterns: true,
            detect_hybrid_patterns: true,
        }
    }
}

/// Collaboration event between agents.
#[derive(Debug, Clone)]
pub struct CollaborationEvent {
    event_id: Uuid,
    agent_ids: Vec<String>,
    task_id: Option<Uuid>,
    success: bool,
    efficiency: f64,
    duration: Duration,
    timestamp: SystemTime,
    metadata: HashMap<String, serde_json::Value>,
}

impl CollaborationEvent {
    fn new(
        agent_ids: Vec<String>,
        task_id: Option<Uuid>,
        success: bool,
        efficiency: f64,
        duration: Duration,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            agent_ids,
            task_id,
            success,
            efficiency: efficiency.clamp(0.0, 1.0),
            duration,
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

/// Collaboration pattern detector for emergent multi-agent behaviors.
#[derive(Debug)]
pub struct CollaborationPatternDetector {
    config: CollaborationConfig,
    events: Arc<RwLock<HashMap<String, VecDeque<CollaborationEvent>>>>,
    patterns: Arc<RwLock<HashMap<String, CollaborationPattern>>>,
    agent_graphs: Arc<RwLock<HashMap<String, AgentCollaborationGraph>>>,
}

impl CollaborationPatternDetector {
    /// Create a new collaboration pattern detector.
    pub fn new(config: CollaborationConfig) -> Self {
        Self {
            config,
            events: Arc::new(RwLock::new(HashMap::new())),
            patterns: Arc::new(RwLock::new(HashMap::new())),
            agent_graphs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a collaboration event.
    pub fn record_collaboration(
        &self,
        agent_ids: Vec<String>,
        task_id: Option<Uuid>,
        success: bool,
        efficiency: f64,
        duration: Duration,
    ) {
        let event =
            CollaborationEvent::new(agent_ids.clone(), task_id, success, efficiency, duration);

        // Record event for each agent
        let mut events = self.events.write().unwrap();
        for agent_id in &agent_ids {
            let agent_events = events.entry(agent_id.clone()).or_default();
            agent_events.push_back(event.clone());
        }

        // Clean up old events
        self.cleanup_events();

        // Update agent collaboration graphs
        self.update_agent_graphs(&agent_ids, &event);

        // Detect patterns
        self.detect_patterns(&agent_ids);
    }

    /// Record collaboration from task result involving multiple agents.
    pub fn record_from_multi_agent_task(
        &self,
        agent_ids: Vec<String>,
        task_result: &TaskResult,
        individual_contributions: HashMap<String, f64>, // Contribution scores 0.0-1.0
    ) {
        // Calculate collaboration efficiency
        let efficiency = if task_result.success {
            // Efficiency based on quality and individual contributions
            let avg_contribution: f64 = individual_contributions.values().sum::<f64>()
                / individual_contributions.len() as f64;
            task_result.quality_score * 0.7 + avg_contribution * 0.3
        } else {
            0.3
        };

        self.record_collaboration(
            agent_ids,
            Some(task_result.task_id),
            task_result.success,
            efficiency,
            Duration::from_millis(task_result.execution_time_ms),
        );
    }

    /// Record collaboration from DTG edges (data flow between agents).
    pub fn record_from_dtg_edges(&self, dtg: &DataTransformationGraph, edges: &[DtgEdge]) {
        for edge in edges {
            // Extract agent IDs from source and target nodes
            if let (Some(source_agent), Some(target_agent)) = (
                self.extract_agent_id_from_node(dtg, &edge.source),
                self.extract_agent_id_from_node(dtg, &edge.target),
            ) && source_agent != target_agent
            {
                // This represents collaboration through data transformation
                let efficiency = 0.7; // Default efficiency since edge doesn't have quality_score
                let success = true; // Edge is considered active if it exists

                self.record_collaboration(
                    vec![source_agent, target_agent],
                    None, // No specific task ID
                    success,
                    efficiency,
                    Duration::from_secs(1), // Default duration
                );
            }
        }
    }

    /// Extract agent ID from DTG node.
    fn extract_agent_id_from_node(
        &self,
        dtg: &DataTransformationGraph,
        node_id: &Uuid,
    ) -> Option<String> {
        dtg.nodes
            .iter()
            .find(|(_, node)| node.id == *node_id)
            .and_then(|(_, node)| node.metadata.get("agent_id"))
            .and_then(|value| value.as_str())
            .map(|s| s.to_string())
    }

    /// Clean up old events.
    fn cleanup_events(&self) {
        let mut events = self.events.write().unwrap();

        for agent_events in events.values_mut() {
            agent_events.retain(|event| event.is_valid(self.config.max_collaboration_age));

            // Keep only recent events
            let max_events = self.config.min_collaborations * 3;
            if agent_events.len() > max_events as usize {
                let to_remove = agent_events.len() - max_events as usize;
                for _ in 0..to_remove {
                    agent_events.pop_front();
                }
            }
        }
    }

    /// Update agent collaboration graphs.
    fn update_agent_graphs(&self, agent_ids: &[String], event: &CollaborationEvent) {
        let mut agent_graphs = self.agent_graphs.write().unwrap();

        for agent_id in agent_ids {
            let graph = agent_graphs
                .entry(agent_id.clone())
                .or_insert_with(|| AgentCollaborationGraph::new(agent_id.clone()));

            // Add connections to other agents in this collaboration
            for other_agent_id in agent_ids {
                if other_agent_id != agent_id {
                    graph.add_connection(
                        other_agent_id.clone(),
                        event.success,
                        event.efficiency,
                        event.duration,
                    );
                }
            }
        }
    }

    /// Detect collaboration patterns for a group of agents.
    fn detect_patterns(&self, agent_ids: &[String]) {
        if agent_ids.len() < 2 {
            return;
        }

        // Get recent events for these agents
        let events = self.events.read().unwrap();
        let mut group_events = Vec::new();

        for agent_id in agent_ids {
            if let Some(agent_events) = events.get(agent_id) {
                for event in agent_events {
                    if event.agent_ids.iter().collect::<HashSet<_>>()
                        == agent_ids.iter().collect::<HashSet<_>>()
                    {
                        group_events.push(event.clone());
                    }
                }
            }
        }

        if group_events.len() < self.config.min_collaborations as usize {
            return;
        }

        // Calculate pattern metrics
        let success_count = group_events.iter().filter(|e| e.success).count() as u32;
        let failure_count = group_events.len() as u32 - success_count;
        let avg_efficiency: f64 =
            group_events.iter().map(|e| e.efficiency).sum::<f64>() / group_events.len() as f64;

        // Detect pattern type
        let pattern_type = self.detect_pattern_type(agent_ids, &group_events);

        // Calculate pattern strength
        let strength = self.calculate_pattern_strength(
            success_count,
            failure_count,
            avg_efficiency,
            pattern_type,
        );

        if strength >= self.config.min_strength_threshold
            && avg_efficiency >= self.config.min_efficiency_threshold
        {
            // Create or update pattern
            let pattern_id = self.generate_pattern_id(agent_ids, pattern_type);
            let mut patterns = self.patterns.write().unwrap();

            if let Some(existing_pattern) = patterns.get_mut(&pattern_id) {
                // Update existing pattern
                existing_pattern.update_with_observation(
                    group_events.last().unwrap().success,
                    avg_efficiency,
                    group_events.last().unwrap().duration,
                );
            } else {
                // Create new pattern
                let pattern = CollaborationPattern::new(
                    agent_ids.to_vec(),
                    pattern_type,
                    strength,
                    avg_efficiency,
                    success_count,
                    failure_count,
                );
                patterns.insert(pattern_id, pattern);
            }
        }
    }

    /// Detect pattern type based on agent interactions.
    fn detect_pattern_type(
        &self,
        agent_ids: &[String],
        events: &[CollaborationEvent],
    ) -> CollaborationPatternType {
        if agent_ids.len() >= self.config.min_agents_for_swarm as usize {
            // Check for swarm characteristics
            if self.is_swarm_pattern(agent_ids, events) {
                return CollaborationPatternType::Swarm;
            }
        }

        // Check for hierarchical structure
        if self.is_hierarchical_pattern(agent_ids, events) {
            return CollaborationPatternType::Hierarchical;
        }

        // Check for specialized roles
        if self.is_specialized_pattern(agent_ids, events) {
            return CollaborationPatternType::Specialized;
        }

        // Check for market-based interactions
        if self.is_market_based_pattern(agent_ids, events) {
            return CollaborationPatternType::MarketBased;
        }

        // Check for adaptive patterns
        if self.config.detect_adaptive_patterns && self.is_adaptive_pattern(agent_ids, events) {
            return CollaborationPatternType::Adaptive;
        }

        // Check for hybrid patterns
        if self.config.detect_hybrid_patterns && self.is_hybrid_pattern(agent_ids, events) {
            return CollaborationPatternType::Hybrid;
        }

        // Default to distributed pattern
        CollaborationPatternType::Distributed
    }

    /// Check if pattern exhibits swarm characteristics.
    fn is_swarm_pattern(&self, agent_ids: &[String], events: &[CollaborationEvent]) -> bool {
        // Swarm patterns have many simple agents with emergent behavior
        // Check for simple individual behaviors that lead to complex group outcomes

        if agent_ids.len() < self.config.min_agents_for_swarm as usize {
            return false;
        }

        // Analyze event metadata for swarm characteristics
        let mut simple_individual_actions = 0;
        let mut complex_group_outcomes = 0;

        for event in events {
            if let Some(is_simple) = event.metadata.get("individual_action_simple")
                && is_simple.as_bool().unwrap_or(false)
            {
                simple_individual_actions += 1;
            }

            if let Some(is_complex) = event.metadata.get("group_outcome_complex")
                && is_complex.as_bool().unwrap_or(false)
            {
                complex_group_outcomes += 1;
            }
        }

        // Swarm pattern if most individual actions are simple but group outcomes are complex
        simple_individual_actions > events.len() / 2 && complex_group_outcomes > events.len() / 3
    }

    /// Check if pattern exhibits hierarchical structure.
    fn is_hierarchical_pattern(&self, agent_ids: &[String], events: &[CollaborationEvent]) -> bool {
        // Hierarchical patterns have clear leader-follower relationships
        // Check for consistent decision-making hierarchy

        let agent_graphs = self.agent_graphs.read().unwrap();

        for agent_id in agent_ids {
            if let Some(graph) = agent_graphs.get(agent_id) {
                // Check if this agent consistently leads or follows
                let leadership_score = graph.calculate_leadership_score();
                if leadership_score > 0.7 {
                    // This agent appears to be a leader
                    // Check if others follow consistently
                    let mut follower_count = 0;
                    for other_agent_id in agent_ids {
                        if other_agent_id != agent_id
                            && let Some(other_graph) = agent_graphs.get(other_agent_id)
                            && other_graph.get_connection_strength(agent_id) > 0.6
                        {
                            follower_count += 1;
                        }
                    }

                    if follower_count >= agent_ids.len() - 1 {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if pattern exhibits specialized roles.
    fn is_specialized_pattern(&self, agent_ids: &[String], events: &[CollaborationEvent]) -> bool {
        // Specialized patterns have agents with complementary skills
        // Check for consistent role specialization across events

        let mut role_consistency = HashMap::new();

        for event in events {
            for agent_id in &event.agent_ids {
                if let Some(role) = event.metadata.get(&format!("{agent_id}_role")) {
                    let role_str = role.as_str().unwrap_or("");
                    *role_consistency
                        .entry((agent_id.clone(), role_str.to_string()))
                        .or_insert(0) += 1;
                }
            }
        }

        // Check if agents have consistent specialized roles
        let mut specialized_agents = 0;

        for agent_id in agent_ids {
            let agent_roles: Vec<_> = role_consistency
                .iter()
                .filter(|((id, _), _)| id == agent_id)
                .collect();

            if !agent_roles.is_empty() {
                // Find the most common role for this agent
                let ((_, most_common_role), _) =
                    agent_roles.iter().max_by_key(|(_, count)| *count).unwrap();

                // Check if this role is specialized (not "general" or "unspecified")
                let specialized_roles = ["expert", "specialist", "analyst", "executor", "planner"];
                if specialized_roles
                    .iter()
                    .any(|r| most_common_role.contains(*r))
                {
                    specialized_agents += 1;
                }
            }
        }

        // Pattern is specialized if most agents have specialized roles
        specialized_agents >= agent_ids.len() / 2
    }

    /// Check if pattern exhibits market-based interactions.
    fn is_market_based_pattern(&self, agent_ids: &[String], events: &[CollaborationEvent]) -> bool {
        // Market-based patterns involve bidding, auction, or resource trading
        // Check for market-like metadata in events

        let mut market_events = 0;

        for event in events {
            if let Some(mechanism) = event.metadata.get("interaction_mechanism") {
                let mechanism_str = mechanism.as_str().unwrap_or("");
                let market_mechanisms = ["bid", "auction", "trade", "market", "price", "cost"];

                if market_mechanisms.iter().any(|m| mechanism_str.contains(m)) {
                    market_events += 1;
                }
            }
        }

        // Pattern is market-based if significant portion of events involve market mechanisms
        market_events >= events.len() / 3
    }

    /// Check if pattern exhibits adaptive behavior.
    fn is_adaptive_pattern(&self, agent_ids: &[String], events: &[CollaborationEvent]) -> bool {
        // Adaptive patterns change structure based on task requirements
        // Check for changing roles or relationships across events

        if events.len() < 3 {
            return false;
        }

        // Analyze how relationships change over time
        let mut relationship_changes = 0;

        for i in 1..events.len() {
            let prev_event = &events[i - 1];
            let curr_event = &events[i];

            // Check if interaction patterns changed
            if let (Some(prev_pattern), Some(curr_pattern)) = (
                prev_event.metadata.get("interaction_pattern"),
                curr_event.metadata.get("interaction_pattern"),
            ) && prev_pattern != curr_pattern
            {
                relationship_changes += 1;
            }
        }

        // Pattern is adaptive if relationships change frequently
        relationship_changes >= events.len() / 3
    }

    /// Check if pattern exhibits hybrid characteristics.
    fn is_hybrid_pattern(&self, agent_ids: &[String], events: &[CollaborationEvent]) -> bool {
        // Hybrid patterns combine multiple pattern types
        // Check for evidence of multiple pattern characteristics

        let mut pattern_evidences = 0;

        // Check for hierarchical evidence
        if self.is_hierarchical_pattern(agent_ids, events) {
            pattern_evidences += 1;
        }

        // Check for specialized evidence
        if self.is_specialized_pattern(agent_ids, events) {
            pattern_evidences += 1;
        }

        // Check for market-based evidence
        if self.is_market_based_pattern(agent_ids, events) {
            pattern_evidences += 1;
        }

        // Pattern is hybrid if it exhibits multiple characteristics
        pattern_evidences >= 2
    }

    /// Calculate pattern strength.
    fn calculate_pattern_strength(
        &self,
        success_count: u32,
        failure_count: u32,
        efficiency: f64,
        pattern_type: CollaborationPatternType,
    ) -> f64 {
        let total = success_count + failure_count;
        let success_rate = if total > 0 {
            success_count as f64 / total as f64
        } else {
            0.0
        };

        // Base strength on success rate and efficiency
        let mut strength = success_rate * 0.6 + efficiency * 0.4;

        // Adjust for pattern type complexity
        match pattern_type {
            CollaborationPatternType::Hierarchical => strength *= 0.9,
            CollaborationPatternType::Distributed => strength *= 0.8,
            CollaborationPatternType::Specialized => strength *= 1.0,
            CollaborationPatternType::Adaptive => strength *= 1.1,
            CollaborationPatternType::Swarm => strength *= 1.2,
            CollaborationPatternType::MarketBased => strength *= 1.0,
            CollaborationPatternType::Consensus => strength *= 0.9,
            CollaborationPatternType::MentorApprentice => strength *= 0.8,
            CollaborationPatternType::Competitive => strength *= 0.7,
            CollaborationPatternType::Hybrid => strength *= 1.3,
        }

        strength.clamp(0.0, 1.0)
    }

    /// Generate pattern ID.
    fn generate_pattern_id(
        &self,
        agent_ids: &[String],
        pattern_type: CollaborationPatternType,
    ) -> String {
        let mut sorted_ids = agent_ids.to_vec();
        sorted_ids.sort();

        format!(
            "{}_{}",
            sorted_ids.join("_"),
            pattern_type.name().to_lowercase()
        )
    }

    /// Get collaboration patterns for an agent.
    pub fn get_agent_patterns(&self, agent_id: &str) -> Vec<CollaborationPattern> {
        let patterns = self.patterns.read().unwrap();

        patterns
            .values()
            .filter(|pattern| pattern.agent_ids.contains(&agent_id.to_string()))
            .cloned()
            .collect()
    }

    /// Get all detected patterns.
    pub fn get_all_patterns(&self) -> Vec<CollaborationPattern> {
        let patterns = self.patterns.read().unwrap();
        patterns.values().cloned().collect()
    }

    /// Get stable patterns (patterns with sufficient observations).
    pub fn get_stable_patterns(&self) -> Vec<CollaborationPattern> {
        self.get_all_patterns()
            .into_iter()
            .filter(|pattern| {
                pattern.is_stable(
                    self.config.min_collaborations,
                    self.config.min_strength_threshold,
                )
            })
            .collect()
    }

    /// Get collaboration efficiency for an agent.
    pub fn get_agent_efficiency(&self, agent_id: &str) -> Option<f64> {
        let events = self.events.read().unwrap();
        let agent_events = events.get(agent_id)?;

        if agent_events.is_empty() {
            return None;
        }

        let total_efficiency: f64 = agent_events.iter().map(|e| e.efficiency).sum();
        Some(total_efficiency / agent_events.len() as f64)
    }

    /// Get collaboration partners for an agent.
    pub fn get_collaboration_partners(&self, agent_id: &str) -> Vec<(String, f64)> {
        let agent_graphs = self.agent_graphs.read().unwrap();

        if let Some(graph) = agent_graphs.get(agent_id) {
            graph.get_connections()
        } else {
            Vec::new()
        }
    }

    /// Estimate collaboration capability κ score.
    pub fn estimate_collaboration_kappa(
        &self,
        agent_id: &str,
    ) -> Option<crate::models::autonomy::KappaScore> {
        let efficiency = self.get_agent_efficiency(agent_id)?;
        let patterns = self.get_agent_patterns(agent_id);

        if patterns.is_empty() {
            return None;
        }

        // Calculate score based on efficiency and pattern diversity
        let avg_pattern_strength: f64 =
            patterns.iter().map(|p| p.strength).sum::<f64>() / patterns.len() as f64;

        let pattern_diversity = self.calculate_pattern_diversity(&patterns);

        // Score is weighted combination of efficiency, strength, and diversity
        let efficiency_weight = 0.4;
        let strength_weight = 0.4;
        let diversity_weight = 0.2;

        let score = efficiency * efficiency_weight
            + avg_pattern_strength * strength_weight
            + pattern_diversity * diversity_weight;

        let confidence = (patterns.len() as f64 / 10.0).min(1.0) * 0.7;

        Some(crate::models::autonomy::KappaScore::new(
            crate::models::autonomy::CapabilityAxis::Collaboration,
            score,
            confidence,
            patterns.len() as u32,
        ))
    }

    /// Calculate pattern diversity for an agent.
    fn calculate_pattern_diversity(&self, patterns: &[CollaborationPattern]) -> f64 {
        if patterns.is_empty() {
            return 0.0;
        }

        let unique_pattern_types: HashSet<_> = patterns.iter().map(|p| p.pattern_type).collect();
        let max_pattern_types = CollaborationPatternType::all().len();

        unique_pattern_types.len() as f64 / max_pattern_types as f64
    }

    /// Generate collaboration report for an agent.
    pub fn generate_collaboration_report(&self, agent_id: &str) -> Option<CollaborationReport> {
        let patterns = self.get_agent_patterns(agent_id);
        let efficiency = self.get_agent_efficiency(agent_id)?;
        let partners = self.get_collaboration_partners(agent_id);

        if patterns.is_empty() && partners.is_empty() {
            return None;
        }

        // Calculate collaboration metrics
        let total_collaborations = patterns
            .iter()
            .map(|p| p.success_count + p.failure_count)
            .sum();
        let success_rate = if total_collaborations > 0 {
            patterns.iter().map(|p| p.success_count).sum::<u32>() as f64
                / total_collaborations as f64
        } else {
            0.0
        };

        let avg_pattern_strength: f64 =
            patterns.iter().map(|p| p.strength).sum::<f64>() / patterns.len().max(1) as f64;

        let pattern_diversity = self.calculate_pattern_diversity(&patterns);

        // Identify most effective patterns
        let effective_patterns: Vec<_> = patterns
            .iter()
            .filter(|p| p.strength >= self.config.min_strength_threshold)
            .map(|p| p.pattern_type.name().to_string())
            .collect();

        let effective_patterns_count = effective_patterns.len();

        // Identify top collaborators
        let top_collaborators: Vec<_> = partners
            .into_iter()
            .filter(|(_, strength)| *strength >= 0.6)
            .collect();

        Some(CollaborationReport {
            agent_id: agent_id.to_string(),
            total_patterns: patterns.len(),
            total_collaborations,
            efficiency,
            success_rate,
            avg_pattern_strength,
            pattern_diversity,
            effective_patterns,
            top_collaborators,
            recommendations: self.generate_collaboration_recommendations(
                efficiency,
                success_rate,
                pattern_diversity,
                effective_patterns_count,
            ),
        })
    }

    /// Generate collaboration recommendations.
    fn generate_collaboration_recommendations(
        &self,
        efficiency: f64,
        success_rate: f64,
        pattern_diversity: f64,
        effective_patterns: usize,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if efficiency < 0.6 {
            recommendations
                .push("Improve collaboration efficiency through better coordination".to_string());
        }

        if success_rate < 0.7 {
            recommendations
                .push("Focus on successful task completion in collaborations".to_string());
        }

        if pattern_diversity < 0.3 {
            recommendations.push("Experiment with different collaboration patterns".to_string());
        }

        if effective_patterns < 2 {
            recommendations
                .push("Develop expertise in multiple collaboration patterns".to_string());
        }

        if efficiency >= 0.8 && success_rate >= 0.8 {
            recommendations
                .push("Share collaboration best practices with other agents".to_string());
        }

        recommendations
    }
}

/// Agent collaboration graph for tracking relationships.
#[derive(Debug, Clone)]
struct AgentCollaborationGraph {
    agent_id: String,
    connections: HashMap<String, ConnectionStats>,
}

impl AgentCollaborationGraph {
    fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            connections: HashMap::new(),
        }
    }

    fn add_connection(
        &mut self,
        other_agent_id: String,
        success: bool,
        efficiency: f64,
        duration: Duration,
    ) {
        let stats = self
            .connections
            .entry(other_agent_id)
            .or_insert_with(ConnectionStats::new);

        stats.add_interaction(success, efficiency, duration);
    }

    fn get_connection_strength(&self, other_agent_id: &str) -> f64 {
        self.connections
            .get(other_agent_id)
            .map(|stats| stats.calculate_strength())
            .unwrap_or(0.0)
    }

    fn calculate_leadership_score(&self) -> f64 {
        // Leadership score based on connection patterns
        // Agents that initiate many successful collaborations are likely leaders

        let mut total_strength = 0.0;
        let mut count = 0;

        for stats in self.connections.values() {
            total_strength += stats.calculate_strength();
            count += 1;
        }

        if count > 0 {
            total_strength / count as f64
        } else {
            0.0
        }
    }

    fn get_connections(&self) -> Vec<(String, f64)> {
        self.connections
            .iter()
            .map(|(id, stats)| (id.clone(), stats.calculate_strength()))
            .collect()
    }
}

/// Statistics for agent connections.
#[derive(Debug, Clone)]
struct ConnectionStats {
    success_count: u32,
    failure_count: u32,
    total_efficiency: f64,
    total_duration: Duration,
    interaction_count: u32,
    last_interaction: SystemTime,
}

impl ConnectionStats {
    fn new() -> Self {
        Self {
            success_count: 0,
            failure_count: 0,
            total_efficiency: 0.0,
            total_duration: Duration::default(),
            interaction_count: 0,
            last_interaction: SystemTime::now(),
        }
    }

    fn add_interaction(&mut self, success: bool, efficiency: f64, duration: Duration) {
        if success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }

        self.total_efficiency += efficiency;
        self.total_duration += duration;
        self.interaction_count += 1;
        self.last_interaction = SystemTime::now();
    }

    fn calculate_strength(&self) -> f64 {
        if self.interaction_count == 0 {
            return 0.0;
        }

        let success_rate = self.success_count as f64 / self.interaction_count as f64;
        let avg_efficiency = self.total_efficiency / self.interaction_count as f64;

        // Strength is weighted combination of success rate and efficiency
        success_rate * 0.7 + avg_efficiency * 0.3
    }
}

/// Collaboration report for an agent.
#[derive(Debug, Clone)]
pub struct CollaborationReport {
    pub agent_id: String,
    pub total_patterns: usize,
    pub total_collaborations: u32,
    pub efficiency: f64,
    pub success_rate: f64,
    pub avg_pattern_strength: f64,
    pub pattern_diversity: f64,
    pub effective_patterns: Vec<String>,
    pub top_collaborators: Vec<(String, f64)>,
    pub recommendations: Vec<String>,
}

impl Default for CollaborationPatternDetector {
    fn default() -> Self {
        Self::new(CollaborationConfig::default())
    }
}
