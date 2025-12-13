//! Example demonstrating hybrid agent architecture.
//!
//! This example shows how to configure LLM strategists with SLM executors
//! for optimal performance and resource utilization.

use constellation_core::{
    HybridAgentConfig, StrategistConfig, ExecutorConfig, CoordinationStrategy,
    ResourceAllocation, PerformanceTargets,
};
use constellation_core::models::hybrid_agent::{
    ModelProvider, ModelSize, StrategistCapability, ExecutorDomain,
    ExecutorModelSize, CoordinationStrategyType, CommunicationPattern,
    DecisionMakingApproach, FeedbackMechanism, AllocationStrategy,
    ScalingStrategy, FallbackStrategy, FallbackTrigger, FallbackAction,
};

fn main() {
    println!("=== Hybrid Agent Architecture Example ===\n");
    
    // Create a hybrid agent for software development
    let mut agent = HybridAgentConfig::new(
        "CodeCraft AI".to_string(),
        "Hybrid agent for autonomous software development".to_string(),
    );
    
    // Configure strategist (LLM)
    let mut strategist = StrategistConfig::default();
    strategist.model_id = "claude-3-opus".to_string();
    strategist.provider = ModelProvider::Anthropic;
    strategist.capabilities.push(StrategistCapability::CreativeThinking);
    strategist.capabilities.push(StrategistCapability::StrategicThinking);
    agent.strategist = strategist;
    
    // Add code generation executor
    let mut code_executor = ExecutorConfig::new(
        "code_gen_v1".to_string(),
        ExecutorDomain::CodeGeneration,
    );
    code_executor.model.model_id = "codellama-13b".to_string();
    code_executor.model.size = ExecutorModelSize::Compact;
    code_executor.performance.throughput_tps = 8.0;
    code_executor.performance.avg_latency_ms = 1500;
    code_executor.local_execution = true;
    code_executor.max_concurrent_tasks = 4;
    
    agent.add_executor(code_executor);
    
    // Add testing executor
    let mut test_executor = ExecutorConfig::new(
        "test_gen_v1".to_string(),
        ExecutorDomain::CodeGeneration,
    );
    test_executor.model.model_id = "deepseek-coder-6.7b".to_string();
    test_executor.model.specialized_capabilities = vec![
        "test_generation".to_string(),
        "test_analysis".to_string(),
    ];
    test_executor.performance.throughput_tps = 6.0;
    test_executor.performance.avg_latency_ms = 2500;
    
    agent.add_executor(test_executor);
    
    // Add research executor
    let mut research_executor = ExecutorConfig::new(
        "research_v1".to_string(),
        ExecutorDomain::Research,
    );
    research_executor.model.model_id = "llama-3-8b".to_string();
    research_executor.model.provider = ModelProvider::Meta;
    research_executor.performance.throughput_tps = 4.0;
    research_executor.performance.avg_latency_ms = 3000;
    
    agent.add_executor(research_executor);
    
    // Configure coordination strategy
    let mut coordination = CoordinationStrategy::default();
    coordination.strategy_type = CoordinationStrategyType::Collaborative;
    coordination.communication_pattern = CommunicationPattern::PeerToPeer;
    coordination.decision_making = DecisionMakingApproach::Consensus;
    coordination.feedback_mechanism = FeedbackMechanism::Continuous;
    agent.coordination = coordination;
    
    // Configure resource allocation
    let mut allocation = ResourceAllocation::default();
    allocation.strategy = AllocationStrategy::Predictive;
    allocation.scaling_strategy = ScalingStrategy::Horizontal;
    agent.resource_allocation = allocation;
    
    // Configure performance targets
    let mut targets = PerformanceTargets::default();
    targets.success_rate_target = 0.98;
    targets.quality_score_target = 0.95;
    targets.latency_target_ms = 8000;
    targets.throughput_target_tps = 15.0;
    agent.performance_targets = targets;
    
    // Add fallback strategies
    agent.add_fallback_strategy(FallbackStrategy {
        trigger: FallbackTrigger::HighLatency,
        action: FallbackAction::SwitchExecutor,
        priority: 50,
        timeout_ms: 5000,
    });
    
    agent.add_fallback_strategy(FallbackStrategy {
        trigger: FallbackTrigger::LowSuccessRate,
        action: FallbackAction::ReduceQuality,
        priority: 75,
        timeout_ms: 10000,
    });
    
    agent.add_fallback_strategy(FallbackStrategy {
        trigger: FallbackTrigger::ResourceExhaustion,
        action: FallbackAction::ScaleResources,
        priority: 100,
        timeout_ms: 3000,
    });
    
    // Display configuration
    println!("Agent: {}", agent.name);
    println!("Description: {}", agent.description);
    println!("ID: {}", agent.id);
    
    println!("\n=== Strategist Configuration ===");
    println!("Model: {}", agent.strategist.model_id);
    println!("Provider: {:?}", agent.strategist.provider);
    println!("Size: {:?}", agent.strategist.model_size);
    println!("Context Window: {} tokens", agent.strategist.context_window);
    println!("Cost per 1K tokens: ${}", agent.strategist.cost_per_1k_tokens);
    println!("Capabilities:");
    for capability in &agent.strategist.capabilities {
        println!("  - {:?}", capability);
    }
    
    println!("\n=== Executor Configurations ===");
    for (i, executor) in agent.executors.iter().enumerate() {
        println!("\nExecutor {}: {}", i + 1, executor.id);
        println!("  Domain: {:?}", executor.domain);
        println!("  Model: {}", executor.model.model_id);
        println!("  Size: {:?}", executor.model.size);
        println!("  Local Execution: {}", executor.local_execution);
        println!("  Max Concurrent Tasks: {}", executor.max_concurrent_tasks);
        println!("  Performance:");
        println!("    Throughput: {:.1} tasks/sec", executor.performance.throughput_tps);
        println!("    Avg Latency: {} ms", executor.performance.avg_latency_ms);
        println!("    Cost per 1K tasks: ${}", executor.performance.cost_per_1k_tasks);
    }
    
    println!("\n=== Coordination Strategy ===");
    println!("Type: {:?}", agent.coordination.strategy_type);
    println!("Communication: {:?}", agent.coordination.communication_pattern);
    println!("Decision Making: {:?}", agent.coordination.decision_making);
    println!("Feedback: {:?}", agent.coordination.feedback_mechanism);
    println!("Sync Frequency: {} ms", agent.coordination.sync_frequency_ms);
    println!("Max Retries: {}", agent.coordination.max_retries);
    
    println!("\n=== Resource Allocation ===");
    println!("Strategy: {:?}", agent.resource_allocation.strategy);
    println!("Scaling: {:?}", agent.resource_allocation.scaling_strategy);
    println!("Budget Allocation:");
    println!("  Strategist: {:.1}%", agent.resource_allocation.budget_allocation.strategist_percentage);
    println!("  Executors: {:.1}%", agent.resource_allocation.budget_allocation.executors_percentage);
    println!("  Infrastructure: {:.1}%", agent.resource_allocation.budget_allocation.infrastructure_percentage);
    
    println!("\n=== Performance Targets ===");
    println!("Success Rate: {:.1}%", agent.performance_targets.success_rate_target * 100.0);
    println!("Quality Score: {:.1}%", agent.performance_targets.quality_score_target * 100.0);
    println!("Latency: {} ms", agent.performance_targets.latency_target_ms);
    println!("Throughput: {:.1} tasks/sec", agent.performance_targets.throughput_target_tps);
    
    println!("\n=== Fallback Strategies ===");
    for (i, strategy) in agent.fallback_strategies.iter().enumerate() {
        println!("Strategy {}: {:?} -> {:?}", i + 1, strategy.trigger, strategy.action);
        println!("  Priority: {}, Timeout: {} ms", strategy.priority, strategy.timeout_ms);
    }
    
    // Calculate metrics
    println!("\n=== Calculated Metrics ===");
    let total_cost = agent.estimated_cost_per_1k_tasks();
    println!("Estimated cost per 1K tasks: ${:.2}", total_cost);
    
    let resources = agent.total_resource_requirements();
    println!("Total resource requirements:");
    println!("  CPU Cores: {}", resources.cpu_cores);
    println!("  Memory: {} MB", resources.memory_mb);
    if let Some(gpu_memory) = resources.gpu_memory_mb {
        println!("  GPU Memory: {} MB", gpu_memory);
    }
    println!("  Disk: {} MB", resources.disk_mb);
    println!("  Network: {} Mbps", resources.network_mbps);
    
    // Calculate total throughput
    let total_throughput: f64 = agent.executors
        .iter()
        .map(|executor| executor.performance.throughput_tps)
        .sum();
    println!("Total executor throughput: {:.1} tasks/sec", total_throughput);
    
    // Serialize to JSON
    let json = serde_json::to_string_pretty(&agent).unwrap();
    println!("\n=== Hybrid Agent JSON (first 600 chars) ===");
    println!("{}...", &json[..600.min(json.len())]);
}