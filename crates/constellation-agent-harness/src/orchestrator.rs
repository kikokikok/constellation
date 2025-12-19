//! Multi-agent orchestrator for coordinating specialized agents
//!
//! This module implements a neuroscience-inspired orchestrator that can:
//! 1. Coordinate specialized agents (research, coding, testing, deployment)
//! 2. Manage token budgets across agents
//! 3. Handle session handoffs and state preservation
//! 4. Use the memory system for knowledge sharing between agents
//! 5. Implement token efficiency strategies based on cognitive load theory

use chrono::{DateTime, Utc};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::plugin::PluginRegistry;
use crate::session::SessionManager;
use crate::skill::SkillRegistry;
use constellation_core::memory::prelude::*;

/// Agent role specialization
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AgentRole {
    /// Research and analysis agent
    Researcher,
    /// Code implementation agent
    Coder,
    /// Testing and quality assurance agent
    Tester,
    /// Deployment and operations agent
    Deployer,
    /// Business and strategy agent
    Strategist,
    /// Integration and coordination agent
    Integrator,
    /// Specialized agent for specific domains
    Specialist(String),
    /// Revenue generation and optimization agent
    RevenueAgent,
    /// Operations and efficiency agent
    OperationsAgent,
    /// Strategic planning and market analysis agent
    StrategyAgent,
    /// Business intelligence and metrics agent
    IntelligenceAgent,
}

impl AgentRole {
    /// Get the default skill set for this role
    pub fn default_skills(&self) -> Vec<&'static str> {
        match self {
            AgentRole::Researcher => vec![
                "research_analysis",
                "problem_decomposition",
                "technical_specification",
                "architecture_design",
            ],
            AgentRole::Coder => vec![
                "code_implementation",
                "refactoring",
                "debugging",
                "code_review",
                "performance_optimization",
            ],
            AgentRole::Tester => vec![
                "test_planning",
                "test_implementation",
                "test_execution",
                "bug_reporting",
                "quality_assurance",
            ],
            AgentRole::Deployer => vec![
                "deployment_planning",
                "infrastructure_setup",
                "monitoring_setup",
                "rollback_procedures",
            ],
            AgentRole::Strategist => vec![
                "business_analysis",
                "market_research",
                "competitive_analysis",
                "strategy_planning",
            ],
            AgentRole::Integrator => vec![
                "system_integration",
                "api_design",
                "data_migration",
                "legacy_system_adaptation",
            ],
            AgentRole::Specialist(domain) => match domain.as_str() {
                "ai" => vec!["machine_learning", "neural_networks", "data_science"],
                "web" => vec!["frontend_development", "backend_development", "api_design"],
                "mobile" => vec!["ios_development", "android_development", "cross_platform"],
                "devops" => vec!["ci_cd", "containerization", "cloud_infrastructure"],
                "security" => vec!["security_audit", "penetration_testing", "cryptography"],
                _ => vec!["domain_expertise"],
            },
            AgentRole::RevenueAgent => vec![
                "financial_analysis",
                "pricing_strategy",
                "revenue_modeling",
                "customer_segmentation",
                "market_research",
            ],
            AgentRole::OperationsAgent => vec![
                "process_optimization",
                "cost_analysis",
                "automation",
                "infrastructure",
                "resource_management",
            ],
            AgentRole::StrategyAgent => vec![
                "market_analysis",
                "strategic_thinking",
                "competitive_analysis",
                "business_model_innovation",
                "scenario_planning",
            ],
            AgentRole::IntelligenceAgent => vec![
                "data_analysis",
                "metrics_tracking",
                "predictive_modeling",
                "dashboard_creation",
                "kpi_definition",
            ],
        }
    }

    /// Get the default token budget for this role
    pub fn default_token_budget(&self) -> u32 {
        match self {
            AgentRole::Researcher => 8000,        // Research requires more context
            AgentRole::Coder => 6000,             // Coding requires moderate context
            AgentRole::Tester => 4000,            // Testing requires less context
            AgentRole::Deployer => 3000,          // Deployment requires minimal context
            AgentRole::Strategist => 10000,       // Strategy requires extensive context
            AgentRole::Integrator => 7000,        // Integration requires moderate context
            AgentRole::Specialist(_) => 5000,     // Specialists vary by domain
            AgentRole::RevenueAgent => 10000,     // Revenue analysis requires extensive context
            AgentRole::OperationsAgent => 8000,   // Operations requires moderate context
            AgentRole::StrategyAgent => 12000,    // Strategic planning requires extensive context
            AgentRole::IntelligenceAgent => 9000, // Intelligence requires data analysis context
        }
    }

    /// Get the default cognitive load factor
    pub fn cognitive_load_factor(&self) -> f32 {
        match self {
            AgentRole::Researcher => 0.8,         // High cognitive load
            AgentRole::Coder => 0.6,              // Moderate cognitive load
            AgentRole::Tester => 0.4,             // Lower cognitive load
            AgentRole::Deployer => 0.3,           // Procedural cognitive load
            AgentRole::Strategist => 0.9,         // Very high cognitive load
            AgentRole::Integrator => 0.7,         // High cognitive load
            AgentRole::Specialist(_) => 0.5,      // Variable cognitive load
            AgentRole::RevenueAgent => 0.85,      // High cognitive load for financial analysis
            AgentRole::OperationsAgent => 0.65, // Moderate cognitive load for process optimization
            AgentRole::StrategyAgent => 0.95,   // Very high cognitive load for strategic thinking
            AgentRole::IntelligenceAgent => 0.75, // High cognitive load for data analysis
        }
    }
}

/// Task assignment for an agent
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskAssignment {
    /// Unique task ID
    pub task_id: String,
    /// Task description
    pub description: String,
    /// Expected outcome
    pub expected_outcome: String,
    /// Priority level (1-10)
    pub priority: u8,
    /// Estimated token cost
    pub estimated_tokens: u32,
    /// Time estimate in minutes
    pub time_estimate: u32,
    /// Dependencies on other tasks
    pub dependencies: Vec<String>,
    /// Required skills
    pub required_skills: Vec<String>,
    /// Context memory IDs to include
    pub context_memory_ids: Vec<String>,
    /// Maximum retries
    pub max_retries: u32,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

impl TaskAssignment {
    /// Create a new task assignment
    pub fn new(
        description: String,
        expected_outcome: String,
        priority: u8,
        estimated_tokens: u32,
        time_estimate: u32,
        required_skills: Vec<String>,
    ) -> Self {
        Self {
            task_id: Uuid::new_v4().to_string(),
            description,
            expected_outcome,
            priority: priority.clamp(1, 10),
            estimated_tokens,
            time_estimate,
            dependencies: Vec::new(),
            required_skills,
            context_memory_ids: Vec::new(),
            max_retries: 3,
            created_at: Utc::now(),
        }
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, task_id: String) {
        if !self.dependencies.contains(&task_id) {
            self.dependencies.push(task_id);
        }
    }

    /// Add context memory
    pub fn add_context_memory(&mut self, memory_id: String) {
        if !self.context_memory_ids.contains(&memory_id) {
            self.context_memory_ids.push(memory_id);
        }
    }

    /// Check if task is ready (all dependencies satisfied)
    pub fn is_ready(&self, completed_tasks: &HashMap<String, bool>) -> bool {
        self.dependencies
            .iter()
            .all(|dep_id| completed_tasks.get(dep_id).copied().unwrap_or(false))
    }

    /// Calculate urgency score based on priority and time
    pub fn urgency_score(&self) -> f32 {
        let priority_weight = self.priority as f32 / 10.0;
        let now = Utc::now();
        let elapsed = now - self.created_at;
        let elapsed_secs = elapsed.num_seconds() as f32;
        let time_weight = 1.0 - (elapsed_secs / (self.time_estimate as f32 * 60.0));
        priority_weight * 0.7 + time_weight * 0.3
    }
}

/// Agent status
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AgentStatus {
    /// Agent is idle and available
    Idle,
    /// Agent is working on a task
    Working {
        task_id: String,
        started_at: DateTime<Utc>,
    },
    /// Agent is waiting for dependencies
    Waiting {
        task_id: String,
        waiting_for: Vec<String>,
    },
    /// Agent has completed a task
    Completed {
        task_id: String,
        completed_at: DateTime<Utc>,
    },
    /// Agent has failed a task
    Failed {
        task_id: String,
        error: String,
        retry_count: u32,
    },
}

/// Agent instance
#[derive(Debug, Clone)]
pub struct AgentInstance {
    /// Agent ID
    pub id: String,
    /// Agent role
    pub role: AgentRole,
    /// Current status
    pub status: AgentStatus,
    /// Available skills
    pub skills: Vec<String>,
    /// Current token usage
    pub token_usage: u32,
    /// Token budget
    pub token_budget: u32,
    /// Cognitive load factor
    pub cognitive_load_factor: f32,
    /// Session ID
    pub session_id: Option<String>,
    /// Performance metrics
    pub metrics: AgentMetrics,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

impl AgentInstance {
    /// Create a new agent instance
    pub fn new(role: AgentRole, skills: Vec<String>) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            id,
            role: role.clone(),
            status: AgentStatus::Idle,
            skills,
            token_usage: 0,
            token_budget: role.default_token_budget(),
            cognitive_load_factor: role.cognitive_load_factor(),
            session_id: None,
            metrics: AgentMetrics::default(),
            created_at: Utc::now(),
        }
    }

    /// Check if agent is available for work
    pub fn is_available(&self) -> bool {
        matches!(self.status, AgentStatus::Idle)
    }

    /// Check if agent has required skills
    pub fn has_skills(&self, required_skills: &[String]) -> bool {
        required_skills
            .iter()
            .all(|skill| self.skills.contains(skill))
    }

    /// Calculate availability score for a task
    pub fn availability_score(&self, task: &TaskAssignment) -> f32 {
        let skill_match = if self.has_skills(&task.required_skills) {
            1.0
        } else {
            // Partial skill match
            let matched = task
                .required_skills
                .iter()
                .filter(|skill| self.skills.contains(skill))
                .count();
            matched as f32 / task.required_skills.len() as f32
        };

        let token_availability = 1.0 - (self.token_usage as f32 / self.token_budget as f32);
        let cognitive_load = 1.0 - self.cognitive_load_factor;

        // Weighted score
        skill_match * 0.5 + token_availability * 0.3 + cognitive_load * 0.2
    }

    /// Start working on a task
    pub fn start_task(&mut self, task_id: String) -> Result<()> {
        if !self.is_available() {
            return Err(Error::AgentBusy(self.id.clone()));
        }

        self.status = AgentStatus::Working {
            task_id: task_id.clone(),
            started_at: Utc::now(),
        };
        self.metrics.tasks_started += 1;

        debug!("Agent {} started task {}", self.id, task_id);
        Ok(())
    }

    /// Complete a task
    pub fn complete_task(&mut self, task_id: String, tokens_used: u32) -> Result<()> {
        match &self.status {
            AgentStatus::Working {
                task_id: current_task_id,
                ..
            } if current_task_id == &task_id => {
                self.status = AgentStatus::Completed {
                    task_id,
                    completed_at: Utc::now(),
                };
                self.token_usage += tokens_used;
                self.metrics.tasks_completed += 1;
                self.metrics.total_tokens_used += tokens_used;
                Ok(())
            }
            _ => Err(Error::InvalidAgentState(format!(
                "Agent {} is not working on task {}",
                self.id, task_id
            ))),
        }
    }

    /// Fail a task
    pub fn fail_task(&mut self, task_id: String, error: String, retry_count: u32) -> Result<()> {
        match &self.status {
            AgentStatus::Working {
                task_id: current_task_id,
                ..
            } if current_task_id == &task_id => {
                self.status = AgentStatus::Failed {
                    task_id,
                    error,
                    retry_count,
                };
                self.metrics.tasks_failed += 1;
                Ok(())
            }
            _ => Err(Error::InvalidAgentState(format!(
                "Agent {} is not working on task {}",
                self.id, task_id
            ))),
        }
    }

    /// Reset agent to idle state
    pub fn reset(&mut self) {
        self.status = AgentStatus::Idle;
        self.session_id = None;
    }

    /// Get current task ID if working
    pub fn current_task_id(&self) -> Option<&str> {
        match &self.status {
            AgentStatus::Working { task_id, .. } => Some(task_id),
            _ => None,
        }
    }
}

/// Agent performance metrics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AgentMetrics {
    /// Total tasks started
    pub tasks_started: u32,
    /// Total tasks completed
    pub tasks_completed: u32,
    /// Total tasks failed
    pub tasks_failed: u32,
    /// Total tokens used
    pub total_tokens_used: u32,
    /// Average task completion time in seconds
    pub avg_completion_time: f32,
    /// Success rate
    pub success_rate: f32,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

impl AgentMetrics {
    /// Update metrics after task completion
    pub fn update_after_completion(&mut self, completion_time: Duration) {
        self.tasks_completed += 1;

        // Update average completion time
        let total_time = self.avg_completion_time * (self.tasks_completed - 1) as f32;
        self.avg_completion_time =
            (total_time + completion_time.as_secs_f32()) / self.tasks_completed as f32;

        // Update success rate
        self.success_rate = self.tasks_completed as f32 / self.tasks_started as f32;
        self.last_updated = Utc::now();
    }

    /// Get efficiency score (higher is better)
    pub fn efficiency_score(&self) -> f32 {
        if self.tasks_started == 0 {
            return 0.0;
        }

        let completion_rate = self.tasks_completed as f32 / self.tasks_started as f32;
        let token_efficiency = if self.total_tokens_used > 0 {
            self.tasks_completed as f32 / self.total_tokens_used as f32 * 1000.0
        } else {
            1.0
        };

        completion_rate * 0.6 + token_efficiency * 0.4
    }
}

/// Multi-agent orchestrator
pub struct Orchestrator {
    /// Available agents
    agents: Arc<RwLock<HashMap<String, AgentInstance>>>,
    /// Task queue
    task_queue: Arc<Mutex<VecDeque<TaskAssignment>>>,
    /// Completed tasks
    completed_tasks: Arc<RwLock<HashMap<String, bool>>>,
    /// Failed tasks
    failed_tasks: Arc<RwLock<HashMap<String, (String, u32)>>>,
    /// Memory system for knowledge sharing
    memory_system: Arc<MemorySystem>,
    /// Communication framework for agent coordination (placeholder)
    _communication: Arc<()>,
    /// Plugin registry
    #[allow(dead_code)]
    plugin_registry: Arc<PluginRegistry>,
    /// Skill registry
    #[allow(dead_code)]
    skill_registry: Arc<SkillRegistry>,
    /// Session manager
    #[allow(dead_code)]
    session_manager: Arc<SessionManager>,
    /// Configuration
    config: OrchestratorConfig,
    /// Background task handle
    #[allow(dead_code)]
    background_task: Option<tokio::task::JoinHandle<()>>,
}

/// Orchestrator configuration
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Maximum concurrent agents
    pub max_concurrent_agents: usize,
    /// Task assignment interval
    pub task_assignment_interval: Duration,
    /// Agent cleanup interval
    pub agent_cleanup_interval: Duration,
    /// Maximum task queue size
    pub max_task_queue_size: usize,
    /// Enable token efficiency optimization
    pub enable_token_efficiency: bool,
    /// Enable cognitive load balancing
    pub enable_cognitive_load_balancing: bool,
    /// Enable memory consolidation
    pub enable_memory_consolidation: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_agents: 10,
            task_assignment_interval: Duration::from_secs(5),
            agent_cleanup_interval: Duration::from_secs(300), // 5 minutes
            max_task_queue_size: 1000,
            enable_token_efficiency: true,
            enable_cognitive_load_balancing: true,
            enable_memory_consolidation: true,
        }
    }
}

impl Orchestrator {
    /// Create a new orchestrator
    pub fn new(
        memory_system: Arc<MemorySystem>,
        communication: Arc<()>,
        plugin_registry: Arc<PluginRegistry>,
        skill_registry: Arc<SkillRegistry>,
        session_manager: Arc<SessionManager>,
    ) -> Self {
        Self::with_config(
            memory_system,
            communication,
            plugin_registry,
            skill_registry,
            session_manager,
            OrchestratorConfig::default(),
        )
    }

    /// Create a new orchestrator with custom configuration
    pub fn with_config(
        memory_system: Arc<MemorySystem>,
        _communication: Arc<()>,
        plugin_registry: Arc<PluginRegistry>,
        skill_registry: Arc<SkillRegistry>,
        session_manager: Arc<SessionManager>,
        config: OrchestratorConfig,
    ) -> Self {
        let orchestrator = Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            completed_tasks: Arc::new(RwLock::new(HashMap::new())),
            failed_tasks: Arc::new(RwLock::new(HashMap::new())),
            memory_system,
            _communication: Arc::new(()),
            plugin_registry,
            skill_registry,
            session_manager,
            config,
            background_task: None,
        };

        // Start background tasks
        orchestrator.start_background_tasks();

        orchestrator
    }

    /// Start background maintenance tasks
    fn start_background_tasks(&self) {
        let _agents = self.agents.clone();
        let task_queue = self.task_queue.clone();
        let completed_tasks = self.completed_tasks.clone();
        let failed_tasks = self.failed_tasks.clone();
        let _memory_system = self.memory_system.clone();
        let config = self.config.clone();
        let agents = self.agents.clone();
        let memory_system = self.memory_system.clone();
        let assignment_interval = config.task_assignment_interval;

        // Clone values for first closure
        let agents1 = agents.clone();
        let task_queue1 = task_queue.clone();
        let completed_tasks1 = completed_tasks.clone();
        let failed_tasks1 = failed_tasks.clone();
        let memory_system1 = memory_system.clone();
        let config1 = config.clone();

        // Task assignment task
        tokio::spawn(async move {
            let mut interval = time::interval(assignment_interval);
            loop {
                interval.tick().await;
                if let Err(e) = Self::assign_tasks_task(
                    &agents1,
                    &task_queue1,
                    &completed_tasks1,
                    &failed_tasks1,
                    &memory_system1,
                    &config1,
                )
                .await
                {
                    warn!("Task assignment failed: {}", e);
                }
            }
        });

        // Clone values for second closure
        let agents2 = agents.clone();
        let config2 = config.clone();

        // Agent cleanup task
        let cleanup_interval = config.agent_cleanup_interval;
        tokio::spawn(async move {
            let mut interval = time::interval(cleanup_interval);
            loop {
                interval.tick().await;
                if let Err(e) = Self::cleanup_agents_task(&agents2, &config2).await {
                    warn!("Agent cleanup failed: {}", e);
                }
            }
        });

        if config.enable_memory_consolidation {
            let _memory_system3 = memory_system.clone();
            tokio::spawn(async move {
                let mut interval = time::interval(Duration::from_secs(3600)); // Every hour
                loop {
                    interval.tick().await;
                    // MemorySystem doesn't have mutable access through Arc
                    // This is a placeholder - real implementation would need different approach
                    warn!("Memory consolidation not implemented (requires mutable access)");
                }
            });
        }
    }

    /// Register a new agent
    pub async fn register_agent(&self, role: AgentRole, skills: Vec<String>) -> Result<String> {
        let agent = AgentInstance::new(role.clone(), skills);
        let agent_id = agent.id.clone();

        let mut agents = self.agents.write().await;
        agents.insert(agent_id.clone(), agent);

        info!("Registered agent {} with role {:?}", agent_id, role);
        Ok(agent_id)
    }

    /// Submit a task to the orchestrator
    pub async fn submit_task(&self, task: TaskAssignment) -> Result<String> {
        let task_id = task.task_id.clone();

        // Check queue size
        let mut queue = self.task_queue.lock().await;
        if queue.len() >= self.config.max_task_queue_size {
            return Err(Error::TaskQueueFull(self.config.max_task_queue_size));
        }

        queue.push_back(task);
        info!("Submitted task {} to orchestrator", task_id);

        Ok(task_id)
    }

    /// Get agent status
    pub async fn get_agent_status(&self, agent_id: &str) -> Result<AgentStatus> {
        let agents = self.agents.read().await;
        agents
            .get(agent_id)
            .map(|agent| agent.status.clone())
            .ok_or_else(|| Error::AgentNotFound(agent_id.to_string()))
    }

    /// Get task status
    pub async fn get_task_status(&self, task_id: &str) -> Result<TaskStatus> {
        let completed_tasks = self.completed_tasks.read().await;
        let failed_tasks = self.failed_tasks.read().await;

        if completed_tasks.contains_key(task_id) {
            Ok(TaskStatus::Completed)
        } else if let Some((error, retry_count)) = failed_tasks.get(task_id) {
            Ok(TaskStatus::Failed {
                error: error.clone(),
                retry_count: *retry_count,
            })
        } else {
            // Check if task is in queue or assigned
            let queue = self.task_queue.lock().await;
            let agents = self.agents.read().await;

            // Check if task is in queue
            if queue.iter().any(|task| task.task_id == task_id) {
                Ok(TaskStatus::Queued)
            } else {
                // Check if task is assigned to an agent
                for agent in agents.values() {
                    if let Some(current_task_id) = agent.current_task_id() {
                        if current_task_id == task_id {
                            return Ok(TaskStatus::InProgress {
                                agent_id: agent.id.clone(),
                                started_at: match &agent.status {
                                    AgentStatus::Working { started_at, .. } => *started_at,
                                    _ => Utc::now(),
                                },
                            });
                        }
                    }
                }
                Ok(TaskStatus::Unknown)
            }
        }
    }

    /// Get orchestrator metrics
    pub async fn get_metrics(&self) -> OrchestratorMetrics {
        let agents = self.agents.read().await;
        let queue = self.task_queue.lock().await;
        let completed_tasks = self.completed_tasks.read().await;
        let failed_tasks = self.failed_tasks.read().await;

        let mut metrics = OrchestratorMetrics::default();

        // Agent metrics
        for agent in agents.values() {
            metrics.total_agents += 1;
            match agent.status {
                AgentStatus::Idle => metrics.idle_agents += 1,
                AgentStatus::Working { .. } => metrics.working_agents += 1,
                AgentStatus::Waiting { .. } => metrics.waiting_agents += 1,
                AgentStatus::Completed { .. } => metrics.completed_agents += 1,
                AgentStatus::Failed { .. } => metrics.failed_agents += 1,
            }

            metrics.total_tokens_used += agent.metrics.total_tokens_used;
            metrics.total_tasks_started += agent.metrics.tasks_started;
            metrics.total_tasks_completed += agent.metrics.tasks_completed;
            metrics.total_tasks_failed += agent.metrics.tasks_failed;
        }

        // Queue metrics
        metrics.queued_tasks = queue.len() as u32;
        metrics.completed_tasks = completed_tasks.len() as u32;
        metrics.failed_tasks = failed_tasks.len() as u32;

        // Calculate success rate
        let total_tasks = metrics.total_tasks_started;
        if total_tasks > 0 {
            metrics.success_rate = metrics.total_tasks_completed as f32 / total_tasks as f32;
        }

        // Calculate token efficiency
        if metrics.total_tokens_used > 0 {
            metrics.token_efficiency =
                metrics.total_tasks_completed as f32 / metrics.total_tokens_used as f32 * 1000.0;
        }

        metrics
    }

    /// Static method for task assignment background task
    async fn assign_tasks_task(
        agents: &Arc<RwLock<HashMap<String, AgentInstance>>>,
        task_queue: &Arc<Mutex<VecDeque<TaskAssignment>>>,
        completed_tasks: &Arc<RwLock<HashMap<String, bool>>>,
        failed_tasks: &Arc<RwLock<HashMap<String, (String, u32)>>>,
        _memory_system: &Arc<MemorySystem>,
        _config: &OrchestratorConfig,
    ) -> Result<()> {
        let mut queue = task_queue.lock().await;
        let mut agents = agents.write().await;
        let completed_tasks = completed_tasks.read().await;
        let _failed_tasks = failed_tasks.read().await;

        // Find available agents
        let mut available_agents: Vec<&mut AgentInstance> = agents
            .values_mut()
            .filter(|agent| agent.is_available())
            .collect();

        if available_agents.is_empty() || queue.is_empty() {
            return Ok(());
        }

        // Sort tasks by urgency
        let mut task_ids: Vec<String> = queue.iter().map(|t| t.task_id.clone()).collect();

        // Sort by urgency score (need to calculate for each)
        task_ids.sort_by(|a_id, b_id| {
            let a = queue.iter().find(|t| t.task_id == *a_id).unwrap();
            let b = queue.iter().find(|t| t.task_id == *b_id).unwrap();
            b.urgency_score().partial_cmp(&a.urgency_score()).unwrap()
        });

        // Assign tasks to available agents
        for task_id in task_ids {
            // Find the task in the queue
            let task_index = queue.iter().position(|t| t.task_id == task_id);
            if task_index.is_none() {
                continue;
            }
            let task_index = task_index.unwrap();
            let task = &queue[task_index];

            // Check if task is ready (dependencies satisfied)
            if !task.is_ready(&completed_tasks) {
                continue;
            }

            // Find best agent for this task
            let best_agent = available_agents.iter_mut().max_by(|a, b| {
                a.availability_score(task)
                    .partial_cmp(&b.availability_score(task))
                    .unwrap()
            });

            if let Some(agent) = best_agent {
                // Remove task from queue
                let task = queue.remove(task_index).unwrap();

                // Start agent on task
                if let Err(e) = agent.start_task(task.task_id.clone()) {
                    warn!(
                        "Failed to start agent {} on task {}: {}",
                        agent.id, task.task_id, e
                    );
                    // Put task back in queue
                    queue.push_back(task);
                } else {
                    // Store task context in memory
                    // Note: Simplified memory system - memory retrieval not implemented
                    if !task.context_memory_ids.is_empty() {
                        debug!(
                            "Task {} has context memory IDs: {:?}",
                            task.task_id, task.context_memory_ids
                        );
                    }

                    debug!("Assigned task {} to agent {}", task.task_id, agent.id);
                }
            }
        }

        Ok(())
    }

    /// Static method for agent cleanup background task
    async fn cleanup_agents_task(
        agents: &Arc<RwLock<HashMap<String, AgentInstance>>>,
        config: &OrchestratorConfig,
    ) -> Result<()> {
        let mut agents = agents.write().await;
        let mut to_remove = Vec::new();

        for (agent_id, agent) in agents.iter() {
            // Remove idle agents that have been idle for too long
            if matches!(agent.status, AgentStatus::Idle) {
                let idle_time = Utc::now().signed_duration_since(agent.created_at);
                if idle_time > chrono::Duration::seconds(3600) {
                    // 1 hour
                    to_remove.push(agent_id.clone());
                }
            }

            // Remove failed agents with too many retries
            if let AgentStatus::Failed { retry_count, .. } = &agent.status {
                if *retry_count >= 5 {
                    to_remove.push(agent_id.clone());
                }
            }
        }

        // Remove agents
        for agent_id in to_remove {
            agents.remove(&agent_id);
            debug!("Removed agent {}", agent_id);
        }

        // Ensure we don't exceed max concurrent agents
        if agents.len() > config.max_concurrent_agents {
            let excess = agents.len() - config.max_concurrent_agents;
            let mut sorted_agents: Vec<(String, DateTime<Utc>)> = agents
                .iter()
                .filter(|(_, agent)| matches!(agent.status, AgentStatus::Idle))
                .map(|(id, agent)| (id.clone(), agent.created_at))
                .collect();

            sorted_agents.sort_by(|a, b| a.1.cmp(&b.1)); // Oldest first

            for (agent_id, _) in sorted_agents.iter().take(excess) {
                agents.remove(agent_id);
                debug!("Removed excess agent {}", agent_id);
            }
        }

        Ok(())
    }
}

/// Task status
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TaskStatus {
    /// Task is queued
    Queued,
    /// Task is in progress
    InProgress {
        agent_id: String,
        started_at: DateTime<Utc>,
    },
    /// Task is completed
    Completed,
    /// Task has failed
    Failed { error: String, retry_count: u32 },
    /// Task status is unknown
    Unknown,
}

/// Orchestrator metrics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct OrchestratorMetrics {
    /// Total agents
    pub total_agents: u32,
    /// Idle agents
    pub idle_agents: u32,
    /// Working agents
    pub working_agents: u32,
    /// Waiting agents
    pub waiting_agents: u32,
    /// Completed agents
    pub completed_agents: u32,
    /// Failed agents
    pub failed_agents: u32,
    /// Queued tasks
    pub queued_tasks: u32,
    /// Completed tasks
    pub completed_tasks: u32,
    /// Failed tasks
    pub failed_tasks: u32,
    /// Total tasks started
    pub total_tasks_started: u32,
    /// Total tasks completed
    pub total_tasks_completed: u32,
    /// Total tasks failed
    pub total_tasks_failed: u32,
    /// Total tokens used
    pub total_tokens_used: u32,
    /// Success rate
    pub success_rate: f32,
    /// Token efficiency (tasks per 1000 tokens)
    pub token_efficiency: f32,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

impl OrchestratorMetrics {
    /// Get overall health score (0-100)
    pub fn health_score(&self) -> f32 {
        let mut score = 0.0;

        // Agent utilization
        if self.total_agents > 0 {
            let utilization =
                (self.working_agents + self.waiting_agents) as f32 / self.total_agents as f32;
            score += utilization * 25.0;
        }

        // Task success rate
        score += self.success_rate * 25.0;

        // Queue health
        let total_tasks = self.queued_tasks + self.completed_tasks + self.failed_tasks;
        if total_tasks > 0 {
            let completion_rate = self.completed_tasks as f32 / total_tasks as f32;
            score += completion_rate * 25.0;
        }

        // Token efficiency (normalized)
        let token_efficiency = self.token_efficiency.min(10.0) / 10.0;
        score += token_efficiency * 25.0;

        score
    }
}

/// Pre-defined task templates for common software development workflows
pub mod task_templates {
    use super::*;

    /// Create a research task
    pub fn research_task(description: String, expected_outcome: String) -> TaskAssignment {
        TaskAssignment::new(
            description,
            expected_outcome,
            7,    // Medium-high priority
            4000, // Estimated tokens
            60,   // 1 hour estimate
            vec![
                "research_analysis".to_string(),
                "problem_decomposition".to_string(),
                "technical_specification".to_string(),
            ],
        )
    }

    /// Create a coding task
    pub fn coding_task(
        description: String,
        expected_outcome: String,
        complexity: u8,
    ) -> TaskAssignment {
        let (estimated_tokens, time_estimate) = match complexity {
            1 => (2000, 30),  // Simple: 30 minutes
            2 => (4000, 60),  // Medium: 1 hour
            3 => (6000, 120), // Complex: 2 hours
            _ => (4000, 60),  // Default: medium
        };

        TaskAssignment::new(
            description,
            expected_outcome,
            8, // High priority
            estimated_tokens,
            time_estimate,
            vec![
                "code_implementation".to_string(),
                "debugging".to_string(),
                "code_review".to_string(),
            ],
        )
    }

    /// Create a testing task
    pub fn testing_task(
        description: String,
        expected_outcome: String,
        test_type: &str,
    ) -> TaskAssignment {
        let required_skills = match test_type {
            "unit" => vec![
                "test_implementation".to_string(),
                "test_execution".to_string(),
            ],
            "integration" => vec![
                "test_planning".to_string(),
                "system_integration".to_string(),
            ],
            "e2e" => vec!["test_planning".to_string(), "quality_assurance".to_string()],
            _ => vec!["test_execution".to_string()],
        };

        TaskAssignment::new(
            description,
            expected_outcome,
            6,    // Medium priority
            3000, // Estimated tokens
            45,   // 45 minutes estimate
            required_skills,
        )
    }

    /// Create a deployment task
    pub fn deployment_task(
        description: String,
        expected_outcome: String,
        environment: &str,
    ) -> TaskAssignment {
        let priority = match environment {
            "production" => 9,  // Very high priority
            "staging" => 7,     // Medium-high priority
            "development" => 5, // Medium priority
            _ => 6,             // Default
        };

        TaskAssignment::new(
            description,
            expected_outcome,
            priority,
            2500, // Estimated tokens
            30,   // 30 minutes estimate
            vec![
                "deployment_planning".to_string(),
                "infrastructure_setup".to_string(),
                "rollback_procedures".to_string(),
            ],
        )
    }

    /// Create a business analysis task
    pub fn business_analysis_task(description: String, expected_outcome: String) -> TaskAssignment {
        TaskAssignment::new(
            description,
            expected_outcome,
            5,    // Medium priority
            5000, // Estimated tokens
            90,   // 1.5 hours estimate
            vec![
                "business_analysis".to_string(),
                "market_research".to_string(),
                "competitive_analysis".to_string(),
            ],
        )
    }
}
