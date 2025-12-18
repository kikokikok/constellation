use constellation_core::dtg::engine::DtgExecutionEngine;
use constellation_core::models::dtg::{DataTransformationGraph, DtgNode, DtgDataRef, DtgMetrics};
use uuid::Uuid;

fn main() {
    println!("Testing DTG optimizations...");
    
    // Create a simple graph
    let mut graph = DataTransformationGraph::new("Test Graph".to_string());
    
    // Create nodes
    let mut node1 = DtgNode::new("skill1".to_string(), "agent1".to_string());
    let mut node2 = DtgNode::new("skill2".to_string(), "agent2".to_string());
    
    // Create data references
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
    node2.add_input(data1.clone());
    node2.add_output(data2.clone());
    
    // Add nodes to graph
    let node1_id = graph.add_node(node1);
    let node2_id = graph.add_node(node2);
    
    // Add edge
    graph.add_edge(node1_id, node2_id, data1.id, "data_flow".to_string());
    graph.mark_ready();
    
    // Test petgraph integration
    println!("Testing petgraph cycle detection...");
    assert!(graph.is_acyclic(), "Graph should be acyclic");
    
    // Test topological order
    match graph.topological_order() {
        Ok(order) => {
            println!("Topological order: {:?}", order);
            assert_eq!(order.len(), 2, "Should have 2 nodes in order");
        }
        Err(cycle) => {
            panic!("Should not have cycles: {:?}", cycle);
        }
    }
    
    // Test validation
    println!("Testing validation...");
    if let Err(err) = graph.validate_graph() {
        panic!("Graph validation failed: {:?}", err);
    }
    
    println!("All DTG optimizations working correctly!");
}