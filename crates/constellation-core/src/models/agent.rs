use serde::{Deserialize, Serialize};
use serde_json;

/// A2A Protocol Binding types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ProtocolBinding {
    /// JSON-RPC protocol binding
    #[serde(rename = "JSONRPC")]
    JsonRpc,
    /// gRPC protocol binding
    Grpc,
    /// HTTP+JSON protocol binding
    #[serde(rename = "HTTP+JSON")]
    HttpJson,
}

/// Security scheme types for A2A authentication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SecuritySchemeType {
    /// API Key security scheme
    ApiKey,
    /// HTTP authentication scheme
    Http,
    /// OAuth2 security scheme
    Oauth2,
    /// OpenID Connect security scheme
    OpenIdConnect,
    /// Mutual TLS security scheme
    MutualTls,
}

/// Agent skill representing a specific task the agent can perform
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentSkill {
    /// Unique identifier for the skill
    pub id: String,
    /// Human-readable name for the skill
    pub name: String,
    /// Detailed description of the skill
    pub description: String,
    /// Keywords describing the skill's capabilities
    pub tags: Vec<String>,
    /// Example prompts or scenarios
    pub examples: Option<Vec<String>>,
    /// Supported input media types for this skill
    pub input_modes: Option<Vec<String>>,
    /// Supported output media types for this skill
    pub output_modes: Option<Vec<String>>,
}

/// Supported interface for agent communication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentInterface {
    /// URL where this interface is available
    pub url: String,
    /// Protocol binding supported at this URL
    pub protocol_binding: ProtocolBinding,
    /// Tenant to be set in the request when calling the agent
    pub tenant: Option<String>,
}

/// Agent capabilities declaration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AgentCapabilities {
    /// Whether the agent supports streaming responses
    pub streaming: Option<bool>,
    /// Whether the agent supports push notifications
    pub push_notifications: Option<bool>,
    /// Whether the agent provides state transition history
    pub state_transition_history: Option<bool>,
}

/// Provider information for the agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentProvider {
    /// Name of the organization or developer
    pub name: String,
    /// URL to the provider's website
    pub url: Option<String>,
    /// Contact information
    pub contact: Option<AgentContact>,
}

/// Contact information for the agent provider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentContact {
    /// Contact email address
    pub email: Option<String>,
}

/// Constellation-specific metadata extensions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConstellationMetadata {
    /// The agent's functional role in Constellation
    pub role: String,
    /// Internal execution status
    pub internal_status: String,
    /// Internal capabilities list
    pub capabilities: Vec<String>,
    /// When the agent was last active internally
    pub last_seen: Option<String>,
}

/// A2A AgentCard representing an agent in the Constellation system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Agent {
    /// Unique identifier for the agent
    pub id: String,
    /// Human-readable name of the agent
    pub name: String,
    /// What the agent does
    pub description: String,
    /// Version of the A2A protocol this agent supports
    pub protocol_version: String,
    /// Version of the agent
    pub version: String,
    /// Default input media types supported
    pub default_input_modes: Vec<String>,
    /// Default output media types supported
    pub default_output_modes: Vec<String>,
    /// Information about the organization/developer providing the agent
    pub provider: AgentProvider,
    /// Declared capabilities of the agent
    pub capabilities: AgentCapabilities,
    /// Specific tasks the agent can perform
    pub skills: Vec<AgentSkill>,
    /// Ordered list of supported interfaces
    pub supported_interfaces: Vec<AgentInterface>,
    /// Constellation platform extensions
    pub metadata: Option<serde_json::Value>,
    /// Whether the agent supports providing an extended agent card when authenticated
    pub supports_extended_agent_card: Option<bool>,
    /// URL to additional documentation about the agent
    pub documentation_url: Option<String>,
    /// URL to an icon for the agent
    pub icon_url: Option<String>,
}

impl Agent {
    /// Create a new A2A-compliant agent
    pub fn new(
        id: String,
        name: String,
        description: String,
        provider_name: String,
        skills: Vec<AgentSkill>,
        supported_interfaces: Vec<AgentInterface>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            protocol_version: "1.0".to_string(),
            version: "1.0.0".to_string(),
            default_input_modes: vec!["text/plain".to_string(), "application/json".to_string()],
            default_output_modes: vec!["text/plain".to_string(), "application/json".to_string()],
            provider: AgentProvider {
                name: provider_name,
                url: None,
                contact: None,
            },
            capabilities: AgentCapabilities::default(),
            skills,
            supported_interfaces,
            metadata: None,
            supports_extended_agent_card: Some(false),
            documentation_url: None,
            icon_url: None,
        }
    }

    /// Create a Constellation agent with role-based metadata
    pub fn new_constellation_agent(
        id: String,
        name: String,
        description: String,
        role: String,
        capabilities: Vec<String>,
        skills: Vec<AgentSkill>,
        supported_interfaces: Vec<AgentInterface>,
    ) -> Self {
        let mut agent = Self::new(id, name, description, "Constellation Team".to_string(), skills, supported_interfaces);
        
        let metadata = ConstellationMetadata {
            role,
            internal_status: "idle".to_string(),
            capabilities,
            last_seen: None,
        };
        
        agent.metadata = Some(serde_json::json!({
            "constellation": metadata
        }));
        
        agent
    }

    /// Check if agent supports a specific protocol binding
    pub fn supports_protocol(&self, protocol: ProtocolBinding) -> bool {
        self.supported_interfaces
            .iter()
            .any(|interface| interface.protocol_binding == protocol)
    }

    /// Get the preferred interface URL for a specific protocol
    pub fn get_interface_url(&self, protocol: ProtocolBinding) -> Option<&String> {
        self.supported_interfaces
            .iter()
            .find(|interface| interface.protocol_binding == protocol)
            .map(|interface| &interface.url)
    }

    /// Check if agent has a specific skill
    pub fn has_skill(&self, skill_id: &str) -> bool {
        self.skills.iter().any(|skill| skill.id == skill_id)
    }

    /// Get a specific skill by ID
    pub fn get_skill(&self, skill_id: &str) -> Option<&AgentSkill> {
        self.skills.iter().find(|skill| skill.id == skill_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_agent_new() {
        let skill = AgentSkill {
            id: "calculation".to_string(),
            name: "Calculation".to_string(),
            description: "Performs mathematical calculations".to_string(),
            tags: vec!["math".to_string(), "analysis".to_string()],
            examples: Some(vec!["Calculate 2 + 2".to_string()]),
            input_modes: None,
            output_modes: None,
        };

        let interface = AgentInterface {
            url: "https://agent.example.com/a2a/v1".to_string(),
            protocol_binding: ProtocolBinding::HttpJson,
            tenant: None,
        };

        let agent = Agent::new(
            "agent-alpha".to_string(),
            "Alpha Agent".to_string(),
            "Handles calculation tasks".to_string(),
            "Constellation Team".to_string(),
            vec![skill.clone()],
            vec![interface.clone()],
        );

        assert_eq!(agent.id, "agent-alpha");
        assert_eq!(agent.name, "Alpha Agent");
        assert_eq!(agent.protocol_version, "1.0");
        assert_eq!(agent.version, "1.0.0");
        assert_eq!(agent.skills.len(), 1);
        assert_eq!(agent.skills[0].id, "calculation");
        assert_eq!(agent.supported_interfaces.len(), 1);
        assert_eq!(agent.supported_interfaces[0].protocol_binding, ProtocolBinding::HttpJson);
    }

    #[test]
    fn test_constellation_agent_new() {
        let skill = AgentSkill {
            id: "system-design".to_string(),
            name: "System Design".to_string(),
            description: "Designs system architecture".to_string(),
            tags: vec!["architecture".to_string(), "design".to_string()],
            examples: None,
            input_modes: None,
            output_modes: None,
        };

        let interface = AgentInterface {
            url: "https://architect.constellation.example.com/a2a/v1".to_string(),
            protocol_binding: ProtocolBinding::HttpJson,
            tenant: None,
        };

        let agent = Agent::new_constellation_agent(
            "architect-001".to_string(),
            "System Architect".to_string(),
            "Designs system architecture and data flows".to_string(),
            "architect".to_string(),
            vec!["system-design".to_string(), "data-modeling".to_string()],
            vec![skill],
            vec![interface],
        );

        assert_eq!(agent.id, "architect-001");
        assert_eq!(agent.name, "System Architect");
        assert_eq!(agent.provider.name, "Constellation Team");
        
        // Check metadata
        assert!(agent.metadata.is_some());
        let metadata = agent.metadata.as_ref().unwrap();
        assert!(metadata.get("constellation").is_some());
    }

    #[test]
    fn test_protocol_support() {
        let agent = Agent::new(
            "test-agent".to_string(),
            "Test Agent".to_string(),
            "Test agent".to_string(),
            "Test Provider".to_string(),
            vec![],
            vec![
                AgentInterface {
                    url: "https://test.com/jsonrpc".to_string(),
                    protocol_binding: ProtocolBinding::JsonRpc,
                    tenant: None,
                },
                AgentInterface {
                    url: "https://test.com/http".to_string(),
                    protocol_binding: ProtocolBinding::HttpJson,
                    tenant: None,
                },
            ],
        );

        assert!(agent.supports_protocol(ProtocolBinding::JsonRpc));
        assert!(agent.supports_protocol(ProtocolBinding::HttpJson));
        assert!(!agent.supports_protocol(ProtocolBinding::Grpc));
        
        assert_eq!(agent.get_interface_url(ProtocolBinding::JsonRpc), Some(&"https://test.com/jsonrpc".to_string()));
        assert_eq!(agent.get_interface_url(ProtocolBinding::Grpc), None);
    }

    #[test]
    fn test_skill_operations() {
        let skill1 = AgentSkill {
            id: "skill-1".to_string(),
            name: "Skill One".to_string(),
            description: "First skill".to_string(),
            tags: vec!["tag1".to_string()],
            examples: None,
            input_modes: None,
            output_modes: None,
        };

        let skill2 = AgentSkill {
            id: "skill-2".to_string(),
            name: "Skill Two".to_string(),
            description: "Second skill".to_string(),
            tags: vec!["tag2".to_string()],
            examples: None,
            input_modes: None,
            output_modes: None,
        };

        let agent = Agent::new(
            "test-agent".to_string(),
            "Test Agent".to_string(),
            "Test agent".to_string(),
            "Test Provider".to_string(),
            vec![skill1.clone(), skill2.clone()],
            vec![],
        );

        assert!(agent.has_skill("skill-1"));
        assert!(agent.has_skill("skill-2"));
        assert!(!agent.has_skill("skill-3"));
        
        assert_eq!(agent.get_skill("skill-1").unwrap().name, "Skill One");
        assert_eq!(agent.get_skill("skill-3"), None);
    }

    #[test]
    fn test_agent_serialization() {
        let skill = AgentSkill {
            id: "test-skill".to_string(),
            name: "Test Skill".to_string(),
            description: "A test skill".to_string(),
            tags: vec!["test".to_string()],
            examples: Some(vec!["Test example".to_string()]),
            input_modes: Some(vec!["text/plain".to_string()]),
            output_modes: Some(vec!["application/json".to_string()]),
        };

        let interface = AgentInterface {
            url: "https://test.com/a2a/v1".to_string(),
            protocol_binding: ProtocolBinding::HttpJson,
            tenant: Some("test-tenant".to_string()),
        };

        let agent = Agent {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            protocol_version: "1.0".to_string(),
            version: "1.0.0".to_string(),
            default_input_modes: vec!["text/plain".to_string()],
            default_output_modes: vec!["application/json".to_string()],
            provider: AgentProvider {
                name: "Test Provider".to_string(),
                url: Some("https://test.com".to_string()),
                contact: Some(AgentContact {
                    email: Some("contact@test.com".to_string()),
                }),
            },
            capabilities: AgentCapabilities {
                streaming: Some(true),
                push_notifications: Some(false),
                state_transition_history: Some(true),
            },
            skills: vec![skill],
            supported_interfaces: vec![interface],
            metadata: Some(serde_json::json!({
                "constellation": {
                    "role": "engineer",
                    "internal_status": "active",
                    "capabilities": ["rust", "system-design"],
                    "last_seen": "2025-01-15T10:25:00Z"
                }
            })),
            supports_extended_agent_card: Some(true),
            documentation_url: Some("https://docs.test.com".to_string()),
            icon_url: Some("https://test.com/icon.png".to_string()),
        };

        let json = serde_json::to_string(&agent).unwrap();
        let deserialized: Agent = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, "test-agent");
        assert_eq!(deserialized.name, "Test Agent");
        assert_eq!(deserialized.protocol_version, "1.0");
        assert_eq!(deserialized.skills.len(), 1);
        assert_eq!(deserialized.skills[0].id, "test-skill");
        assert_eq!(deserialized.supported_interfaces.len(), 1);
        assert_eq!(deserialized.supported_interfaces[0].protocol_binding, ProtocolBinding::HttpJson);
        assert!(deserialized.metadata.is_some());
    }

    #[test]
    fn test_protocol_binding_enum_values() {
        let test_cases = vec![
            (ProtocolBinding::JsonRpc, "JSONRPC"),
            (ProtocolBinding::Grpc, "GRPC"),
            (ProtocolBinding::HttpJson, "HTTP+JSON"),
        ];

        for (protocol, expected_str) in test_cases {
            let json = serde_json::to_string(&protocol).unwrap();
            assert_eq!(json, format!("\"{}\"", expected_str));
        }
    }

    #[test]
    fn test_security_scheme_enum_values() {
        let test_cases = vec![
            (SecuritySchemeType::ApiKey, "api_key"),
            (SecuritySchemeType::Http, "http"),
            (SecuritySchemeType::Oauth2, "oauth2"),
            (SecuritySchemeType::OpenIdConnect, "open_id_connect"),
            (SecuritySchemeType::MutualTls, "mutual_tls"),
        ];

        for (scheme, expected_str) in test_cases {
            let json = serde_json::to_string(&scheme).unwrap();
            assert_eq!(json, format!("\"{}\"", expected_str));
        }
    }
}
