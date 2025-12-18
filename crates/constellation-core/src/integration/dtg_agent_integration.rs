//! Integration between DTG execution and agent task execution.
//!
//! This module connects Data Transformation Graphs with agent execution,
//! allowing DTG nodes to be executed by agents and tracking the results
//! back to the DTG for provenance and quality scoring.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::dtg::engine::DtgExecutionEngine;
use crate::hybrid::coordinator::{LlmStrategistCoordinator, Task, TaskResult};
use crate::models::agent::AgentSkill;
use crate::models::dtg::{DtgNode, DtgNodeStatus};

/// Integration engine that connects DTG execution with agent task execution.
pub struct DtgAgentIntegrationEngine {
    /// DTG execution engine.
    dtg_engine: Arc<RwLock<DtgExecutionEngine>>,

    /// Agent coordinator for task execution.
    agent_coordinator: Arc<RwLock<LlmStrategistCoordinator>>,

    /// Mapping from DTG node IDs to agent task IDs.
    node_to_task_map: Arc<RwLock<HashMap<Uuid, Uuid>>>,

    /// Mapping from agent task IDs to DTG node IDs.
    task_to_node_map: Arc<RwLock<HashMap<Uuid, Uuid>>>,

    /// Agent skill registry for matching nodes to agents.
    skill_registry: Arc<RwLock<HashMap<String, Vec<AgentSkill>>>>,
}

impl DtgAgentIntegrationEngine {
    /// Create a new DTG-agent integration engine.
    pub fn new(
        dtg_engine: DtgExecutionEngine,
        agent_coordinator: LlmStrategistCoordinator,
    ) -> Self {
        Self {
            dtg_engine: Arc::new(RwLock::new(dtg_engine)),
            agent_coordinator: Arc::new(RwLock::new(agent_coordinator)),
            node_to_task_map: Arc::new(RwLock::new(HashMap::new())),
            task_to_node_map: Arc::new(RwLock::new(HashMap::new())),
            skill_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an agent skill for DTG node matching.
    pub async fn register_agent_skill(&self, agent_id: &str, skill: AgentSkill) {
        let mut registry = self.skill_registry.write().await;
        registry
            .entry(agent_id.to_string())
            .or_insert_with(Vec::new)
            .push(skill);
    }

    /// Find the best agent for a DTG node based on skills.
    pub async fn find_best_agent_for_node(&self, node: &DtgNode) -> Option<String> {
        let registry = self.skill_registry.read().await;

        // Extract node requirements from metadata
        let node_requirements: HashSet<String> = node
            .metadata
            .get("required_skills")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if node_requirements.is_empty() {
            // If no specific requirements, find any available agent
            return registry.keys().next().cloned();
        }

        // Find agent with the best skill match
        let mut best_agent = None;
        let mut best_match_score = 0.0;

        for (agent_id, skills) in registry.iter() {
            let agent_skills: HashSet<String> = skills
                .iter()
                .map(|skill| skill.name.to_lowercase())
                .collect();

            let matched_skills: HashSet<_> = node_requirements
                .iter()
                .filter(|req| agent_skills.contains(&req.to_lowercase()))
                .collect();

            let match_score = matched_skills.len() as f64 / node_requirements.len() as f64;

            if match_score > best_match_score {
                best_match_score = match_score;
                best_agent = Some(agent_id.clone());
            }
        }

        best_agent
    }

    /// Convert a DTG node to an agent task.
    pub async fn convert_node_to_task(&self, node_id: Uuid) -> Result<Task, String> {
        let dtg_engine = self.dtg_engine.read().await;
        let node = dtg_engine
            .graph()
            .nodes
            .get(&node_id)
            .ok_or_else(|| format!("Node {node_id} not found"))?;

        // Find the best agent for this node
        let best_agent = self
            .find_best_agent_for_node(node)
            .await
            .ok_or_else(|| "No suitable agent found for node".to_string())?;

        // Extract task parameters from node metadata
        let task_type = node
            .metadata
            .get("task_type")
            .and_then(|v| v.as_str())
            .unwrap_or("data_processing")
            .to_string();

        let input_data = node
            .metadata
            .get("input_data")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        let expected_output = node.metadata.get("expected_output").cloned();

        let timeout_ms = node
            .metadata
            .get("timeout_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(30000) as u32;

        let priority = node
            .metadata
            .get("priority")
            .and_then(|v| v.as_u64())
            .unwrap_or(5) as u32;

        Ok(Task {
            id: Uuid::new_v4(),
            task_type,
            input: input_data,
            expected_output,
            assigned_to: Some(best_agent),
            priority,
            timeout_ms,
            created_at: chrono::Utc::now(),
            status: crate::hybrid::coordinator::TaskStatus::Pending,
            metadata: node.metadata.clone(),
            deadline: None,
            quality_requirement: 0.8,
            budget_allocation: 1.0,
            resource_requirements: crate::hybrid::coordinator::ResourceRequirements {
                min_cpu_cores: 1,
                min_memory_mb: 1024,
                gpu_memory_mb: None,
                network_mbps: 10,
            },
        })
    }

    /// Execute a DTG node using an agent.
    pub async fn execute_node_with_agent(&self, node_id: Uuid) -> Result<Uuid, String> {
        // Convert node to task
        let task = self.convert_node_to_task(node_id).await?;
        let task_id = task.id;

        // Submit task to agent coordinator
        let coordinator = self.agent_coordinator.write().await;
        coordinator
            .submit_task(task)
            .map_err(|e| format!("Failed to submit task: {e}"))?;

        // Store mappings
        let mut node_to_task = self.node_to_task_map.write().await;
        let mut task_to_node = self.task_to_node_map.write().await;

        node_to_task.insert(node_id, task_id);
        task_to_node.insert(task_id, node_id);

        Ok(task_id)
    }

    /// Process task results and update DTG node.
    pub async fn process_task_result(&self, task_result: TaskResult) -> Result<(), String> {
        let task_to_node = self.task_to_node_map.read().await;
        let node_id = task_to_node
            .get(&task_result.task_id)
            .ok_or_else(|| format!("No node found for task {}", task_result.task_id))?;

        let mut dtg_engine = self.dtg_engine.write().await;
        let node = dtg_engine
            .graph_mut()
            .nodes
            .get_mut(node_id)
            .ok_or_else(|| format!("Node {node_id} not found"))?;

        // Update node status based on task result
        if task_result.success {
            node.status = DtgNodeStatus::Completed;

            // Store result in node metadata
            node.metadata.insert(
                "execution_result".to_string(),
                serde_json::to_value(&task_result.result).unwrap_or(serde_json::Value::Null),
            );

            // Update node metrics
            node.metrics.quality_score = task_result.quality_score;
            // Note: DtgMetrics doesn't have latency_ms or cost fields
            // Store in metadata instead
            node.metadata.insert(
                "execution_time_ms".to_string(),
                serde_json::Value::from(task_result.execution_time_ms),
            );
            node.metadata.insert(
                "execution_cost".to_string(),
                serde_json::Value::from(task_result.cost),
            );

            // Add provenance information
            node.metadata.insert(
                "executed_by_agent".to_string(),
                serde_json::Value::String(task_result.executor_id),
            );

            node.metadata.insert(
                "completed_at".to_string(),
                serde_json::to_value(task_result.completed_at).unwrap_or(serde_json::Value::Null),
            );

            if let Some(error) = task_result.error {
                node.metadata.insert(
                    "execution_error".to_string(),
                    serde_json::Value::String(error),
                );
            }
        } else {
            node.status = DtgNodeStatus::Failed;

            if let Some(error) = task_result.error {
                node.metadata.insert(
                    "failure_reason".to_string(),
                    serde_json::Value::String(error),
                );
            }
        }

        // Update DTG engine state
        if task_result.success {
            dtg_engine.completed.insert(*node_id);
            dtg_engine.executing.remove(node_id);
            dtg_engine.stats.nodes_completed += 1;
            dtg_engine.stats.nodes_executing -= 1;
        } else {
            dtg_engine.failed.insert(*node_id);
            dtg_engine.executing.remove(node_id);
            dtg_engine.stats.nodes_failed += 1;
            dtg_engine.stats.nodes_executing -= 1;
        }

        Ok(())
    }

    /// Execute the entire DTG using agents.
    pub async fn execute_dtg_with_agents(&self) -> Result<ExecutionResult, String> {
        // TODO: Implement proper DTG execution with agents
        // This requires integration with the DtgExecutionEngine's execution methods
        Err("DTG execution with agents not yet implemented".to_string())
    }

    /// Get execution status for a DTG.
    pub async fn get_execution_status(&self) -> ExecutionStatus {
        let dtg_engine = self.dtg_engine.read().await;
        let node_to_task = self.node_to_task_map.read().await;

        let mut node_statuses = HashMap::new();

        for (node_id, _) in dtg_engine.graph.nodes.iter() {
            let status = if dtg_engine.completed.contains(node_id) {
                NodeExecutionStatus::Completed
            } else if dtg_engine.failed.contains(node_id) {
                NodeExecutionStatus::Failed("Execution failed".to_string())
            } else if dtg_engine.executing.contains(node_id) {
                if node_to_task.contains_key(node_id) {
                    NodeExecutionStatus::Executing
                } else {
                    NodeExecutionStatus::Ready
                }
            } else if dtg_engine.ready_queue.contains(node_id) {
                NodeExecutionStatus::Ready
            } else {
                NodeExecutionStatus::WaitingForDependencies
            };

            node_statuses.insert(*node_id, status);
        }

        ExecutionStatus {
            total_nodes: dtg_engine.stats.total_nodes,
            completed: dtg_engine.stats.nodes_completed,
            failed: dtg_engine.stats.nodes_failed,
            executing: dtg_engine.stats.nodes_executing,
            waiting: dtg_engine.stats.nodes_waiting,
            node_statuses,
        }
    }

    /// Get a reference to the DTG engine (for testing).
    pub fn get_dtg_engine(&self) -> &Arc<RwLock<DtgExecutionEngine>> {
        &self.dtg_engine
    }

    /// Get a reference to the agent coordinator (for testing).
    pub fn get_coordinator(&self) -> &Arc<RwLock<LlmStrategistCoordinator>> {
        &self.agent_coordinator
    }
}

/// Result of DTG execution with agents.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Total nodes in the DTG.
    pub total_nodes: usize,

    /// Number of nodes submitted for execution.
    pub submitted_nodes: usize,

    /// Individual node execution results.
    pub results: Vec<NodeExecutionResult>,
}

/// Result of a single node execution.
#[derive(Debug, Clone)]
pub struct NodeExecutionResult {
    /// DTG node ID.
    pub node_id: Uuid,

    /// Agent task ID (if submitted).
    pub task_id: Uuid,

    /// Execution status.
    pub status: NodeExecutionStatus,
}

/// Status of a node execution.
#[derive(Debug, Clone)]
pub enum NodeExecutionStatus {
    /// Node is waiting for dependencies.
    WaitingForDependencies,

    /// Node is ready for execution.
    Ready,

    /// Node has been submitted to an agent.
    Submitted,

    /// Node is currently executing.
    Executing,

    /// Node execution completed successfully.
    Completed,

    /// Node execution failed.
    Failed(String),
}

/// Overall execution status.
#[derive(Debug, Clone)]
pub struct ExecutionStatus {
    /// Total nodes in the DTG.
    pub total_nodes: usize,

    /// Nodes completed successfully.
    pub completed: usize,

    /// Nodes that failed.
    pub failed: usize,

    /// Nodes currently executing.
    pub executing: usize,

    /// Nodes waiting for dependencies.
    pub waiting: usize,

    /// Status of individual nodes.
    pub node_statuses: HashMap<Uuid, NodeExecutionStatus>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hybrid::coordinator::{ExecutorStats, PerformanceMetrics, QueueStats};
    use crate::models::dtg::{DataTransformationGraph, DtgDataRef, DtgGraphStatus, DtgMetrics};
    use crate::models::hybrid_agent::{
        CommunicationPattern, CoordinationStrategy, CoordinationStrategyType,
        DecisionMakingApproach, FeedbackMechanism, HybridAgentConfig, ModelProvider, ModelSize,
        PerformanceTargets, ResourceAllocation, StrategistConfig,
    };
    use std::collections::HashMap;

    #[tokio::test]

    async fn test_dtg_agent_integration() {
        // Create a simple DTG
        let mut graph = DataTransformationGraph {
            id: Uuid::new_v4(),
            name: "Test DTG".to_string(),
            root_nodes: vec![],
            nodes: HashMap::new(),
            edges: vec![],
            graph_inputs: vec![],
            graph_outputs: vec![],
            metadata: HashMap::new(),
            started_at: chrono::Utc::now(),
            completed_at: None,
            status: DtgGraphStatus::Ready,
            tags: vec![],
        };

        // Add a test node
        let node_id = Uuid::new_v4();
        let node = DtgNode {
            id: node_id,
            skill_id: "data_processing".to_string(),
            agent_id: "".to_string(),
            inputs: vec![],
            outputs: vec![DtgDataRef {
                id: Uuid::new_v4(),
                data_type: "test_data".to_string(),
                schema: None,
                size_bytes: Some(100),
                content_hash: Some("test_hash".to_string()),
                storage_ref: Some("memory".to_string()),
            }],
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "task_type".to_string(),
                    serde_json::Value::String("data_processing".to_string()),
                );
                metadata.insert(
                    "required_skills".to_string(),
                    serde_json::json!(["data_analysis", "python"]),
                );
                metadata
            },
            started_at: chrono::Utc::now(),
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
                collected_at: chrono::Utc::now(),
            },
        };

        graph.nodes.insert(node_id, node);
        graph.root_nodes.push(node_id);

        // Create DTG engine
        let dtg_engine = DtgExecutionEngine::new(
            graph,
            Box::new(|_node| {
                Ok(DtgMetrics {
                    cpu_time_ms: 100,
                    memory_bytes: 1024,
                    network_bytes: 0,
                    disk_bytes: 0,
                    retry_count: 0,
                    quality_score: 0.9,
                    confidence_score: 0.8,
                    execution_time_ms: 50,
                    throughput_ops_per_sec: 10.0,
                    error_rate: 0.0,
                    data_consistency_score: 0.9,
                    schema_compliance_score: 0.9,
                    business_value_score: 0.8,
                    cost: 0.0,
                    collected_at: chrono::Utc::now(),
                })
            }),
        );

        // Create agent coordinator
        let coordinator = LlmStrategistCoordinator::new(HybridAgentConfig {
            id: Uuid::new_v4(),
            name: "Test Coordinator".to_string(),
            description: "Test".to_string(),
            strategist: StrategistConfig {
                model_id: "test".to_string(),
                provider: ModelProvider::Openai,
                model_size: ModelSize::Small,
                capabilities: vec![],
                context_window: 4096,
                temperature: 0.7,
                max_tokens: 1024,
                cost_per_1k_tokens: 0.01,
                latency_target_ms: 1000,
                streaming: false,
            },
            executors: vec![],
            coordination: CoordinationStrategy {
                strategy_type: CoordinationStrategyType::Hierarchical,
                communication_pattern: CommunicationPattern::Centralized,
                decision_making: DecisionMakingApproach::Centralized,
                feedback_mechanism: FeedbackMechanism::Immediate,
                sync_frequency_ms: 1000,
                max_retries: 3,
                timeout_ms: 5000,
            },
            resource_allocation: ResourceAllocation::default(),
            performance_targets: PerformanceTargets::default(),
            fallback_strategies: vec![],
        });

        // Create integration engine
        let integration = DtgAgentIntegrationEngine::new(
            dtg_engine.expect("Failed to create DTG engine"),
            coordinator,
        );

        // Test skill registration
        let skill = AgentSkill {
            id: "data_analysis".to_string(),
            name: "data_analysis".to_string(),
            description: "Data analysis skill".to_string(),
            tags: vec!["data".to_string(), "analysis".to_string()],
            examples: Some(vec![
                "Analyze dataset".to_string(),
                "Generate report".to_string(),
            ]),
            input_modes: None,
            output_modes: None,
        };

        integration.register_agent_skill("agent_1", skill).await;

        // Also register python skill
        let python_skill = AgentSkill {
            id: "python".to_string(),
            name: "python".to_string(),
            description: "Python programming".to_string(),
            tags: vec!["programming".to_string(), "python".to_string()],
            examples: Some(vec![
                "Write Python script".to_string(),
                "Debug code".to_string(),
            ]),
            input_modes: None,
            output_modes: None,
        };
        integration
            .register_agent_skill("agent_1", python_skill)
            .await;

        // Test finding best agent
        let node = DtgNode {
            id: Uuid::new_v4(),
            skill_id: "data_analysis".to_string(),
            agent_id: "".to_string(), // Will be filled by integration
            inputs: vec![],
            outputs: vec![DtgDataRef {
                id: Uuid::new_v4(),
                data_type: "test_data".to_string(),
                schema: None,
                size_bytes: Some(100),
                content_hash: Some("test_hash".to_string()),
                storage_ref: Some("memory".to_string()),
            }],
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "required_skills".to_string(),
                    serde_json::json!(["data_analysis"]),
                );
                metadata
            },
            started_at: chrono::Utc::now(),
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
                collected_at: chrono::Utc::now(),
            },
        };

        let best_agent = integration.find_best_agent_for_node(&node).await;
        assert_eq!(best_agent, Some("agent_1".to_string()));
    }
}
