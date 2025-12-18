//! DTG visualization and analysis tools.
//!
//! Provides graph visualization, export capabilities, and analysis tools
//! for Data Transformation Graphs.

#[allow(unused_imports)]
use crate::models::dtg::{DataTransformationGraph, DtgGraphStatus, DtgNode, DtgNodeStatus};
use std::collections::HashMap;
use uuid::Uuid;

/// DTG visualization formats.
#[derive(Debug, Clone, PartialEq)]
pub enum VisualizationFormat {
    /// Graphviz DOT format for graph visualization.
    Dot,

    /// Mermaid format for documentation and diagrams.
    Mermaid,

    /// JSON format for programmatic analysis.
    Json,

    /// CSV format for tabular analysis.
    Csv,

    /// HTML format for web visualization.
    Html,
}

/// Graph analysis metrics.
#[derive(Debug, Clone)]
pub struct GraphAnalysis {
    /// Total number of nodes.
    pub node_count: usize,

    /// Total number of edges.
    pub edge_count: usize,

    /// Graph depth (longest path).
    pub max_depth: usize,

    /// Graph width (maximum parallel nodes).
    pub max_width: usize,

    /// Average node quality score.
    pub avg_quality_score: f64,

    /// Total execution time in milliseconds.
    pub total_latency_ms: u64,

    /// Total cost.
    pub total_cost: f64,

    /// Node status distribution.
    pub status_distribution: HashMap<DtgNodeStatus, usize>,

    /// Performance bottlenecks (nodes with longest execution time).
    pub bottlenecks: Vec<(Uuid, String, u64)>,

    /// Quality issues (nodes with lowest quality scores).
    pub quality_issues: Vec<(Uuid, String, f64)>,

    /// Cost issues (nodes with highest cost).
    pub cost_issues: Vec<(Uuid, String, f64)>,
}

/// DTG visualization and analysis engine.
#[derive(Debug)]
pub struct DtgVisualizationEngine {
    /// Color scheme for visualization.
    color_scheme: HashMap<DtgNodeStatus, String>,

    /// Node shape mapping.
    node_shapes: HashMap<String, String>,
}

impl DtgVisualizationEngine {
    /// Create a new visualization engine.
    pub fn new() -> Self {
        let mut color_scheme = HashMap::new();
        color_scheme.insert(DtgNodeStatus::Pending, "#FF6B6B".to_string()); // Red
        color_scheme.insert(DtgNodeStatus::Executing, "#4ECDC4".to_string()); // Teal
        color_scheme.insert(DtgNodeStatus::Completed, "#45B7D1".to_string()); // Blue
        color_scheme.insert(DtgNodeStatus::Failed, "#96CEB4".to_string()); // Green (failed)
        // Note: Skipped variant doesn't exist in DtgNodeStatus, using Cancelled instead
        color_scheme.insert(DtgNodeStatus::Cancelled, "#FFEAA7".to_string()); // Yellow

        let mut node_shapes = HashMap::new();
        node_shapes.insert("data_source".to_string(), "cylinder".to_string());
        node_shapes.insert("transformation".to_string(), "box".to_string());
        node_shapes.insert("analysis".to_string(), "ellipse".to_string());
        node_shapes.insert("export".to_string(), "parallelogram".to_string());
        node_shapes.insert("validation".to_string(), "diamond".to_string());

        Self {
            color_scheme,
            node_shapes,
        }
    }

    /// Export DTG to Graphviz DOT format.
    pub fn export_to_dot(&self, dtg: &DataTransformationGraph) -> String {
        let mut dot = String::new();

        // Graph header
        dot.push_str(&format!("digraph \"{}\" {{\n", dtg.name));
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [fontname=\"Helvetica\", fontsize=10];\n");
        dot.push_str("  edge [fontname=\"Helvetica\", fontsize=8];\n\n");

        // Add nodes
        for (node_id, node) in &dtg.nodes {
            let default_color = "#CCCCCC".to_string();
            let color = self
                .color_scheme
                .get(&node.status)
                .unwrap_or(&default_color);
            let shape = self.get_node_shape(node);
            let label = self.format_node_label(node);

            dot.push_str(&format!(
                "  \"{node_id}\" [label=\"{label}\", shape={shape}, style=filled, fillcolor=\"{color}\"];\n"
            ));
        }

        // Add edges
        for edge in &dtg.edges {
            dot.push_str(&format!("  \"{}\" -> \"{}\";\n", edge.source, edge.target));
        }

        // Add subgraphs for parallel execution groups
        self.add_execution_groups(dtg, &mut dot);

        dot.push_str("}\n");
        dot
    }

    /// Export DTG to Mermaid format.
    pub fn export_to_mermaid(&self, dtg: &DataTransformationGraph) -> String {
        let mut mermaid = String::new();

        mermaid.push_str("graph LR\n");

        // Add nodes
        for (node_id, node) in &dtg.nodes {
            let default_color = "#CCCCCC".to_string();
            let color = self
                .color_scheme
                .get(&node.status)
                .unwrap_or(&default_color);
            let shape = self.get_mermaid_shape(node);
            let label = self.format_node_label(node);

            mermaid.push_str(&format!("  {node_id}(\"{label}\")\n"));
            mermaid.push_str(&format!("  style {node_id} fill:{color}\n"));
            if shape != "default" {
                mermaid.push_str(&format!("  style {node_id} shape:{shape}\n"));
            }
        }

        // Add edges
        for edge in &dtg.edges {
            mermaid.push_str(&format!("  {} --> {}\n", edge.source, edge.target));
        }

        mermaid
    }

    /// Export DTG to JSON format.
    pub fn export_to_json(&self, dtg: &DataTransformationGraph) -> String {
        serde_json::to_string_pretty(dtg).unwrap_or_else(|_| "{}".to_string())
    }

    /// Export DTG metrics to CSV format.
    pub fn export_metrics_to_csv(&self, dtg: &DataTransformationGraph) -> String {
        let mut csv = String::new();

        // CSV header
        csv.push_str(
            "node_id,name,status,quality_score,latency_ms,cpu_time_ms,data_type,location\n",
        );

        // CSV rows
        for (node_id, node) in &dtg.nodes {
            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{:?}\",{},{},{},\"{}\",\"{}\"\n",
                node_id,
                &node.skill_id,
                node.status,
                node.metrics.quality_score,
                node.metrics.execution_time_ms,
                node.metrics.cpu_time_ms as f64,
                node.inputs
                    .first()
                    .map(|r| r.data_type.as_str())
                    .unwrap_or("unknown"),
                node.outputs
                    .first()
                    .map(|r| r.id.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            ));
        }

        csv
    }

    /// Generate HTML visualization.
    pub fn export_to_html(&self, dtg: &DataTransformationGraph) -> String {
        let dot = self.export_to_dot(dtg);
        let analysis = self.analyze_graph(dtg);

        format!(
            r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>DTG Visualization: {}</title>
    <script src="https://cdn.jsdelivr.net/npm/@hpcc-js/wasm@1.12.5/dist/index.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/d3@7.8.5/dist/d3.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/d3-graphviz@3.1.0/build/d3-graphviz.min.js"></script>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .container {{ display: flex; flex-direction: column; gap: 20px; }}
        .graph-container {{ border: 1px solid #ddd; padding: 10px; }}
        .metrics {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 10px; }}
        .metric-card {{ background: #f5f5f5; padding: 10px; border-radius: 5px; }}
        .metric-value {{ font-size: 24px; font-weight: bold; }}
        .metric-label {{ font-size: 12px; color: #666; }}
        .issues {{ margin-top: 20px; }}
        .issue-list {{ list-style: none; padding: 0; }}
        .issue-item {{ padding: 5px; margin: 2px 0; background: #fff3cd; border: 1px solid #ffeaa7; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>DTG Visualization: {}</h1>
        <p>{}</p>
        
        <div class="metrics">
            <div class="metric-card">
                <div class="metric-value">{}</div>
                <div class="metric-label">Nodes</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{}</div>
                <div class="metric-label">Edges</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{:.2}</div>
                <div class="metric-label">Avg Quality</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">${{{:.2}}}</div>
                <div class="metric-label">Total Cost</div>
            </div>
        </div>
        
        <div class="graph-container">
            <div id="graph"></div>
        </div>
        
        <div class="issues">
            <h3>Performance Analysis</h3>
            <ul class="issue-list">
                <li class="issue-item"><strong>Max Depth:</strong> {} levels</li>
                <li class="issue-item"><strong>Max Width:</strong> {} parallel nodes</li>
                <li class="issue-item"><strong>Total Execution Time:</strong> {}ms</li>
            </ul>
            
            <h3>Bottlenecks (Top 3)</h3>
            <ul class="issue-list">
                {}
            </ul>
            
            <h3>Quality Issues (Top 3)</h3>
            <ul class="issue-list">
                {}
            </ul>
        </div>
    </div>
    
    <script>
        const dot = "{{{}}}";
        
        d3.select("#graph").graphviz()
            .width(window.innerWidth - 100)
            .height(600)
            .fit(true)
            .renderDot(dot);
    </script>
</body>
</html>"##,
            dtg.name,
            dtg.name,
            &dtg.name, // Using name as description since description field doesn't exist
            analysis.node_count,
            analysis.edge_count,
            analysis.avg_quality_score,
            analysis.total_cost,
            analysis.max_depth,
            analysis.max_width,
            analysis.total_latency_ms,
            self.format_issues(&analysis.bottlenecks, "ms"),
            self.format_issues(&analysis.quality_issues, "quality"),
            dot
        )
    }

    /// Analyze DTG graph structure and performance.
    pub fn analyze_graph(&self, dtg: &DataTransformationGraph) -> GraphAnalysis {
        let node_count = dtg.nodes.len();
        let edge_count = dtg.edges.len();

        // Calculate metrics
        let mut total_quality = 0.0;
        let mut total_execution_time = 0;
        let mut total_cost = 0.0;
        let mut status_distribution = HashMap::new();

        let mut bottlenecks = Vec::new();
        let mut quality_issues = Vec::new();
        let mut cost_issues = Vec::new();

        for (node_id, node) in &dtg.nodes {
            // Update totals
            total_quality += node.metrics.quality_score;
            total_execution_time += node.metrics.execution_time_ms;
            total_cost += node.metrics.cpu_time_ms as f64;

            // Update status distribution
            *status_distribution.entry(node.status.clone()).or_insert(0) += 1;

            // Collect issues
            if node.metrics.execution_time_ms > 1000 {
                bottlenecks.push((
                    *node_id,
                    node.skill_id.clone(),
                    node.metrics.execution_time_ms,
                ));
            }

            if node.metrics.quality_score < 0.7 {
                quality_issues.push((*node_id, node.skill_id.clone(), node.metrics.quality_score));
            }

            if node.metrics.cpu_time_ms as f64 > 1000.0 {
                // Note: DtgMetrics doesn't have cost field, using cpu_time_ms as proxy
                cost_issues.push((
                    *node_id,
                    node.skill_id.clone(),
                    node.metrics.cpu_time_ms as f64,
                ));
            }
        }

        // Sort issues
        bottlenecks.sort_by(|a, b| b.2.cmp(&a.2));
        quality_issues.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
        cost_issues.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        // Calculate graph depth and width
        let (max_depth, max_width) = self.calculate_graph_metrics(dtg);

        GraphAnalysis {
            node_count,
            edge_count,
            max_depth,
            max_width,
            avg_quality_score: if node_count > 0 {
                total_quality / node_count as f64
            } else {
                0.0
            },
            total_latency_ms: total_execution_time,
            total_cost,
            status_distribution,
            bottlenecks: bottlenecks.into_iter().take(3).collect(),
            quality_issues: quality_issues.into_iter().take(3).collect(),
            cost_issues: cost_issues.into_iter().take(3).collect(),
        }
    }

    /// Calculate graph depth (longest path) and width (maximum parallel nodes).
    fn calculate_graph_metrics(&self, dtg: &DataTransformationGraph) -> (usize, usize) {
        // Simple BFS for depth calculation
        let mut max_depth = 0;
        let mut max_width = 0;

        // Find source nodes (nodes with no incoming edges)
        let mut incoming_counts: HashMap<Uuid, usize> = HashMap::new();
        for edge in &dtg.edges {
            *incoming_counts.entry(edge.target).or_insert(0) += 1;
        }

        let source_nodes: Vec<Uuid> = dtg
            .nodes
            .keys()
            .filter(|&node_id| incoming_counts.get(node_id).unwrap_or(&0) == &0)
            .cloned()
            .collect();

        // BFS from each source
        for source in source_nodes {
            let mut queue = vec![(source, 1)];
            let mut visited = HashMap::new();

            while let Some((node_id, depth)) = queue.pop() {
                visited.insert(node_id, depth);
                max_depth = max_depth.max(depth);

                // Find children
                let children: Vec<Uuid> = dtg
                    .edges
                    .iter()
                    .filter(|edge| edge.source == node_id)
                    .map(|edge| edge.target)
                    .collect();

                max_width = max_width.max(children.len());

                for child in children {
                    if !visited.contains_key(&child) {
                        queue.push((child, depth + 1));
                    }
                }
            }
        }

        (max_depth, max_width)
    }

    /// Get node shape based on metadata.
    fn get_node_shape(&self, node: &DtgNode) -> String {
        if let Some(task_type) = node.metadata.get("task_type")
            && let Some(task_type_str) = task_type.as_str()
        {
            return self
                .node_shapes
                .get(task_type_str)
                .cloned()
                .unwrap_or_else(|| "box".to_string());
        }
        "box".to_string()
    }

    /// Get Mermaid shape.
    fn get_mermaid_shape(&self, node: &DtgNode) -> String {
        match self.get_node_shape(node).as_str() {
            "cylinder" => "cylinder",
            "ellipse" => "ellipse",
            "parallelogram" => "parallelogram",
            "diamond" => "diamond",
            _ => "default",
        }
        .to_string()
    }

    /// Format node label for visualization.
    fn format_node_label(&self, node: &DtgNode) -> String {
        format!(
            "{}|Quality: {:.2}|Time: {}ms|Cost: ${:.2}",
            &node.skill_id,
            node.metrics.quality_score,
            node.metrics.execution_time_ms,
            node.metrics.cpu_time_ms as f64
        )
    }

    /// Add execution groups to DOT output.
    fn add_execution_groups(&self, dtg: &DataTransformationGraph, dot: &mut String) {
        // Group nodes by status for visual clustering
        let mut groups: HashMap<DtgNodeStatus, Vec<Uuid>> = HashMap::new();

        for (node_id, node) in &dtg.nodes {
            groups
                .entry(node.status.clone())
                .or_default()
                .push(*node_id);
        }

        for (status, nodes) in groups {
            if nodes.len() > 1 {
                dot.push_str(&format!("  subgraph cluster_{status:?} {{\n"));
                dot.push_str(&format!("    label = \"{status:?} Nodes\";\n"));
                dot.push_str("    style = filled;\n");
                dot.push_str("    color = lightgrey;\n");

                for node_id in nodes {
                    dot.push_str(&format!("    \"{node_id}\";\n"));
                }

                dot.push_str("  }\n");
            }
        }
    }

    /// Format issues for HTML display.
    fn format_issues(
        &self,
        issues: &[(Uuid, String, impl std::fmt::Display)],
        unit: &str,
    ) -> String {
        issues
            .iter()
            .map(|(id, name, value)| {
                format!("<li class=\"issue-item\"><strong>{name}:</strong> {value} ({unit})</li>")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Generate performance report.
    pub fn generate_performance_report(&self, dtg: &DataTransformationGraph) -> String {
        let analysis = self.analyze_graph(dtg);

        format!(
            r#"DTG Performance Report
=======================

Graph: {}
Description: {}

Summary Metrics:
---------------
- Total Nodes: {}
- Total Edges: {}
- Graph Depth: {} levels
- Graph Width: {} parallel nodes
- Average Quality Score: {:.2}
- Total Execution Time: {}ms
- Total Cost: ${:.2}

Status Distribution:
-------------------"#,
            dtg.name,
            &dtg.name, // Using name as description since description field doesn't exist
            analysis.node_count,
            analysis.edge_count,
            analysis.max_depth,
            analysis.max_width,
            analysis.avg_quality_score,
            analysis.total_latency_ms,
            analysis.total_cost
        )
    }
}

impl Default for DtgVisualizationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::dtg::{DtgDataRef, DtgEdge, DtgMetrics};

    #[test]
    fn test_visualization_engine_creation() {
        let engine = DtgVisualizationEngine::new();
        assert!(!engine.color_scheme.is_empty());
        assert!(!engine.node_shapes.is_empty());
    }

    #[test]
    fn test_dot_export() {
        let dtg = create_test_dtg();
        let engine = DtgVisualizationEngine::new();

        let dot = engine.export_to_dot(&dtg);

        assert!(dot.contains("digraph"));
        assert!(dot.contains(&dtg.name));
        assert!(dot.contains("node"));
        assert!(dot.contains("edge"));
    }

    #[test]
    fn test_mermaid_export() {
        let dtg = create_test_dtg();
        let engine = DtgVisualizationEngine::new();

        let mermaid = engine.export_to_mermaid(&dtg);

        assert!(mermaid.contains("graph LR"));
        assert!(mermaid.contains(&dtg.nodes.keys().next().unwrap().to_string()));
    }

    #[test]
    fn test_json_export() {
        let dtg = create_test_dtg();
        let engine = DtgVisualizationEngine::new();

        let json = engine.export_to_json(&dtg);

        assert!(json.contains(&dtg.name));
        assert!(json.contains("\"nodes\""));
        assert!(json.contains("\"edges\""));
    }

    #[test]
    fn test_csv_export() {
        let dtg = create_test_dtg();
        let engine = DtgVisualizationEngine::new();

        let csv = engine.export_metrics_to_csv(&dtg);

        assert!(csv.contains("node_id,name,status"));
        assert!(csv.contains(&dtg.nodes.keys().next().unwrap().to_string()));
    }

    #[test]
    fn test_graph_analysis() {
        let dtg = create_test_dtg();
        let engine = DtgVisualizationEngine::new();

        let analysis = engine.analyze_graph(&dtg);

        assert_eq!(analysis.node_count, 3);
        assert_eq!(analysis.edge_count, 2);
        assert!(analysis.avg_quality_score >= 0.0);
        assert!(analysis.avg_quality_score <= 1.0);
    }

    #[test]
    fn test_performance_report() {
        let dtg = create_test_dtg();
        let engine = DtgVisualizationEngine::new();

        let report = engine.generate_performance_report(&dtg);

        assert!(report.contains("DTG Performance Report"));
        assert!(report.contains(&dtg.name));
        assert!(report.contains("Summary Metrics"));
    }

    fn create_test_dtg() -> DataTransformationGraph {
        let mut dtg = DataTransformationGraph {
            id: Uuid::new_v4(),
            name: "Test DTG".to_string(),
            root_nodes: vec![],
            nodes: HashMap::new(),
            edges: vec![],
            graph_inputs: vec![],
            graph_outputs: vec![],
            metadata: HashMap::new(),
            started_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
            status: DtgGraphStatus::Completed,
            tags: vec![],
        };

        // Add test nodes
        let node1 = DtgNode {
            id: Uuid::new_v4(),
            skill_id: "data_source".to_string(),
            agent_id: "agent_1".to_string(),
            inputs: vec![],
            outputs: vec![DtgDataRef {
                id: Uuid::new_v4(),
                data_type: "csv".to_string(),
                schema: Some("id,name,value".to_string()),
                size_bytes: Some(1000),
                content_hash: Some("abc123".to_string()),
                storage_ref: Some("file://test.csv".to_string()),
            }],
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "task_type".to_string(),
                    serde_json::Value::String("data_source".to_string()),
                );
                metadata
            },
            started_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
            status: DtgNodeStatus::Completed,
            error: None,
            metrics: DtgMetrics {
                cpu_time_ms: 100,
                memory_bytes: 1024,
                network_bytes: 0,
                disk_bytes: 0,
                retry_count: 0,
                quality_score: 0.9,
                confidence_score: 0.8,
                execution_time_ms: 100,
                throughput_ops_per_sec: 10.0,
                error_rate: 0.0,
                data_consistency_score: 0.9,
                schema_compliance_score: 0.9,
                business_value_score: 0.8,
                cost: 0.0,
                collected_at: chrono::Utc::now(),
            },
        };

        let node2 = DtgNode {
            id: Uuid::new_v4(),
            skill_id: "transformation".to_string(),
            agent_id: "agent_2".to_string(),
            inputs: vec![DtgDataRef {
                id: Uuid::new_v4(),
                data_type: "csv".to_string(),
                schema: Some("id,name,value".to_string()),
                size_bytes: Some(1000),
                content_hash: Some("abc123".to_string()),
                storage_ref: Some("memory".to_string()),
            }],
            outputs: vec![DtgDataRef {
                id: Uuid::new_v4(),
                data_type: "json".to_string(),
                schema: Some("{\"type\":\"object\"}".to_string()),
                size_bytes: Some(500),
                content_hash: Some("def456".to_string()),
                storage_ref: Some("memory".to_string()),
            }],
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "task_type".to_string(),
                    serde_json::Value::String("transformation".to_string()),
                );
                metadata
            },
            started_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
            status: DtgNodeStatus::Completed,
            error: None,
            metrics: DtgMetrics {
                cpu_time_ms: 200,
                memory_bytes: 2048,
                network_bytes: 0,
                disk_bytes: 0,
                retry_count: 0,
                quality_score: 0.8,
                confidence_score: 0.7,
                execution_time_ms: 200,
                throughput_ops_per_sec: 5.0,
                error_rate: 0.0,
                data_consistency_score: 0.8,
                schema_compliance_score: 0.8,
                business_value_score: 0.7,
                cost: 0.0,
                collected_at: chrono::Utc::now(),
            },
        };

        let node3 = DtgNode {
            id: Uuid::new_v4(),
            skill_id: "export".to_string(),
            agent_id: "agent_3".to_string(),
            inputs: vec![DtgDataRef {
                id: Uuid::new_v4(),
                data_type: "json".to_string(),
                schema: Some("{\"type\":\"object\"}".to_string()),
                size_bytes: Some(500),
                content_hash: Some("def456".to_string()),
                storage_ref: Some("memory".to_string()),
            }],
            outputs: vec![DtgDataRef {
                id: Uuid::new_v4(),
                data_type: "database".to_string(),
                schema: Some("table_schema".to_string()),
                size_bytes: Some(800),
                content_hash: Some("ghi789".to_string()),
                storage_ref: Some("postgresql://localhost/test".to_string()),
            }],
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "task_type".to_string(),
                    serde_json::Value::String("export".to_string()),
                );
                metadata
            },
            started_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
            status: DtgNodeStatus::Completed,
            error: None,
            metrics: DtgMetrics {
                cpu_time_ms: 150,
                memory_bytes: 1024,
                network_bytes: 0,
                disk_bytes: 0,
                retry_count: 0,
                quality_score: 0.95,
                confidence_score: 0.9,
                execution_time_ms: 150,
                throughput_ops_per_sec: 8.0,
                error_rate: 0.0,
                data_consistency_score: 0.95,
                schema_compliance_score: 0.95,
                business_value_score: 0.9,
                cost: 0.0,
                collected_at: chrono::Utc::now(),
            },
        };

        dtg.nodes.insert(node1.id, node1.clone());
        dtg.nodes.insert(node2.id, node2.clone());
        dtg.nodes.insert(node3.id, node3.clone());
        dtg.root_nodes.push(node1.id);

        // Add edges
        dtg.edges.push(DtgEdge {
            source: node1.id,
            target: node2.id,
            data_ref: node1.outputs[0].id,
            edge_type: "data_flow".to_string(),
            metadata: HashMap::new(),
        });

        dtg.edges.push(DtgEdge {
            source: node2.id,
            target: node3.id,
            data_ref: node2.outputs[0].id,
            edge_type: "data_flow".to_string(),
            metadata: HashMap::new(),
        });

        dtg
    }
}
