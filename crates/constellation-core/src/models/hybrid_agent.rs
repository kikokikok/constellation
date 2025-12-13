//! Hybrid agent architecture models.
//!
//! Combines LLM strategists (large language models for planning) with
//! SLM executors (smaller, specialized models for execution).
//!
//! Based on research: "Hybrid Agentic AI architectures" for optimal
//! performance and resource utilization.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Hybrid agent configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridAgentConfig {
    /// Unique identifier for this hybrid agent.
    pub id: Uuid,
    
    /// Name of the hybrid agent.
    pub name: String,
    
    /// Description of the agent's purpose.
    pub description: String,
    
    /// Strategist configuration (LLM).
    pub strategist: StrategistConfig,
    
    /// Executor configurations (SLMs).
    pub executors: Vec<ExecutorConfig>,
    
    /// Coordination strategy between strategist and executors.
    pub coordination: CoordinationStrategy,
    
    /// Resource allocation strategy.
    pub resource_allocation: ResourceAllocation,
    
    /// Performance targets.
    pub performance_targets: PerformanceTargets,
    
    /// Fallback strategies.
    pub fallback_strategies: Vec<FallbackStrategy>,
}

/// Strategist configuration (LLM).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategistConfig {
    /// Model identifier.
    pub model_id: String,
    
    /// Model provider.
    pub provider: ModelProvider,
    
    /// Model size in parameters.
    pub model_size: ModelSize,
    
    /// Capabilities of the strategist.
    pub capabilities: Vec<StrategistCapability>,
    
    /// Context window size in tokens.
    pub context_window: u32,
    
    /// Temperature for generation.
    pub temperature: f64,
    
    /// Maximum tokens per request.
    pub max_tokens: u32,
    
    /// Cost per 1K tokens.
    pub cost_per_1k_tokens: f64,
    
    /// Latency target in milliseconds.
    pub latency_target_ms: u32,
    
    /// Whether to use streaming.
    pub streaming: bool,
}

/// Model provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModelProvider {
    Openai,
    Anthropic,
    Google,
    Meta,
    Mistral,
    Cohere,
    Local,
    Custom(String),
}

/// Model size category.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModelSize {
    Tiny,      // < 1B params
    Small,     // 1B-7B params
    Medium,    // 7B-30B params
    Large,     // 30B-100B params
    XLarge,    // 100B-500B params
    XXLarge,   // 500B+ params
}

/// Strategist capability.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StrategistCapability {
    Planning,
    Reasoning,
    ProblemDecomposition,
    TaskAllocation,
    QualityAssessment,
    RiskAssessment,
    CreativeThinking,
    StrategicThinking,
    MetaCognition,
    SelfReflection,
}

/// Executor configuration (SLM).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorConfig {
    /// Executor identifier.
    pub id: String,
    
    /// Specialized domain.
    pub domain: ExecutorDomain,
    
    /// Model configuration.
    pub model: ExecutorModel,
    
    /// Skills available to this executor.
    pub skills: Vec<ExecutorSkill>,
    
    /// Performance characteristics.
    pub performance: ExecutorPerformance,
    
    /// Resource requirements.
    pub resource_requirements: ResourceRequirements,
    
    /// Whether this executor can run locally.
    pub local_execution: bool,
    
    /// Maximum concurrent tasks.
    pub max_concurrent_tasks: u32,
}

/// Executor domain specialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutorDomain {
    CodeGeneration,
    DataAnalysis,
    Research,
    Writing,
    Mathematics,
    Science,
    Engineering,
    Design,
    Business,
    Legal,
    Medical,
    Security,
    Operations,
    Monitoring,
    Debugging,
}

/// Executor model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorModel {
    /// Model identifier.
    pub model_id: String,
    
    /// Model provider.
    pub provider: ModelProvider,
    
    /// Model size.
    pub size: ExecutorModelSize,
    
    /// Whether model is fine-tuned.
    pub fine_tuned: bool,
    
    /// Fine-tuning dataset.
    pub fine_tuning_dataset: Option<String>,
    
    /// Specialized capabilities.
    pub specialized_capabilities: Vec<String>,
}

/// Executor model size.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutorModelSize {
    Nano,      // < 100M params
    Micro,     // 100M-500M params
    Mini,      // 500M-1B params
    Small,     // 1B-3B params
    Compact,   // 3B-7B params
}

/// Executor skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorSkill {
    /// Skill identifier.
    pub id: String,
    
    /// Skill name.
    pub name: String,
    
    /// Skill description.
    pub description: String,
    
    /// Input schema.
    pub input_schema: serde_json::Value,
    
    /// Output schema.
    pub output_schema: serde_json::Value,
    
    /// Average execution time in milliseconds.
    pub avg_execution_time_ms: u32,
    
    /// Success rate (0.0 to 1.0).
    pub success_rate: f64,
    
    /// Quality score (0.0 to 1.0).
    pub quality_score: f64,
    
    /// Whether skill is deterministic.
    pub deterministic: bool,
}

/// Executor performance characteristics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorPerformance {
    /// Throughput in tasks per second.
    pub throughput_tps: f64,
    
    /// Average latency in milliseconds.
    pub avg_latency_ms: u32,
    
    /// P95 latency in milliseconds.
    pub p95_latency_ms: u32,
    
    /// P99 latency in milliseconds.
    pub p99_latency_ms: u32,
    
    /// Error rate (0.0 to 1.0).
    pub error_rate: f64,
    
    /// Availability (0.0 to 1.0).
    pub availability: f64,
    
    /// Cost per 1K tasks.
    pub cost_per_1k_tasks: f64,
}

/// Resource requirements for executor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU cores required.
    pub cpu_cores: u32,
    
    /// Memory required in MB.
    pub memory_mb: u32,
    
    /// GPU memory required in MB.
    pub gpu_memory_mb: Option<u32>,
    
    /// Disk space required in MB.
    pub disk_mb: u32,
    
    /// Network bandwidth required in Mbps.
    pub network_mbps: u32,
}

/// Coordination strategy between strategist and executors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationStrategy {
    /// Strategy type.
    pub strategy_type: CoordinationStrategyType,
    
    /// Communication pattern.
    pub communication_pattern: CommunicationPattern,
    
    /// Decision-making approach.
    pub decision_making: DecisionMakingApproach,
    
    /// Feedback mechanism.
    pub feedback_mechanism: FeedbackMechanism,
    
    /// Synchronization frequency in milliseconds.
    pub sync_frequency_ms: u32,
    
    /// Maximum retry attempts.
    pub max_retries: u32,
    
    /// Timeout in milliseconds.
    pub timeout_ms: u32,
}

/// Coordination strategy type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CoordinationStrategyType {
    Hierarchical,
    Collaborative,
    Competitive,
    MarketBased,
    Federated,
    Swarm,
}

/// Communication pattern.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationPattern {
    Centralized,
    Decentralized,
    PeerToPeer,
    PubSub,
    RequestResponse,
    Streaming,
}

/// Decision-making approach.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionMakingApproach {
    Centralized,
    Distributed,
    Consensus,
    Voting,
    Auction,
    ReinforcementLearning,
}

/// Feedback mechanism.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackMechanism {
    Immediate,
    Delayed,
    Batch,
    Continuous,
    EventDriven,
    Periodic,
}

/// Resource allocation strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    /// Allocation strategy.
    pub strategy: AllocationStrategy,
    
    /// CPU allocation policy.
    pub cpu_policy: AllocationPolicy,
    
    /// Memory allocation policy.
    pub memory_policy: AllocationPolicy,
    
    /// GPU allocation policy.
    pub gpu_policy: Option<AllocationPolicy>,
    
    /// Budget allocation.
    pub budget_allocation: BudgetAllocation,
    
    /// Scaling strategy.
    pub scaling_strategy: ScalingStrategy,
    
    /// Priority levels.
    pub priority_levels: Vec<PriorityLevel>,
}

/// Allocation strategy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AllocationStrategy {
    Static,
    Dynamic,
    Predictive,
    Reactive,
    Optimistic,
    Conservative,
}

/// Allocation policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationPolicy {
    /// Minimum allocation.
    pub min: u32,
    
    /// Maximum allocation.
    pub max: u32,
    
    /// Default allocation.
    pub default: u32,
    
    /// Scaling factor.
    pub scaling_factor: f64,
}

/// Budget allocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAllocation {
    /// Total budget.
    pub total_budget: f64,
    
    /// Strategist budget percentage.
    pub strategist_percentage: f64,
    
    /// Executors budget percentage.
    pub executors_percentage: f64,
    
    /// Infrastructure budget percentage.
    pub infrastructure_percentage: f64,
    
    /// Reserve budget percentage.
    pub reserve_percentage: f64,
}

/// Scaling strategy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScalingStrategy {
    Horizontal,
    Vertical,
    Hybrid,
    Burstable,
    Reserved,
    Spot,
}

/// Priority level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityLevel {
    /// Level name.
    pub name: String,
    
    /// Priority value (higher = more important).
    pub value: u32,
    
    /// Resource multiplier.
    pub resource_multiplier: f64,
    
    /// Cost multiplier.
    pub cost_multiplier: f64,
}

/// Performance targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargets {
    /// Overall success rate target.
    pub success_rate_target: f64,
    
    /// Quality score target.
    pub quality_score_target: f64,
    
    /// Latency target in milliseconds.
    pub latency_target_ms: u32,
    
    /// Throughput target in tasks per second.
    pub throughput_target_tps: f64,
    
    /// Cost efficiency target.
    pub cost_efficiency_target: f64,
    
    /// Resource utilization target.
    pub resource_utilization_target: f64,
    
    /// Availability target.
    pub availability_target: f64,
}

/// Fallback strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackStrategy {
    /// Trigger condition.
    pub trigger: FallbackTrigger,
    
    /// Action to take.
    pub action: FallbackAction,
    
    /// Priority level.
    pub priority: u32,
    
    /// Timeout in milliseconds.
    pub timeout_ms: u32,
}

/// Fallback trigger condition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FallbackTrigger {
    HighLatency,
    LowSuccessRate,
    HighErrorRate,
    ResourceExhaustion,
    BudgetExceeded,
    QualityBelowThreshold,
    AvailabilityBelowThreshold,
    Timeout,
}

/// Fallback action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FallbackAction {
    SwitchExecutor,
    ReduceQuality,
    IncreaseBudget,
    ScaleResources,
    NotifyHuman,
    RetryWithBackoff,
    AbortTask,
    UseAlternativeStrategy,
}

impl HybridAgentConfig {
    /// Create a new hybrid agent configuration.
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            strategist: StrategistConfig::default(),
            executors: Vec::new(),
            coordination: CoordinationStrategy::default(),
            resource_allocation: ResourceAllocation::default(),
            performance_targets: PerformanceTargets::default(),
            fallback_strategies: Vec::new(),
        }
    }
    
    /// Add an executor configuration.
    pub fn add_executor(&mut self, executor: ExecutorConfig) {
        self.executors.push(executor);
    }
    
    /// Add a fallback strategy.
    pub fn add_fallback_strategy(&mut self, strategy: FallbackStrategy) {
        self.fallback_strategies.push(strategy);
    }
    
    /// Calculate total estimated cost per 1K tasks.
    pub fn estimated_cost_per_1k_tasks(&self) -> f64 {
        let strategist_cost = self.strategist.cost_per_1k_tokens * 10.0; // Estimate 10 tokens per task
        
        let executors_cost: f64 = self.executors
            .iter()
            .map(|executor| executor.performance.cost_per_1k_tasks)
            .sum();
        
        strategist_cost + executors_cost
    }
    
    /// Calculate total resource requirements.
    pub fn total_resource_requirements(&self) -> ResourceRequirements {
        let mut total = ResourceRequirements {
            cpu_cores: 0,
            memory_mb: 0,
            gpu_memory_mb: None,
            disk_mb: 0,
            network_mbps: 0,
        };
        
        // Add strategist requirements (estimate)
        total.cpu_cores += 4; // LLM API calls
        total.memory_mb += 1024; // Context management
        total.network_mbps += 100; // API communication
        
        // Add executor requirements
        for executor in &self.executors {
            total.cpu_cores += executor.resource_requirements.cpu_cores;
            total.memory_mb += executor.resource_requirements.memory_mb;
            
            if let Some(executor_gpu) = executor.resource_requirements.gpu_memory_mb {
                total.gpu_memory_mb = Some(
                    total.gpu_memory_mb.unwrap_or(0) + executor_gpu
                );
            }
            
            total.disk_mb += executor.resource_requirements.disk_mb;
            total.network_mbps += executor.resource_requirements.network_mbps;
        }
        
        total
    }
}

impl Default for StrategistConfig {
    fn default() -> Self {
        Self {
            model_id: "gpt-4".to_string(),
            provider: ModelProvider::Openai,
            model_size: ModelSize::XLarge,
            capabilities: vec![
                StrategistCapability::Planning,
                StrategistCapability::Reasoning,
                StrategistCapability::ProblemDecomposition,
            ],
            context_window: 128000,
            temperature: 0.7,
            max_tokens: 4096,
            cost_per_1k_tokens: 0.03,
            latency_target_ms: 5000,
            streaming: false,
        }
    }
}

impl Default for CoordinationStrategy {
    fn default() -> Self {
        Self {
            strategy_type: CoordinationStrategyType::Hierarchical,
            communication_pattern: CommunicationPattern::Centralized,
            decision_making: DecisionMakingApproach::Centralized,
            feedback_mechanism: FeedbackMechanism::Immediate,
            sync_frequency_ms: 1000,
            max_retries: 3,
            timeout_ms: 30000,
        }
    }
}

impl Default for ResourceAllocation {
    fn default() -> Self {
        Self {
            strategy: AllocationStrategy::Dynamic,
            cpu_policy: AllocationPolicy {
                min: 1,
                max: 16,
                default: 4,
                scaling_factor: 1.5,
            },
            memory_policy: AllocationPolicy {
                min: 1024,
                max: 32768,
                default: 8192,
                scaling_factor: 2.0,
            },
            gpu_policy: Some(AllocationPolicy {
                min: 0,
                max: 16384,
                default: 4096,
                scaling_factor: 2.0,
            }),
            budget_allocation: BudgetAllocation {
                total_budget: 1000.0,
                strategist_percentage: 40.0,
                executors_percentage: 40.0,
                infrastructure_percentage: 15.0,
                reserve_percentage: 5.0,
            },
            scaling_strategy: ScalingStrategy::Hybrid,
            priority_levels: vec![
                PriorityLevel {
                    name: "critical".to_string(),
                    value: 100,
                    resource_multiplier: 2.0,
                    cost_multiplier: 3.0,
                },
                PriorityLevel {
                    name: "high".to_string(),
                    value: 75,
                    resource_multiplier: 1.5,
                    cost_multiplier: 2.0,
                },
                PriorityLevel {
                    name: "normal".to_string(),
                    value: 50,
                    resource_multiplier: 1.0,
                    cost_multiplier: 1.0,
                },
                PriorityLevel {
                    name: "low".to_string(),
                    value: 25,
                    resource_multiplier: 0.5,
                    cost_multiplier: 0.5,
                },
            ],
        }
    }
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            success_rate_target: 0.95,
            quality_score_target: 0.90,
            latency_target_ms: 10000,
            throughput_target_tps: 10.0,
            cost_efficiency_target: 0.80,
            resource_utilization_target: 0.70,
            availability_target: 0.99,
        }
    }
}

impl ExecutorConfig {
    /// Create a new executor configuration.
    pub fn new(id: String, domain: ExecutorDomain) -> Self {
        Self {
            id,
            domain,
            model: ExecutorModel::default(),
            skills: Vec::new(),
            performance: ExecutorPerformance::default(),
            resource_requirements: ResourceRequirements::default(),
            local_execution: false,
            max_concurrent_tasks: 1,
        }
    }
    
    /// Add a skill to the executor.
    pub fn add_skill(&mut self, skill: ExecutorSkill) {
        self.skills.push(skill);
    }
}

impl Default for ExecutorModel {
    fn default() -> Self {
        Self {
            model_id: "codellama-7b".to_string(),
            provider: ModelProvider::Meta,
            size: ExecutorModelSize::Small,
            fine_tuned: false,
            fine_tuning_dataset: None,
            specialized_capabilities: vec!["code_generation".to_string()],
        }
    }
}

impl Default for ExecutorPerformance {
    fn default() -> Self {
        Self {
            throughput_tps: 5.0,
            avg_latency_ms: 2000,
            p95_latency_ms: 5000,
            p99_latency_ms: 10000,
            error_rate: 0.05,
            availability: 0.99,
            cost_per_1k_tasks: 0.50,
        }
    }
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu_cores: 2,
            memory_mb: 4096,
            gpu_memory_mb: Some(4096),
            disk_mb: 1024,
            network_mbps: 100,
        }
    }
}