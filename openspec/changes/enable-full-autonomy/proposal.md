# Change: Enable Full Autonomous AI-Driven Software Business

## Why
Current Constellation implementation provides basic A2A communication but lacks critical components for full business autonomy based on state-of-the-art research from Anthropic. To achieve a fully autonomous software business, we need long-running agent harnesses, dynamic skill frameworks, and multi-agent research systems.

## What Changes
- **ADD** Long-running agent harness with initializer/coding agent pattern
- **ADD** Agent Skills framework with progressive disclosure (SKILL.md)
- **ADD** Multi-agent research system with orchestrator-worker pattern
- **ADD** Business autonomy components: revenue, operations, strategy agents
- **ADD** Memory compression and context management across sessions
- **MODIFY** Existing A2A protocol to support skill discovery and execution
- **BREAKING** Agents must implement clean state management and progress tracking

## Impact
- Affected specs: agent-a2a-protocol, agent-harness (new), agent-skills (new), multi-agent-research (new)
- Affected code: Complete overhaul of agent execution model
- Infrastructure: Git integration, browser automation, external tool integration
- Performance: 15x token usage increase for multi-agent systems
- Business: Enables fully autonomous software business operations