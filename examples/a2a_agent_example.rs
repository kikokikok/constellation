//! Example demonstrating A2A-compliant agent creation

use constellation_core::{Agent, AgentInterface, AgentSkill, ProtocolBinding};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a CEO agent with A2A compliance
    let ceo_agent = create_ceo_agent();
    
    println!("CEO Agent created:");
    println!("ID: {}", ceo_agent.id);
    println!("Name: {}", ceo_agent.name);
    println!("Description: {}", ceo_agent.description);
    println!("Protocol Version: {}", ceo_agent.protocol_version);
    println!("Version: {}", ceo_agent.version);
    
    println!("\nSkills:");
    for skill in &ceo_agent.skills {
        println!("  - {}: {}", skill.id, skill.description);
    }
    
    println!("\nSupported Interfaces:");
    for interface in &ceo_agent.supported_interfaces {
        println!("  - {:?}: {}", interface.protocol_binding, interface.url);
    }
    
    println!("\nCapabilities:");
    println!("  Streaming: {:?}", ceo_agent.capabilities.streaming);
    println!("  Push Notifications: {:?}", ceo_agent.capabilities.push_notifications);
    println!("  State Transition History: {:?}", ceo_agent.capabilities.state_transition_history);
    
    // Serialize to JSON (A2A AgentCard format)
    let json = serde_json::to_string_pretty(&ceo_agent)?;
    println!("\nA2A AgentCard JSON:\n{}", json);
    
    Ok(())
}

fn create_ceo_agent() -> Agent {
    // Define CEO skills
    let strategic_decision_skill = AgentSkill {
        id: "strategic-decision".to_string(),
        name: "Strategic Decision Making".to_string(),
        description: "Makes final strategic decisions for the organization".to_string(),
        tags: vec!["strategy".to_string(), "decision".to_string(), "leadership".to_string()],
        examples: Some(vec![
            "Approve new product development".to_string(),
            "Make final budget allocation decisions".to_string(),
        ]),
        input_modes: Some(vec!["application/json".to_string()]),
        output_modes: Some(vec!["application/json".to_string()]),
    };
    
    let budget_approval_skill = AgentSkill {
        id: "budget-approval".to_string(),
        name: "Budget Approval".to_string(),
        description: "Approves budget allocations after CFO review".to_string(),
        tags: vec!["budget".to_string(), "finance".to_string(), "approval".to_string()],
        examples: Some(vec![
            "Approve department budget requests".to_string(),
            "Review and approve quarterly financial plans".to_string(),
        ]),
        input_modes: Some(vec!["application/json".to_string()]),
        output_modes: Some(vec!["application/json".to_string()]),
    };
    
    // Define supported interfaces
    let http_interface = AgentInterface {
        url: "https://ceo.constellation.example.com/a2a/v1".to_string(),
        protocol_binding: ProtocolBinding::HttpJson,
        tenant: Some("constellation".to_string()),
    };
    
    // Create the agent
    Agent::new_constellation_agent(
        "ceo-001".to_string(),
        "CEO Agent".to_string(),
        "Chief Executive Officer - final arbitrator for strategic decisions".to_string(),
        "ceo".to_string(),
        vec![
            "strategic-decision".to_string(),
            "budget-approval".to_string(),
            "leadership".to_string(),
        ],
        vec![strategic_decision_skill, budget_approval_skill],
        vec![http_interface],
    )
}