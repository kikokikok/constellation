//! DTG execution engine with cycle detection and dependency resolution.

use crate::models::dtg::{
    DataTransformationGraph, DtgGraphStatus, DtgMetrics, DtgNode, DtgNodeStatus,
};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

/// Execution engine for Data Transformation Graphs.
pub struct DtgExecutionEngine {
    /// Current graph being executed.
    graph: DataTransformationGraph,
    
    /// Nodes ready for execution (no pending dependencies).
    ready_queue: VecDeque<Uuid>,
    
    /// Nodes currently executing.
    executing: HashSet<Uuid>,
    
    /// Nodes that have completed execution.
    completed: HashSet<Uuid>,
    
    /// Nodes that have failed.
    failed: HashSet<Uuid>,
    
    /// Execution statistics.
    stats: ExecutionStats,
    
    /// Callback for node execution.
    node_executor: Box<dyn Fn(&mut DtgNode) -> Result<DtgMetrics, String> + Send + Sync>,
}

/// Execution statistics for the DTG engine.
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    /// Total nodes in the graph.
    pub total_nodes: usize,
    
    /// Nodes successfully executed.
    pub nodes_completed: usize,
    
    /// Nodes that failed.
    pub nodes_failed: usize,
    
    /// Nodes currently executing.
    pub nodes_executing: usize,
    
    /// Nodes waiting for dependencies.
    pub nodes_waiting: usize,
    
    /// Total execution time in milliseconds.
    pub total_execution_time_ms: u64,
    
    /// Start time of execution.
    pub started_at: chrono::DateTime<chrono::Utc>,
    
    /// End time of execution.
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl DtgExecutionEngine {
    /// Create a new execution engine for a DTG.
    pub fn new(
        graph: DataTransformationGraph,
        node_executor: Box<dyn Fn(&mut DtgNode) -> Result<DtgMetrics, String> + Send + Sync>,
    ) -> Self {
        // Validate the graph is acyclic before execution
        if !graph.is_acyclic() {
            panic!("Cannot execute DTG with cycles. Use validate() to check for cycles first.");
        }
        
        let total_nodes = graph.nodes.len();
        let mut ready_queue = VecDeque::new();
        
        // Initialize ready queue with nodes that have no graph dependencies
        // (nodes that are not targets of any edges)
        for node_id in graph.nodes.keys() {
            let has_incoming_edges = graph.edges.iter().any(|edge| edge.target == *node_id);
            if !has_incoming_edges {
                ready_queue.push_back(*node_id);
            }
        }
        
        let ready_queue_len = ready_queue.len();
        Self {
            graph,
            ready_queue,
            executing: HashSet::new(),
            completed: HashSet::new(),
            failed: HashSet::new(),
            stats: ExecutionStats {
                total_nodes,
                nodes_completed: 0,
                nodes_failed: 0,
                nodes_executing: 0,
                nodes_waiting: total_nodes - ready_queue_len,
                total_execution_time_ms: 0,
                started_at: chrono::Utc::now(),
                ended_at: None,
            },
            node_executor,
        }
    }
    
    /// Execute the DTG synchronously.
    pub fn execute(&mut self) -> Result<(), String> {
        self.graph.mark_executing();
        
        while !self.is_complete() {
            self.execute_step()?;
        }
        
        self.finalize_execution();
        Ok(())
    }
    
    /// Execute a single step of the DTG.
    pub fn execute_step(&mut self) -> Result<(), String> {
        // Check for completed nodes and update dependents
        self.update_completed_nodes();
        
        // Execute ready nodes
        self.execute_ready_nodes()?;
        
        // Update statistics
        self.update_stats();
        
        Ok(())
    }
    
    /// Check if execution is complete.
    pub fn is_complete(&self) -> bool {
        self.completed.len() + self.failed.len() == self.graph.nodes.len()
    }
    
    /// Get the current execution status.
    pub fn status(&self) -> DtgGraphStatus {
        if self.is_complete() {
            if self.failed.is_empty() {
                DtgGraphStatus::Completed
            } else if self.completed.is_empty() {
                DtgGraphStatus::Failed
            } else {
                DtgGraphStatus::PartiallyCompleted
            }
        } else {
            DtgGraphStatus::Executing
        }
    }
    
    /// Get execution statistics.
    pub fn stats(&self) -> &ExecutionStats {
        &self.stats
    }
    
    /// Get the underlying graph.
    pub fn graph(&self) -> &DataTransformationGraph {
        &self.graph
    }
    
    /// Get a mutable reference to the underlying graph.
    pub fn graph_mut(&mut self) -> &mut DataTransformationGraph {
        &mut self.graph
    }
    
    /// Validate the DTG for execution.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Check for cycles
        if !self.graph.is_acyclic() {
            errors.push("DTG contains cycles".to_string());
        }
        
        // Check for orphaned nodes (no incoming or outgoing edges and not a root)
        for (node_id, _node) in &self.graph.nodes {
            let has_incoming = self.graph.edges.iter().any(|e| e.target == *node_id);
            let has_outgoing = self.graph.edges.iter().any(|e| e.source == *node_id);
            
            if !has_incoming && !has_outgoing && !self.graph.root_nodes.contains(node_id) {
                errors.push(format!("Node {} is orphaned (no connections)", node_id));
            }
        }
        
        // Check for duplicate node IDs
        let node_ids: HashSet<Uuid> = self.graph.nodes.keys().cloned().collect();
        if node_ids.len() != self.graph.nodes.len() {
            errors.push("Duplicate node IDs detected".to_string());
        }
        
        // Check for invalid edges (references to non-existent nodes)
        for edge in &self.graph.edges {
            if !self.graph.nodes.contains_key(&edge.source) {
                errors.push(format!("Edge references non-existent source node: {}", edge.source));
            }
            if !self.graph.nodes.contains_key(&edge.target) {
                errors.push(format!("Edge references non-existent target node: {}", edge.target));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Find cycles in the DTG using Tarjan's strongly connected components algorithm.
    pub fn find_cycles(&self) -> Vec<Vec<Uuid>> {
        let mut index = 0;
        let mut indices = HashMap::new();
        let mut lowlinks = HashMap::new();
        let mut stack = Vec::new();
        let mut on_stack = HashSet::new();
        let mut cycles = Vec::new();
        
        for node_id in self.graph.nodes.keys() {
            if !indices.contains_key(node_id) {
                self.strong_connect(
                    *node_id,
                    &mut index,
                    &mut indices,
                    &mut lowlinks,
                    &mut stack,
                    &mut on_stack,
                    &mut cycles,
                );
            }
        }
        
        cycles
    }
    
    fn strong_connect(
        &self,
        node_id: Uuid,
        index: &mut u32,
        indices: &mut HashMap<Uuid, u32>,
        lowlinks: &mut HashMap<Uuid, u32>,
        stack: &mut Vec<Uuid>,
        on_stack: &mut HashSet<Uuid>,
        cycles: &mut Vec<Vec<Uuid>>,
    ) {
        indices.insert(node_id, *index);
        lowlinks.insert(node_id, *index);
        *index += 1;
        stack.push(node_id);
        on_stack.insert(node_id);
        
        // Consider successors
        for dependent_id in self.graph.get_dependents(node_id) {
            if !indices.contains_key(&dependent_id) {
                self.strong_connect(
                    dependent_id,
                    index,
                    indices,
                    lowlinks,
                    stack,
                    on_stack,
                    cycles,
                );
                let lowlink = std::cmp::min(
                    *lowlinks.get(&node_id).unwrap(),
                    *lowlinks.get(&dependent_id).unwrap(),
                );
                lowlinks.insert(node_id, lowlink);
            } else if on_stack.contains(&dependent_id) {
                let lowlink = std::cmp::min(
                    *lowlinks.get(&node_id).unwrap(),
                    *indices.get(&dependent_id).unwrap(),
                );
                lowlinks.insert(node_id, lowlink);
            }
        }
        
        // If node is a root node, pop the stack and generate an SCC
        if lowlinks.get(&node_id) == indices.get(&node_id) {
            let mut scc = Vec::new();
            loop {
                let w = stack.pop().unwrap();
                on_stack.remove(&w);
                scc.push(w);
                if w == node_id {
                    break;
                }
            }
            
            // SCCs with more than one node are cycles
            if scc.len() > 1 {
                cycles.push(scc);
            }
        }
    }
    
    fn update_completed_nodes(&mut self) {
        let completed_nodes: Vec<Uuid> = self.executing.iter()
            .filter(|node_id| {
                let node = self.graph.nodes.get(node_id).unwrap();
                matches!(node.status, DtgNodeStatus::Completed | DtgNodeStatus::Failed)
            })
            .cloned()
            .collect();
        
        for node_id in completed_nodes {
            self.executing.remove(&node_id);
            
            let node = self.graph.nodes.get(&node_id).unwrap();
            match node.status {
                DtgNodeStatus::Completed => {
                    self.completed.insert(node_id);
                    self.add_dependents_to_ready_queue(node_id);
                }
                DtgNodeStatus::Failed => {
                    self.failed.insert(node_id);
                    // Optionally: propagate failure to dependents
                    // self.mark_dependents_as_failed(node_id);
                }
                _ => {}
            }
        }
    }
    
    fn execute_ready_nodes(&mut self) -> Result<(), String> {
        while let Some(node_id) = self.ready_queue.pop_front() {
            if self.executing.contains(&node_id) 
                || self.completed.contains(&node_id) 
                || self.failed.contains(&node_id) {
                continue;
            }
            
            // Check if all dependencies are completed
            let dependencies = self.graph.get_dependencies(node_id);
            let all_deps_completed = dependencies.iter()
                .all(|dep_id| self.completed.contains(dep_id));
            
            if !all_deps_completed && !dependencies.is_empty() {
                // Not all dependencies are ready yet
                if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                    node.status = DtgNodeStatus::Waiting;
                }
                continue;
            }
            
            // Execute the node
            if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                node.mark_executing();
                self.executing.insert(node_id);
                
                match (self.node_executor)(node) {
                    Ok(metrics) => {
                        node.mark_completed(metrics);
                    }
                    Err(error) => {
                        node.mark_failed(error);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn add_dependents_to_ready_queue(&mut self, node_id: Uuid) {
        for dependent_id in self.graph.get_dependents(node_id) {
            // Check if all dependencies of this dependent are completed
            let dependencies = self.graph.get_dependencies(dependent_id);
            let all_deps_completed = dependencies.iter()
                .all(|dep_id| self.completed.contains(dep_id));
            
            if all_deps_completed {
                self.ready_queue.push_back(dependent_id);
            }
        }
    }
    
    fn update_stats(&mut self) {
        self.stats.nodes_completed = self.completed.len();
        self.stats.nodes_failed = self.failed.len();
        self.stats.nodes_executing = self.executing.len();
        self.stats.nodes_waiting = self.graph.nodes.len() 
            - self.completed.len() 
            - self.failed.len() 
            - self.executing.len();
        
        if self.is_complete() {
            self.stats.ended_at = Some(chrono::Utc::now());
            if let Some(started_at) = self.stats.ended_at {
                self.stats.total_execution_time_ms = started_at
                    .signed_duration_since(self.stats.started_at)
                    .num_milliseconds() as u64;
            }
        }
    }
    
    fn finalize_execution(&mut self) {
        match self.status() {
            DtgGraphStatus::Completed => {
                self.graph.mark_completed();
            }
            DtgGraphStatus::PartiallyCompleted => {
                self.graph.status = DtgGraphStatus::PartiallyCompleted;
                self.graph.completed_at = Some(chrono::Utc::now());
            }
            DtgGraphStatus::Failed => {
                self.graph.status = DtgGraphStatus::Failed;
                self.graph.completed_at = Some(chrono::Utc::now());
            }
            _ => {}
        }
    }
}

impl std::fmt::Debug for DtgExecutionEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DtgExecutionEngine")
            .field("graph", &self.graph)
            .field("ready_queue", &self.ready_queue)
            .field("executing", &self.executing)
            .field("completed", &self.completed)
            .field("failed", &self.failed)
            .field("stats", &self.stats)
            .field("node_executor", &"<function>")
            .finish()
    }
}

impl Default for ExecutionStats {
    fn default() -> Self {
        Self {
            total_nodes: 0,
            nodes_completed: 0,
            nodes_failed: 0,
            nodes_executing: 0,
            nodes_waiting: 0,
            total_execution_time_ms: 0,
            started_at: chrono::Utc::now(),
            ended_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::dtg::DtgDataRef;
    
    fn create_test_graph() -> DataTransformationGraph {
        let mut graph = DataTransformationGraph::new("Test Graph".to_string());
        
        // Create nodes
        let mut node1 = DtgNode::new("skill1".to_string(), "agent1".to_string());
        let mut node2 = DtgNode::new("skill2".to_string(), "agent2".to_string());
        let mut node3 = DtgNode::new("skill3".to_string(), "agent3".to_string());
        
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
        
        let data3 = DtgDataRef {
            id: Uuid::new_v4(),
            data_type: "json".to_string(),
            schema: None,
            size_bytes: None,
            content_hash: None,
            storage_ref: None,
        };
        
        // Add data references to nodes
        node1.add_output(data1.clone());
        node2.add_input(data1.clone());
        node2.add_output(data2.clone());
        node3.add_input(data2.clone());
        node3.add_output(data3.clone());
        
        // Add nodes to graph
        let node1_id = graph.add_node(node1);
        let node2_id = graph.add_node(node2);
        let node3_id = graph.add_node(node3);
        
        // Add edges
        graph.add_edge(node1_id, node2_id, data1.id, "data_flow".to_string());
        graph.add_edge(node2_id, node3_id, data2.id, "data_flow".to_string());
        
        graph.mark_ready();
        graph
    }
    
    #[test]
    fn test_engine_creation() {
        let graph = create_test_graph();
        let executor = Box::new(|node: &mut DtgNode| {
            node.mark_executing();
            Ok(DtgMetrics::default())
        });
        
        let engine = DtgExecutionEngine::new(graph, executor);
        assert_eq!(engine.stats.total_nodes, 3);
        assert_eq!(engine.ready_queue.len(), 1); // Only node1 should be ready initially
    }
    
    #[test]
    fn test_cycle_detection() {
        let mut graph = DataTransformationGraph::new("Cyclic Graph".to_string());
        
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
        
        let node1_id = graph.add_node(node1);
        let node2_id = graph.add_node(node2);
        
        graph.add_edge(node1_id, node2_id, data1.id, "data_flow".to_string());
        graph.add_edge(node2_id, node1_id, data2.id, "data_flow".to_string());
        
        let cycles = graph.is_acyclic();
        assert!(!cycles, "Graph should be detected as cyclic");
    }
    
    #[test]
    fn test_find_cycles() {
        let mut graph = DataTransformationGraph::new("Cyclic Graph".to_string());
        
        let mut node1 = DtgNode::new("skill1".to_string(), "agent1".to_string());
        let mut node2 = DtgNode::new("skill2".to_string(), "agent2".to_string());
        let mut node3 = DtgNode::new("skill3".to_string(), "agent3".to_string());
        
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
        
        let data3 = DtgDataRef {
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
        node3.add_input(data2.clone());
        node3.add_output(data3.clone());
        
        let node1_id = graph.add_node(node1);
        let node2_id = graph.add_node(node2);
        let node3_id = graph.add_node(node3);
        
        // Create a cycle: 1 -> 2 -> 3 -> 1
        graph.add_edge(node1_id, node2_id, data1.id, "data_flow".to_string());
        graph.add_edge(node2_id, node3_id, data2.id, "data_flow".to_string());
        graph.add_edge(node3_id, node1_id, data3.id, "data_flow".to_string());
        
        // Create engine without validation (bypass the panic)
        let executor = Box::new(|_node: &mut DtgNode| Ok(DtgMetrics::default()));
        
        // Manually create engine to bypass validation
        let total_nodes = graph.nodes.len();
        let mut ready_queue = VecDeque::new();
        for node_id in graph.root_nodes.iter() {
            ready_queue.push_back(*node_id);
        }
        
        let ready_queue_len = ready_queue.len();
        let engine = DtgExecutionEngine {
            graph,
            ready_queue,
            executing: HashSet::new(),
            completed: HashSet::new(),
            failed: HashSet::new(),
            stats: ExecutionStats {
                total_nodes,
                nodes_completed: 0,
                nodes_failed: 0,
                nodes_executing: 0,
                nodes_waiting: total_nodes - ready_queue_len,
                total_execution_time_ms: 0,
                started_at: chrono::Utc::now(),
                ended_at: None,
            },
            node_executor: executor,
        };
        
        let cycles = engine.find_cycles();
        assert!(!cycles.is_empty(), "Should find at least one cycle");
        assert_eq!(cycles[0].len(), 3, "Cycle should contain 3 nodes");
    }
    
    #[test]
    fn test_validation() {
        let graph = create_test_graph();
        let executor = Box::new(|_node: &mut DtgNode| Ok(DtgMetrics::default()));
        let engine = DtgExecutionEngine::new(graph, executor);
        
        let validation_result = engine.validate();
        assert!(validation_result.is_ok(), "Valid graph should pass validation");
    }
}