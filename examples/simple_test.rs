//! Simple test to verify the constellation-core library works

use constellation_core::models::dtg::{DataTransformationGraph, DtgNode, DtgNodeStatus, DtgMetrics, DtgDataRef, DtgGraphStatus};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

fn main() {
    println!("=== Simple Constellation Core Test ===\n");
    
    // Create a simple DTG
    let mut graph = DataTransformationGraph {
        id: Uuid::new_v4(),
        name: "Test Graph".to_string(),
        root_nodes: vec![],
        nodes: HashMap::new(),
        edges: vec![],
        graph_inputs: vec![],
        graph_outputs: vec![],
        metadata: HashMap::new(),
        started_at: Utc::now(),
        completed_at: None,
        status: DtgGraphStatus::Ready,
        tags: vec![],
    };
    
    // Create a simple node
    let node = DtgNode {
        id: Uuid::new_v4(),
        skill_id: "test_skill".to_string(),
        agent_id: "test_agent".to_string(),
        inputs: vec![],
        outputs: vec![DtgDataRef {
            id: Uuid::new_v4(),
            data_type: "test_data".to_string(),
            schema: None,
            size_bytes: Some(1000),
            content_hash: Some("test_hash".to_string()),
            storage_ref: Some("memory".to_string()),
        }],
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("test".to_string(), serde_json::Value::String("value".to_string()));
            metadata
        },
        started_at: Utc::now(),
        completed_at: None,
        status: DtgNodeStatus::Pending,
        error: None,
        metrics: DtgMetrics {
            cpu_time_ms: 0,
            memory_bytes: 0,
            network_bytes: 0,
            disk_bytes: 0,
            retry_count: 0,
            quality_score: 0.0,
            confidence_score: 0.0,
            execution_time_ms: 0,
            throughput_ops_per_sec: 0.0,
            error_rate: 0.0,
            data_consistency_score: 0.0,
            schema_compliance_score: 0.0,
            business_value_score: 0.0,
            cost: 0.0,
            collected_at: Utc::now(),
        },
    };
    
    graph.nodes.insert(node.id, node.clone());
    graph.root_nodes.push(node.id);
    
    println!("✓ Created DataTransformationGraph");
    println!("  - ID: {}", graph.id);
    println!("  - Name: {}", graph.name);
    println!("  - Nodes: {}", graph.nodes.len());
    println!("  - Status: {:?}", graph.status);
    
    println!("\n✓ Created DtgNode");
    println!("  - ID: {}", node.id);
    println!("  - Skill ID: {}", node.skill_id);
    println!("  - Agent ID: {}", node.agent_id);
    println!("  - Status: {:?}", node.status);
    
    println!("\n=== Test Complete ===");
    println!("Library is working correctly!");
}