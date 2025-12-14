//! DTG metrics collection and quality scoring algorithms.

use crate::models::dtg::{DtgMetrics, DtgNode};
use std::collections::HashMap;

/// Collector for DTG metrics with quality scoring.
#[derive(Debug, Clone)]
pub struct DtgMetricsCollector {
    /// Historical metrics for trend analysis.
    historical_metrics: HashMap<String, Vec<DtgMetrics>>,
    
    /// Quality scoring configuration.
    config: QualityScoringConfig,
    
    /// Baseline metrics for comparison.
    baseline_metrics: Option<DtgMetrics>,
}

/// Configuration for quality scoring algorithms.
#[derive(Debug, Clone)]
pub struct QualityScoringConfig {
    /// Weight for performance metrics (0.0 to 1.0).
    pub performance_weight: f64,
    
    /// Weight for reliability metrics (0.0 to 1.0).
    pub reliability_weight: f64,
    
    /// Weight for data quality metrics (0.0 to 1.0).
    pub data_quality_weight: f64,
    
    /// Weight for business value metrics (0.0 to 1.0).
    pub business_value_weight: f64,
    
    /// Minimum acceptable quality score (0.0 to 1.0).
    pub min_acceptable_score: f64,
    
    /// Target quality score (0.0 to 1.0).
    pub target_score: f64,
    
    /// Whether to enable adaptive scoring based on historical data.
    pub adaptive_scoring: bool,
    
    /// Window size for historical analysis.
    pub historical_window: usize,
}

impl Default for QualityScoringConfig {
    fn default() -> Self {
        Self {
            performance_weight: 0.25,
            reliability_weight: 0.25,
            data_quality_weight: 0.25,
            business_value_weight: 0.25,
            min_acceptable_score: 0.7,
            target_score: 0.9,
            adaptive_scoring: true,
            historical_window: 100,
        }
    }
}

impl DtgMetricsCollector {
    /// Create a new metrics collector with default configuration.
    pub fn new() -> Self {
        Self {
            historical_metrics: HashMap::new(),
            config: QualityScoringConfig::default(),
            baseline_metrics: None,
        }
    }
    
    /// Create a new metrics collector with custom configuration.
    pub fn with_config(config: QualityScoringConfig) -> Self {
        Self {
            historical_metrics: HashMap::new(),
            config,
            baseline_metrics: None,
        }
    }
    
    /// Collect metrics from a DTG node.
    pub fn collect_node_metrics(&mut self, node: &DtgNode) -> DtgMetrics {
        let mut metrics = node.metrics.clone();
        
        // Compute comprehensive quality score
        metrics.quality_score = self.compute_comprehensive_quality_score(&metrics);
        
        // Update confidence score based on historical data
        if self.config.adaptive_scoring {
            metrics.confidence_score = self.compute_confidence_score(&metrics, &node.skill_id);
        }
        
        // Store historical metrics for trend analysis
        self.store_historical_metrics(&node.skill_id, metrics.clone());
        
        // Update baseline if needed
        self.update_baseline(&metrics);
        
        metrics
    }
    
    /// Compute a comprehensive quality score from multiple dimensions.
    pub fn compute_comprehensive_quality_score(&self, metrics: &DtgMetrics) -> f64 {
        let performance_score = self.compute_performance_score(metrics);
        let reliability_score = self.compute_reliability_score(metrics);
        let data_quality_score = self.compute_data_quality_score(metrics);
        let business_value_score = self.compute_business_value_score(metrics);
        
        // Weighted average of all dimensions
        let total_weight = self.config.performance_weight
            + self.config.reliability_weight
            + self.config.data_quality_weight
            + self.config.business_value_weight;
        
        if total_weight == 0.0 {
            return 0.0;
        }
        
        (performance_score * self.config.performance_weight
            + reliability_score * self.config.reliability_weight
            + data_quality_score * self.config.data_quality_weight
            + business_value_score * self.config.business_value_weight)
            / total_weight
    }
    
    /// Compute performance score based on resource usage and latency.
    fn compute_performance_score(&self, metrics: &DtgMetrics) -> f64 {
        // Normalize metrics (lower is better for these)
        let cpu_score = self.normalize_metric(metrics.cpu_time_ms as f64, 0.0, 10000.0, true);
        let memory_score = self.normalize_metric(metrics.memory_bytes as f64, 0.0, 1_000_000_000.0, true); // 1GB
        let latency_score = self.normalize_metric(metrics.latency_ms as f64, 0.0, 10000.0, true);
        
        // Throughput (higher is better)
        let throughput_score = self.normalize_metric(metrics.throughput_ops_per_sec, 0.0, 1000.0, false);
        
        // Weighted average of performance metrics
        cpu_score * 0.25 + memory_score * 0.25 + latency_score * 0.25 + throughput_score * 0.25
    }
    
    /// Compute reliability score based on error rate and retries.
    fn compute_reliability_score(&self, metrics: &DtgMetrics) -> f64 {
        // Error rate (lower is better)
        let error_score = 1.0 - metrics.error_rate;
        
        // Retry count (lower is better)
        let retry_score = self.normalize_metric(metrics.retry_count as f64, 0.0, 10.0, true);
        
        // Confidence score
        let confidence_score = metrics.confidence_score;
        
        // Weighted average
        error_score * 0.4 + retry_score * 0.3 + confidence_score * 0.3
    }
    
    /// Compute data quality score based on consistency and schema compliance.
    fn compute_data_quality_score(&self, metrics: &DtgMetrics) -> f64 {
        // Direct scores from metrics
        let consistency_score = metrics.data_consistency_score;
        let schema_score = metrics.schema_compliance_score;
        
        // Weighted average
        consistency_score * 0.6 + schema_score * 0.4
    }
    
    /// Compute business value score.
    fn compute_business_value_score(&self, metrics: &DtgMetrics) -> f64 {
        // Direct score from metrics
        metrics.business_value_score
    }
    
    /// Compute confidence score based on historical performance.
    fn compute_confidence_score(&self, metrics: &DtgMetrics, skill_id: &str) -> f64 {
        if let Some(historical) = self.historical_metrics.get(skill_id) {
            if historical.is_empty() {
                return metrics.confidence_score;
            }
            
            // Compute average historical quality
            let avg_historical_quality: f64 = historical
                .iter()
                .map(|m| m.quality_score)
                .sum::<f64>()
                / historical.len() as f64;
            
            // Compute stability (variance)
            let variance: f64 = historical
                .iter()
                .map(|m| (m.quality_score - avg_historical_quality).powi(2))
                .sum::<f64>()
                / historical.len() as f64;
            let stability = 1.0 / (1.0 + variance.sqrt());
            
            // Current performance relative to historical average
            let performance_ratio = if avg_historical_quality > 0.0 {
                metrics.quality_score / avg_historical_quality
            } else {
                1.0
            };
            
            // Combine factors
            let base_confidence = metrics.confidence_score;
            let historical_confidence = avg_historical_quality * stability;
            
            base_confidence * 0.4 + historical_confidence * 0.4 + performance_ratio.min(1.0) * 0.2
        } else {
            metrics.confidence_score
        }
    }
    
    /// Normalize a metric value to 0.0-1.0 range.
    fn normalize_metric(&self, value: f64, min: f64, max: f64, invert: bool) -> f64 {
        if max <= min {
            return 0.5;
        }
        
        let normalized = (value - min) / (max - min);
        let clamped = normalized.clamp(0.0, 1.0);
        
        if invert {
            1.0 - clamped
        } else {
            clamped
        }
    }
    
    /// Store historical metrics for a skill.
    fn store_historical_metrics(&mut self, skill_id: &str, metrics: DtgMetrics) {
        let entry = self.historical_metrics.entry(skill_id.to_string()).or_default();
        entry.push(metrics);
        
        // Trim to window size
        if entry.len() > self.config.historical_window {
            entry.remove(0);
        }
    }
    
    /// Update baseline metrics.
    fn update_baseline(&mut self, metrics: &DtgMetrics) {
        if self.baseline_metrics.is_none() {
            self.baseline_metrics = Some(metrics.clone());
        } else if let Some(baseline) = &mut self.baseline_metrics {
            // Moving average update
            let alpha = 0.1; // Learning rate
            baseline.cpu_time_ms = ((1.0 - alpha) * baseline.cpu_time_ms as f64 + alpha * metrics.cpu_time_ms as f64) as u64;
            baseline.memory_bytes = ((1.0 - alpha) * baseline.memory_bytes as f64 + alpha * metrics.memory_bytes as f64) as u64;
            baseline.latency_ms = ((1.0 - alpha) * baseline.latency_ms as f64 + alpha * metrics.latency_ms as f64) as u64;
            baseline.throughput_ops_per_sec = (1.0 - alpha) * baseline.throughput_ops_per_sec + alpha * metrics.throughput_ops_per_sec;
            baseline.quality_score = (1.0 - alpha) * baseline.quality_score + alpha * metrics.quality_score;
        }
    }
    
    /// Get historical metrics for a skill.
    pub fn get_historical_metrics(&self, skill_id: &str) -> Option<&Vec<DtgMetrics>> {
        self.historical_metrics.get(skill_id)
    }
    
    /// Get baseline metrics.
    pub fn get_baseline_metrics(&self) -> Option<&DtgMetrics> {
        self.baseline_metrics.as_ref()
    }
    
    /// Get quality scoring configuration.
    pub fn config(&self) -> &QualityScoringConfig {
        &self.config
    }
    
    /// Update quality scoring configuration.
    pub fn update_config(&mut self, config: QualityScoringConfig) {
        self.config = config;
    }
    
    /// Analyze node performance and provide recommendations.
    pub fn analyze_node_performance(&self, node: &DtgNode) -> PerformanceAnalysis {
        let metrics = &node.metrics;
        let quality_score = self.compute_comprehensive_quality_score(metrics);
        
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        
        // Check performance issues
        if metrics.cpu_time_ms > 5000 {
            issues.push("High CPU usage".to_string());
            recommendations.push("Consider optimizing algorithm or using more efficient data structures".to_string());
        }
        
        if metrics.memory_bytes > 100_000_000 {
            issues.push("High memory usage".to_string());
            recommendations.push("Consider implementing memory pooling or streaming processing".to_string());
        }
        
        if metrics.latency_ms > 1000 {
            issues.push("High latency".to_string());
            recommendations.push("Consider parallelizing operations or using caching".to_string());
        }
        
        // Check reliability issues
        if metrics.error_rate > 0.1 {
            issues.push("High error rate".to_string());
            recommendations.push("Improve error handling and add retry logic with exponential backoff".to_string());
        }
        
        if metrics.retry_count > 3 {
            issues.push("Excessive retries".to_string());
            recommendations.push("Investigate root cause of failures and improve stability".to_string());
        }
        
        // Check data quality issues
        if metrics.data_consistency_score < 0.8 {
            issues.push("Low data consistency".to_string());
            recommendations.push("Implement data validation and consistency checks".to_string());
        }
        
        if metrics.schema_compliance_score < 0.8 {
            issues.push("Low schema compliance".to_string());
            recommendations.push("Enforce schema validation and versioning".to_string());
        }
        
        // Check business value
        if metrics.business_value_score < 0.7 {
            issues.push("Low business value".to_string());
            recommendations.push("Re-evaluate transformation goals and alignment with business objectives".to_string());
        }
        
        // Overall quality check
        let status = if quality_score >= self.config.target_score {
            PerformanceStatus::Excellent
        } else if quality_score >= self.config.min_acceptable_score {
            PerformanceStatus::Acceptable
        } else {
            PerformanceStatus::Poor
        };
        
        PerformanceAnalysis {
            node_id: node.id,
            skill_id: node.skill_id.clone(),
            quality_score,
            status,
            issues,
            recommendations,
            metrics: metrics.clone(),
        }
    }
    
    /// Generate a quality report for multiple nodes.
    pub fn generate_quality_report(&self, nodes: &[&DtgNode]) -> QualityReport {
        let analyses: Vec<PerformanceAnalysis> = nodes
            .iter()
            .map(|node| self.analyze_node_performance(node))
            .collect();
        
        let avg_quality: f64 = analyses.iter().map(|a| a.quality_score).sum::<f64>() / analyses.len() as f64;
        
        let mut issue_counts = HashMap::new();
        for analysis in &analyses {
            for issue in &analysis.issues {
                *issue_counts.entry(issue.clone()).or_insert(0) += 1;
            }
        }
        
        let common_issues: Vec<String> = issue_counts
            .into_iter()
            .filter(|(_, count)| *count > analyses.len() / 3) // Issues affecting >1/3 of nodes
            .map(|(issue, _)| issue)
            .collect();
        
        QualityReport {
            timestamp: chrono::Utc::now(),
            total_nodes: analyses.len(),
            average_quality: avg_quality,
            analyses,
            common_issues,
            overall_status: if avg_quality >= self.config.target_score {
                PerformanceStatus::Excellent
            } else if avg_quality >= self.config.min_acceptable_score {
                PerformanceStatus::Acceptable
            } else {
                PerformanceStatus::Poor
            },
        }
    }
}

/// Performance analysis for a DTG node.
#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    /// Node ID.
    pub node_id: uuid::Uuid,
    
    /// Skill ID.
    pub skill_id: String,
    
    /// Comprehensive quality score.
    pub quality_score: f64,
    
    /// Performance status.
    pub status: PerformanceStatus,
    
    /// Identified issues.
    pub issues: Vec<String>,
    
    /// Recommendations for improvement.
    pub recommendations: Vec<String>,
    
    /// Detailed metrics.
    pub metrics: DtgMetrics,
}

/// Performance status.
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceStatus {
    /// Excellent performance.
    Excellent,
    
    /// Acceptable performance.
    Acceptable,
    
    /// Poor performance needs improvement.
    Poor,
}

/// Quality report for multiple DTG nodes.
#[derive(Debug, Clone)]
pub struct QualityReport {
    /// Report timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Total nodes analyzed.
    pub total_nodes: usize,
    
    /// Average quality score.
    pub average_quality: f64,
    
    /// Individual node analyses.
    pub analyses: Vec<PerformanceAnalysis>,
    
    /// Common issues across nodes.
    pub common_issues: Vec<String>,
    
    /// Overall status.
    pub overall_status: PerformanceStatus,
}

impl Default for DtgMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::dtg::DtgNode;
    
    #[test]
    fn test_metrics_collector_creation() {
        let collector = DtgMetricsCollector::new();
        assert_eq!(collector.config.performance_weight, 0.25);
        assert_eq!(collector.config.reliability_weight, 0.25);
        assert_eq!(collector.config.min_acceptable_score, 0.7);
    }
    
    #[test]
    fn test_quality_score_computation() {
        let collector = DtgMetricsCollector::new();
        
        let metrics = DtgMetrics {
            cpu_time_ms: 100,
            memory_bytes: 1024 * 1024, // 1MB
            network_bytes: 1024,
            disk_bytes: 2048,
            retry_count: 0,
            quality_score: 0.9,
            confidence_score: 0.95,
            latency_ms: 50,
            throughput_ops_per_sec: 100.0,
            error_rate: 0.01,
            data_consistency_score: 0.98,
            schema_compliance_score: 0.95,
            business_value_score: 0.9,
            collected_at: chrono::Utc::now(),
        };
        
        let score = collector.compute_comprehensive_quality_score(&metrics);
        assert!(score >= 0.0 && score <= 1.0, "Score should be between 0 and 1");
        assert!(score > 0.7, "Good metrics should yield high score");
    }
    
    #[test]
    fn test_performance_analysis() {
        let collector = DtgMetricsCollector::new();
        
        let node = DtgNode::new("test_skill".to_string(), "test_agent".to_string());
        let analysis = collector.analyze_node_performance(&node);
        
        assert_eq!(analysis.skill_id, "test_skill");
        assert!(analysis.quality_score >= 0.0 && analysis.quality_score <= 1.0);
    }
    
    #[test]
    fn test_historical_metrics_storage() {
        let mut collector = DtgMetricsCollector::new();
        
        let node = DtgNode::new("test_skill".to_string(), "test_agent".to_string());
        let metrics = collector.collect_node_metrics(&node);
        
        assert!(collector.get_historical_metrics("test_skill").is_some());
        let historical = collector.get_historical_metrics("test_skill").unwrap();
        assert_eq!(historical.len(), 1);
        assert_eq!(historical[0].quality_score, metrics.quality_score);
    }
    
    #[test]
    fn test_baseline_update() {
        let mut collector = DtgMetricsCollector::new();
        
        let node = DtgNode::new("test_skill".to_string(), "test_agent".to_string());
        let _ = collector.collect_node_metrics(&node);
        
        assert!(collector.get_baseline_metrics().is_some());
    }
}