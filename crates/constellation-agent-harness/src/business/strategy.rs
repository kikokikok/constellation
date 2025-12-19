use super::*;
use crate::orchestrator::{AgentRole, TaskStatus};
use crate::skill::{DifficultyLevel, Skill};

#[derive(Debug, Clone)]
pub struct StrategyAgent {
    pub id: String,
    pub skills: Vec<Skill>,
    pub token_budget: u32,
    pub cognitive_load: f32,
}

impl StrategyAgent {
    pub fn new(id: String) -> Self {
        Self {
            id,
            skills: vec![
                Skill::new("market_analysis".to_string(), DifficultyLevel::Expert),
                Skill::new("strategic_thinking".to_string(), DifficultyLevel::Expert),
                Skill::new(
                    "competitive_analysis".to_string(),
                    DifficultyLevel::Advanced,
                ),
                Skill::new(
                    "business_model_innovation".to_string(),
                    DifficultyLevel::Advanced,
                ),
                Skill::new("risk_assessment".to_string(), DifficultyLevel::Intermediate),
                Skill::new(
                    "scenario_planning".to_string(),
                    DifficultyLevel::Intermediate,
                ),
            ],
            token_budget: 12000,
            cognitive_load: 0.0,
        }
    }

    pub fn analyze_market_position(
        &self,
        context: &BusinessContext,
    ) -> Result<MarketPositionAnalysis, BusinessError> {
        let market_analysis = &context.market_analysis;
        let current_metrics = &context.current_metrics;

        let market_share = if market_analysis.market_size > 0.0 {
            current_metrics.revenue / market_analysis.market_size
        } else {
            0.0
        };

        let competitive_position = if !market_analysis.competitors.is_empty() {
            let avg_competitor_share: f64 = market_analysis
                .competitors
                .iter()
                .map(|c| c.market_share)
                .sum::<f64>()
                / market_analysis.competitors.len() as f64;

            if market_share > avg_competitor_share * 1.5 {
                CompetitivePosition::Leader
            } else if market_share > avg_competitor_share {
                CompetitivePosition::StrongContender
            } else {
                CompetitivePosition::Challenger
            }
        } else {
            CompetitivePosition::FirstMover
        };

        let mut strategic_options = vec![];

        match competitive_position {
            CompetitivePosition::Leader => {
                strategic_options.push(StrategicOption {
                    name: "Market Defense".to_string(),
                    description: "Protect market share through innovation and customer loyalty"
                        .to_string(),
                    expected_value: current_metrics.revenue * 0.1,
                    risk_level: RiskLevel::Low,
                });
                strategic_options.push(StrategicOption {
                    name: "Adjacent Market Expansion".to_string(),
                    description: "Expand into related markets using existing capabilities"
                        .to_string(),
                    expected_value: current_metrics.revenue * 0.25,
                    risk_level: RiskLevel::Medium,
                });
            }
            CompetitivePosition::Challenger => {
                strategic_options.push(StrategicOption {
                    name: "Differentiated Offering".to_string(),
                    description: "Create unique value proposition to capture market share"
                        .to_string(),
                    expected_value: current_metrics.revenue * 0.3,
                    risk_level: RiskLevel::Medium,
                });
                strategic_options.push(StrategicOption {
                    name: "Niche Focus".to_string(),
                    description: "Focus on underserved market segments".to_string(),
                    expected_value: current_metrics.revenue * 0.2,
                    risk_level: RiskLevel::Low,
                });
            }
            _ => {
                strategic_options.push(StrategicOption {
                    name: "Growth Acceleration".to_string(),
                    description: "Invest in marketing and sales to accelerate growth".to_string(),
                    expected_value: current_metrics.revenue * 0.15,
                    risk_level: RiskLevel::Medium,
                });
            }
        }

        let strategic_options_clone = strategic_options.clone();
        Ok(MarketPositionAnalysis {
            market_share,
            competitive_position,
            growth_potential: market_analysis.growth_rate * (1.0 - market_share),
            strategic_options,
            recommended_focus: strategic_options_clone
                .iter()
                .max_by(|a, b| a.expected_value.partial_cmp(&b.expected_value).unwrap())
                .map(|opt| opt.name.clone())
                .unwrap_or_default(),
        })
    }

    pub fn develop_strategic_roadmap(
        &self,
        context: &BusinessContext,
    ) -> Result<StrategicRoadmap, BusinessError> {
        let _goals = &context.goals;
        let market_analysis = &context.market_analysis;

        let mut initiatives = vec![];
        let timeframe_years = 3;

        for (year, _year_offset) in (1..=timeframe_years).enumerate() {
            let year_num = year + 1;

            let mut year_initiatives = vec![];

            if year_num == 1 {
                year_initiatives.push(StrategicInitiative {
                    name: "Market Validation".to_string(),
                    description: "Validate product-market fit in target segments".to_string(),
                    expected_outcome:
                        "Clear understanding of customer needs and willingness to pay".to_string(),
                    resources_required: vec![
                        "Market Research Budget".to_string(),
                        "Product Team".to_string(),
                    ],
                    success_metrics: vec![
                        "Customer Satisfaction > 4.0".to_string(),
                        "Conversion Rate > 5%".to_string(),
                    ],
                });

                year_initiatives.push(StrategicInitiative {
                    name: "Core Product Enhancement".to_string(),
                    description: "Improve core product based on customer feedback".to_string(),
                    expected_outcome: "Increased user engagement and retention".to_string(),
                    resources_required: vec![
                        "Development Team".to_string(),
                        "UX Design".to_string(),
                    ],
                    success_metrics: vec![
                        "User Retention > 80%".to_string(),
                        "Feature Adoption > 60%".to_string(),
                    ],
                });
            }

            if year_num == 2 {
                year_initiatives.push(StrategicInitiative {
                    name: "Market Expansion".to_string(),
                    description: "Expand to new geographic markets or customer segments"
                        .to_string(),
                    expected_outcome: "20% revenue growth from new markets".to_string(),
                    resources_required: vec!["Sales Team".to_string(), "Localization".to_string()],
                    success_metrics: vec![
                        "New Market Revenue > $100k".to_string(),
                        "Market Share > 5%".to_string(),
                    ],
                });
            }

            if year_num == 3 {
                year_initiatives.push(StrategicInitiative {
                    name: "Product Diversification".to_string(),
                    description: "Launch complementary products or services".to_string(),
                    expected_outcome: "Additional revenue streams reducing dependency on core product".to_string(),
                    resources_required: vec!["R&D Team".to_string(), "Partnership Development".to_string()],
                    success_metrics: vec!["New Product Revenue > 30% of total".to_string(), "Customer LTV Increase > 20%".to_string()],
                });
            }

            initiatives.push(RoadmapYear {
                year: year_num,
                theme: match year_num {
                    1 => "Foundation Building".to_string(),
                    2 => "Growth Acceleration".to_string(),
                    3 => "Scale and Diversify".to_string(),
                    _ => "Strategic Evolution".to_string(),
                },
                initiatives: year_initiatives,
                budget_allocation: match year_num {
                    1 => 0.4,
                    2 => 0.35,
                    3 => 0.25,
                    _ => 0.0,
                },
            });
        }

        let total_expected_growth = market_analysis.growth_rate * timeframe_years as f64;
        let expected_revenue_multiplier = 1.0 + total_expected_growth;

        Ok(StrategicRoadmap {
            timeframe_years,
            initiatives,
            total_expected_growth,
            expected_revenue_multiplier,
            key_assumptions: vec![
                "Market continues growing at current rate".to_string(),
                "Competitive landscape remains stable".to_string(),
                "Sufficient funding available for initiatives".to_string(),
            ],
            risk_factors: vec![
                "Market disruption by new competitors".to_string(),
                "Regulatory changes affecting operations".to_string(),
                "Economic downturn reducing customer spending".to_string(),
            ],
        })
    }

    pub fn assess_competitive_landscape(
        &self,
        context: &BusinessContext,
    ) -> Result<CompetitiveAssessment, BusinessError> {
        let competitors = &context.market_analysis.competitors;
        let current_metrics = &context.current_metrics;

        let mut competitive_advantages = vec![];
        let mut vulnerabilities = vec![];

        for competitor in competitors {
            let relative_strength = competitor.market_share
                / (current_metrics.revenue / context.market_analysis.market_size);

            if relative_strength > 1.5 {
                vulnerabilities.push(CompetitiveVulnerability {
                    competitor: competitor.name.clone(),
                    area: "Market Share".to_string(),
                    threat_level: ThreatLevel::High,
                    mitigation: format!(
                        "Differentiate through {}",
                        competitor.weaknesses.join(", ")
                    ),
                });
            }

            if !competitor.strengths.is_empty() {
                competitive_advantages.push(CompetitiveAdvantage {
                    area: competitor.strengths[0].clone(),
                    relative_position: if relative_strength < 1.0 {
                        "Ahead".to_string()
                    } else {
                        "Behind".to_string()
                    },
                    strategic_implication: if relative_strength < 1.0 {
                        "Leverage as differentiator".to_string()
                    } else {
                        "Develop counter-capability".to_string()
                    },
                });
            }
        }

        let overall_competitive_position =
            if vulnerabilities.is_empty() && !competitive_advantages.is_empty() {
                CompetitivePosition::Leader
            } else if vulnerabilities.len() > competitive_advantages.len() {
                CompetitivePosition::Challenger
            } else {
                CompetitivePosition::StrongContender
            };

        Ok(CompetitiveAssessment {
            competitive_advantages,
            vulnerabilities,
            overall_competitive_position: overall_competitive_position.clone(),
            recommended_actions: if overall_competitive_position == CompetitivePosition::Challenger
            {
                vec![
                    "Focus on underserved customer segments".to_string(),
                    "Develop unique value proposition".to_string(),
                    "Form strategic partnerships".to_string(),
                ]
            } else {
                vec![
                    "Strengthen customer loyalty programs".to_string(),
                    "Invest in continuous innovation".to_string(),
                    "Expand into adjacent markets".to_string(),
                ]
            },
        })
    }
}

impl BusinessAgent for StrategyAgent {
    fn analyze(&self, context: &BusinessContext) -> Result<BusinessAnalysis, BusinessError> {
        let market_position = self.analyze_market_position(context)?;
        let strategic_roadmap = self.develop_strategic_roadmap(context)?;
        let _competitive_assessment = self.assess_competitive_landscape(context)?;

        let recommendations = market_position
            .strategic_options
            .iter()
            .map(|option| Recommendation {
                action: option.name.clone(),
                rationale: option.description.clone(),
                priority: match option.risk_level {
                    RiskLevel::Low => 9,    // High priority
                    RiskLevel::Medium => 7, // Medium priority
                    RiskLevel::High => 5,   // Low priority
                },
                estimated_impact: option.expected_value,
                effort_required: 90,
            })
            .collect();

        let risks = vec![
            Risk {
                description: "Market conditions may change rapidly".to_string(),
                probability: 0.6,
                impact: 0.4,
                severity: 0.24,
            },
            Risk {
                description: "New competitors may enter the market".to_string(),
                probability: 0.5,
                impact: 0.5,
                severity: 0.25,
            },
        ];

        Ok(BusinessAnalysis {
            recommendations,
            risk_assessment: RiskAssessment {
                risks,
                mitigation_strategies: vec![MitigationStrategy {
                    risk_id: "market_changes".to_string(),
                    strategy: "Maintain strategic flexibility and regular market scans".to_string(),
                    effectiveness: 0.7,
                    cost: 0.05,
                }],
                overall_risk_level: match market_position.competitive_position {
                    CompetitivePosition::Leader => RiskLevel::Low,
                    CompetitivePosition::StrongContender => RiskLevel::Medium,
                    CompetitivePosition::Challenger => RiskLevel::High,
                    CompetitivePosition::FirstMover => RiskLevel::Medium,
                },
            },
            expected_outcomes: vec![
                ExpectedOutcome {
                    metric: "Market Share".to_string(),
                    expected_value: market_position.market_share
                        * strategic_roadmap.expected_revenue_multiplier,
                    confidence: 0.7,
                    timeframe_days: 365 * strategic_roadmap.timeframe_years as u32,
                },
                ExpectedOutcome {
                    metric: "Competitive Position".to_string(),
                    expected_value: 1.0,
                    confidence: 0.8,
                    timeframe_days: 180,
                },
            ],
            resource_requirements: {
                let mut map = HashMap::new();
                map.insert("strategy_team".to_string(), 2.0);
                map.insert("market_research_budget".to_string(), 20000.0);
                map.insert("competitive_intelligence".to_string(), 10000.0);
                map
            },
        })
    }

    fn execute(&self, task: &BusinessTask) -> Result<BusinessResult, BusinessError> {
        match task.task_type {
            BusinessTaskType::StrategicPlanning => {
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
                        competitors: vec![Competitor {
                            name: "Competitor A".to_string(),
                            market_share: 0.1,
                            strengths: vec!["Brand recognition".to_string()],
                            weaknesses: vec!["High prices".to_string()],
                            pricing_strategy: "Premium".to_string(),
                        }],
                        trends: vec!["AI integration".to_string(), "Mobile-first".to_string()],
                        opportunities: vec![],
                        threats: vec![],
                    },
                    constraints: vec![],
                    available_resources: HashMap::new(),
                };

                let analysis = self.analyze(&context)?;
                let roadmap = self.develop_strategic_roadmap(&context)?;

                Ok(BusinessResult {
                    task_id: task.id.clone(),
                    status: TaskStatus::Completed,
                    outcomes: analysis
                        .expected_outcomes
                        .iter()
                        .map(|outcome| Outcome {
                            description: format!("Strategic improvement in {}", outcome.metric),
                            value: outcome.expected_value,
                            metric_affected: outcome.metric.clone(),
                            confidence: outcome.confidence,
                        })
                        .collect(),
                    metrics_impact: {
                        let mut map = HashMap::new();
                        map.insert("market_share".to_string(), 0.02);
                        map.insert("competitive_position".to_string(), 0.5);
                        map.insert("strategic_clarity".to_string(), 0.8);
                        map
                    },
                    lessons_learned: vec![
                        "Regular market analysis is crucial".to_string(),
                        "Strategic flexibility beats rigid planning".to_string(),
                        "Competitive intelligence drives better decisions".to_string(),
                    ],
                    next_steps: roadmap
                        .initiatives
                        .iter()
                        .flat_map(|year| &year.initiatives)
                        .map(|initiative| BusinessTask {
                            id: format!(
                                "strategy_{}",
                                initiative.name.to_lowercase().replace(" ", "_")
                            ),
                            description: initiative.description.clone(),
                            task_type: BusinessTaskType::StrategicPlanning,
                            priority: 7, // Medium priority
                            dependencies: vec![],
                            required_skills: vec![
                                Skill::new(
                                    "strategic_thinking".to_string(),
                                    DifficultyLevel::Intermediate,
                                ),
                                Skill::new(
                                    "project_management".to_string(),
                                    DifficultyLevel::Intermediate,
                                ),
                            ],
                            budget: Some(10000.0),
                            deadline: None,
                        })
                        .collect(),
                })
            }
            _ => Err(BusinessError::ExecutionFailed(format!(
                "Task type {:?} not supported by StrategyAgent",
                task.task_type
            ))),
        }
    }

    fn get_skills(&self) -> Vec<Skill> {
        self.skills.clone()
    }

    fn get_role(&self) -> AgentRole {
        AgentRole::StrategyAgent
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPositionAnalysis {
    pub market_share: f64,
    pub competitive_position: CompetitivePosition,
    pub growth_potential: f64,
    pub strategic_options: Vec<StrategicOption>,
    pub recommended_focus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompetitivePosition {
    Leader,
    StrongContender,
    Challenger,
    FirstMover,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicOption {
    pub name: String,
    pub description: String,
    pub expected_value: f64,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicRoadmap {
    pub timeframe_years: usize,
    pub initiatives: Vec<RoadmapYear>,
    pub total_expected_growth: f64,
    pub expected_revenue_multiplier: f64,
    pub key_assumptions: Vec<String>,
    pub risk_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapYear {
    pub year: usize,
    pub theme: String,
    pub initiatives: Vec<StrategicInitiative>,
    pub budget_allocation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicInitiative {
    pub name: String,
    pub description: String,
    pub expected_outcome: String,
    pub resources_required: Vec<String>,
    pub success_metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitiveAssessment {
    pub competitive_advantages: Vec<CompetitiveAdvantage>,
    pub vulnerabilities: Vec<CompetitiveVulnerability>,
    pub overall_competitive_position: CompetitivePosition,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitiveAdvantage {
    pub area: String,
    pub relative_position: String,
    pub strategic_implication: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitiveVulnerability {
    pub competitor: String,
    pub area: String,
    pub threat_level: ThreatLevel,
    pub mitigation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}
