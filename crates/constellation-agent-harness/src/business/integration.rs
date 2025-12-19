use super::*;
use crate::orchestrator::Orchestrator;
use crate::skill::{DifficultyLevel, Skill};

/// Business orchestrator that coordinates all business agents
pub struct BusinessOrchestrator {
    pub orchestrator: Orchestrator,
    pub revenue_agent: revenue::RevenueAgent,
    pub operations_agent: operations::OperationsAgent,
    pub strategy_agent: strategy::StrategyAgent,
    pub intelligence_agent: intelligence::IntelligenceAgent,
}

impl BusinessOrchestrator {
    pub fn new(orchestrator: Orchestrator) -> Self {
        Self {
            orchestrator,
            revenue_agent: revenue::RevenueAgent::new("revenue_agent_1".to_string()),
            operations_agent: operations::OperationsAgent::new("operations_agent_1".to_string()),
            strategy_agent: strategy::StrategyAgent::new("strategy_agent_1".to_string()),
            intelligence_agent: intelligence::IntelligenceAgent::new(
                "intelligence_agent_1".to_string(),
            ),
        }
    }

    pub async fn run_business_cycle(
        &self,
        context: &BusinessContext,
    ) -> Result<BusinessCycleResult, BusinessError> {
        let mut results = vec![];

        let tasks = self.create_business_tasks(context);

        for task in tasks {
            let result = self.execute_business_task(&task, context).await?;
            results.push(result);
        }

        let consolidated_analysis = self.consolidate_analyses(&results, context)?;
        let strategic_plan = self.create_strategic_plan(&consolidated_analysis, context)?;

        let overall_health_score = self.calculate_overall_health_score(&consolidated_analysis);
        let recommendations = self.generate_recommendations(&consolidated_analysis);

        Ok(BusinessCycleResult {
            task_results: results,
            consolidated_analysis,
            strategic_plan,
            overall_health_score,
            recommendations,
        })
    }

    fn create_business_tasks(&self, _context: &BusinessContext) -> Vec<BusinessTask> {
        vec![
            BusinessTask {
                id: "revenue_optimization_1".to_string(),
                description: "Analyze and optimize revenue streams".to_string(),
                task_type: BusinessTaskType::RevenueOptimization,
                priority: 9, // High priority
                dependencies: vec![],
                required_skills: vec![
                    Skill::new("financial_analysis".to_string(), DifficultyLevel::Expert),
                    Skill::new("pricing_strategy".to_string(), DifficultyLevel::Advanced),
                ],
                budget: Some(5000.0),
                deadline: None,
            },
            BusinessTask {
                id: "operational_efficiency_1".to_string(),
                description: "Improve operational efficiency".to_string(),
                task_type: BusinessTaskType::OperationalEfficiency,
                priority: 7, // Medium priority
                dependencies: vec![],
                required_skills: vec![
                    Skill::new(
                        "process_optimization".to_string(),
                        DifficultyLevel::Advanced,
                    ),
                    Skill::new("cost_analysis".to_string(), DifficultyLevel::Intermediate),
                ],
                budget: Some(3000.0),
                deadline: None,
            },
            BusinessTask {
                id: "strategic_planning_1".to_string(),
                description: "Develop strategic roadmap".to_string(),
                task_type: BusinessTaskType::StrategicPlanning,
                priority: 9, // High priority
                dependencies: vec![
                    "revenue_optimization_1".to_string(),
                    "operational_efficiency_1".to_string(),
                ],
                required_skills: vec![
                    Skill::new("strategic_thinking".to_string(), DifficultyLevel::Expert),
                    Skill::new("market_analysis".to_string(), DifficultyLevel::Advanced),
                ],
                budget: Some(10000.0),
                deadline: None,
            },
            BusinessTask {
                id: "performance_analysis_1".to_string(),
                description: "Analyze business performance metrics".to_string(),
                task_type: BusinessTaskType::PerformanceAnalysis,
                priority: 7, // Medium priority
                dependencies: vec![],
                required_skills: vec![
                    Skill::new("data_analysis".to_string(), DifficultyLevel::Advanced),
                    Skill::new(
                        "metrics_tracking".to_string(),
                        DifficultyLevel::Intermediate,
                    ),
                ],
                budget: Some(2000.0),
                deadline: None,
            },
        ]
    }

    async fn execute_business_task(
        &self,
        task: &BusinessTask,
        _context: &BusinessContext,
    ) -> Result<BusinessResult, BusinessError> {
        match task.task_type {
            BusinessTaskType::RevenueOptimization => self.revenue_agent.execute(task),
            BusinessTaskType::OperationalEfficiency => self.operations_agent.execute(task),
            BusinessTaskType::StrategicPlanning => self.strategy_agent.execute(task),
            BusinessTaskType::PerformanceAnalysis => self.intelligence_agent.execute(task),
            _ => Err(BusinessError::ExecutionFailed(format!(
                "Unsupported business task type: {:?}",
                task.task_type
            ))),
        }
    }

    fn consolidate_analyses(
        &self,
        results: &[BusinessResult],
        context: &BusinessContext,
    ) -> Result<ConsolidatedAnalysis, BusinessError> {
        let mut revenue_insights = vec![];
        let mut operations_insights = vec![];
        let mut strategy_insights = vec![];
        let mut intelligence_insights = vec![];

        for result in results {
            match result.task_id.as_str() {
                id if id.contains("revenue") => {
                    revenue_insights.extend(result.lessons_learned.clone());
                }
                id if id.contains("operational") => {
                    operations_insights.extend(result.lessons_learned.clone());
                }
                id if id.contains("strategic") => {
                    strategy_insights.extend(result.lessons_learned.clone());
                }
                id if id.contains("performance") => {
                    intelligence_insights.extend(result.lessons_learned.clone());
                }
                _ => {}
            }
        }

        let mut combined_metrics = HashMap::new();
        for result in results {
            for (metric, impact) in &result.metrics_impact {
                *combined_metrics.entry(metric.clone()).or_insert(0.0) += impact;
            }
        }

        let overall_health = self.calculate_business_health(context, &combined_metrics);

        let revenue_insights_clone = revenue_insights.clone();
        let operations_insights_clone = operations_insights.clone();
        let strategy_insights_clone = strategy_insights.clone();
        let intelligence_insights_clone = intelligence_insights.clone();

        Ok(ConsolidatedAnalysis {
            revenue_insights,
            operations_insights,
            strategy_insights,
            intelligence_insights,
            combined_metrics,
            overall_health,
            cross_cutting_themes: self.identify_cross_cutting_themes(&[
                &revenue_insights_clone,
                &operations_insights_clone,
                &strategy_insights_clone,
                &intelligence_insights_clone,
            ]),
        })
    }

    fn create_strategic_plan(
        &self,
        analysis: &ConsolidatedAnalysis,
        context: &BusinessContext,
    ) -> Result<StrategicPlan, BusinessError> {
        let mut initiatives = vec![];

        if analysis.overall_health.financial_health < 0.7 {
            initiatives.push(StrategicInitiative {
                name: "Financial Health Improvement".to_string(),
                description: "Address revenue and cost structure issues".to_string(),
                priority: InitiativePriority::Critical,
                timeframe_months: 6,
                resources_required: vec![ResourceRequirement {
                    resource_type: "Budget".to_string(),
                    amount: 20000.0,
                    purpose: "Financial analysis and restructuring".to_string(),
                }],
                success_metrics: vec![
                    "Profit margin > 20%".to_string(),
                    "LTV:CAC ratio > 3.0".to_string(),
                ],
            });
        }

        if analysis.overall_health.operational_efficiency < 0.7 {
            initiatives.push(StrategicInitiative {
                name: "Operational Excellence".to_string(),
                description: "Streamline processes and reduce costs".to_string(),
                priority: InitiativePriority::High,
                timeframe_months: 9,
                resources_required: vec![ResourceRequirement {
                    resource_type: "Process Engineers".to_string(),
                    amount: 2.0,
                    purpose: "Process optimization".to_string(),
                }],
                success_metrics: vec![
                    "Operational efficiency > 80%".to_string(),
                    "Cost reduction > 15%".to_string(),
                ],
            });
        }

        if analysis.overall_health.strategic_position < 0.7 {
            initiatives.push(StrategicInitiative {
                name: "Market Positioning".to_string(),
                description: "Strengthen competitive position and market share".to_string(),
                priority: InitiativePriority::Medium,
                timeframe_months: 12,
                resources_required: vec![ResourceRequirement {
                    resource_type: "Market Research".to_string(),
                    amount: 15000.0,
                    purpose: "Competitive analysis and positioning".to_string(),
                }],
                success_metrics: vec![
                    "Market share increase > 5%".to_string(),
                    "Brand recognition improvement".to_string(),
                ],
            });
        }

        let initiatives_clone = initiatives.clone();
        Ok(StrategicPlan {
            initiatives,
            timeframe_years: 3,
            total_budget_required: initiatives_clone
                .iter()
                .flat_map(|i| &i.resources_required)
                .filter(|r| r.resource_type == "Budget")
                .map(|r| r.amount)
                .sum(),
            expected_roi: self.calculate_expected_roi(&initiatives_clone, context),
            risk_assessment: self.assess_strategic_risks(&initiatives_clone),
        })
    }

    fn calculate_business_health(
        &self,
        context: &BusinessContext,
        metrics: &HashMap<String, f64>,
    ) -> BusinessHealth {
        let financial_health = if context.current_metrics.profit_margin > 0.2 {
            0.8
        } else if context.current_metrics.profit_margin > 0.1 {
            0.6
        } else {
            0.4
        };

        let operational_efficiency = context.current_metrics.operational_efficiency;

        let strategic_position = if let Some(market_share) = metrics.get("market_share") {
            if *market_share > 0.1 {
                0.9
            } else if *market_share > 0.05 {
                0.7
            } else {
                0.5
            }
        } else {
            0.5
        };

        BusinessHealth {
            financial_health,
            operational_efficiency,
            strategic_position,
            overall_score: (financial_health + operational_efficiency + strategic_position) / 3.0,
        }
    }

    fn identify_cross_cutting_themes(&self, insights_lists: &[&Vec<String>]) -> Vec<String> {
        let mut theme_counts = HashMap::new();

        for insights in insights_lists {
            for insight in *insights {
                let words: Vec<&str> = insight.split_whitespace().collect();
                for word in words {
                    if word.len() > 4 {
                        *theme_counts.entry(word.to_lowercase()).or_insert(0) += 1;
                    }
                }
            }
        }

        theme_counts
            .into_iter()
            .filter(|(_, count)| *count >= 2)
            .map(|(word, _)| word)
            .collect()
    }

    fn calculate_expected_roi(
        &self,
        initiatives: &[StrategicInitiative],
        context: &BusinessContext,
    ) -> f64 {
        let total_investment: f64 = initiatives
            .iter()
            .flat_map(|i| &i.resources_required)
            .filter(|r| r.resource_type == "Budget")
            .map(|r| r.amount)
            .sum();

        if total_investment > 0.0 {
            let expected_revenue_increase = context.current_metrics.revenue * 0.3;
            expected_revenue_increase / total_investment
        } else {
            0.0
        }
    }

    fn assess_strategic_risks(
        &self,
        initiatives: &[StrategicInitiative],
    ) -> StrategicRiskAssessment {
        let mut risks = vec![];

        for initiative in initiatives {
            if initiative.timeframe_months > 12 {
                risks.push(StrategicRisk {
                    description: format!("Long timeline for {}", initiative.name),
                    probability: 0.6,
                    impact: 0.4,
                    mitigation: "Break into phased delivery".to_string(),
                });
            }

            if initiative.priority == InitiativePriority::Critical {
                risks.push(StrategicRisk {
                    description: format!("Critical initiative {}", initiative.name),
                    probability: 0.3,
                    impact: 0.8,
                    mitigation: "Allocate additional resources and oversight".to_string(),
                });
            }
        }

        let risks_clone = risks.clone();
        StrategicRiskAssessment {
            risks,
            overall_risk_level: if risks_clone.is_empty() {
                RiskLevel::Low
            } else if risks_clone.len() > 3 {
                RiskLevel::High
            } else {
                RiskLevel::Medium
            },
        }
    }

    fn calculate_overall_health_score(&self, analysis: &ConsolidatedAnalysis) -> f64 {
        analysis.overall_health.overall_score
    }

    fn generate_recommendations(
        &self,
        analysis: &ConsolidatedAnalysis,
    ) -> Vec<BusinessRecommendation> {
        let mut recommendations = vec![];

        if analysis.overall_health.financial_health < 0.7 {
            recommendations.push(BusinessRecommendation {
                area: "Financial Health".to_string(),
                action: "Conduct comprehensive financial review".to_string(),
                priority: RecommendationPriority::High,
                expected_impact: 0.3,
            });
        }

        if analysis.overall_health.operational_efficiency < 0.7 {
            recommendations.push(BusinessRecommendation {
                area: "Operations".to_string(),
                action: "Implement process automation initiatives".to_string(),
                priority: RecommendationPriority::Medium,
                expected_impact: 0.25,
            });
        }

        if !analysis.cross_cutting_themes.is_empty() {
            recommendations.push(BusinessRecommendation {
                area: "Strategic Alignment".to_string(),
                action: format!(
                    "Address cross-cutting themes: {}",
                    analysis.cross_cutting_themes.join(", ")
                ),
                priority: RecommendationPriority::Medium,
                expected_impact: 0.2,
            });
        }

        recommendations
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessCycleResult {
    pub task_results: Vec<BusinessResult>,
    pub consolidated_analysis: ConsolidatedAnalysis,
    pub strategic_plan: StrategicPlan,
    pub overall_health_score: f64,
    pub recommendations: Vec<BusinessRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidatedAnalysis {
    pub revenue_insights: Vec<String>,
    pub operations_insights: Vec<String>,
    pub strategy_insights: Vec<String>,
    pub intelligence_insights: Vec<String>,
    pub combined_metrics: HashMap<String, f64>,
    pub overall_health: BusinessHealth,
    pub cross_cutting_themes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessHealth {
    pub financial_health: f64,
    pub operational_efficiency: f64,
    pub strategic_position: f64,
    pub overall_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicPlan {
    pub initiatives: Vec<StrategicInitiative>,
    pub timeframe_years: u32,
    pub total_budget_required: f64,
    pub expected_roi: f64,
    pub risk_assessment: StrategicRiskAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicInitiative {
    pub name: String,
    pub description: String,
    pub priority: InitiativePriority,
    pub timeframe_months: u32,
    pub resources_required: Vec<ResourceRequirement>,
    pub success_metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InitiativePriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirement {
    pub resource_type: String,
    pub amount: f64,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicRiskAssessment {
    pub risks: Vec<StrategicRisk>,
    pub overall_risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicRisk {
    pub description: String,
    pub probability: f64,
    pub impact: f64,
    pub mitigation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRecommendation {
    pub area: String,
    pub action: String,
    pub priority: RecommendationPriority,
    pub expected_impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}
