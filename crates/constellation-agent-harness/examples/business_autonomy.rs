//! Example of business autonomy system in action
//!
//! This example demonstrates how the business autonomy components work together
//! to analyze and optimize a software business.

use constellation_agent_harness::business::{
    BusinessContext, BusinessGoal, BusinessMetrics, Competitor, MarketAnalysis,
};
use constellation_agent_harness::{BusinessOrchestrator, Orchestrator};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Business Autonomy System Demo");
    println!("=========================================\n");

    // Create a simple orchestrator for the business orchestrator
    let base_orchestrator = Orchestrator::new(
        Arc::new(constellation_core::memory::MemorySystem::new()),
        Arc::new(()),
        Arc::new(constellation_agent_harness::PluginRegistry::new()),
        Arc::new(constellation_agent_harness::SkillRegistry::new()),
        Arc::new(constellation_agent_harness::SessionManager::new()),
    );

    let orchestrator = BusinessOrchestrator::new(base_orchestrator);

    let context = create_sample_business_context();

    println!("📊 Initial Business Context:");
    println!("Revenue: ${:.2}", context.current_metrics.revenue);
    println!(
        "Monthly Recurring Revenue: ${:.2}",
        context.current_metrics.monthly_recurring_revenue
    );
    println!(
        "Profit Margin: {:.1}%",
        context.current_metrics.profit_margin * 100.0
    );
    println!(
        "Customer Acquisition Cost: ${:.2}",
        context.current_metrics.customer_acquisition_cost
    );
    println!(
        "Customer Lifetime Value: ${:.2}",
        context.current_metrics.lifetime_value
    );
    println!(
        "Churn Rate: {:.1}%",
        context.current_metrics.churn_rate * 100.0
    );
    println!("Active Users: {}", context.current_metrics.active_users);
    println!(
        "Conversion Rate: {:.1}%",
        context.current_metrics.conversion_rate * 100.0
    );
    println!(
        "Operational Efficiency: {:.1}%",
        context.current_metrics.operational_efficiency * 100.0
    );
    println!();

    println!("🎯 Business Goals:");
    for goal in &context.goals {
        println!("  • {}: {}", goal.id, goal.description);
        println!("    Target Revenue: ${:.2}", goal.target_metrics.revenue);
        println!(
            "    Target Profit Margin: {:.1}%",
            goal.target_metrics.profit_margin * 100.0
        );
    }
    println!();

    println!("📈 Market Analysis:");
    println!("  Market Size: ${:.2}", context.market_analysis.market_size);
    println!(
        "  Growth Rate: {:.1}%",
        context.market_analysis.growth_rate * 100.0
    );
    println!(
        "  Competitors: {}",
        context.market_analysis.competitors.len()
    );
    for competitor in &context.market_analysis.competitors {
        println!(
            "    • {}: {:.1}% market share",
            competitor.name,
            competitor.market_share * 100.0
        );
    }
    println!();

    println!("🔍 Running Business Cycle Analysis...\n");

    match orchestrator.run_business_cycle(&context).await {
        Ok(result) => {
            println!("✅ Business Cycle Completed Successfully");
            println!("=======================================\n");

            println!(
                "📈 Overall Health Score: {:.1}%",
                result.overall_health_score * 100.0
            );
            println!();

            println!("🎯 Strategic Plan:");
            println!(
                "  Timeframe: {} years",
                result.strategic_plan.timeframe_years
            );
            println!(
                "  Total Budget Required: ${:.2}",
                result.strategic_plan.total_budget_required
            );
            println!("  Expected ROI: {:.1}x", result.strategic_plan.expected_roi);
            println!(
                "  Overall Risk Level: {:?}",
                result.strategic_plan.risk_assessment.overall_risk_level
            );
            println!();

            println!("🚀 Strategic Initiatives:");
            for initiative in &result.strategic_plan.initiatives {
                println!(
                    "  • {} ({:?} priority)",
                    initiative.name, initiative.priority
                );
                println!("    Description: {}", initiative.description);
                println!("    Timeframe: {} months", initiative.timeframe_months);
                println!("    Success Metrics:");
                for metric in &initiative.success_metrics {
                    println!("      - {}", metric);
                }
                println!();
            }

            println!("💡 Key Recommendations:");
            for recommendation in &result.recommendations {
                println!("  • {}: {}", recommendation.area, recommendation.action);
                println!("    Priority: {:?}", recommendation.priority);
                println!(
                    "    Expected Impact: {:.1}%",
                    recommendation.expected_impact * 100.0
                );
                println!();
            }

            println!("📊 Task Results Summary:");
            for task_result in &result.task_results {
                println!("  • {}: {:?}", task_result.task_id, task_result.status);
                println!("    Metrics Impact:");
                for (metric, impact) in &task_result.metrics_impact {
                    println!("      - {}: {:.2}", metric, impact);
                }
                println!("    Lessons Learned:");
                for lesson in &task_result.lessons_learned {
                    println!("      - {}", lesson);
                }
                println!();
            }

            println!("🎯 Consolidated Analysis:");
            println!(
                "  Financial Health: {:.1}%",
                result.consolidated_analysis.overall_health.financial_health * 100.0
            );
            println!(
                "  Operational Efficiency: {:.1}%",
                result
                    .consolidated_analysis
                    .overall_health
                    .operational_efficiency
                    * 100.0
            );
            println!(
                "  Strategic Position: {:.1}%",
                result
                    .consolidated_analysis
                    .overall_health
                    .strategic_position
                    * 100.0
            );
            println!(
                "  Overall Score: {:.1}%",
                result.consolidated_analysis.overall_health.overall_score * 100.0
            );
            println!();

            println!("🔗 Cross-Cutting Themes:");
            for theme in &result.consolidated_analysis.cross_cutting_themes {
                println!("  • {}", theme);
            }
            println!();

            println!("✅ Business Autonomy System Demo Completed Successfully!");
            println!("The system has analyzed the business, identified opportunities,");
            println!("created a strategic plan, and generated actionable recommendations.");
        }
        Err(e) => {
            println!("❌ Business Cycle Failed: {}", e);
        }
    }

    Ok(())
}

fn create_sample_business_context() -> BusinessContext {
    BusinessContext {
        current_metrics: BusinessMetrics {
            revenue: 250000.0,
            expenses: 150000.0,
            profit_margin: 0.4,
            customer_acquisition_cost: 800.0,
            lifetime_value: 2400.0,
            monthly_recurring_revenue: 120000.0,
            churn_rate: 0.08,
            active_users: 2500,
            conversion_rate: 0.035,
            operational_efficiency: 0.65,
        },
        goals: vec![
            BusinessGoal {
                id: "increase_revenue".to_string(),
                description: "Increase annual revenue by 30%".to_string(),
                target_metrics: BusinessMetrics {
                    revenue: 325000.0,
                    expenses: 180000.0,
                    profit_margin: 0.45,
                    customer_acquisition_cost: 700.0,
                    lifetime_value: 3000.0,
                    monthly_recurring_revenue: 156000.0,
                    churn_rate: 0.06,
                    active_users: 3500,
                    conversion_rate: 0.045,
                    operational_efficiency: 0.75,
                },
                deadline: None,
                priority: 9, // High priority
                dependencies: vec![],
            },
            BusinessGoal {
                id: "improve_efficiency".to_string(),
                description: "Improve operational efficiency by 15%".to_string(),
                target_metrics: BusinessMetrics {
                    revenue: 250000.0,
                    expenses: 135000.0,
                    profit_margin: 0.46,
                    customer_acquisition_cost: 800.0,
                    lifetime_value: 2400.0,
                    monthly_recurring_revenue: 120000.0,
                    churn_rate: 0.08,
                    active_users: 2500,
                    conversion_rate: 0.035,
                    operational_efficiency: 0.8,
                },
                deadline: None,
                priority: 7, // Medium priority
                dependencies: vec![],
            },
        ],
        market_analysis: MarketAnalysis {
            market_size: 5000000.0,
            growth_rate: 0.18,
            competitors: vec![
                Competitor {
                    name: "TechCorp Solutions".to_string(),
                    market_share: 0.15,
                    strengths: vec![
                        "Strong brand recognition".to_string(),
                        "Enterprise sales team".to_string(),
                        "Comprehensive feature set".to_string(),
                    ],
                    weaknesses: vec![
                        "High prices".to_string(),
                        "Complex implementation".to_string(),
                        "Slow innovation".to_string(),
                    ],
                    pricing_strategy: "Enterprise tiered".to_string(),
                },
                Competitor {
                    name: "StartupFast".to_string(),
                    market_share: 0.08,
                    strengths: vec![
                        "Modern technology stack".to_string(),
                        "Agile development".to_string(),
                        "Competitive pricing".to_string(),
                    ],
                    weaknesses: vec![
                        "Limited features".to_string(),
                        "Small team".to_string(),
                        "Unproven scalability".to_string(),
                    ],
                    pricing_strategy: "Freemium".to_string(),
                },
            ],
            trends: vec![
                "AI integration becoming standard".to_string(),
                "Mobile-first approach".to_string(),
                "Subscription models preferred".to_string(),
                "Data privacy concerns increasing".to_string(),
            ],
            opportunities: vec![],
            threats: vec![],
        },
        constraints: vec![
            "Limited marketing budget".to_string(),
            "Small development team".to_string(),
            "Need to maintain existing customers".to_string(),
        ],
        available_resources: {
            let mut map = HashMap::new();
            map.insert("budget".to_string(), 50000.0);
            map.insert("development_hours".to_string(), 2000.0);
            map.insert("marketing_budget".to_string(), 20000.0);
            map
        },
    }
}
