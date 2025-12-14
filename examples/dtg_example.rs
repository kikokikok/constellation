//! Example demonstrating Data Transformation Graph (DTG) usage.
//!
//! This example shows how to create and execute a DTG for tracking
//! multi-agent skill execution as data transformations.

use constellation_core::{
    dtg::engine::DtgExecutionEngine, DataTransformationGraph, DtgDataRef, DtgMetrics, DtgNode,
};
use uuid::Uuid;

fn main() {
    println!("=== Data Transformation Graph Example ===\n");

    // Create a new DTG for a data processing pipeline
    let mut dtg = DataTransformationGraph::new("Data Processing Pipeline".to_string());

    // Add input data references
    let input_data = DtgDataRef {
        id: Uuid::new_v4(),
        data_type: "json".to_string(),
        schema: Some("{\"type\": \"object\"}".to_string()),
        size_bytes: Some(1024),
        content_hash: Some("abc123".to_string()),
        storage_ref: Some("s3://bucket/input.json".to_string()),
    };

    dtg.graph_inputs.push(input_data.clone());

    // Create transformation nodes
    let mut node1 = DtgNode::new("data_validation".to_string(), "validator_agent".to_string());
    // Node1 is a root node - it takes graph input, not another node's output
    // In a real scenario, this would be connected to graph_inputs

    let validation_output = DtgDataRef {
        id: Uuid::new_v4(),
        data_type: "json".to_string(),
        schema: Some("{\"type\": \"object\", \"validated\": true}".to_string()),
        size_bytes: Some(1024),
        content_hash: Some("def456".to_string()),
        storage_ref: Some("s3://bucket/validated.json".to_string()),
    };
    node1.add_output(validation_output.clone());

    let node1_id = dtg.add_node(node1);

    let mut node2 = DtgNode::new("data_enrichment".to_string(), "enricher_agent".to_string());
    node2.add_input(validation_output.clone());

    let enriched_output = DtgDataRef {
        id: Uuid::new_v4(),
        data_type: "json".to_string(),
        schema: Some("{\"type\": \"object\", \"enriched\": true}".to_string()),
        size_bytes: Some(2048),
        content_hash: Some("ghi789".to_string()),
        storage_ref: Some("s3://bucket/enriched.json".to_string()),
    };
    node2.add_output(enriched_output.clone());

    let node2_id = dtg.add_node(node2);

    let mut node3 = DtgNode::new("data_analysis".to_string(), "analyzer_agent".to_string());
    node3.add_input(enriched_output.clone());

    let analysis_output = DtgDataRef {
        id: Uuid::new_v4(),
        data_type: "json".to_string(),
        schema: Some("{\"type\": \"object\", \"analysis\": true}".to_string()),
        size_bytes: Some(512),
        content_hash: Some("jkl012".to_string()),
        storage_ref: Some("s3://bucket/analysis.json".to_string()),
    };
    node3.add_output(analysis_output.clone());

    let node3_id = dtg.add_node(node3);

    // Add edges to represent data flow
    dtg.add_edge(
        node1_id,
        node2_id,
        validation_output.id,
        "data_flow".to_string(),
    );
    dtg.add_edge(
        node2_id,
        node3_id,
        enriched_output.id,
        "data_flow".to_string(),
    );

    // Add final output
    dtg.graph_outputs.push(analysis_output);

    // Mark the graph as ready
    dtg.mark_ready();

    println!("Created DTG with ID: {}", dtg.id);
    println!("Graph name: {}", dtg.name);
    println!("Number of nodes: {}", dtg.nodes.len());
    println!("Number of edges: {}", dtg.edges.len());
    // Calculate root nodes (nodes with no incoming edges)
    let root_nodes: Vec<Uuid> = dtg.nodes.keys()
        .filter(|node_id| !dtg.edges.iter().any(|edge| edge.target == **node_id))
        .cloned()
        .collect();
    println!("Root nodes: {:?}", root_nodes);
    println!("Graph status: {:?}", dtg.status);
    println!("Is acyclic: {}", dtg.is_acyclic());

    // Create execution engine
    println!("\n=== Creating DTG Execution Engine ===\n");
    
    let executor = Box::new(|node: &mut DtgNode| {
        println!("  Executing node: {} (skill: {})", node.id, node.skill_id);
        
        // Simulate different execution times based on skill
        let metrics = match node.skill_id.as_str() {
            "data_validation" => DtgMetrics {
                cpu_time_ms: 100,
                memory_bytes: 1024 * 1024,
                network_bytes: 1024,
                disk_bytes: 2048,
                retry_count: 0,
                quality_score: 0.95,
                confidence_score: 0.98,
                latency_ms: 50,
                throughput_ops_per_sec: 100.0,
                error_rate: 0.01,
                data_consistency_score: 0.98,
                schema_compliance_score: 0.95,
                business_value_score: 0.9,
                collected_at: chrono::Utc::now(),
            },
            "data_enrichment" => DtgMetrics {
                cpu_time_ms: 200,
                memory_bytes: 2 * 1024 * 1024,
                network_bytes: 2048,
                disk_bytes: 4096,
                retry_count: 1,
                quality_score: 0.90,
                confidence_score: 0.95,
                latency_ms: 100,
                throughput_ops_per_sec: 50.0,
                error_rate: 0.05,
                data_consistency_score: 0.92,
                schema_compliance_score: 0.88,
                business_value_score: 0.85,
                collected_at: chrono::Utc::now(),
            },
            "data_analysis" => DtgMetrics {
                cpu_time_ms: 150,
                memory_bytes: 3 * 1024 * 1024,
                network_bytes: 1024,
                disk_bytes: 1024,
                retry_count: 0,
                quality_score: 0.98,
                confidence_score: 0.99,
                latency_ms: 75,
                throughput_ops_per_sec: 80.0,
                error_rate: 0.02,
                data_consistency_score: 0.96,
                schema_compliance_score: 0.94,
                business_value_score: 0.95,
                collected_at: chrono::Utc::now(),
            },
            _ => DtgMetrics::default(),
        };
        
        // Simulate work
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        Ok(metrics)
    });
    
    let mut engine = DtgExecutionEngine::new(dtg, executor);
    
    // Validate the graph
    match engine.validate() {
        Ok(_) => println!("✓ Graph validation passed"),
        Err(errors) => {
            println!("✗ Graph validation failed:");
            for error in errors {
                println!("  - {}", error);
            }
            return;
        }
    }
    
    // Execute the graph
    println!("\n=== Executing DTG ===\n");
    
    match engine.execute() {
        Ok(_) => {
            println!("✓ DTG execution completed successfully");
            
            let stats = engine.stats();
            println!("\nExecution Statistics:");
            println!("  Total nodes: {}", stats.total_nodes);
            println!("  Completed: {}", stats.nodes_completed);
            println!("  Failed: {}", stats.nodes_failed);
            println!("  Execution time: {}ms", stats.total_execution_time_ms);
            
            let graph = engine.graph();
            println!("\nFinal graph status: {:?}", graph.status);
            println!(
                "Execution time: {:?}",
                graph.completed_at.unwrap() - graph.started_at
            );
        }
        Err(error) => {
            println!("✗ DTG execution failed: {}", error);
        }
    }

    // Calculate overall quality
    let graph = engine.graph();
    let total_quality: f64 = graph
        .nodes
        .values()
        .map(|node| node.metrics.quality_score)
        .sum();
    let avg_quality = total_quality / graph.nodes.len() as f64;
    println!("Average quality score: {:.2}", avg_quality);

    // Show dependencies
    println!("\n=== Dependency Analysis ===\n");
    for (node_id, node) in &graph.nodes {
        let deps = graph.get_dependencies(*node_id);
        let dependents = graph.get_dependents(*node_id);

        println!("Node {} ({}):", node_id, node.skill_id);
        println!("  Dependencies: {:?}", deps);
        println!("  Dependents: {:?}", dependents);
        println!("  Status: {:?}", node.status);
    }

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&graph).unwrap();
    println!("\n=== DTG JSON Representation (first 500 chars) ===\n");
    println!("{}...", &json[..500.min(json.len())]);
}
