//! Example demonstrating hybrid agent architecture.
//!
//! This example shows how to configure LLM strategists with SLM executors
//! for optimal performance and resource utilization.

use constellation_core::hybrid::coordinator::ExecutorStatus;
use constellation_core::hybrid::{LlmStrategistCoordinator, Task, TaskResult};
use constellation_core::models::hybrid_agent::{
    AllocationStrategy, CommunicationPattern, CoordinationStrategy, CoordinationStrategyType,
    DecisionMakingApproach, ExecutorConfig, ExecutorDomain, ExecutorModelSize, FallbackAction,
    FallbackStrategy, FallbackTrigger, FeedbackMechanism, HybridAgentConfig, ModelProvider,
    PerformanceTargets, ResourceAllocation, ScalingStrategy, StrategistCapability,
    StrategistConfig,
};
use serde_json::json;

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
    strategist
        .capabilities
        .push(StrategistCapability::CreativeThinking);
    strategist
        .capabilities
        .push(StrategistCapability::StrategicThinking);
    agent.strategist = strategist;

    // Add code generation executor
    let mut code_executor =
        ExecutorConfig::new("code_gen_v1".to_string(), ExecutorDomain::CodeGeneration);
    code_executor.model.model_id = "codellama-13b".to_string();
    code_executor.model.size = ExecutorModelSize::Compact;
    code_executor.performance.throughput_tps = 8.0;
    code_executor.performance.avg_latency_ms = 1500;
    code_executor.local_execution = true;
    code_executor.max_concurrent_tasks = 4;

    agent.add_executor(code_executor);

    // Add testing executor
    let mut test_executor =
        ExecutorConfig::new("test_gen_v1".to_string(), ExecutorDomain::CodeGeneration);
    test_executor.model.model_id = "deepseek-coder-6.7b".to_string();
    test_executor.model.specialized_capabilities =
        vec!["test_generation".to_string(), "test_analysis".to_string()];
    test_executor.performance.throughput_tps = 6.0;
    test_executor.performance.avg_latency_ms = 2500;

    agent.add_executor(test_executor);

    // Add research executor
    let mut research_executor =
        ExecutorConfig::new("research_v1".to_string(), ExecutorDomain::Research);
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
    println!(
        "Cost per 1K tokens: ${}",
        agent.strategist.cost_per_1k_tokens
    );
    println!("Capabilities:");
    for capability in &agent.strategist.capabilities {
        println!("  - {capability:?}");
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
        println!(
            "    Throughput: {:.1} tasks/sec",
            executor.performance.throughput_tps
        );
        println!(
            "    Avg Latency: {} ms",
            executor.performance.avg_latency_ms
        );
        println!(
            "    Cost per 1K tasks: ${}",
            executor.performance.cost_per_1k_tasks
        );
    }

    println!("\n=== Coordination Strategy ===");
    println!("Type: {:?}", agent.coordination.strategy_type);
    println!(
        "Communication: {:?}",
        agent.coordination.communication_pattern
    );
    println!("Decision Making: {:?}", agent.coordination.decision_making);
    println!("Feedback: {:?}", agent.coordination.feedback_mechanism);
    println!(
        "Sync Frequency: {} ms",
        agent.coordination.sync_frequency_ms
    );
    println!("Max Retries: {}", agent.coordination.max_retries);

    println!("\n=== Resource Allocation ===");
    println!("Strategy: {:?}", agent.resource_allocation.strategy);
    println!("Scaling: {:?}", agent.resource_allocation.scaling_strategy);
    println!("Budget Allocation:");
    println!(
        "  Strategist: {:.1}%",
        agent
            .resource_allocation
            .budget_allocation
            .strategist_percentage
    );
    println!(
        "  Executors: {:.1}%",
        agent
            .resource_allocation
            .budget_allocation
            .executors_percentage
    );
    println!(
        "  Infrastructure: {:.1}%",
        agent
            .resource_allocation
            .budget_allocation
            .infrastructure_percentage
    );

    println!("\n=== Performance Targets ===");
    println!(
        "Success Rate: {:.1}%",
        agent.performance_targets.success_rate_target * 100.0
    );
    println!(
        "Quality Score: {:.1}%",
        agent.performance_targets.quality_score_target * 100.0
    );
    println!(
        "Latency: {} ms",
        agent.performance_targets.latency_target_ms
    );
    println!(
        "Throughput: {:.1} tasks/sec",
        agent.performance_targets.throughput_target_tps
    );

    println!("\n=== Fallback Strategies ===");
    for (i, strategy) in agent.fallback_strategies.iter().enumerate() {
        println!(
            "Strategy {}: {:?} -> {:?}",
            i + 1,
            strategy.trigger,
            strategy.action
        );
        println!(
            "  Priority: {}, Timeout: {} ms",
            strategy.priority, strategy.timeout_ms
        );
    }

    // Calculate metrics
    println!("\n=== Calculated Metrics ===");
    let total_cost = agent.estimated_cost_per_1k_tasks();
    println!("Estimated cost per 1K tasks: ${total_cost:.2}");

    let resources = agent.total_resource_requirements();
    println!("Total resource requirements:");
    println!("  CPU Cores: {}", resources.cpu_cores);
    println!("  Memory: {} MB", resources.memory_mb);
    if let Some(gpu_memory) = resources.gpu_memory_mb {
        println!("  GPU Memory: {gpu_memory} MB");
    }
    println!("  Disk: {} MB", resources.disk_mb);
    println!("  Network: {} Mbps", resources.network_mbps);

    // Calculate total throughput
    let total_throughput: f64 = agent
        .executors
        .iter()
        .map(|executor| executor.performance.throughput_tps)
        .sum();
    println!("Total executor throughput: {total_throughput:.1} tasks/sec");

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&agent).unwrap();
    println!("\n=== Hybrid Agent JSON (first 600 chars) ===");
    println!("{}...", &json[..600.min(json.len())]);

    // Demonstrate LLM strategist coordinator
    println!("\n=== LLM Strategist Coordinator Demo ===");
    demonstrate_coordinator(agent);
}

fn demonstrate_coordinator(config: HybridAgentConfig) {
    println!("\n--- Initializing Coordinator ---");

    // Create coordinator
    let coordinator = LlmStrategistCoordinator::new(config.clone())
        .with_fallback_strategies(config.fallback_strategies.clone());

    // Register executors
    println!("Registering executors...");
    for executor in &config.executors {
        let status = ExecutorStatus::new(executor.id.clone())
            .with_load(0.3)
            .with_performance(
                executor.performance.availability,
                0.85,
                executor.performance.avg_latency_ms as f64,
            )
            .with_cost(executor.performance.cost_per_1k_tasks / 1000.0)
            .with_availability(true);

        coordinator
            .update_executor_status(status)
            .expect("Failed to update executor status");
        println!("  - {}: registered", executor.id);
    }

    // Submit tasks
    println!("\nSubmitting tasks...");
    let tasks = vec![
        Task::new(
            "code_generation".to_string(),
            json!({"language": "rust", "description": "Create a sorting function"}),
        )
        .with_priority(75)
        .with_quality_requirement(0.9)
        .with_budget_allocation(2.0),
        Task::new(
            "test_generation".to_string(),
            json!({"function": "sort", "language": "rust"}),
        )
        .with_priority(50)
        .with_quality_requirement(0.8)
        .with_budget_allocation(1.5),
        Task::new(
            "research".to_string(),
            json!({"topic": "sorting algorithms", "depth": "intermediate"}),
        )
        .with_priority(25)
        .with_quality_requirement(0.7)
        .with_budget_allocation(1.0),
        Task::new(
            "code_review".to_string(),
            json!({"code": "fn sort() {}", "language": "rust"}),
        )
        .with_priority(100)
        .with_quality_requirement(0.95)
        .with_budget_allocation(3.0),
    ];

    for task in &tasks {
        let task_id = coordinator
            .submit_task(task.clone())
            .expect("Failed to submit task");
        println!(
            "  - Task {} submitted (type: {}, priority: {})",
            task_id, task.task_type, task.priority
        );
    }

    // Assign tasks
    println!("\nAssigning tasks...");
    let assignments = coordinator.assign_tasks().expect("Failed to assign tasks");

    for assignment in &assignments {
        println!(
            "  - Task {} assigned to executor {}",
            assignment.task_id, assignment.executor_id
        );
    }

    // Simulate task completion
    println!("\nSimulating task completion...");
    for assignment in assignments {
        let result = TaskResult {
            task_id: assignment.task_id,
            executor_id: assignment.executor_id,
            completed_at: chrono::Utc::now(),
            result: json!({"output": "Task completed successfully", "quality": 0.9}),
            success: true,
            error: None,
            quality_score: 0.9,
            execution_time_ms: 1500,
            resource_usage: constellation_core::hybrid::ResourceUsage {
                cpu_core_seconds: 0.5,
                memory_mb_seconds: 512.0,
                gpu_memory_mb_seconds: None,
                network_mb: 0.1,
            },
            cost: 0.5,
        };

        coordinator
            .complete_task(result)
            .expect("Failed to complete task");
        println!("  - Task {} completed", assignment.task_id);
    }

    // Get performance metrics
    println!("\nPerformance Metrics:");
    let metrics = coordinator.get_performance_metrics();
    println!("  - Throughput: {:.2} tasks/sec", metrics.throughput_tps);
    println!("  - Avg Latency: {:.2} ms", metrics.avg_latency_ms);
    println!("  - Success Rate: {:.2}%", metrics.success_rate * 100.0);
    println!("  - Avg Quality: {:.2}%", metrics.avg_quality_score * 100.0);
    println!(
        "  - Resource Utilization: {:.2}%",
        metrics.resource_utilization * 100.0
    );
    println!(
        "  - Cost Efficiency: {:.2}%",
        metrics.cost_efficiency * 100.0
    );
    println!("  - Availability: {:.2}%", metrics.availability * 100.0);

    // Get queue stats
    println!("\nQueue Statistics:");
    let queue_stats = coordinator.get_queue_stats();
    println!("  - Pending Tasks: {}", queue_stats.pending_tasks);
    println!("  - Active Tasks: {}", queue_stats.active_tasks);
    println!("  - Completed Tasks: {}", queue_stats.completed_tasks);
    println!("  - Total Processed: {}", queue_stats.total_tasks_processed);
    println!(
        "  - Total Budget Spent: ${:.2}",
        queue_stats.total_budget_spent
    );

    // Get executor stats
    println!("\nExecutor Statistics:");
    let executor_stats = coordinator.get_executor_stats();
    for stats in executor_stats {
        println!("  - {}:", stats.executor_id);
        println!("      Load: {:.1}%", stats.current_load * 100.0);
        println!("      Available Capacity: {}", stats.available_capacity);
        println!("      Success Rate: {:.1}%", stats.success_rate * 100.0);
        println!("      Quality Score: {:.1}%", stats.quality_score * 100.0);
        println!("      Cost per Task: ${:.4}", stats.cost_per_task);
        println!("      Available: {}", stats.is_available);
    }

    // Check fallback conditions
    println!("\nChecking fallback conditions...");
    let fallback_actions = coordinator.check_fallback_conditions();
    if fallback_actions.is_empty() {
        println!("  - No fallback conditions triggered");
    } else {
        println!("  - Fallback actions needed:");
        for action in fallback_actions {
            println!("    - {action:?}");
        }
    }

    println!("\n=== Coordinator Demo Complete ===");
}
