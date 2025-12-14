//! Example demonstrating DTG execution engine with cycle detection.

use constellation_core::{
    dtg::engine::DtgExecutionEngine,
    models::dtg::{DataTransformationGraph, DtgDataRef, DtgMetrics, DtgNode},
};
use std::collections::HashMap;
use uuid::Uuid;

fn main() {
    println!("=== DTG Execution Engine Example ===\n");
    
    // Create a simple linear DTG: A -> B -> C
    let mut graph = DataTransformationGraph::new("Linear Transformation Pipeline".to_string());
    
    // Create data references
    let data_a = DtgDataRef {
        id: Uuid::new_v4(),
        data_type: "json".to_string(),
        schema: None,
        size_bytes: None,
        content_hash: None,
        storage_ref: None,
    };
    
    let data_b = DtgDataRef {
        id: Uuid::new_v4(),
        data_type: "json".to_string(),
        schema: None,
        size_bytes: None,
        content_hash: None,
        storage_ref: None,
    };
    
    let data_c = DtgDataRef {
        id: Uuid::new_v4(),
        data_type: "json".to_string(),
        schema: None,
        size_bytes: None,
        content_hash: None,
        storage_ref: None,
    };
    
    // Create nodes
    let mut node_a = DtgNode::new("data_ingestion".to_string(), "agent_ingest".to_string());
    let mut node_b = DtgNode::new("data_processing".to_string(), "agent_process".to_string());
    let mut node_c = DtgNode::new("data_export".to_string(), "agent_export".to_string());
    
    // Configure node inputs/outputs
    node_a.add_output(data_a.clone());
    node_b.add_input(data_a.clone());
    node_b.add_output(data_b.clone());
    node_c.add_input(data_b.clone());
    node_c.add_output(data_c.clone());
    
    // Add nodes to graph
    let node_a_id = graph.add_node(node_a);
    let node_b_id = graph.add_node(node_b);
    let node_c_id = graph.add_node(node_c);
    
    // Add edges
    graph.add_edge(node_a_id, node_b_id, data_a.id, "data_flow".to_string());
    graph.add_edge(node_b_id, node_c_id, data_b.id, "data_flow".to_string());
    
    // Mark graph as ready
    graph.mark_ready();
    
    println!("Created DTG with 3 nodes:");
    println!("  Node A ({}): data_ingestion", node_a_id);
    println!("  Node B ({}): data_processing", node_b_id);
    println!("  Node C ({}): data_export", node_c_id);
    println!();
    
    // Create execution engine
    let executor = Box::new(|node: &mut DtgNode| {
        println!("  Executing node: {} (skill: {})", node.id, node.skill_id);
        
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Return metrics
        Ok(DtgMetrics {
            cpu_time_ms: 50,
            memory_bytes: 1024 * 1024, // 1MB
            network_bytes: 0,
            disk_bytes: 0,
            retry_count: 0,
            quality_score: 0.95,
            confidence_score: 0.98,
        })
    });
    
    let mut engine = DtgExecutionEngine::new(graph, executor);
    
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
    
    println!("\n=== Executing DTG ===");
    
    // Execute the graph
    match engine.execute() {
        Ok(_) => {
            println!("✓ DTG execution completed successfully");
            
            let stats = engine.stats();
            println!("\nExecution Statistics:");
            println!("  Total nodes: {}", stats.total_nodes);
            println!("  Completed: {}", stats.nodes_completed);
            println!("  Failed: {}", stats.nodes_failed);
            println!("  Execution time: {}ms", stats.total_execution_time_ms);
        }
        Err(error) => {
            println!("✗ DTG execution failed: {}", error);
        }
    }
    
    println!("\n=== Cycle Detection Example ===");
    
    // Create a cyclic graph to demonstrate cycle detection
    let mut cyclic_graph = DataTransformationGraph::new("Cyclic Graph Example".to_string());
    
    let mut node1 = DtgNode::new("skill1".to_string(), "agent1".to_string());
    let mut node2 = DtgNode::new("skill2".to_string(), "agent2".to_string());
    
    let data1 = DtgDataRef {
        id: Uuid::new_v4(),
        data_type: "json".to_string(),
        schema: None,
        size_bytes: None,
        content_hash: None,
        storage_ref: None,
    };
    
    let data2 = DtgDataRef {
        id: Uuid::new_v4(),
        data_type: "json".to_string(),
        schema: None,
        size_bytes: None,
        content_hash: None,
        storage_ref: None,
    };
    
    node1.add_output(data1.clone());
    node1.add_input(data2.clone());
    node2.add_output(data2.clone());
    node2.add_input(data1.clone());
    
    let node1_id = cyclic_graph.add_node(node1);
    let node2_id = cyclic_graph.add_node(node2);
    
    cyclic_graph.add_edge(node1_id, node2_id, data1.id, "data_flow".to_string());
    cyclic_graph.add_edge(node2_id, node1_id, data2.id, "data_flow".to_string());
    
    // Try to create engine with cyclic graph (should panic)
    println!("\nAttempting to create engine with cyclic graph...");
    
    let cyclic_executor = Box::new(|_node: &mut DtgNode| Ok(DtgMetrics::default()));
    
    // Use catch_unwind to handle the expected panic
    let result = std::panic::catch_unwind(|| {
        let _engine = DtgExecutionEngine::new(cyclic_graph, cyclic_executor);
    });
    
    match result {
        Ok(_) => println!("✗ Unexpected: Engine created without panic"),
        Err(_) => println!("✓ Expected: Engine creation panicked due to cycles"),
    }
    
    println!("\n=== Example Complete ===");
}