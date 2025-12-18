//! Integration of hybrid agents with A2A (Agent-to-Agent) protocol.
//!
//! This module connects hybrid agent architecture (LLM strategist + SLM executors)
//! with the A2A communication protocol, enabling coordinated multi-agent workflows.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::hybrid::coordinator::{LlmStrategistCoordinator, Task, TaskResult};
use crate::integration::mcp_security_integration::McpSecurityIntegration;
use crate::models::agent::{
    Agent, AgentCapabilities, AgentContact, AgentInterface, AgentProvider, AgentSkill,
    ProtocolBinding,
};
use crate::models::hybrid_agent::HybridAgentConfig;

/// Integration of hybrid agents with A2A protocol.
pub struct HybridA2AIntegration {
    /// Hybrid agent coordinator.
    coordinator: Arc<RwLock<LlmStrategistCoordinator>>,

    /// MCP security integration.
    security: Arc<RwLock<McpSecurityIntegration>>,

    /// Registered agents.
    agents: Arc<RwLock<HashMap<String, Agent>>>,

    /// Agent skills registry.
    skills: Arc<RwLock<HashMap<String, Vec<AgentSkill>>>>,

    /// A2A protocol bindings.
    protocol_bindings: Arc<RwLock<HashMap<ProtocolBinding, String>>>,
}

impl HybridA2AIntegration {
    /// Create a new hybrid A2A integration.
    pub fn new(coordinator: LlmStrategistCoordinator, security: McpSecurityIntegration) -> Self {
        Self {
            coordinator: Arc::new(RwLock::new(coordinator)),
            security: Arc::new(RwLock::new(security)),
            agents: Arc::new(RwLock::new(HashMap::new())),
            skills: Arc::new(RwLock::new(HashMap::new())),
            protocol_bindings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a hybrid agent with A2A protocol.
    pub async fn register_hybrid_agent(
        &self,
        agent_config: HybridAgentConfig,
        a2a_interface: AgentInterface,
    ) -> Result<Agent, String> {
        let agent_id = agent_config.id.to_string();

        // Extract skills from executor configurations
        let mut skills = Vec::new();
        for executor in &agent_config.executors {
            let skill = AgentSkill {
                id: executor.id.clone(),
                name: format!("{:?}", executor.domain),
                description: format!("Executor for {:?}", executor.domain),
                tags: vec![format!("{:?}", executor.domain), "executor".to_string()],
                examples: None,
                input_modes: Some(vec!["text".to_string(), "json".to_string()]),
                output_modes: Some(vec!["text".to_string(), "json".to_string()]),
            };
            skills.push(skill);
        }

        // Create agent from hybrid config
        let agent = Agent {
            id: agent_id.clone(),
            name: agent_config.name.clone(),
            description: agent_config.description.clone(),
            protocol_version: "1.0.0".to_string(),
            version: "1.0.0".to_string(),
            default_input_modes: vec!["application/json".to_string()],
            default_output_modes: vec!["application/json".to_string()],
            provider: AgentProvider {
                name: "Constellation".to_string(),
                url: None,
                contact: Some(AgentContact { email: None }),
            },
            capabilities: AgentCapabilities {
                streaming: Some(true),
                push_notifications: Some(true),
                state_transition_history: Some(true),
            },
            skills: skills.clone(),
            supported_interfaces: vec![a2a_interface],
            metadata: {
                let mut metadata = serde_json::Map::new();
                metadata.insert(
                    "hybrid_agent_id".to_string(),
                    serde_json::Value::String(agent_config.id.to_string()),
                );
                metadata.insert(
                    "coordination_strategy".to_string(),
                    serde_json::Value::String(format!("{:?}", agent_config.coordination)),
                );
                metadata.insert(
                    "max_concurrent_tasks".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(
                        agent_config.executors.len() as u64,
                    )),
                );
                Some(serde_json::Value::Object(metadata))
            },
            supports_extended_agent_card: Some(true),
            documentation_url: None,
            icon_url: None,
        };

        // Store agent and skills
        let mut agents = self.agents.write().await;
        let mut skills_registry = self.skills.write().await;

        agents.insert(agent_id.clone(), agent.clone());
        skills_registry.insert(agent_id.clone(), skills);

        // Register with MCP security
        let security = self.security.read().await;
        security
            .register_agent(&agent)
            .await
            .map_err(|e| format!("Failed to register with MCP security: {e}"))?;

        Ok(agent)
    }

    /// Submit a task to a hybrid agent via A2A protocol.
    pub async fn submit_task_to_agent(&self, agent_id: &str, task: Task) -> Result<Uuid, String> {
        let coordinator = self.coordinator.write().await;

        // Submit task through coordinator
        coordinator
            .submit_task(task)
            .map_err(|e| format!("Failed to submit task: {e}"))
    }

    /// Send an A2A message to another agent.
    pub async fn send_a2a_message(
        &self,
        sender_id: &str,
        recipient_id: &str,
        message_type: &str,
        payload: serde_json::Value,
        security_level: crate::models::mcp::SecurityLevel,
    ) -> Result<A2AMessageResponse, String> {
        let security = self.security.read().await;
        let agents = self.agents.read().await;

        // Check if agents exist
        if !agents.contains_key(sender_id) {
            return Err(format!("Sender agent {sender_id} not found"));
        }

        if !agents.contains_key(recipient_id) {
            return Err(format!("Recipient agent {recipient_id} not found"));
        }

        // Create A2A message
        let message = A2AMessage {
            message_id: Uuid::new_v4(),
            sender_id: sender_id.to_string(),
            recipient_id: recipient_id.to_string(),
            message_type: message_type.to_string(),
            payload: payload.clone(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        // Serialize message
        let message_bytes = serde_json::to_vec(&message)
            .map_err(|e| format!("Failed to serialize message: {e}"))?;

        // Secure the message with MCP
        let envelope = security
            .secure_message(sender_id, recipient_id, &message_bytes, security_level)
            .await
            .map_err(|e| format!("Failed to secure message: {e}"))?;

        // Get recipient's endpoint
        let recipient = agents.get(recipient_id).unwrap();

        // In a real implementation, this would send the envelope to the recipient's endpoint
        // For now, we'll simulate the response
        let response = A2AMessageResponse {
            message_id: message.message_id,
            recipient_id: recipient_id.to_string(),
            status: A2AMessageStatus::Delivered,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        Ok(response)
    }

    /// Process an incoming A2A message.
    pub async fn process_incoming_message(
        &self,
        envelope: crate::models::mcp::McpSecureEnvelope,
    ) -> Result<A2AMessage, String> {
        let security = self.security.read().await;

        // Verify and decrypt the message
        let decrypted_bytes = security
            .verify_and_decrypt_message(&envelope)
            .await
            .map_err(|e| format!("Failed to verify/decrypt message: {e}"))?;

        // Deserialize the A2A message
        let message: A2AMessage = serde_json::from_slice(&decrypted_bytes)
            .map_err(|e| format!("Failed to deserialize message: {e}"))?;

        // Process based on message type
        match message.message_type.as_str() {
            "task_request" => {
                self.process_task_request(&message).await?;
            }
            "task_result" => {
                self.process_task_result(&message).await?;
            }
            "coordination" => {
                self.process_coordination_message(&message).await?;
            }
            _ => {
                // Unknown message type, but we still received it
            }
        }

        Ok(message)
    }

    /// Process a task request message.
    async fn process_task_request(&self, message: &A2AMessage) -> Result<(), String> {
        let coordinator = self.coordinator.write().await;

        // Extract task from message
        let task: Task = serde_json::from_value(message.payload.clone())
            .map_err(|e| format!("Failed to parse task from message: {e}"))?;

        // Submit task to coordinator
        coordinator
            .submit_task(task)
            .map_err(|e| format!("Failed to submit task: {e}"))?;

        Ok(())
    }

    /// Process a task result message.
    async fn process_task_result(&self, message: &A2AMessage) -> Result<(), String> {
        let coordinator = self.coordinator.write().await;

        // Extract task result from message
        let task_result: TaskResult = serde_json::from_value(message.payload.clone())
            .map_err(|e| format!("Failed to parse task result from message: {e}"))?;

        // Update coordinator with result
        // Note: In a real implementation, this would update the coordinator's state
        // For now, we'll just acknowledge receipt

        Ok(())
    }

    /// Process a coordination message.
    async fn process_coordination_message(&self, message: &A2AMessage) -> Result<(), String> {
        // Coordination messages are for multi-agent workflows
        // They can include:
        // - Resource allocation requests
        // - Task dependencies
        // - Performance feedback
        // - Strategy adjustments

        // For now, we'll just log the coordination message
        println!(
            "Received coordination message from {}: {:?}",
            message.sender_id, message.payload
        );

        Ok(())
    }

    /// Get agent performance metrics.
    pub async fn get_agent_performance(&self, agent_id: &str) -> Result<AgentPerformance, String> {
        let coordinator = self.coordinator.read().await;

        // Get coordinator metrics
        let metrics = coordinator.get_performance_metrics();

        // Calculate agent-specific metrics
        // In a real implementation, this would filter metrics by agent
        // For now, we'll use placeholder values since PerformanceMetrics doesn't have these fields
        let performance = AgentPerformance {
            agent_id: agent_id.to_string(),
            total_tasks_completed: 0, // Placeholder - would need to track this separately
            total_tasks_failed: 0,    // Placeholder - would need to track this separately
            average_execution_time_ms: metrics.avg_latency_ms,
            average_quality_score: metrics.avg_quality_score,
            total_cost: 0.0, // Placeholder - would need to track this separately
            timestamp: chrono::Utc::now(),
        };

        Ok(performance)
    }

    /// Update protocol binding for an agent.
    pub async fn update_protocol_binding(
        &self,
        agent_id: &str,
        protocol: ProtocolBinding,
        endpoint: &str,
    ) -> Result<(), String> {
        let mut agents = self.agents.write().await;
        let mut bindings = self.protocol_bindings.write().await;

        let agent = agents
            .get_mut(agent_id)
            .ok_or_else(|| format!("Agent {agent_id} not found"))?;

        // Update agent interface
        if let Some(interface) = agent.supported_interfaces.first_mut() {
            interface.protocol_binding = protocol.clone();
            interface.url = endpoint.to_string();
        }

        // Update provider contact if it exists
        if let Some(contact) = &mut agent.provider.contact {
            // Note: AgentContact only has email field in the new structure
            // We'll update metadata instead
        }

        // Update protocol bindings
        bindings.insert(protocol, endpoint.to_string());

        Ok(())
    }

    /// Get all registered agents.
    pub async fn get_registered_agents(&self) -> Vec<Agent> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }

    /// Get agent skills.
    pub async fn get_agent_skills(&self, agent_id: &str) -> Option<Vec<AgentSkill>> {
        let skills = self.skills.read().await;
        skills.get(agent_id).cloned()
    }
}

/// A2A message structure.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct A2AMessage {
    /// Unique message ID.
    pub message_id: Uuid,

    /// Sender agent ID.
    pub sender_id: String,

    /// Recipient agent ID.
    pub recipient_id: String,

    /// Type of message (task_request, task_result, coordination, etc.).
    pub message_type: String,

    /// Message payload (JSON).
    pub payload: serde_json::Value,

    /// Message timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Additional metadata.
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A2A message response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct A2AMessageResponse {
    /// Original message ID.
    pub message_id: Uuid,

    /// Recipient agent ID.
    pub recipient_id: String,

    /// Delivery status.
    pub status: A2AMessageStatus,

    /// Response timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Additional metadata.
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A2A message status.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum A2AMessageStatus {
    /// Message delivered successfully.
    Delivered,

    /// Message failed to deliver.
    Failed,

    /// Message is being processed.
    Processing,

    /// Message requires acknowledgment.
    RequiresAck,
}

/// Agent performance metrics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentPerformance {
    /// Agent ID.
    pub agent_id: String,

    /// Total tasks completed.
    pub total_tasks_completed: u64,

    /// Total tasks failed.
    pub total_tasks_failed: u64,

    /// Average execution time in milliseconds.
    pub average_execution_time_ms: f64,

    /// Average quality score (0.0 to 1.0).
    pub average_quality_score: f64,

    /// Total cost incurred.
    pub total_cost: f64,

    /// Metrics timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HybridA2AIntegration {
    /// Get a reference to the coordinator.
    pub fn get_coordinator(&self) -> &Arc<RwLock<LlmStrategistCoordinator>> {
        &self.coordinator
    }

    /// Get a reference to the security integration.
    pub fn get_security_integration(&self) -> &Arc<RwLock<McpSecurityIntegration>> {
        &self.security
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hybrid::coordinator::{
        ExecutorStats, LlmStrategistCoordinator, PerformanceMetrics, QueueStats,
    };
    use crate::models::hybrid_agent::{
        CommunicationPattern, CoordinationStrategy, CoordinationStrategyType,
        DecisionMakingApproach, ExecutorConfig, ExecutorDomain, ExecutorModel, ExecutorModelSize,
        ExecutorPerformance, FeedbackMechanism, ModelProvider, ModelSize, PerformanceTargets,
        ResourceAllocation, ResourceRequirements, StrategistConfig,
    };

    #[tokio::test]

    async fn test_hybrid_a2a_integration() {
        // Create hybrid agent config first
        let agent_config = HybridAgentConfig {
            id: Uuid::new_v4(),
            name: "Test Hybrid Agent".to_string(),
            description: "Test agent".to_string(),
            strategist: StrategistConfig {
                model_id: "gpt-4".to_string(),
                provider: ModelProvider::Openai,
                model_size: ModelSize::Large,
                capabilities: vec![],
                context_window: 8192,
                temperature: 0.7,
                max_tokens: 2048,
                cost_per_1k_tokens: 0.03,
                latency_target_ms: 1000,
                streaming: false,
            },
            executors: vec![ExecutorConfig {
                id: "executor_1".to_string(),
                domain: ExecutorDomain::DataAnalysis,
                model: ExecutorModel {
                    model_id: "claude-3-haiku".to_string(),
                    provider: ModelProvider::Anthropic,
                    size: ExecutorModelSize::Small,
                    fine_tuned: false,
                    fine_tuning_dataset: None,
                    specialized_capabilities: vec!["data_analysis".to_string()],
                },
                skills: vec![],
                performance: ExecutorPerformance::default(),
                resource_requirements: ResourceRequirements::default(),
                local_execution: false,
                max_concurrent_tasks: 5,
            }],
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
        };

        // Create test coordinator
        let coordinator = LlmStrategistCoordinator::new(agent_config.clone());

        // Create test security integration
        let security = McpSecurityIntegration::new().unwrap();

        // Create integration
        let integration = HybridA2AIntegration::new(coordinator, security);

        // Create A2A interface
        let a2a_interface = AgentInterface {
            url: "http://localhost:8080/agent".to_string(),
            protocol_binding: ProtocolBinding::HttpJson,
            tenant: None,
        };

        // Register hybrid agent
        let agent = integration
            .register_hybrid_agent(agent_config, a2a_interface)
            .await
            .unwrap();

        // Verify agent was registered
        let agents = integration.get_registered_agents().await;
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, agent.id);

        // Test getting agent skills
        let skills = integration.get_agent_skills(&agent.id).await;
        assert!(skills.is_some());
        assert_eq!(skills.unwrap().len(), 1);
    }
}
