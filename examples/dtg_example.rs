//! Example demonstrating Data Transformation Graph (DTG) usage.
//!
//! This example shows how to create and execute a DTG for tracking
//! multi-agent skill execution as data transformations.

use constellation_core::{
    DataTransformationGraph, DtgDataRef, DtgMetrics, DtgNode, DtgNodeStatus,
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
    node1.add_input(input_data.clone());
    
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
    dtg.add_edge(node1_id, node2_id, validation_output.id, "data_flow".to_string());
    dtg.add_edge(node2_id, node3_id, enriched_output.id, "data_flow".to_string());
    
    // Add final output
    dtg.graph_outputs.push(analysis_output);
    
    // Mark the graph as ready
    dtg.mark_ready();
    
    println!("Created DTG with ID: {}", dtg.id);
    println!("Graph name: {}", dtg.name);
    println!("Number of nodes: {}", dtg.nodes.len());
    println!("Number of edges: {}", dtg.edges.len());
    println!("Root nodes: {:?}", dtg.root_nodes);
    println!("Graph status: {:?}", dtg.status);
    println!("Is acyclic: {}", dtg.is_acyclic());
    
    // Simulate execution
    println!("\n=== Simulating DTG Execution ===\n");
    
    dtg.mark_executing();
    
    // Update node statuses
    if let Some(node) = dtg.nodes.get_mut(&node1_id) {
        node.mark_executing();
        println!("Node 1 ({}) started execution", node.skill_id);
        
        // Simulate completion
        let metrics = DtgMetrics {
            cpu_time_ms: 100,
            memory_bytes: 1024 * 1024,
            network_bytes: 1024,
            disk_bytes: 2048,
            retry_count: 0,
            quality_score: 0.95,
            confidence_score: 0.98,
        };
        node.mark_completed(metrics);
        println!("Node 1 completed with quality score: {:.2}", node.metrics.quality_score);
    }
    
    if let Some(node) = dtg.nodes.get_mut(&node2_id) {
        node.mark_executing();
        println!("Node 2 ({}) started execution", node.skill_id);
        
        let metrics = DtgMetrics {
            cpu_time_ms: 200,
            memory_bytes: 2 * 1024 * 1024,
            network_bytes: 2048,
            disk_bytes: 4096,
            retry_count: 1,
            quality_score: 0.90,
            confidence_score: 0.95,
        };
        node.mark_completed(metrics);
        println!("Node 2 completed with quality score: {:.2}", node.metrics.quality_score);
    }
    
    if let Some(node) = dtg.nodes.get_mut(&node3_id) {
        node.mark_executing();
        println!("Node 3 ({}) started execution", node.skill_id);
        
        let metrics = DtgMetrics {
            cpu_time_ms: 150,
            memory_bytes: 3 * 1024 * 1024,
            network_bytes: 1024,
            disk_bytes: 1024,
            retry_count: 0,
            quality_score: 0.98,
            confidence_score: 0.99,
        };
        node.mark_completed(metrics);
        println!("Node 3 completed with quality score: {:.2}", node.metrics.quality_score);
    }
    
    // Mark graph as completed
    dtg.mark_completed();
    
    println!("\n=== DTG Execution Complete ===\n");
    println!("Final graph status: {:?}", dtg.status);
    println!("Execution time: {:?}", dtg.completed_at.unwrap() - dtg.started_at);
    
    // Calculate overall quality
    let total_quality: f64 = dtg.nodes.values()
        .map(|node| node.metrics.quality_score)
        .sum();
    let avg_quality = total_quality / dtg.nodes.len() as f64;
    println!("Average quality score: {:.2}", avg_quality);
    
    // Show dependencies
    println!("\n=== Dependency Analysis ===\n");
    for (node_id, node) in &dtg.nodes {
        let deps = dtg.get_dependencies(*node_id);
        let dependents = dtg.get_dependents(*node_id);
        
        println!("Node {} ({}):", node_id, node.skill_id);
        println!("  Dependencies: {:?}", deps);
        println!("  Dependents: {:?}", dependents);
        println!("  Status: {:?}", node.status);
    }
    
    // Serialize to JSON
    let json = serde_json::to_string_pretty(&dtg).unwrap();
    println!("\n=== DTG JSON Representation (first 500 chars) ===\n");
    println!("{}...", &json[..500.min(json.len())]);
}