//! Integration layer connecting DTG, agents, MCP security, and autonomy measurement.
//!
//! This module provides the glue between different Constellation components:
//! - DTG execution with agent task execution
//! - MCP security for agent communications
//! - Hybrid agents with A2A protocol
//! - Autonomy measurement across all operations

pub mod autonomy_integration;
pub mod dtg_agent_integration;
pub mod hybrid_a2a_integration;
pub mod mcp_security_integration;

pub use autonomy_integration::AutonomyIntegrationEngine;
pub use dtg_agent_integration::DtgAgentIntegrationEngine;
pub use hybrid_a2a_integration::HybridA2AIntegration;
pub use mcp_security_integration::McpSecurityIntegration;
