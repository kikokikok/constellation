# Data Transformation Graph (DTG) User Guide

**Version:** 1.0.0  
**Last Updated:** December 14, 2025

## Overview

The Data Transformation Graph (DTG) is a powerful workflow engine that allows you to define, execute, and track complex data transformation pipelines. DTG provides cryptographic provenance, quality scoring, and agent-based execution.

## Quick Start

### 1. Installation

Add Constellation to your `Cargo.toml`:

```toml
[dependencies]
constellation-core = { git = "https://github.com/your-org/constellation" }
```

### 2. Create Your First DTG

```rust
use constellation_core::models::dtg::{DataTransformationGraph, DtgNode, DtgDataRef};
use uuid::Uuid;

fn create_simple_dtg() -> DataTransformationGraph {
    let mut dtg = DataTransformationGraph::new(
        "My First Pipeline".to_string(),
        "A simple data processing pipeline".to_string(),
    );
    
    // Create data source node
    let source_node = DtgNode::new(
        "Load Data".to_string(),
        "Load data from source".to_string(),
        DtgDataRef::new("csv_data".to_string(), "s3://bucket/data.csv".to_string()),
    );
    
    // Create transformation node
    let transform_node = DtgNode::new(
        "Clean Data".to_string(),
        "Clean and preprocess data".to_string(),
        DtgDataRef::new("cleaned_data".to_string(), "memory".to_string()),
    );
    
    // Add nodes to DTG
    dtg.add_node(source_node);
    dtg.add_node(transform_node);
    
    // Add edge between nodes
    dtg.add_edge(source_node.id, transform_node.id);
    
    dtg
}
```

### 3. Execute the DTG

```rust
use constellation_core::dtg::engine::DtgExecutionEngine;

fn execute_dtg() -> Result<(), Box<dyn std::error::Error>> {
    let dtg = create_simple_dtg();
    
    // Create execution engine with custom executor
    let execution_fn = Box::new(|node: &DtgNode| -> Result<DtgMetrics, String> {
        println!("Executing node: {}", node.name);
        
        // Your execution logic here
        Ok(DtgMetrics {
            quality_score: 0.95,
            execution_time_ms: 500,
            cost: 0.05,
            ..Default::default()
        })
    });
    
    let mut engine = DtgExecutionEngine::new_with_executor(dtg, execution_fn);
    
    // Execute all nodes
    engine.execute_all()?;
    
    // Get execution results
    let metrics = engine.get_metrics();
    println!("Total quality: {}", metrics.quality_score);
    println!("Total cost: ${}", metrics.cost);
    
    Ok(())
}
```

## Core Concepts

### Nodes

Nodes represent individual transformation steps in your pipeline. Each node has:

- **ID**: Unique identifier (UUID)
- **Name**: Human-readable name
- **Description**: What the node does
- **Data Reference**: Source or destination of data
- **Status**: Current execution state
- **Metrics**: Execution results

### Edges

Edges define dependencies between nodes. They create a directed acyclic graph (DAG) that determines execution order.

### Data References

Data references describe where data comes from or goes to:

```rust
let data_ref = DtgDataRef {
    id: Uuid::new_v4(),
    data_type: "csv".to_string(),
    location: "s3://bucket/data.csv".to_string(),
    hash: "sha256:abc123...".to_string(),  // Cryptographic hash
    size_bytes: 1024 * 1024,  // 1MB
    metadata: HashMap::new(),
};
```

### Metrics

Each node execution produces metrics:

- **Quality Score**: 0.0 to 1.0 (higher is better)
- **Execution Time**: Milliseconds
- **Cost**: Monetary cost (e.g., $0.10)
- **Resource Usage**: CPU, memory, network
- **Error Rate**: Percentage of failures

## Advanced Usage

### 1. Agent-Based Execution

Execute DTG nodes using AI agents:

```rust
use constellation_core::integration::DtgAgentIntegrationEngine;

async fn execute_with_agents() -> Result<(), Box<dyn std::error::Error>> {
    let dtg = create_complex_dtg();
    let dtg_engine = DtgExecutionEngine::new(dtg);
    
    // Create agent coordinator
    let coordinator = LlmStrategistCoordinator::default();
    
    // Create integration engine
    let integration = DtgAgentIntegrationEngine::new(dtg_engine, coordinator);
    
    // Register agent skills
    let skills = vec![
        AgentSkill {
            name: "data_cleaning".to_string(),
            level: 0.9,
            description: "Clean and preprocess data".to_string(),
            ..Default::default()
        },
        AgentSkill {
            name: "analysis".to_string(),
            level: 0.8,
            description: "Data analysis and insights".to_string(),
            ..Default::default()
        },
    ];
    
    integration.register_agent_skills("data_scientist", skills)?;
    
    // Execute nodes with agents
    for node_id in integration.get_pending_nodes() {
        let result = integration.execute_node_with_agent(node_id, "data_scientist")?;
        println!("Node {} executed with quality: {}", node_id, result.quality_score);
    }
    
    Ok(())
}
```

### 2. Quality Scoring

Implement custom quality scoring:

```rust
fn custom_quality_scorer(node: &DtgNode, result: &ExecutionResult) -> f64 {
    // Base quality from execution
    let mut quality = result.success_rate;
    
    // Penalize for high cost
    if result.cost > 1.0 {
        quality *= 0.9;
    }
    
    // Reward for fast execution
    if result.execution_time_ms < 1000 {
        quality *= 1.1;
    }
    
    // Cap at 1.0
    quality.min(1.0)
}
```

### 3. Provenance Tracking

DTG automatically tracks cryptographic provenance:

```rust
// Get provenance for a node
let provenance = dtg_engine.get_provenance(node_id)?;

println!("Node: {}", provenance.node_id);
println!("Input Hash: {}", provenance.input_hash);
println!("Output Hash: {}", provenance.output_hash);
println!("Executor: {}", provenance.executor_id);
println!("Timestamp: {}", provenance.timestamp);

// Verify provenance
let is_valid = dtg_engine.verify_provenance(&provenance)?;
println!("Provenance valid: {}", is_valid);
```

### 4. Error Handling and Retry

```rust
use constellation_core::models::dtg::DtgError;

async fn execute_with_retry(
    engine: &mut DtgExecutionEngine,
    node_id: Uuid,
    max_retries: usize,
) -> Result<DtgMetrics, DtgError> {
    let mut retries = 0;
    
    loop {
        match engine.execute_node(node_id) {
            Ok(metrics) => return Ok(metrics),
            Err(DtgError::ExecutionError(e)) if retries < max_retries => {
                retries += 1;
                println!("Retry {} for node {}: {}", retries, node_id, e);
                tokio::time::sleep(Duration::from_secs(2u64.pow(retries))).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}
```

## Real-World Examples

### Example 1: Data Pipeline

```rust
fn create_data_pipeline() -> DataTransformationGraph {
    let mut dtg = DataTransformationGraph::new(
        "Customer Analytics Pipeline".to_string(),
        "Process customer data for analytics".to_string(),
    );
    
    // Define nodes
    let nodes = vec![
        ("ingest", "Ingest raw data", "s3://raw/customers.csv"),
        ("clean", "Clean data", "memory"),
        ("enrich", "Enrich with external data", "memory"),
        ("aggregate", "Aggregate metrics", "memory"),
        ("export", "Export to data warehouse", "redshift://warehouse"),
    ];
    
    // Add nodes and edges
    let mut prev_id = None;
    
    for (name, description, location) in nodes {
        let node = DtgNode::new(
            name.to_string(),
            description.to_string(),
            DtgDataRef::new("data".to_string(), location.to_string()),
        );
        
        let node_id = node.id;
        dtg.add_node(node);
        
        if let Some(prev) = prev_id {
            dtg.add_edge(prev, node_id);
        }
        
        prev_id = Some(node_id);
    }
    
    dtg
}
```

### Example 2: Machine Learning Pipeline

```rust
fn create_ml_pipeline() -> DataTransformationGraph {
    let mut dtg = DataTransformationGraph::new(
        "ML Model Training".to_string(),
        "Train and evaluate machine learning model".to_string(),
    );
    
    // Training pipeline
    let train_nodes = vec![
        ("load_data", "Load training data"),
        ("preprocess", "Preprocess features"),
        ("train", "Train model"),
        ("validate", "Validate model"),
    ];
    
    // Inference pipeline
    let infer_nodes = vec![
        ("load_new", "Load new data"),
        ("preprocess_infer", "Preprocess for inference"),
        ("predict", "Make predictions"),
        ("export_results", "Export predictions"),
    ];
    
    // Add training nodes
    let mut train_ids = Vec::new();
    for (name, description) in train_nodes {
        let node = DtgNode::new(
            name.to_string(),
            description.to_string(),
            DtgDataRef::new("data".to_string(), "memory".to_string()),
        );
        train_ids.push(node.id);
        dtg.add_node(node);
    }
    
    // Add inference nodes
    let mut infer_ids = Vec::new();
    for (name, description) in infer_nodes {
        let node = DtgNode::new(
            name.to_string(),
            description.to_string(),
            DtgDataRef::new("data".to_string(), "memory".to_string()),
        );
        infer_ids.push(node.id);
        dtg.add_node(node);
    }
    
    // Connect training pipeline
    for i in 0..train_ids.len() - 1 {
        dtg.add_edge(train_ids[i], train_ids[i + 1]);
    }
    
    // Connect inference pipeline
    for i in 0..infer_ids.len() - 1 {
        dtg.add_edge(infer_ids[i], infer_ids[i + 1]);
    }
    
    // Connect trained model to inference
    dtg.add_edge(train_ids[2], infer_ids[2]);  // trained model → predict
    
    dtg
}
```

## Best Practices

### 1. Node Design

- **Keep nodes focused**: Each node should do one thing well
- **Define clear inputs/outputs**: Use descriptive data references
- **Set reasonable timeouts**: Prevent hanging executions
- **Include error handling**: Nodes should handle failures gracefully

### 2. Graph Design

- **Avoid cycles**: DTG must be a directed acyclic graph (DAG)
- **Limit fan-out**: Too many dependencies can be hard to manage
- **Use subgraphs**: Break complex pipelines into manageable pieces
- **Document dependencies**: Clearly document why edges exist

### 3. Execution Strategy

- **Parallel execution**: Execute independent nodes concurrently
- **Resource awareness**: Consider CPU/memory constraints
- **Cost optimization**: Balance speed vs. cost
- **Quality monitoring**: Track and improve quality scores over time

### 4. Monitoring and Debugging

```rust
// Monitor DTG execution
fn monitor_dtg(engine: &DtgExecutionEngine) {
    println!("=== DTG Status ===");
    println!("Total Nodes: {}", engine.get_node_count());
    println!("Completed: {}", engine.get_completed_count());
    println!("Failed: {}", engine.get_failed_count());
    println!("Pending: {}", engine.get_pending_count());
    
    // Node status
    for (node_id, status) in engine.get_node_statuses() {
        println!("  {}: {:?}", node_id, status);
    }
    
    // Metrics
    let metrics = engine.get_metrics();
    println!("Overall Quality: {:.3}", metrics.quality_score);
    println!("Total Cost: ${:.2}", metrics.cost);
    println!("Total Time: {}ms", metrics.execution_time_ms);
}
```

## Troubleshooting

### Common Issues

1. **Cycle Detection Error**
   - **Cause**: Graph contains a cycle
   - **Solution**: Review edges and ensure DAG structure

2. **Node Execution Failure**
   - **Cause**: Execution function returns error
   - **Solution**: Check execution logic and error handling

3. **Memory Issues**
   - **Cause**: Large graphs or data
   - **Solution**: Use streaming or chunked processing

4. **Performance Problems**
   - **Cause**: Too many sequential dependencies
   - **Solution**: Identify and parallelize independent nodes

### Debugging Tips

```rust
// Enable debug logging
use tracing::Level;
use tracing_subscriber;

fn setup_debug_logging() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
}

// Add detailed logging to execution
let execution_fn = Box::new(|node: &DtgNode| -> Result<DtgMetrics, String> {
    tracing::debug!("Executing node: {} ({})", node.name, node.id);
    
    // Your execution logic with detailed error reporting
    match execute_node_logic(node) {
        Ok(result) => {
            tracing::debug!("Node {} completed successfully", node.id);
            Ok(result)
        }
        Err(e) => {
            tracing::error!("Node {} failed: {}", node.id, e);
            Err(e)
        }
    }
});
```

## Performance Optimization

### 1. Parallel Execution

```rust
use tokio::task;

async fn execute_parallel(engine: &DtgExecutionEngine) -> Result<(), Box<dyn std::error::Error>> {
    // Get independent nodes (no dependencies on each other)
    let independent_nodes = engine.get_independent_nodes();
    
    // Execute in parallel
    let tasks: Vec<_> = independent_nodes
        .into_iter()
        .map(|node_id| {
            let engine_clone = engine.clone();
            task::spawn(async move {
                engine_clone.execute_node(node_id)
            })
        })
        .collect();
    
    // Wait for all tasks
    for task in tasks {
        let _ = task.await?;
    }
    
    Ok(())
}
```

### 2. Caching Intermediate Results

```rust
use std::collections::HashMap;

struct CachedDtgEngine {
    engine: DtgExecutionEngine,
    cache: HashMap<Uuid, DtgMetrics>,
}

impl CachedDtgEngine {
    fn execute_node_cached(&mut self, node_id: Uuid) -> Result<DtgMetrics, DtgError> {
        // Check cache first
        if let Some(metrics) = self.cache.get(&node_id) {
            return Ok(metrics.clone());
        }
        
        // Execute and cache
        let metrics = self.engine.execute_node(node_id)?;
        self.cache.insert(node_id, metrics.clone());
        
        Ok(metrics)
    }
}
```

### 3. Resource Management

```rust
struct ResourceAwareExecutor {
    max_concurrent: usize,
    semaphore: tokio::sync::Semaphore,
}

impl ResourceAwareExecutor {
    async fn execute_with_limits(
        &self,
        node_ids: Vec<Uuid>,
        engine: DtgExecutionEngine,
    ) -> Result<(), DtgError> {
        let tasks: Vec<_> = node_ids
            .into_iter()
            .map(|node_id| {
                let engine_clone = engine.clone();
                let permit = self.semaphore.clone().acquire_owned();
                
                tokio::spawn(async move {
                    let _permit = permit.await;
                    engine_clone.execute_node(node_id)
                })
            })
            .collect();
        
        for task in tasks {
            let _ = task.await??;
        }
        
        Ok(())
    }
}
```

## Integration with Other Systems

### 1. Database Integration

```rust
use sqlx::{PgPool, postgres::PgPoolOptions};

struct DatabaseBackedDtg {
    pool: PgPool,
}

impl DatabaseBackedDtg {
    async fn save_to_db(&self, dtg: &DataTransformationGraph) -> Result<(), sqlx::Error> {
        // Save DTG to PostgreSQL
        let json = serde_json::to_value(dtg)?;
        
        sqlx::query!(
            r#"
            INSERT INTO dtg_graphs (id, name, graph_data)
            VALUES ($1, $2, $3)
            "#,
            dtg.id,
            dtg.name,
            json
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn load_from_db(&self, dtg_id: Uuid) -> Result<DataTransformationGraph, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT graph_data FROM dtg_graphs WHERE id = $1
            "#,
            dtg_id
        )
        .fetch_one(&self.pool)
        .await?;
        
        let dtg: DataTransformationGraph = serde_json::from_value(row.graph_data)?;
        Ok(dtg)
    }
}
```

### 2. Cloud Storage Integration

```rust
use aws_sdk_s3::Client as S3Client;

struct S3BackedDtg {
    s3_client: S3Client,
    bucket: String,
}

impl S3BackedDtg {
    async fn save_to_s3(&self, dtg: &DataTransformationGraph) -> Result<(), Box<dyn std::error::Error>> {
        let key = format!("dtg/{}.json", dtg.id);
        let body = serde_json::to_vec(dtg)?.into();
        
        self.s3_client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(body)
            .send()
            .await?;
        
        Ok(())
    }
}
```

## Next Steps

1. **Explore examples**: Check the `/examples` directory
2. **Read API docs**: Run `cargo doc --open`
3. **Join community**: GitHub discussions and Discord
4. **Contribute**: Submit issues and pull requests

## Support

- **Documentation**: This guide and API reference
- **Examples**: Complete working examples
- **Issues**: GitHub issue tracker
- **Questions**: GitHub discussions

---

**Happy transforming!** 🚀