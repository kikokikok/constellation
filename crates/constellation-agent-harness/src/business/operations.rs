use super::*;
use crate::orchestrator::{AgentRole, TaskStatus};
use crate::skill::{DifficultyLevel, Skill};

#[derive(Debug, Clone)]
pub struct OperationsAgent {
    pub id: String,
    pub skills: Vec<Skill>,
    pub token_budget: u32,
    pub cognitive_load: f32,
}

impl OperationsAgent {
    pub fn new(id: String) -> Self {
        Self {
            id,
            skills: vec![
                Skill::new(
                    "process_optimization".to_string(),
                    DifficultyLevel::Advanced,
                ),
                Skill::new("cost_analysis".to_string(), DifficultyLevel::Intermediate),
                Skill::new("automation".to_string(), DifficultyLevel::Intermediate),
                Skill::new("infrastructure".to_string(), DifficultyLevel::Advanced),
                Skill::new(
                    "resource_management".to_string(),
                    DifficultyLevel::Intermediate,
                ),
                Skill::new(
                    "performance_monitoring".to_string(),
                    DifficultyLevel::Intermediate,
                ),
            ],
            token_budget: 8000,
            cognitive_load: 0.0,
        }
    }

    pub fn analyze_operational_efficiency(
        &self,
        context: &BusinessContext,
    ) -> Result<EfficiencyAnalysis, BusinessError> {
        let current_efficiency = context.current_metrics.operational_efficiency;
        let expenses = context.current_metrics.expenses;
        let revenue = context.current_metrics.revenue;

        let expense_ratio = if revenue > 0.0 {
            expenses / revenue
        } else {
            1.0
        };

        let mut bottlenecks = vec![];
        let mut optimization_opportunities = vec![];

        if current_efficiency < 0.8 {
            bottlenecks.push(OperationalBottleneck {
                area: "Process Automation".to_string(),
                impact: 0.3,
                solution: "Implement workflow automation for repetitive tasks".to_string(),
                estimated_savings: expenses * 0.05,
            });
        }

        if expense_ratio > 0.6 {
            optimization_opportunities.push(CostOptimization {
                category: "Infrastructure Costs".to_string(),
                current_cost: expenses * 0.3,
                potential_savings: expenses * 0.1,
                actions: vec![
                    "Right-size cloud resources".to_string(),
                    "Implement auto-scaling".to_string(),
                    "Use reserved instances".to_string(),
                ],
            });
        }

        let total_potential_savings: f64 = optimization_opportunities
            .iter()
            .map(|opt| opt.potential_savings)
            .sum();

        Ok(EfficiencyAnalysis {
            current_efficiency,
            expense_ratio,
            bottlenecks,
            optimization_opportunities,
            total_potential_savings,
            implementation_priority: if total_potential_savings > expenses * 0.1 {
                vec![
                    "Infrastructure Costs".to_string(),
                    "Process Automation".to_string(),
                ]
            } else {
                vec![
                    "Process Automation".to_string(),
                    "Infrastructure Costs".to_string(),
                ]
            },
        })
    }

    pub fn create_automation_plan(
        &self,
        context: &BusinessContext,
    ) -> Result<AutomationPlan, BusinessError> {
        let mut automatable_processes = vec![];
        let active_users = context.current_metrics.active_users as f64;

        if active_users > 100.0 {
            automatable_processes.push(AutomatableProcess {
                name: "Customer Support Ticket Routing".to_string(),
                manual_effort_hours_per_month: 80.0,
                automation_potential: 0.8,
                implementation_complexity: ImplementationComplexity::Medium,
                estimated_roi: 3.5,
            });
        }

        if context.current_metrics.revenue > 50000.0 {
            automatable_processes.push(AutomatableProcess {
                name: "Billing and Invoicing".to_string(),
                manual_effort_hours_per_month: 40.0,
                automation_potential: 0.9,
                implementation_complexity: ImplementationComplexity::Low,
                estimated_roi: 4.2,
            });
        }

        automatable_processes.push(AutomatableProcess {
            name: "Infrastructure Monitoring and Alerts".to_string(),
            manual_effort_hours_per_month: 60.0,
            automation_potential: 0.95,
            implementation_complexity: ImplementationComplexity::High,
            estimated_roi: 2.8,
        });

        let total_savings: f64 = automatable_processes
            .iter()
            .map(|p| p.manual_effort_hours_per_month * p.automation_potential * 50.0)
            .sum();

        let total_implementation_cost: f64 = automatable_processes
            .iter()
            .map(|p| match p.implementation_complexity {
                ImplementationComplexity::Low => 2000.0,
                ImplementationComplexity::Medium => 5000.0,
                ImplementationComplexity::High => 10000.0,
            })
            .sum();

        let processes_clone = automatable_processes.clone();
        Ok(AutomationPlan {
            processes: automatable_processes,
            total_savings_per_year: total_savings * 12.0,
            total_implementation_cost,
            payback_period_months: if total_savings > 0.0 {
                (total_implementation_cost / total_savings).ceil() as u32
            } else {
                0
            },
            implementation_priority: processes_clone
                .iter()
                .sorted_by(|a, b| b.estimated_roi.partial_cmp(&a.estimated_roi).unwrap())
                .map(|p| p.name.clone())
                .collect(),
        })
    }

    pub fn optimize_resource_allocation(
        &self,
        context: &BusinessContext,
    ) -> Result<ResourceAllocationPlan, BusinessError> {
        let available_resources = &context.available_resources;
        let goals = &context.goals;

        let mut allocations = HashMap::new();
        let mut recommendations = vec![];

        for goal in goals {
            let required_budget = goal.target_metrics.revenue * 0.1;

            if let Some(&available) = available_resources.get("budget") {
                if available >= required_budget {
                    allocations.insert(goal.id.clone(), required_budget);
                    recommendations.push(ResourceRecommendation {
                        resource_type: "Budget".to_string(),
                        allocation: required_budget,
                        purpose: goal.description.clone(),
                        expected_return: goal.target_metrics.revenue * 0.3,
                    });
                }
            }
        }

        let total_allocated: f64 = allocations.values().sum();
        let available_budget = available_resources.get("budget").unwrap_or(&0.0);
        let utilization_rate = if *available_budget > 0.0 {
            total_allocated / available_budget
        } else {
            0.0
        };

        Ok(ResourceAllocationPlan {
            allocations,
            recommendations,
            total_allocated,
            utilization_rate,
            efficiency_score: utilization_rate * 0.8,
        })
    }
}

impl BusinessAgent for OperationsAgent {
    fn analyze(&self, context: &BusinessContext) -> Result<BusinessAnalysis, BusinessError> {
        let efficiency_analysis = self.analyze_operational_efficiency(context)?;
        let automation_plan = self.create_automation_plan(context)?;

        let recommendations = efficiency_analysis
            .optimization_opportunities
            .iter()
            .map(|opt| Recommendation {
                action: format!("Optimize {} costs", opt.category),
                rationale: format!("Potential savings of ${:.2}", opt.potential_savings),
                priority: 7, // Medium priority
                estimated_impact: opt.potential_savings,
                effort_required: 20,
            })
            .chain(
                automation_plan
                    .processes
                    .iter()
                    .map(|process| Recommendation {
                        action: format!("Automate {}", process.name),
                        rationale: format!(
                            "ROI of {:.1}x, saves {:.0} hours/month",
                            process.estimated_roi,
                            process.manual_effort_hours_per_month * process.automation_potential
                        ),
                        priority: 9, // High priority
                        estimated_impact: process.manual_effort_hours_per_month
                            * process.automation_potential
                            * 50.0,
                        effort_required: match process.implementation_complexity {
                            ImplementationComplexity::Low => 10,
                            ImplementationComplexity::Medium => 25,
                            ImplementationComplexity::High => 50,
                        },
                    }),
            )
            .collect();

        let risks = vec![
            Risk {
                description: "Automation may introduce new failure points".to_string(),
                probability: 0.4,
                impact: 0.3,
                severity: 0.12,
            },
            Risk {
                description: "Cost optimization may affect service quality".to_string(),
                probability: 0.3,
                impact: 0.5,
                severity: 0.15,
            },
        ];

        Ok(BusinessAnalysis {
            recommendations,
            risk_assessment: RiskAssessment {
                risks,
                mitigation_strategies: vec![MitigationStrategy {
                    risk_id: "automation_failure".to_string(),
                    strategy: "Implement gradual rollout with monitoring".to_string(),
                    effectiveness: 0.8,
                    cost: 0.05,
                }],
                overall_risk_level: if efficiency_analysis.current_efficiency > 0.7 {
                    RiskLevel::Low
                } else {
                    RiskLevel::Medium
                },
            },
            expected_outcomes: vec![
                ExpectedOutcome {
                    metric: "Operational Efficiency".to_string(),
                    expected_value: efficiency_analysis.current_efficiency + 0.15,
                    confidence: 0.85,
                    timeframe_days: 60,
                },
                ExpectedOutcome {
                    metric: "Monthly Expenses".to_string(),
                    expected_value: context.current_metrics.expenses * 0.9,
                    confidence: 0.75,
                    timeframe_days: 90,
                },
            ],
            resource_requirements: {
                let mut map = HashMap::new();
                map.insert("development_hours".to_string(), 60.0);
                map.insert("infrastructure_budget".to_string(), 10000.0);
                map.insert("monitoring_tools".to_string(), 2000.0);
                map
            },
        })
    }

    fn execute(&self, task: &BusinessTask) -> Result<BusinessResult, BusinessError> {
        match task.task_type {
            BusinessTaskType::OperationalEfficiency => {
                let context = BusinessContext {
                    current_metrics: BusinessMetrics {
                        revenue: 100000.0,
                        expenses: 60000.0,
                        profit_margin: 0.4,
                        customer_acquisition_cost: 500.0,
                        lifetime_value: 1500.0,
                        monthly_recurring_revenue: 50000.0,
                        churn_rate: 0.05,
                        active_users: 1000,
                        conversion_rate: 0.03,
                        operational_efficiency: 0.7,
                    },
                    goals: vec![],
                    market_analysis: MarketAnalysis {
                        market_size: 10000000.0,
                        growth_rate: 0.15,
                        competitors: vec![],
                        trends: vec![],
                        opportunities: vec![],
                        threats: vec![],
                    },
                    constraints: vec![],
                    available_resources: HashMap::new(),
                };

                let analysis = self.analyze(&context)?;

                Ok(BusinessResult {
                    task_id: task.id.clone(),
                    status: TaskStatus::Completed,
                    outcomes: analysis
                        .expected_outcomes
                        .iter()
                        .map(|outcome| Outcome {
                            description: format!("Improved {}", outcome.metric),
                            value: outcome.expected_value,
                            metric_affected: outcome.metric.clone(),
                            confidence: outcome.confidence,
                        })
                        .collect(),
                    metrics_impact: {
                        let mut map = HashMap::new();
                        map.insert("expenses".to_string(), -6000.0);
                        map.insert("operational_efficiency".to_string(), 0.15);
                        map.insert("profit_margin".to_string(), 0.06);
                        map
                    },
                    lessons_learned: vec![
                        "Automation requires careful monitoring".to_string(),
                        "Cost optimization should not compromise quality".to_string(),
                        "Resource allocation needs regular review".to_string(),
                    ],
                    next_steps: vec![],
                })
            }
            _ => Err(BusinessError::ExecutionFailed(format!(
                "Task type {:?} not supported by OperationsAgent",
                task.task_type
            ))),
        }
    }

    fn get_skills(&self) -> Vec<Skill> {
        self.skills.clone()
    }

    fn get_role(&self) -> AgentRole {
        AgentRole::OperationsAgent
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyAnalysis {
    pub current_efficiency: f64,
    pub expense_ratio: f64,
    pub bottlenecks: Vec<OperationalBottleneck>,
    pub optimization_opportunities: Vec<CostOptimization>,
    pub total_potential_savings: f64,
    pub implementation_priority: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalBottleneck {
    pub area: String,
    pub impact: f64,
    pub solution: String,
    pub estimated_savings: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostOptimization {
    pub category: String,
    pub current_cost: f64,
    pub potential_savings: f64,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationPlan {
    pub processes: Vec<AutomatableProcess>,
    pub total_savings_per_year: f64,
    pub total_implementation_cost: f64,
    pub payback_period_months: u32,
    pub implementation_priority: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomatableProcess {
    pub name: String,
    pub manual_effort_hours_per_month: f64,
    pub automation_potential: f64,
    pub implementation_complexity: ImplementationComplexity,
    pub estimated_roi: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationComplexity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocationPlan {
    pub allocations: HashMap<String, f64>,
    pub recommendations: Vec<ResourceRecommendation>,
    pub total_allocated: f64,
    pub utilization_rate: f64,
    pub efficiency_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRecommendation {
    pub resource_type: String,
    pub allocation: f64,
    pub purpose: String,
    pub expected_return: f64,
}

use std::cmp::Ordering;
use std::iter::Iterator;

trait IteratorExt: Iterator {
    fn sorted_by<F>(self, compare: F) -> std::vec::IntoIter<Self::Item>
    where
        Self: Sized,
        F: FnMut(&Self::Item, &Self::Item) -> Ordering,
    {
        let mut vec: Vec<_> = self.collect();
        vec.sort_by(compare);
        vec.into_iter()
    }
}

impl<T: Iterator> IteratorExt for T {}
