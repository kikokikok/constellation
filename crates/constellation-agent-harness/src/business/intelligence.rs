use super::*;
use crate::orchestrator::{AgentRole, TaskStatus};
use crate::skill::{DifficultyLevel, Skill};

#[derive(Debug, Clone)]
pub struct IntelligenceAgent {
    pub id: String,
    pub skills: Vec<Skill>,
    pub token_budget: u32,
    pub cognitive_load: f32,
}

impl IntelligenceAgent {
    pub fn new(id: String) -> Self {
        Self {
            id,
            skills: vec![
                Skill::new("data_analysis".to_string(), DifficultyLevel::Expert),
                Skill::new("metrics_tracking".to_string(), DifficultyLevel::Advanced),
                Skill::new(
                    "predictive_modeling".to_string(),
                    DifficultyLevel::Intermediate,
                ),
                Skill::new(
                    "dashboard_creation".to_string(),
                    DifficultyLevel::Intermediate,
                ),
                Skill::new("kpi_definition".to_string(), DifficultyLevel::Advanced),
                Skill::new(
                    "performance_optimization".to_string(),
                    DifficultyLevel::Intermediate,
                ),
            ],
            token_budget: 9000,
            cognitive_load: 0.0,
        }
    }

    pub fn define_key_performance_indicators(
        &self,
        context: &BusinessContext,
    ) -> Result<KpiFramework, BusinessError> {
        let _goals = &context.goals;
        let current_metrics = &context.current_metrics;

        let kpis = vec![
            KeyPerformanceIndicator {
                name: "Monthly Recurring Revenue (MRR)".to_string(),
                description: "Predictable recurring revenue from subscriptions".to_string(),
                current_value: current_metrics.monthly_recurring_revenue,
                target_value: current_metrics.monthly_recurring_revenue * 1.2,
                unit: "USD".to_string(),
                frequency: MeasurementFrequency::Monthly,
                owner: "Revenue Team".to_string(),
            },
            KeyPerformanceIndicator {
                name: "Customer Acquisition Cost (CAC)".to_string(),
                description: "Cost to acquire a new customer".to_string(),
                current_value: current_metrics.customer_acquisition_cost,
                target_value: current_metrics.customer_acquisition_cost * 0.9,
                unit: "USD".to_string(),
                frequency: MeasurementFrequency::Monthly,
                owner: "Marketing Team".to_string(),
            },
            KeyPerformanceIndicator {
                name: "Customer Lifetime Value (LTV)".to_string(),
                description: "Total revenue expected from a customer".to_string(),
                current_value: current_metrics.lifetime_value,
                target_value: current_metrics.lifetime_value * 1.15,
                unit: "USD".to_string(),
                frequency: MeasurementFrequency::Quarterly,
                owner: "Product Team".to_string(),
            },
            KeyPerformanceIndicator {
                name: "Churn Rate".to_string(),
                description: "Percentage of customers lost over period".to_string(),
                current_value: current_metrics.churn_rate,
                target_value: current_metrics.churn_rate * 0.8,
                unit: "Percentage".to_string(),
                frequency: MeasurementFrequency::Monthly,
                owner: "Customer Success".to_string(),
            },
            KeyPerformanceIndicator {
                name: "Conversion Rate".to_string(),
                description: "Percentage of visitors who become customers".to_string(),
                current_value: current_metrics.conversion_rate,
                target_value: current_metrics.conversion_rate * 1.25,
                unit: "Percentage".to_string(),
                frequency: MeasurementFrequency::Weekly,
                owner: "Sales Team".to_string(),
            },
        ];

        let ltv_to_cac_ratio = if current_metrics.customer_acquisition_cost > 0.0 {
            current_metrics.lifetime_value / current_metrics.customer_acquisition_cost
        } else {
            0.0
        };

        Ok(KpiFramework {
            kpis,
            ltv_to_cac_ratio,
            healthy_ratio_threshold: 3.0,
            overall_health_score: self.calculate_health_score(current_metrics),
            improvement_priorities: self.identify_improvement_priorities(current_metrics),
        })
    }

    pub fn create_performance_dashboard(
        &self,
        context: &BusinessContext,
    ) -> Result<DashboardDesign, BusinessError> {
        let kpi_framework = self.define_key_performance_indicators(context)?;

        let widgets = vec![
            DashboardWidget {
                title: "Revenue Overview".to_string(),
                widget_type: WidgetType::MetricCard,
                metrics: vec![
                    "Monthly Recurring Revenue".to_string(),
                    "Total Revenue".to_string(),
                    "Growth Rate".to_string(),
                ],
                refresh_frequency: RefreshFrequency::RealTime,
                visualization: VisualizationType::Sparkline,
            },
            DashboardWidget {
                title: "Customer Health".to_string(),
                widget_type: WidgetType::MultiMetric,
                metrics: vec![
                    "Customer Acquisition Cost".to_string(),
                    "Lifetime Value".to_string(),
                    "Churn Rate".to_string(),
                    "Conversion Rate".to_string(),
                ],
                refresh_frequency: RefreshFrequency::Daily,
                visualization: VisualizationType::Gauge,
            },
            DashboardWidget {
                title: "Operational Efficiency".to_string(),
                widget_type: WidgetType::TimeSeries,
                metrics: vec![
                    "Operational Efficiency".to_string(),
                    "Expense Ratio".to_string(),
                    "Profit Margin".to_string(),
                ],
                refresh_frequency: RefreshFrequency::Weekly,
                visualization: VisualizationType::LineChart,
            },
            DashboardWidget {
                title: "Strategic Goals Progress".to_string(),
                widget_type: WidgetType::ProgressBar,
                metrics: context.goals.iter().map(|g| g.id.clone()).collect(),
                refresh_frequency: RefreshFrequency::Monthly,
                visualization: VisualizationType::ProgressBar,
            },
        ];

        Ok(DashboardDesign {
            widgets,
            layout: DashboardLayout::Grid,
            theme: DashboardTheme::Dark,
            accessibility: true,
            export_capabilities: vec![ExportFormat::Pdf, ExportFormat::Csv, ExportFormat::Image],
            alert_rules: self.create_alert_rules(&kpi_framework),
        })
    }

    pub fn analyze_trends_and_forecasts(
        &self,
        context: &BusinessContext,
    ) -> Result<TrendAnalysis, BusinessError> {
        let current_metrics = &context.current_metrics;
        let market_analysis = &context.market_analysis;

        let _historical_data_points = 12;
        let mut revenue_forecast = vec![current_metrics.revenue];
        let mut growth_rates = vec![];

        for _month in 1..=12 {
            let growth_rate = market_analysis.growth_rate / 12.0;
            let previous_revenue = revenue_forecast.last().unwrap();
            let new_revenue = previous_revenue * (1.0 + growth_rate);
            revenue_forecast.push(new_revenue);
            growth_rates.push(growth_rate);
        }

        let _average_growth_rate: f64 =
            growth_rates.iter().sum::<f64>() / growth_rates.len() as f64;

        let mut trends = vec![];

        if current_metrics.churn_rate > 0.1 {
            trends.push(Trend {
                metric: "Churn Rate".to_string(),
                direction: TrendDirection::Negative,
                strength: 0.8,
                implication: "High customer turnover affecting LTV".to_string(),
                recommendation: "Improve customer onboarding and support".to_string(),
            });
        }

        if current_metrics.conversion_rate < 0.02 {
            trends.push(Trend {
                metric: "Conversion Rate".to_string(),
                direction: TrendDirection::Negative,
                strength: 0.6,
                implication: "Inefficient sales funnel".to_string(),
                recommendation: "Optimize landing pages and CTAs".to_string(),
            });
        }

        if current_metrics.operational_efficiency > 0.8 {
            trends.push(Trend {
                metric: "Operational Efficiency".to_string(),
                direction: TrendDirection::Positive,
                strength: 0.7,
                implication: "Good resource utilization".to_string(),
                recommendation: "Maintain current processes".to_string(),
            });
        }

        let revenue_forecast_clone = revenue_forecast.clone();
        let trends_clone = trends.clone();
        Ok(TrendAnalysis {
            revenue_forecast,
            average_growth_rate: 0.15,
            forecast_horizon_months: 12,
            trends,
            confidence_interval: 0.85,
            key_insights: vec![
                format!(
                    "Revenue expected to grow to ${:.2} in 12 months",
                    revenue_forecast_clone.last().unwrap()
                ),
                if trends_clone
                    .iter()
                    .any(|t| t.direction == TrendDirection::Negative)
                {
                    "Warning: Negative trends detected in key metrics".to_string()
                } else {
                    "All trends are positive or neutral".to_string()
                },
            ],
        })
    }

    fn calculate_health_score(&self, metrics: &BusinessMetrics) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        if metrics.profit_margin > 0.2 {
            score += 0.2;
            factors += 1;
        }

        let ltv_to_cac = if metrics.customer_acquisition_cost > 0.0 {
            metrics.lifetime_value / metrics.customer_acquisition_cost
        } else {
            0.0
        };

        if ltv_to_cac > 3.0 {
            score += 0.3;
            factors += 1;
        }

        if metrics.churn_rate < 0.1 {
            score += 0.2;
            factors += 1;
        }

        if metrics.conversion_rate > 0.02 {
            score += 0.2;
            factors += 1;
        }

        if metrics.operational_efficiency > 0.7 {
            score += 0.1;
            factors += 1;
        }

        if factors > 0 {
            score / factors as f64
        } else {
            0.0
        }
    }

    fn identify_improvement_priorities(
        &self,
        metrics: &BusinessMetrics,
    ) -> Vec<ImprovementPriority> {
        let mut priorities = vec![];

        if metrics.churn_rate > 0.1 {
            priorities.push(ImprovementPriority {
                metric: "Churn Rate".to_string(),
                current_value: metrics.churn_rate,
                target_value: metrics.churn_rate * 0.7,
                impact_score: 0.9,
                effort_required: 0.7,
            });
        }

        if metrics.conversion_rate < 0.02 {
            priorities.push(ImprovementPriority {
                metric: "Conversion Rate".to_string(),
                current_value: metrics.conversion_rate,
                target_value: metrics.conversion_rate * 1.5,
                impact_score: 0.8,
                effort_required: 0.6,
            });
        }

        let ltv_to_cac = if metrics.customer_acquisition_cost > 0.0 {
            metrics.lifetime_value / metrics.customer_acquisition_cost
        } else {
            0.0
        };

        if ltv_to_cac < 3.0 {
            priorities.push(ImprovementPriority {
                metric: "LTV:CAC Ratio".to_string(),
                current_value: ltv_to_cac,
                target_value: 3.0,
                impact_score: 0.95,
                effort_required: 0.8,
            });
        }

        priorities.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap());
        priorities
    }

    fn create_alert_rules(&self, _kpi_framework: &KpiFramework) -> Vec<AlertRule> {
        vec![
            AlertRule {
                metric: "Churn Rate".to_string(),
                condition: AlertCondition::GreaterThan(0.15),
                severity: AlertSeverity::Critical,
                notification_channels: vec![NotificationChannel::Email, NotificationChannel::Slack],
                auto_remediation: Some("Trigger customer retention campaign".to_string()),
            },
            AlertRule {
                metric: "LTV:CAC Ratio".to_string(),
                condition: AlertCondition::LessThan(2.0),
                severity: AlertSeverity::High,
                notification_channels: vec![NotificationChannel::Email],
                auto_remediation: Some("Review acquisition channels and pricing".to_string()),
            },
            AlertRule {
                metric: "Monthly Recurring Revenue".to_string(),
                condition: AlertCondition::DecreaseByPercent(10.0),
                severity: AlertSeverity::Medium,
                notification_channels: vec![NotificationChannel::Slack],
                auto_remediation: None,
            },
        ]
    }
}

impl BusinessAgent for IntelligenceAgent {
    fn analyze(&self, context: &BusinessContext) -> Result<BusinessAnalysis, BusinessError> {
        let kpi_framework = self.define_key_performance_indicators(context)?;
        let _trend_analysis = self.analyze_trends_and_forecasts(context)?;

        let recommendations = kpi_framework
            .improvement_priorities
            .iter()
            .map(|priority| Recommendation {
                action: format!("Improve {}", priority.metric),
                rationale: format!(
                    "Current: {:.2}, Target: {:.2}, Impact: {:.0}%",
                    priority.current_value,
                    priority.target_value,
                    priority.impact_score * 100.0
                ),
                priority: if priority.impact_score > 0.8 { 9 } else { 7 }, // High or Medium priority
                estimated_impact: (priority.target_value - priority.current_value).abs() * 1000.0,
                effort_required: (priority.effort_required * 100.0) as u32,
            })
            .collect();

        let risks = vec![
            Risk {
                description: "KPIs may not reflect true business health".to_string(),
                probability: 0.3,
                impact: 0.4,
                severity: 0.12,
            },
            Risk {
                description: "Data quality issues affecting analysis".to_string(),
                probability: 0.4,
                impact: 0.6,
                severity: 0.24,
            },
        ];

        Ok(BusinessAnalysis {
            recommendations,
            risk_assessment: RiskAssessment {
                risks,
                mitigation_strategies: vec![MitigationStrategy {
                    risk_id: "data_quality".to_string(),
                    strategy: "Implement data validation and monitoring".to_string(),
                    effectiveness: 0.9,
                    cost: 0.1,
                }],
                overall_risk_level: if kpi_framework.overall_health_score > 0.7 {
                    RiskLevel::Low
                } else if kpi_framework.overall_health_score > 0.5 {
                    RiskLevel::Medium
                } else {
                    RiskLevel::High
                },
            },
            expected_outcomes: vec![
                ExpectedOutcome {
                    metric: "Business Health Score".to_string(),
                    expected_value: kpi_framework.overall_health_score + 0.15,
                    confidence: 0.8,
                    timeframe_days: 90,
                },
                ExpectedOutcome {
                    metric: "Revenue Forecast Accuracy".to_string(),
                    expected_value: 0.85,
                    confidence: 0.75,
                    timeframe_days: 180,
                },
            ],
            resource_requirements: {
                let mut map = HashMap::new();
                map.insert("analytics_tools".to_string(), 5000.0);
                map.insert("data_engineer_hours".to_string(), 80.0);
                map.insert("dashboard_development".to_string(), 40.0);
                map
            },
        })
    }

    fn execute(&self, task: &BusinessTask) -> Result<BusinessResult, BusinessError> {
        match task.task_type {
            BusinessTaskType::PerformanceAnalysis => {
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
                let _kpi_framework = self.define_key_performance_indicators(&context)?;
                let _dashboard = self.create_performance_dashboard(&context)?;

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
                        map.insert("health_score".to_string(), 0.15);
                        map.insert("forecast_accuracy".to_string(), 0.1);
                        map.insert("kpi_coverage".to_string(), 0.9);
                        map
                    },
                    lessons_learned: vec![
                        "Regular KPI review drives better decisions".to_string(),
                        "Dashboard visualization improves understanding".to_string(),
                        "Alert rules prevent small issues from becoming big problems".to_string(),
                    ],
                    next_steps: vec![],
                })
            }
            _ => Err(BusinessError::ExecutionFailed(format!(
                "Task type {:?} not supported by IntelligenceAgent",
                task.task_type
            ))),
        }
    }

    fn get_skills(&self) -> Vec<Skill> {
        self.skills.clone()
    }

    fn get_role(&self) -> AgentRole {
        AgentRole::IntelligenceAgent
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiFramework {
    pub kpis: Vec<KeyPerformanceIndicator>,
    pub ltv_to_cac_ratio: f64,
    pub healthy_ratio_threshold: f64,
    pub overall_health_score: f64,
    pub improvement_priorities: Vec<ImprovementPriority>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPerformanceIndicator {
    pub name: String,
    pub description: String,
    pub current_value: f64,
    pub target_value: f64,
    pub unit: String,
    pub frequency: MeasurementFrequency,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeasurementFrequency {
    RealTime,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementPriority {
    pub metric: String,
    pub current_value: f64,
    pub target_value: f64,
    pub impact_score: f64,
    pub effort_required: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardDesign {
    pub widgets: Vec<DashboardWidget>,
    pub layout: DashboardLayout,
    pub theme: DashboardTheme,
    pub accessibility: bool,
    pub export_capabilities: Vec<ExportFormat>,
    pub alert_rules: Vec<AlertRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub title: String,
    pub widget_type: WidgetType,
    pub metrics: Vec<String>,
    pub refresh_frequency: RefreshFrequency,
    pub visualization: VisualizationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    MetricCard,
    MultiMetric,
    TimeSeries,
    ProgressBar,
    HeatMap,
    Distribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefreshFrequency {
    RealTime,
    Minute,
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationType {
    Sparkline,
    Gauge,
    LineChart,
    BarChart,
    PieChart,
    ProgressBar,
    HeatMap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardLayout {
    Grid,
    SingleColumn,
    MultiColumn,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardTheme {
    Light,
    Dark,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Pdf,
    Csv,
    Excel,
    Image,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub metric: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub notification_channels: Vec<NotificationChannel>,
    pub auto_remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    GreaterThan(f64),
    LessThan(f64),
    EqualTo(f64),
    NotEqualTo(f64),
    IncreaseByPercent(f64),
    DecreaseByPercent(f64),
    OutsideRange(f64, f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email,
    Slack,
    Webhook,
    Sms,
    Push,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub revenue_forecast: Vec<f64>,
    pub average_growth_rate: f64,
    pub forecast_horizon_months: u32,
    pub confidence_interval: f64,
    pub trends: Vec<Trend>,
    pub key_insights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trend {
    pub metric: String,
    pub direction: TrendDirection,
    pub strength: f64,
    pub implication: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    Positive,
    Negative,
    Neutral,
}
