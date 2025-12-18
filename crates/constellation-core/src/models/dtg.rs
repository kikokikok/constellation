//! Data Transformation Graph (DTG) models for tracking skill execution.
//!
//! DTG represents agent skill execution as data transformations rather than
//! traditional control flow. Each node represents a data transformation,
//! edges represent data flow dependencies.
//!
//! Based on research: "Data Transformation Graphs vs. Code Property Graphs"
//! for tracking multi-agent execution provenance.

use petgraph::algo::{is_cyclic_directed, toposort};
use petgraph::graph::DiGraph;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use validator::Validate;

/// A data transformation node in the DTG.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DtgNode {
    /// Unique identifier for this transformation.
    pub id: Uuid,

    /// The agent skill that performed this transformation.
    #[validate(length(min = 1, message = "Skill ID cannot be empty"))]
    pub skill_id: String,

    /// The agent that executed this transformation.
    #[validate(length(min = 1, message = "Agent ID cannot be empty"))]
    pub agent_id: String,

    /// Input data references (UUIDs of previous nodes or external data).
    pub inputs: Vec<DtgDataRef>,

    /// Output data produced by this transformation.
    pub outputs: Vec<DtgDataRef>,

    /// Transformation metadata (parameters, configuration, etc.).
    pub metadata: HashMap<String, serde_json::Value>,

    /// Timestamp when transformation started.
    pub started_at: chrono::DateTime<chrono::Utc>,

    /// Timestamp when transformation completed.
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Status of the transformation.
    pub status: DtgNodeStatus,

    /// Error information if transformation failed.
    pub error: Option<String>,

    /// Performance metrics (execution time, resource usage, etc.).
    pub metrics: DtgMetrics,
}

/// Reference to data in the DTG.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DtgDataRef {
    /// Unique identifier for this data reference.
    pub id: Uuid,

    /// Data type (e.g., "json", "text", "binary", "structured").
    #[validate(length(min = 1, message = "Data type cannot be empty"))]
    pub data_type: String,

    /// Schema or format information.
    pub schema: Option<String>,

    /// Size in bytes.
    pub size_bytes: Option<u64>,

    /// Hash of the data content for integrity verification.
    pub content_hash: Option<String>,

    /// Storage location or reference.
    pub storage_ref: Option<String>,
}

/// Status of a DTG node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DtgNodeStatus {
    /// Transformation is pending execution.
    Pending,

    /// Transformation is currently executing.
    Executing,

    /// Transformation completed successfully.
    Completed,

    /// Transformation failed.
    Failed,

    /// Transformation was cancelled.
    Cancelled,

    /// Transformation is waiting for dependencies.
    Waiting,
}

/// Performance metrics for a DTG node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtgMetrics {
    /// CPU time used in milliseconds.
    pub cpu_time_ms: u64,

    /// Memory usage in bytes.
    pub memory_bytes: u64,

    /// Network I/O in bytes.
    pub network_bytes: u64,

    /// Disk I/O in bytes.
    pub disk_bytes: u64,

    /// Number of retries attempted.
    pub retry_count: u32,

    /// Quality score (0.0 to 1.0) of the transformation.
    pub quality_score: f64,

    /// Confidence score (0.0 to 1.0) of the transformation.
    pub confidence_score: f64,

    /// Execution time in milliseconds.
    pub execution_time_ms: u64,

    /// Throughput in operations per second.
    pub throughput_ops_per_sec: f64,

    /// Error rate (0.0 to 1.0).
    pub error_rate: f64,

    /// Data consistency score (0.0 to 1.0).
    pub data_consistency_score: f64,

    /// Schema compliance score (0.0 to 1.0).
    pub schema_compliance_score: f64,

    /// Business value score (0.0 to 1.0).
    pub business_value_score: f64,

    /// Cost incurred for this transformation.
    pub cost: f64,

    /// Timestamp when metrics were collected.
    pub collected_at: chrono::DateTime<chrono::Utc>,
}

/// A complete Data Transformation Graph.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DataTransformationGraph {
    /// Unique identifier for this DTG.
    pub id: Uuid,

    /// Name or description of this transformation graph.
    #[validate(length(min = 1, message = "Graph name cannot be empty"))]
    pub name: String,

    /// Root nodes (nodes with no dependencies).
    pub root_nodes: Vec<Uuid>,

    /// All nodes in the graph.
    pub nodes: HashMap<Uuid, DtgNode>,

    /// Edges representing data flow dependencies.
    pub edges: Vec<DtgEdge>,

    /// Input data references for the entire graph.
    pub graph_inputs: Vec<DtgDataRef>,

    /// Output data references for the entire graph.
    pub graph_outputs: Vec<DtgDataRef>,

    /// Metadata about the graph execution.
    pub metadata: HashMap<String, serde_json::Value>,

    /// Timestamp when graph execution started.
    pub started_at: chrono::DateTime<chrono::Utc>,

    /// Timestamp when graph execution completed.
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Overall status of the graph.
    pub status: DtgGraphStatus,

    /// Tags for categorization and search.
    pub tags: Vec<String>,
}

/// Edge representing data flow between DTG nodes.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DtgEdge {
    /// Source node ID.
    pub source: Uuid,

    /// Target node ID.
    pub target: Uuid,

    /// Data reference being transferred.
    pub data_ref: Uuid,

    /// Edge type (e.g., "data_flow", "control_flow", "dependency").
    #[validate(length(min = 1, message = "Edge type cannot be empty"))]
    pub edge_type: String,

    /// Metadata about this edge.
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Status of a complete DTG.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DtgGraphStatus {
    /// Graph is being constructed.
    Constructing,

    /// Graph is ready for execution.
    Ready,

    /// Graph is currently executing.
    Executing,

    /// Graph completed successfully.
    Completed,

    /// Graph partially completed (some nodes failed).
    PartiallyCompleted,

    /// Graph failed.
    Failed,

    /// Graph was cancelled.
    Cancelled,
}

/// Provenance information for DTG execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtgProvenance {
    /// DTG ID.
    pub dtg_id: Uuid,

    /// Chain of transformations with full audit trail.
    pub transformation_chain: Vec<TransformationRecord>,

    /// Input data lineage.
    pub input_lineage: Vec<DataLineage>,

    /// Output data lineage.
    pub output_lineage: Vec<DataLineage>,

    /// Agent execution records.
    pub agent_records: Vec<AgentExecutionRecord>,

    /// Cryptographic signatures for verification.
    pub signatures: Vec<CryptographicSignature>,

    /// Timestamp when provenance was recorded.
    pub recorded_at: chrono::DateTime<chrono::Utc>,
}

/// Record of a single transformation in the provenance chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationRecord {
    /// Node ID.
    pub node_id: Uuid,

    /// Agent ID that performed the transformation.
    pub agent_id: String,

    /// Skill ID used.
    pub skill_id: String,

    /// Input data references.
    pub inputs: Vec<DtgDataRef>,

    /// Output data references.
    pub outputs: Vec<DtgDataRef>,

    /// Transformation parameters.
    pub parameters: HashMap<String, serde_json::Value>,

    /// Timestamp when transformation occurred.
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Hash of the transformation for integrity.
    pub transformation_hash: String,
}

/// Lineage information for data in the DTG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLineage {
    /// Data reference ID.
    pub data_ref_id: Uuid,

    /// Source transformation nodes.
    pub source_nodes: Vec<Uuid>,

    /// Destination transformation nodes.
    pub destination_nodes: Vec<Uuid>,

    /// Data type.
    pub data_type: String,

    /// Schema evolution history.
    pub schema_history: Vec<SchemaVersion>,

    /// Quality metrics over time.
    pub quality_history: Vec<QualityMetric>,
}

/// Record of agent execution in the DTG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionRecord {
    /// Agent ID.
    pub agent_id: String,

    /// Nodes executed by this agent.
    pub executed_nodes: Vec<Uuid>,

    /// Performance metrics.
    pub metrics: AgentMetrics,

    /// Resource usage.
    pub resource_usage: ResourceUsage,

    /// Error rate.
    pub error_rate: f64,

    /// Success rate.
    pub success_rate: f64,
}

/// Cryptographic signature for DTG verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptographicSignature {
    /// Signer identifier (agent ID or system component).
    pub signer: String,

    /// Signature algorithm.
    pub algorithm: String,

    /// Signature value.
    pub signature: String,

    /// Public key or certificate.
    pub public_key: Option<String>,

    /// Timestamp when signed.
    pub signed_at: chrono::DateTime<chrono::Utc>,
}

/// Schema version information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaVersion {
    /// Version number.
    pub version: String,

    /// Schema definition.
    pub schema: serde_json::Value,

    /// Timestamp when this version was created.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Agent that created this version.
    pub created_by: String,
}

/// Quality metric for data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetric {
    /// Metric name.
    pub metric: String,

    /// Metric value.
    pub value: f64,

    /// Timestamp when measured.
    pub measured_at: chrono::DateTime<chrono::Utc>,

    /// Measurement method.
    pub measurement_method: String,
}

/// Agent performance metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    /// Total execution time in milliseconds.
    pub total_execution_time_ms: u64,

    /// Average execution time per node.
    pub avg_execution_time_ms: u64,

    /// Total nodes executed.
    pub total_nodes_executed: u32,

    /// Successfully executed nodes.
    pub successful_nodes: u32,

    /// Failed nodes.
    pub failed_nodes: u32,

    /// Resource efficiency score.
    pub resource_efficiency: f64,

    /// Quality score.
    pub quality_score: f64,
}

/// Resource usage information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage percentage.
    pub cpu_percent: f64,

    /// Memory usage in bytes.
    pub memory_bytes: u64,

    /// Network usage in bytes.
    pub network_bytes: u64,

    /// Disk usage in bytes.
    pub disk_bytes: u64,

    /// GPU usage if applicable.
    pub gpu_usage: Option<GpuUsage>,
}

/// GPU usage information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuUsage {
    /// GPU utilization percentage.
    pub utilization_percent: f64,

    /// Memory usage in bytes.
    pub memory_bytes: u64,

    /// GPU temperature in Celsius.
    pub temperature_c: f64,

    /// Power draw in watts.
    pub power_watts: f64,
}

impl DtgNode {
    /// Create a new DTG node.
    pub fn new(skill_id: String, agent_id: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            skill_id,
            agent_id,
            inputs: Vec::new(),
            outputs: Vec::new(),
            metadata: HashMap::new(),
            started_at: chrono::Utc::now(),
            completed_at: None,
            status: DtgNodeStatus::Pending,
            error: None,
            metrics: DtgMetrics::default(),
        }
    }

    /// Mark the node as executing.
    pub fn mark_executing(&mut self) {
        self.status = DtgNodeStatus::Executing;
        self.started_at = chrono::Utc::now();
    }

    /// Mark the node as completed.
    pub fn mark_completed(&mut self, metrics: DtgMetrics) {
        self.status = DtgNodeStatus::Completed;
        self.completed_at = Some(chrono::Utc::now());
        self.metrics = metrics;
    }

    /// Mark the node as failed.
    pub fn mark_failed(&mut self, error: String) {
        self.status = DtgNodeStatus::Failed;
        self.completed_at = Some(chrono::Utc::now());
        self.error = Some(error);
    }

    /// Add an input data reference.
    pub fn add_input(&mut self, data_ref: DtgDataRef) {
        self.inputs.push(data_ref);
    }

    /// Add an output data reference.
    pub fn add_output(&mut self, data_ref: DtgDataRef) {
        self.outputs.push(data_ref);
    }
}

impl Default for DtgMetrics {
    fn default() -> Self {
        Self {
            cpu_time_ms: 0,
            memory_bytes: 0,
            network_bytes: 0,
            disk_bytes: 0,
            retry_count: 0,
            quality_score: 1.0,
            confidence_score: 1.0,
            execution_time_ms: 0,
            throughput_ops_per_sec: 0.0,
            error_rate: 0.0,
            data_consistency_score: 1.0,
            schema_compliance_score: 1.0,
            business_value_score: 1.0,
            cost: 0.0,
            collected_at: chrono::Utc::now(),
        }
    }
}

impl DataTransformationGraph {
    /// Create a new empty DTG.
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            root_nodes: Vec::new(),
            nodes: HashMap::new(),
            edges: Vec::new(),
            graph_inputs: Vec::new(),
            graph_outputs: Vec::new(),
            metadata: HashMap::new(),
            started_at: chrono::Utc::now(),
            completed_at: None,
            status: DtgGraphStatus::Constructing,
            tags: Vec::new(),
        }
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: DtgNode) -> Uuid {
        let node_id = node.id;
        self.nodes.insert(node_id, node);
        node_id
    }

    /// Add an edge between nodes.
    pub fn add_edge(&mut self, source: Uuid, target: Uuid, data_ref: Uuid, edge_type: String) {
        let edge = DtgEdge {
            source,
            target,
            data_ref,
            edge_type,
            metadata: HashMap::new(),
        };
        self.edges.push(edge);
    }

    /// Mark the graph as ready for execution.
    pub fn mark_ready(&mut self) {
        self.status = DtgGraphStatus::Ready;
    }

    /// Mark the graph as executing.
    pub fn mark_executing(&mut self) {
        self.status = DtgGraphStatus::Executing;
        self.started_at = chrono::Utc::now();
    }

    /// Mark the graph as completed.
    pub fn mark_completed(&mut self) {
        self.status = DtgGraphStatus::Completed;
        self.completed_at = Some(chrono::Utc::now());
    }

    /// Validate the graph structure and data.
    pub fn validate_graph(&self) -> Result<(), validator::ValidationErrors> {
        // Validate the graph itself
        self.validate()?;

        // Validate all nodes
        for node in self.nodes.values() {
            node.validate()?;
        }

        // Validate all edges
        for edge in &self.edges {
            edge.validate()?;
        }

        Ok(())
    }

    /// Get all dependencies for a node.
    pub fn get_dependencies(&self, node_id: Uuid) -> Vec<Uuid> {
        self.edges
            .iter()
            .filter(|edge| edge.target == node_id)
            .map(|edge| edge.source)
            .collect()
    }

    /// Get all dependents for a node.
    pub fn get_dependents(&self, node_id: Uuid) -> Vec<Uuid> {
        self.edges
            .iter()
            .filter(|edge| edge.source == node_id)
            .map(|edge| edge.target)
            .collect()
    }

    /// Check if the graph is acyclic using petgraph.
    pub fn is_acyclic(&self) -> bool {
        // Build petgraph representation
        let mut graph = DiGraph::<Uuid, ()>::new();
        let mut node_indices = HashMap::new();

        // Add all nodes to the graph
        for node_id in self.nodes.keys() {
            let idx = graph.add_node(*node_id);
            node_indices.insert(*node_id, idx);
        }

        // Add all edges to the graph
        for edge in &self.edges {
            if let (Some(&source_idx), Some(&target_idx)) = (
                node_indices.get(&edge.source),
                node_indices.get(&edge.target),
            ) {
                graph.add_edge(source_idx, target_idx, ());
            }
        }

        // Use petgraph's cycle detection
        !is_cyclic_directed(&graph)
    }

    /// Get topological order of nodes using petgraph.
    pub fn topological_order(&self) -> Result<Vec<Uuid>, Vec<Uuid>> {
        // Build petgraph representation
        let mut graph = DiGraph::<Uuid, ()>::new();
        let mut node_indices = HashMap::new();

        // Add all nodes to the graph
        for node_id in self.nodes.keys() {
            let idx = graph.add_node(*node_id);
            node_indices.insert(*node_id, idx);
        }

        // Add all edges to the graph
        for edge in &self.edges {
            if let (Some(&source_idx), Some(&target_idx)) = (
                node_indices.get(&edge.source),
                node_indices.get(&edge.target),
            ) {
                graph.add_edge(source_idx, target_idx, ());
            }
        }

        // Get topological order
        match toposort(&graph, None) {
            Ok(order) => {
                let uuids: Vec<Uuid> = order.iter().map(|&idx| graph[idx]).collect();
                Ok(uuids)
            }
            Err(cycle) => {
                // Find the cycle nodes
                let mut cycle_nodes = Vec::new();
                let mut visited = HashSet::new();
                let mut current = cycle.node_id();

                while !visited.contains(&current) {
                    visited.insert(current);
                    cycle_nodes.push(graph[current]);

                    // Find an outgoing edge that's part of the cycle
                    for neighbor in graph.neighbors_directed(current, petgraph::Direction::Outgoing)
                    {
                        if !visited.contains(&neighbor) {
                            current = neighbor;
                            break;
                        }
                    }
                }

                Err(cycle_nodes)
            }
        }
    }
}

impl DtgProvenance {
    /// Create new provenance for a DTG.
    pub fn new(dtg_id: Uuid) -> Self {
        Self {
            dtg_id,
            transformation_chain: Vec::new(),
            input_lineage: Vec::new(),
            output_lineage: Vec::new(),
            agent_records: Vec::new(),
            signatures: Vec::new(),
            recorded_at: chrono::Utc::now(),
        }
    }

    /// Add a transformation record.
    pub fn add_transformation(&mut self, record: TransformationRecord) {
        self.transformation_chain.push(record);
    }

    /// Add a cryptographic signature.
    pub fn add_signature(&mut self, signer: String, algorithm: String, signature: String) {
        let crypto_signature = CryptographicSignature {
            signer,
            algorithm,
            signature,
            public_key: None,
            signed_at: chrono::Utc::now(),
        };
        self.signatures.push(crypto_signature);
    }

    /// Verify all signatures in the provenance.
    pub fn verify_signatures(&self) -> bool {
        // In a real implementation, this would verify cryptographic signatures
        // For now, return true if we have any signatures
        !self.signatures.is_empty()
    }
}
