//! Example demonstrating DTG visualization and analysis tools.
//!
//! This example shows how to:
//! 1. Create a Data Transformation Graph
//! 2. Export it to various formats (DOT, Mermaid, JSON, CSV, HTML)
//! 3. Analyze graph performance and identify issues
//! 4. Generate visualizations and reports

use constellation_core::dtg::visualization::{DtgVisualizationEngine, VisualizationFormat};
use constellation_core::models::dtg::{
    DataTransformationGraph, DtgDataRef, DtgEdge, DtgMetrics, DtgNode, DtgNodeStatus,
};
use std::collections::HashMap;
use uuid::Uuid;

fn main() {
    println!("=== DTG Visualization Example ===\n");

    // Step 1: Create a sample DTG
    println!("1. Creating Sample DTG");
    println!("------------------------");

    let dtg = create_sample_dtg();

    println!("✓ Created DTG: {}", dtg.name);
    println!("  - Nodes: {}", dtg.nodes.len());
    println!("  - Edges: {}", dtg.edges.len());
    println!("  - Status: {:?}\n", dtg.status);

    // Step 2: Create visualization engine
    println!("2. Creating Visualization Engine");
    println!("--------------------------------");

    let viz_engine = DtgVisualizationEngine::new();

    println!("✓ Created visualization engine\n");

    // Step 3: Export to various formats
    println!("3. Exporting DTG to Different Formats");
    println!("--------------------------------------");

    // Export to Graphviz DOT
    let dot = viz_engine.export_to_dot(&dtg);
    println!("✓ Graphviz DOT format:");
    println!("  - Size: {} characters", dot.len());
    println!("  - First 100 chars: {}...\n", &dot[..100.min(dot.len())]);

    // Export to Mermaid
    let mermaid = viz_engine.export_to_mermaid(&dtg);
    println!("✓ Mermaid format:");
    println!("  - Size: {} characters", mermaid.len());
    println!("  - First line: {}\n", mermaid.lines().next().unwrap_or(""));

    // Export to JSON
    let json = viz_engine.export_to_json(&dtg);
    println!("✓ JSON format:");
    println!("  - Size: {} characters", json.len());
    println!("  - Contains nodes: {}", json.contains("\"nodes\""));
    println!("  - Contains edges: {}\n", json.contains("\"edges\""));

    // Export metrics to CSV
    let csv = viz_engine.export_metrics_to_csv(&dtg);
    println!("✓ CSV format:");
    println!("  - Size: {} characters", csv.len());
    println!("  - Lines: {}", csv.lines().count());
    println!("  - Header: {}\n", csv.lines().next().unwrap_or(""));

    // Step 4: Analyze graph
    println!("4. Analyzing DTG Graph");
    println!("----------------------");

    let analysis = viz_engine.analyze_graph(&dtg);

    println!("✓ Graph Analysis Results:");
    println!("  - Node Count: {}", analysis.node_count);
    println!("  - Edge Count: {}", analysis.edge_count);
    println!("  - Max Depth: {} levels", analysis.max_depth);
    println!("  - Max Width: {} parallel nodes", analysis.max_width);
    println!("  - Avg Quality Score: {:.2}", analysis.avg_quality_score);
    println!("  - Total Execution Time: {}ms", analysis.total_latency_ms);
    println!("  - Total Cost: ${:.2}\n", analysis.total_cost);

    // Step 5: Show status distribution
    println!("5. Node Status Distribution");
    println!("---------------------------");

    for (status, count) in &analysis.status_distribution {
        println!("  - {status:?}: {count} nodes");
    }
    println!();

    // Step 6: Identify issues
    println!("6. Performance Issues Identified");
    println!("--------------------------------");

    if !analysis.bottlenecks.is_empty() {
        println!("✓ Bottlenecks (slowest nodes):");
        for (node_id, name, time) in &analysis.bottlenecks {
            println!("  - {name}: {time}ms");
        }
    } else {
        println!("✓ No significant bottlenecks found");
    }

    if !analysis.quality_issues.is_empty() {
        println!("\n✓ Quality Issues (lowest scores):");
        for (node_id, name, score) in &analysis.quality_issues {
            println!("  - {name}: {score:.2}");
        }
    } else {
        println!("\n✓ No quality issues found");
    }

    if !analysis.cost_issues.is_empty() {
        println!("\n✓ Cost Issues (highest costs):");
        for (node_id, name, cost) in &analysis.cost_issues {
            println!("  - {name}: ${cost:.2}");
        }
    } else {
        println!("\n✓ No cost issues found");
    }
    println!();

    // Step 7: Generate HTML visualization
    println!("7. Generating HTML Visualization");
    println!("--------------------------------");

    let html = viz_engine.export_to_html(&dtg);
    println!("✓ HTML visualization generated");
    println!("  - Size: {} characters", html.len());
    println!("  - Contains D3.js: {}", html.contains("d3-graphviz"));
    println!("  - Contains Graphviz: {}", html.contains("graphviz"));

    // Save HTML to file
    let html_path = "dtg_visualization.html";
    std::fs::write(html_path, html).expect("Failed to write HTML file");
    println!("  - Saved to: {html_path}\n");

    // Step 8: Generate performance report
    println!("8. Generating Performance Report");
    println!("--------------------------------");

    let report = viz_engine.generate_performance_report(&dtg);
    println!("✓ Performance report generated");
    println!("  - Size: {} characters", report.len());

    // Save report to file
    let report_path = "dtg_performance_report.txt";
    std::fs::write(report_path, report).expect("Failed to write report file");
    println!("  - Saved to: {report_path}\n");

    // Step 9: Demonstrate format selection
    println!("9. Format Selection Demo");
    println!("------------------------");

    let formats = vec![
        VisualizationFormat::Dot,
        VisualizationFormat::Mermaid,
        VisualizationFormat::Json,
        VisualizationFormat::Csv,
        VisualizationFormat::Html,
    ];

    for format in formats {
        match format {
            VisualizationFormat::Dot => println!("  - DOT: Graph visualization"),
            VisualizationFormat::Mermaid => println!("  - Mermaid: Documentation diagrams"),
            VisualizationFormat::Json => println!("  - JSON: Programmatic analysis"),
            VisualizationFormat::Csv => println!("  - CSV: Tabular data analysis"),
            VisualizationFormat::Html => println!("  - HTML: Interactive web visualization"),
        }
    }

    println!("\n=== Example Complete ===");
    println!("\nSummary:");
    println!("- Successfully created DTG visualization engine");
    println!("- Exported DTG to 5 different formats");
    println!("- Analyzed graph structure and performance");
    println!("- Identified potential issues and bottlenecks");
    println!("- Generated interactive HTML visualization");
    println!("- Created comprehensive performance report");
    println!("\nGenerated files:");
    println!("- dtg_visualization.html (open in browser)");
    println!("- dtg_performance_report.txt");
}

/// Create a sample Data Transformation Graph for demonstration.
fn create_sample_dtg() -> DataTransformationGraph {
    let mut dtg = DataTransformationGraph {
        id: Uuid::new_v4(),
        name: "Data Processing Pipeline".to_string(),
        root_nodes: vec![],
        nodes: HashMap::new(),
        edges: Vec::new(),
        graph_inputs: vec![],
        graph_outputs: vec![],
        metadata: HashMap::new(),
        started_at: chrono::Utc::now(),
        completed_at: None,
        status: constellation_core::models::dtg::DtgGraphStatus::Ready,
        tags: vec![],
    };

    // Create nodes for a typical analytics pipeline
    let nodes = vec![
        create_node(
            "data_ingestion",
            "ingest_agent",
            DtgNodeStatus::Completed,
            0.95,
            500,
            0.05,
            "data_source",
        ),
        create_node(
            "data_cleaning",
            "cleaning_agent",
            DtgNodeStatus::Completed,
            0.98,
            800,
            0.08,
            "data_cleaning",
        ),
        create_node(
            "data_enrichment",
            "enrichment_agent",
            DtgNodeStatus::Completed,
            0.92,
            1200,
            0.12,
            "data_enrichment",
        ),
        create_node(
            "metrics_calculation",
            "metrics_agent",
            DtgNodeStatus::Completed,
            0.96,
            600,
            0.06,
            "metrics_calculation",
        ),
        create_node(
            "report_generation",
            "report_agent",
            DtgNodeStatus::Completed,
            0.94,
            900,
            0.09,
            "report_generation",
        ),
        create_node(
            "validation",
            "validation_agent",
            DtgNodeStatus::Completed,
            0.99,
            400,
            0.04,
            "validation",
        ),
        create_node(
            "archiving",
            "archive_agent",
            DtgNodeStatus::Completed,
            0.97,
            300,
            0.03,
            "archiving",
        ),
        create_node(
            "data_cleaning",
            "cleaning_agent_2",
            DtgNodeStatus::Completed,
            0.88,
            1200,
            0.15,
            "transformation",
        ),
        create_node(
            "data_enrichment",
            "enrichment_agent_2",
            DtgNodeStatus::Completed,
            0.92,
            800,
            0.10,
            "transformation",
        ),
        create_node(
            "metrics_calculation",
            "metrics_agent_2",
            DtgNodeStatus::Completed,
            0.85,
            2500,
            0.30,
            "analysis",
        ),
        create_node(
            "report_generation",
            "report_agent_2",
            DtgNodeStatus::Completed,
            0.98,
            1500,
            0.20,
            "export",
        ),
        create_node(
            "validation",
            "validation_agent_2",
            DtgNodeStatus::Completed,
            0.75,
            3000,
            0.40,
            "validation",
        ),
        create_node(
            "archiving",
            "archive_agent_2",
            DtgNodeStatus::Completed,
            0.99,
            700,
            0.08,
            "export",
        ),
    ];

    // Add nodes to graph
    for node in nodes {
        dtg.nodes.insert(node.id, node);
    }

    // Create edges (dependencies)
    let node_ids: Vec<Uuid> = dtg.nodes.keys().cloned().collect();

    // Linear pipeline with some parallel branches
    dtg.edges.push(DtgEdge {
        source: node_ids[0],
        target: node_ids[1],
        data_ref: Uuid::new_v4(),
        edge_type: "data_flow".to_string(),
        metadata: HashMap::new(),
    });

    dtg.edges.push(DtgEdge {
        source: node_ids[1],
        target: node_ids[2],
        data_ref: Uuid::new_v4(),
        edge_type: "data_flow".to_string(),
        metadata: HashMap::new(),
    });

    // Parallel processing after enrichment
    dtg.edges.push(DtgEdge {
        source: node_ids[2],
        target: node_ids[3],
        data_ref: Uuid::new_v4(),
        edge_type: "data_flow".to_string(),
        metadata: HashMap::new(),
    });

    dtg.edges.push(DtgEdge {
        source: node_ids[2],
        target: node_ids[5],
        data_ref: Uuid::new_v4(),
        edge_type: "data_flow".to_string(),
        metadata: HashMap::new(),
    });

    // Report generation depends on metrics
    dtg.edges.push(DtgEdge {
        source: node_ids[3],
        target: node_ids[4],
        data_ref: Uuid::new_v4(),
        edge_type: "data_flow".to_string(),
        metadata: HashMap::new(),
    });

    // Archiving depends on validation
    dtg.edges.push(DtgEdge {
        source: node_ids[5],
        target: node_ids[6],
        data_ref: Uuid::new_v4(),
        edge_type: "data_flow".to_string(),
        metadata: HashMap::new(),
    });

    dtg
}

/// Helper function to create a DTG node.
fn create_node(
    skill_id: &str,
    agent_id: &str,
    status: DtgNodeStatus,
    quality: f64,
    time_ms: u64,
    cost: f64,
    task_type: &str,
) -> DtgNode {
    DtgNode {
        id: Uuid::new_v4(),
        skill_id: skill_id.to_string(),
        agent_id: agent_id.to_string(),
        inputs: vec![],
        outputs: vec![DtgDataRef {
            id: Uuid::new_v4(),
            data_type: "processed_data".to_string(),
            schema: None,
            size_bytes: Some(1000),
            content_hash: Some(Uuid::new_v4().to_string()),
            storage_ref: Some("memory".to_string()),
        }],
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert(
                "task_type".to_string(),
                serde_json::Value::String(task_type.to_string()),
            );
            metadata.insert(
                "complexity".to_string(),
                serde_json::Value::String("medium".to_string()),
            );
            metadata
        },
        started_at: chrono::Utc::now(),
        completed_at: None,
        status,
        error: None,
        metrics: DtgMetrics {
            cpu_time_ms: 0,
            memory_bytes: 0,
            network_bytes: 0,
            disk_bytes: 0,
            retry_count: 0,
            quality_score: quality,
            confidence_score: 0.0,
            execution_time_ms: time_ms,
            throughput_ops_per_sec: 0.0,
            error_rate: 0.0,
            data_consistency_score: 0.0,
            schema_compliance_score: 0.0,
            business_value_score: 0.0,
            cost,
            collected_at: chrono::Utc::now(),
        },
    }
}
