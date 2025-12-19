pub mod integration;
pub mod intelligence;
pub mod operations;
pub mod revenue;
pub mod strategy;

use crate::orchestrator::{AgentRole, TaskAssignment, TaskStatus};
use crate::skill::Skill;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetrics {
    pub revenue: f64,
    pub expenses: f64,
    pub profit_margin: f64,
    pub customer_acquisition_cost: f64,
    pub lifetime_value: f64,
    pub monthly_recurring_revenue: f64,
    pub churn_rate: f64,
    pub active_users: u64,
    pub conversion_rate: f64,
    pub operational_efficiency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessGoal {
    pub id: String,
    pub description: String,
    pub target_metrics: BusinessMetrics,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub priority: u8,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketAnalysis {
    pub market_size: f64,
    pub growth_rate: f64,
    pub competitors: Vec<Competitor>,
    pub trends: Vec<String>,
    pub opportunities: Vec<BusinessOpportunity>,
    pub threats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Competitor {
    pub name: String,
    pub market_share: f64,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub pricing_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessOpportunity {
    pub description: String,
    pub estimated_value: f64,
    pub effort_required: u32,
    pub risk_level: RiskLevel,
    pub time_to_market: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingStrategy {
    pub model: PricingModel,
    pub price_points: Vec<PricePoint>,
    pub discount_strategy: Option<DiscountStrategy>,
    pub tiered_pricing: Option<Vec<Tier>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricingModel {
    Subscription,
    Freemium,
    Tiered,
    UsageBased,
    OneTime,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub name: String,
    pub price: f64,
    pub features: Vec<String>,
    pub target_audience: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscountStrategy {
    pub annual_discount: f64,
    pub volume_discount: f64,
    pub promotional_discounts: Vec<Promotion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier {
    pub name: String,
    pub price: f64,
    pub features: Vec<String>,
    pub limits: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Promotion {
    pub name: String,
    pub discount_percentage: f64,
    pub duration_days: u32,
    pub target_segment: String,
}

pub trait BusinessAgent {
    fn analyze(&self, context: &BusinessContext) -> Result<BusinessAnalysis, BusinessError>;
    fn execute(&self, task: &BusinessTask) -> Result<BusinessResult, BusinessError>;
    fn get_skills(&self) -> Vec<Skill>;
    fn get_role(&self) -> AgentRole;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessContext {
    pub current_metrics: BusinessMetrics,
    pub goals: Vec<BusinessGoal>,
    pub market_analysis: MarketAnalysis,
    pub constraints: Vec<String>,
    pub available_resources: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessAnalysis {
    pub recommendations: Vec<Recommendation>,
    pub risk_assessment: RiskAssessment,
    pub expected_outcomes: Vec<ExpectedOutcome>,
    pub resource_requirements: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub action: String,
    pub rationale: String,
    pub priority: u8,
    pub estimated_impact: f64,
    pub effort_required: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub risks: Vec<Risk>,
    pub mitigation_strategies: Vec<MitigationStrategy>,
    pub overall_risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub description: String,
    pub probability: f64,
    pub impact: f64,
    pub severity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationStrategy {
    pub risk_id: String,
    pub strategy: String,
    pub effectiveness: f64,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedOutcome {
    pub metric: String,
    pub expected_value: f64,
    pub confidence: f64,
    pub timeframe_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessTask {
    pub id: String,
    pub description: String,
    pub task_type: BusinessTaskType,
    pub priority: u8,
    pub dependencies: Vec<String>,
    pub required_skills: Vec<Skill>,
    pub budget: Option<f64>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BusinessTaskType {
    RevenueOptimization,
    CostReduction,
    MarketExpansion,
    ProductDevelopment,
    OperationalEfficiency,
    RiskMitigation,
    StrategicPlanning,
    PerformanceAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessResult {
    pub task_id: String,
    pub status: TaskStatus,
    pub outcomes: Vec<Outcome>,
    pub metrics_impact: HashMap<String, f64>,
    pub lessons_learned: Vec<String>,
    pub next_steps: Vec<BusinessTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub description: String,
    pub value: f64,
    pub metric_affected: String,
    pub confidence: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum BusinessError {
    #[error("Insufficient resources: {0}")]
    InsufficientResources(String),
    #[error("Market conditions unfavorable: {0}")]
    MarketConditions(String),
    #[error("Strategic misalignment: {0}")]
    StrategicMisalignment(String),
    #[error("Operational constraint: {0}")]
    OperationalConstraint(String),
    #[error("Risk threshold exceeded: {0}")]
    RiskThresholdExceeded(String),
    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}

pub fn create_business_task_templates() -> HashMap<BusinessTaskType, TaskAssignment> {
    let mut templates = HashMap::new();

    templates.insert(
        BusinessTaskType::RevenueOptimization,
        TaskAssignment::new(
            "Analyze and optimize revenue streams".to_string(),
            "Revenue optimization plan with pricing recommendations".to_string(),
            9,    // High priority
            8000, // Estimated tokens
            120,  // Time estimate in minutes
            vec![
                "financial_analysis".to_string(),
                "pricing_strategy".to_string(),
                "data_analysis".to_string(),
            ],
        ),
    );

    templates.insert(
        BusinessTaskType::OperationalEfficiency,
        TaskAssignment::new(
            "Improve operational efficiency and reduce costs".to_string(),
            "Operational efficiency improvement plan".to_string(),
            7,    // Medium priority
            6000, // Estimated tokens
            90,   // Time estimate in minutes
            vec![
                "process_optimization".to_string(),
                "cost_analysis".to_string(),
                "automation".to_string(),
            ],
        ),
    );

    templates.insert(
        BusinessTaskType::StrategicPlanning,
        TaskAssignment::new(
            "Develop long-term strategic plans".to_string(),
            "Strategic roadmap and competitive analysis".to_string(),
            9,     // High priority
            10000, // Estimated tokens
            180,   // Time estimate in minutes
            vec![
                "market_analysis".to_string(),
                "strategic_thinking".to_string(),
                "competitive_analysis".to_string(),
            ],
        ),
    );

    templates
}
