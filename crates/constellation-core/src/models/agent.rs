use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Agent role in the Constellation system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    /// Chief Executive Officer - final arbitrator for strategic decisions.
    Ceo,
    /// Chief Financial Officer - manages budget pool and evaluates ROI.
    Cfo,
    /// System Architect - designs architecture and data flows.
    Architect,
    /// Engineer - implements performance-critical Rust code.
    Engineer,
    /// DevOps Engineer - manages containerization and deployment.
    Devops,
    /// Quality Assurance - creates test suites and validates system integrity.
    Qa,
    /// Specification Manager - maintains OpenSpec as source of truth.
    SpecManager,
    /// GitHub Automation - automates GitHub workflows and project management.
    GithubAutomation,
}

/// Current runtime status of an agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    /// Agent is available but not currently executing tasks.
    Idle,
    /// Agent is currently executing tasks.
    Active,
    /// Agent has been terminated and cannot be reactivated.
    Terminated,
}

/// Core agent entity representing a participant in the Constellation system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Agent {
    /// Unique agent identifier.
    pub id: Uuid,
    /// The agent's functional role.
    pub role: AgentRole,
    /// Current runtime status.
    pub status: AgentStatus,
    /// List of actions this agent can perform.
    pub capabilities: Vec<String>,
}

impl Agent {
    /// Create a new agent with the given role and capabilities.
    pub fn new(role: AgentRole, capabilities: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            role,
            status: AgentStatus::Idle,
            capabilities,
        }
    }

    /// Activate the agent.
    /// Returns `Ok(())` if successful, or `Err` if the agent is terminated.
    pub fn activate(&mut self) -> Result<(), &'static str> {
        if self.status == AgentStatus::Terminated {
            return Err("Cannot activate a terminated agent");
        }
        self.status = AgentStatus::Active;
        Ok(())
    }

    /// Deactivate the agent (set to idle).
    /// Returns `Ok(())` if successful, or `Err` if the agent is terminated.
    pub fn deactivate(&mut self) -> Result<(), &'static str> {
        if self.status == AgentStatus::Terminated {
            return Err("Cannot deactivate a terminated agent");
        }
        self.status = AgentStatus::Idle;
        Ok(())
    }

    /// Terminate the agent.
    /// Returns `Ok(())` if successful, or `Err` if the agent is already terminated.
    pub fn terminate(&mut self) -> Result<(), &'static str> {
        if self.status == AgentStatus::Terminated {
            return Err("Agent is already terminated");
        }
        self.status = AgentStatus::Terminated;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_agent_new() {
        let capabilities = vec!["calculate".to_string(), "analyze".to_string()];
        let agent = Agent::new(AgentRole::Engineer, capabilities.clone());

        assert_eq!(agent.role, AgentRole::Engineer);
        assert_eq!(agent.status, AgentStatus::Idle);
        assert_eq!(agent.capabilities, capabilities);
    }

    #[test]
    fn test_agent_status_transitions() {
        let mut agent = Agent::new(AgentRole::Architect, vec![]);

        assert!(agent.activate().is_ok());
        assert_eq!(agent.status, AgentStatus::Active);

        assert!(agent.deactivate().is_ok());
        assert_eq!(agent.status, AgentStatus::Idle);

        assert!(agent.terminate().is_ok());
        assert_eq!(agent.status, AgentStatus::Terminated);
    }

    #[test]
    fn test_terminated_agent_cannot_be_activated() {
        let mut agent = Agent::new(AgentRole::Engineer, vec![]);

        // Terminate the agent
        assert!(agent.terminate().is_ok());
        assert_eq!(agent.status, AgentStatus::Terminated);

        // Attempt to activate should fail
        let result = agent.activate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot activate a terminated agent");
        assert_eq!(agent.status, AgentStatus::Terminated);
    }

    #[test]
    fn test_terminated_agent_cannot_be_deactivated() {
        let mut agent = Agent::new(AgentRole::Engineer, vec![]);

        // Terminate the agent
        assert!(agent.terminate().is_ok());
        assert_eq!(agent.status, AgentStatus::Terminated);

        // Attempt to deactivate should fail
        let result = agent.deactivate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot deactivate a terminated agent");
        assert_eq!(agent.status, AgentStatus::Terminated);
    }

    #[test]
    fn test_terminated_agent_cannot_be_reterminated() {
        let mut agent = Agent::new(AgentRole::Engineer, vec![]);

        // Terminate the agent
        assert!(agent.terminate().is_ok());
        assert_eq!(agent.status, AgentStatus::Terminated);

        // Attempt to terminate again should fail
        let result = agent.terminate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Agent is already terminated");
        assert_eq!(agent.status, AgentStatus::Terminated);
    }

    #[test]
    fn test_agent_serialization() {
        let agent = Agent {
            id: uuid::Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap(),
            role: AgentRole::Ceo,
            status: AgentStatus::Active,
            capabilities: vec!["decide".to_string(), "approve".to_string()],
        };

        let json = serde_json::to_string(&agent).unwrap();
        let expected_json = r#"{"id":"123e4567-e89b-12d3-a456-426614174000","role":"ceo","status":"active","capabilities":["decide","approve"]}"#;

        assert_eq!(json, expected_json);
    }

    #[test]
    fn test_agent_deserialization() {
        let json = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "role": "cfo",
            "status": "idle",
            "capabilities": ["budget", "allocate"]
        }"#;

        let agent: Agent = serde_json::from_str(json).unwrap();

        assert_eq!(
            agent.id,
            uuid::Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap()
        );
        assert_eq!(agent.role, AgentRole::Cfo);
        assert_eq!(agent.status, AgentStatus::Idle);
        assert_eq!(
            agent.capabilities,
            vec!["budget".to_string(), "allocate".to_string()]
        );
    }

    #[test]
    fn test_agent_role_enum_values() {
        // Test all enum values match OpenSpec schema
        let test_cases = vec![
            (AgentRole::Ceo, "ceo"),
            (AgentRole::Cfo, "cfo"),
            (AgentRole::Architect, "architect"),
            (AgentRole::Engineer, "engineer"),
            (AgentRole::Devops, "devops"),
            (AgentRole::Qa, "qa"),
            (AgentRole::SpecManager, "spec_manager"),
            (AgentRole::GithubAutomation, "github_automation"),
        ];

        for (role, expected_str) in test_cases {
            let json = serde_json::to_string(&role).unwrap();
            assert_eq!(json, format!("\"{}\"", expected_str));
        }
    }

    #[test]
    fn test_agent_status_enum_values() {
        // Test all enum values match OpenSpec schema
        let test_cases = vec![
            (AgentStatus::Idle, "idle"),
            (AgentStatus::Active, "active"),
            (AgentStatus::Terminated, "terminated"),
        ];

        for (status, expected_str) in test_cases {
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, format!("\"{}\"", expected_str));
        }
    }
}
