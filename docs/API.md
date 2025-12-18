# Constellation Platform API Documentation

**Version:** 1.0.0  
**Last Updated:** December 14, 2025

## Overview

Constellation is a Rust-based multi-agent platform with integrated DTG (Data Transformation Graph), MCP security, hybrid agents, and autonomy measurement. This document describes the API for the integrated platform.

## Table of Contents

1. [Core Concepts](#core-concepts)
2. [Data Transformation Graph (DTG) API](#data-transformation-graph-dtg-api)
3. [MCP Security API](#mcp-security-api)
4. [Hybrid Agent API](#hybrid-agent-api)
5. [Autonomy Measurement API](#autonomy-measurement-api)
6. [Integration API](#integration-api)
7. [Examples](#examples)
8. [Error Handling](#error-handling)

## Core Concepts

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Constellation Platform                    │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │   DTG    │  │   MCP    │  │  Hybrid  │  │ Autonomy │   │
│  │  Engine  │◄─┤ Security │◄─┤  Agents  │◄─┤  Engine  │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
│         │            │             │              │        │
│         └────────────┴─────────────┴──────────────┘        │
│                    Integration Layer                        │
└─────────────────────────────────────────────────────────────┘
```

### Key Components

1. **DTG Engine**: Data Transformation Graph for workflow definition and execution
2. **MCP Security**: Model Context Protocol for cryptographic security
3. **Hybrid Agents**: LLM strategist + SLM executor architecture
4. **Autonomy Measurement**: Kardashev-style autonomy scale with κ scoring
5. **Integration Layer**: Connects all components into cohesive workflows

## Data Transformation Graph (DTG) API

### DTG Graph Management

```rust
// Create a new DTG
let dtg = DataTransformationGraph::new(
    "Research Pipeline".to_string(),
    "Research workflow for AI autonomy measurement".to_string(),
);

// Add nodes to DTG
let node = DtgNode::new(
    "Literature Review".to_string(),
    "Review existing research".to_string(),
    DtgDataRef::new("research_papers".to_string(), "database".to_string()),
);

dtg.add_node(node);

// Execute DTG
let dtg_engine = DtgExecutionEngine::new(dtg);
let metrics = dtg_engine.execute_node(node_id)?;
```

### DTG Node Execution

```rust
// Custom execution function
let execution_fn = Box::new(|node: &DtgNode| -> Result<DtgMetrics, String> {
    // Execute node logic
    Ok(DtgMetrics {
        quality_score: 0.9,
        execution_time_ms: 100,
        cost: 0.1,
        ..Default::default()
    })
});

let dtg_engine = DtgExecutionEngine::new_with_executor(dtg, execution_fn);
```

### DTG Metrics

```rust
// Access DTG metrics
let metrics = dtg_engine.get_metrics();
println!("Quality Score: {}", metrics.quality_score);
println!("Execution Time: {}ms", metrics.execution_time_ms);
println!("Cost: ${}", metrics.cost);
```

## MCP Security API

### Key Management

```rust
// Create MCP crypto instance
let mut crypto = McpCrypto::new()?;

// Generate key pairs
let (private_key_id, public_key_id) = crypto.generate_key_pair(
    "Ed25519",          // Algorithm
    "agent1",           // Owner
    KeyUsage::Signing,  // Usage
)?;

// Generate encryption key
let (enc_key_id, _) = crypto.generate_key_pair(
    "AES-256-GCM",
    "agent1",
    KeyUsage::Encryption,
)?;
```

### Encryption & Signing

```rust
// Encrypt data
let encrypted_message = crypto.encrypt(
    &enc_key_id,        // Key ID
    b"Secret message",  // Data
    "AES-256-GCM",      // Algorithm
)?;

// Create signature
let signature = crypto.create_signature(
    &private_key_id,    // Signing key
    "agent1",           // Signer
    "Ed25519",          // Algorithm
    b"Data to sign",    // Data
)?;

// Verify signature
let verified = crypto.verify_signature(&signature, b"Data to sign")?;
```

### Secure Envelopes

```rust
// Create secure envelope
let envelope = crypto.create_secure_envelope(
    &sender_private_key_id,
    &recipient_public_key_id,
    "sender@example.com",
    "recipient@example.com",
    "task_assignment",
    b"Task payload",
    "AES-256-GCM",
    "Ed25519",
)?;

// Verify and decrypt envelope
let decrypted = crypto.verify_and_decrypt_envelope(
    &envelope,
    &recipient_private_key_id,
    &sender_public_key_id,
)?;
```

## Hybrid Agent API

### Agent Configuration

```rust
// Create hybrid agent configuration
let agent_config = HybridAgentConfig {
    id: Uuid::new_v4(),
    name: "Research Agent".to_string(),
    description: "Agent for research tasks".to_string(),
    strategist: StrategistConfig {
        model_id: "gpt-4".to_string(),
        provider: ModelProvider::Openai,
        model_size: ModelSize::Large,
        context_window: 8192,
        temperature: 0.7,
        max_tokens: 2048,
        cost_per_1k_tokens: 0.03,
        latency_target_ms: 1000,
        ..Default::default()
    },
    executors: vec![
        ExecutorConfig {
            id: "data_analyzer".to_string(),
            domain: ExecutorDomain::DataAnalysis,
            model_id: "claude-3-haiku".to_string(),
            provider: ModelProvider::Anthropic,
            model_size: ModelSize::Small,
            specialization: "data_analysis".to_string(),
            max_concurrent_tasks: 5,
            ..Default::default()
        },
    ],
    coordination: CoordinationStrategy::Hierarchical,
    ..Default::default()
};
```

### Task Management

```rust
// Create coordinator
let coordinator = LlmStrategistCoordinator::new(
    ExecutorStats::default(),
    PerformanceMetrics::default(),
    QueueStats::default(),
);

// Submit task
let task = Task::new(
    "research_planning".to_string(),
    json!({
        "research_topic": "AI autonomy measurement",
        "objectives": ["Define metrics", "Implement measurement"]
    }),
).with_priority(75);

let task_id = coordinator.submit_task(task)?;

// Assign tasks
let assignments = coordinator.assign_tasks()?;

// Complete task
let result = TaskResult {
    task_id,
    executor_id: "data_analyzer".to_string(),
    completed_at: Utc::now(),
    result: json!({"plan": "Research plan created"}),
    success: true,
    quality_score: 0.88,
    execution_time_ms: 1800000,
    cost: 0.25,
    ..Default::default()
};

coordinator.complete_task(result)?;
```

### Performance Monitoring

```rust
// Get performance metrics
let metrics = coordinator.get_performance_metrics();
println!("Success Rate: {:.2}%", metrics.success_rate * 100.0);
println!("Average Latency: {}ms", metrics.avg_latency_ms);
println!("Throughput: {:.2} tasks/sec", metrics.throughput_tps);

// Get queue stats
let stats = coordinator.get_queue_stats();
println!("Pending Tasks: {}", stats.pending_tasks);
println!("Active Tasks: {}", stats.active_tasks);
println!("Completed Tasks: {}", stats.completed_tasks);
```

## Autonomy Measurement API

### Autonomy Engine

```rust
// Create autonomy measurement engine
let autonomy_engine = AutonomyMeasurementEngine::new();

// Measure agent autonomy
let measurement = autonomy_engine.measure_agent(
    "agent1".to_string(),
    &agent_capabilities,
    &task_history,
)?;

println!("Autonomy Level: {}", measurement.autonomy_level);
println!("κ Score: {:.3}", measurement.kappa_score);
println!("Progress: {:.1}%", measurement.progress_percentage);

// Get capability scores
for (axis, score) in &measurement.capability_scores {
    println!("{}: {:.3}", axis, score);
}
```

### Self-Assessment

```rust
// Create self-assessment engine
let self_assessment = SelfAssessmentEngine::new();

// Assess accuracy
let accuracy = self_assessment.assess_accuracy(
    &predicted_scores,
    &actual_scores,
)?;

println!("Self-Assessment Accuracy: {:.2}%", accuracy * 100.0);

// Get calibration curve
let calibration = self_assessment.get_calibration_curve();
```

### Collaboration Detection

```rust
// Create collaboration detector
let collaboration_detector = CollaborationPatternDetector::new();

// Detect collaboration patterns
let patterns = collaboration_detector.detect_patterns(
    &agent_interactions,
    &task_dependencies,
)?;

for pattern in patterns {
    println!("Pattern: {}", pattern.pattern_type);
    println!("Strength: {:.3}", pattern.strength);
    println!("Agents: {:?}", pattern.agents_involved);
}
```

### Open-World Research

```rust
// Create open-world research environment
let open_world_config = OpenWorldConfig {
    max_concurrent_experiments: 5,
    collaboration_enabled: true,
    peer_review_enabled: true,
    discovery_validation_required: true,
    exploration_exploitation_ratio: 0.3,
    minimum_evidence_strength: 0.7,
    minimum_reproducibility: 0.8,
    ..Default::default()
};

let open_world = OpenWorldResearchEnvironment::new(open_world_config);

// Submit discovery
let discovery = Discovery {
    id: Uuid::new_v4(),
    title: "New autonomy metric".to_string(),
    description: "Discovered more accurate way to measure agent autonomy".to_string(),
    evidence_strength: 0.85,
    reproducibility: 0.9,
    submitted_by: "agent1".to_string(),
    submitted_at: Utc::now(),
    status: DiscoveryStatus::UnderReview,
    ..Default::default()
};

open_world.submit_discovery(discovery)?;

// Get validated discoveries
let discoveries = open_world.get_validated_discoveries();
```

## Integration API

### DTG-Agent Integration

```rust
// Create DTG-agent integration engine
let dtg_integration = DtgAgentIntegrationEngine::new(dtg_engine, coordinator);

// Register agent skills
let skills = vec![
    AgentSkill {
        name: "research".to_string(),
        level: 0.9,
        description: "Research and analysis".to_string(),
        ..Default::default()
    },
];

dtg_integration.register_agent_skills("agent1", skills)?;

// Execute DTG node with agent
let result = dtg_integration.execute_node_with_agent(node_id, "agent1")?;
```

### MCP Security Integration

```rust
// Create MCP security integration
let security_integration = McpSecurityIntegration::new()?;

// Register agent with security
security_integration.register_agent(
    "agent1".to_string(),
    &agent_public_key_id,
    &access_policies,
)?;

// Send secure message
let secure_message = security_integration.send_secure_message(
    "agent1".to_string(),
    "agent2".to_string(),
    "task_result".to_string(),
    b"Task completed successfully",
)?;
```

### Hybrid A2A Integration

```rust
// Create hybrid A2A integration
let hybrid_a2a_integration = HybridA2AIntegration::new(coordinator, security_integration);

// Register hybrid agent
hybrid_a2a_integration.register_hybrid_agent(
    &agent_config,
    &a2a_interface,
)?;

// Send A2A message
let a2a_message = A2AMessage {
    sender: "agent1".to_string(),
    recipient: "agent2".to_string(),
    message_type: "task_coordination".to_string(),
    payload: json!({"task_id": "123", "action": "start"}),
    timestamp: Utc::now(),
    ..Default::default()
};

hybrid_a2a_integration.send_a2a_message(a2a_message)?;
```

### Autonomy Integration

```rust
// Create autonomy integration engine
let autonomy_integration = AutonomyIntegrationEngine::new(
    autonomy_engine,
    self_assessment,
    collaboration_detector,
    open_world,
)
.with_dtg_integration(dtg_integration)
.with_hybrid_a2a_integration(hybrid_a2a_integration);

// Track task execution for autonomy measurement
autonomy_integration.track_task_execution(&task, &task_result)?;

// Get autonomy recommendations
let recommendations = autonomy_integration.get_improvement_recommendations("agent1")?;

for recommendation in recommendations {
    println!("Recommendation: {}", recommendation.description);
    println!("Priority: {}", recommendation.priority);
    println!("Expected Impact: {:.1}%", recommendation.expected_impact * 100.0);
}
```

## Examples

### Complete Workflow Example

```rust
use constellation_core::integration::{
    DtgAgentIntegrationEngine, McpSecurityIntegration, HybridA2AIntegration, AutonomyIntegrationEngine,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup all components
    let dtg = create_research_dtg();
    let dtg_engine = DtgExecutionEngine::new(dtg);
    
    let coordinator = LlmStrategistCoordinator::default();
    let autonomy_engine = AutonomyMeasurementEngine::new();
    
    // 2. Create integration engines
    let dtg_integration = DtgAgentIntegrationEngine::new(dtg_engine, coordinator);
    let security_integration = McpSecurityIntegration::new()?;
    let hybrid_a2a_integration = HybridA2AIntegration::new(
        LlmStrategistCoordinator::default(),
        security_integration,
    );
    
    let autonomy_integration = AutonomyIntegrationEngine::new(
        autonomy_engine,
        SelfAssessmentEngine::new(),
        CollaborationPatternDetector::new(),
        OpenWorldResearchEnvironment::default(),
    )
    .with_dtg_integration(dtg_integration)
    .with_hybrid_a2a_integration(hybrid_a2a_integration);
    
    // 3. Execute complete workflow
    println!("Starting research workflow...");
    
    // Execute DTG nodes with agents
    // Secure agent communications
    // Measure autonomy throughout
    // Integrate discoveries
    
    println!("Workflow completed successfully!");
    
    Ok(())
}
```

### Error Handling Example

```rust
use constellation_core::integration::IntegrationError;

async fn execute_workflow() -> Result<(), IntegrationError> {
    match autonomy_integration.track_task_execution(&task, &task_result) {
        Ok(_) => {
            println!("Task tracked successfully");
            Ok(())
        }
        Err(IntegrationError::SecurityError(e)) => {
            eprintln!("Security error: {}", e);
            Err(IntegrationError::SecurityError(e))
        }
        Err(IntegrationError::AutonomyError(e)) => {
            eprintln!("Autonomy measurement error: {}", e);
            Err(IntegrationError::AutonomyError(e))
        }
        Err(e) => {
            eprintln!("Integration error: {}", e);
            Err(e)
        }
    }
}
```

## Error Handling

### Error Types

```rust
pub enum IntegrationError {
    /// DTG-related errors
    DtgError(String),
    
    /// Security-related errors
    SecurityError(String),
    
    /// Agent-related errors
    AgentError(String),
    
    /// Autonomy measurement errors
    AutonomyError(String),
    
    /// Configuration errors
    ConfigurationError(String),
    
    /// Network/communication errors
    CommunicationError(String),
    
    /// Resource exhaustion errors
    ResourceError(String),
    
    /// Timeout errors
    TimeoutError(String),
}
```

### Error Recovery

```rust
// Retry with exponential backoff
async fn execute_with_retry<F, T>(mut operation: F) -> Result<T, IntegrationError>
where
    F: FnMut() -> Result<T, IntegrationError>,
{
    let mut retries = 0;
    let max_retries = 3;
    
    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if retries < max_retries => {
                retries += 1;
                let delay = std::time::Duration::from_secs(2u64.pow(retries));
                tokio::time::sleep(delay).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}
```

## Performance Considerations

### Memory Usage

- DTG graphs: ~1KB per node
- Agent configurations: ~500 bytes per agent
- Security keys: 32-64 bytes per key
- Autonomy measurements: ~100 bytes per measurement

### Latency

- DTG node execution: < 100ms
- Encryption/decryption: < 5ms
- Signature verification: < 2ms
- Autonomy measurement: < 10ms
- Complete workflow: < 1s for typical research tasks

### Scalability

- Supports 1000+ concurrent agents
- Handles 10,000+ DTG nodes
- Processes 100+ tasks per second
- Scales horizontally with Redis clustering

## Security Best Practices

1. **Always use MCP security** for agent communications
2. **Rotate keys regularly** (90 days for signing, 180 days for encryption)
3. **Validate all inputs** before processing
4. **Use secure random number generation** for nonces and keys
5. **Implement rate limiting** to prevent DoS attacks
6. **Audit log all security events**
7. **Regularly update dependencies** for security patches

## Getting Help

- **Documentation:** This API reference
- **Examples:** See `/examples` directory
- **Issues:** GitHub issue tracker
- **Security:** Report security issues to security@constellation.example.com

---

**Copyright © 2025 Constellation Platform**  
**License:** MIT  
**Version:** 1.0.0