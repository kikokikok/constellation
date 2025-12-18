# 🛰️ Constellation Platform - Multi-Agent System

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust-based, multi-agent collaborative platform simulating a company with hierarchical decision-making, budget arbitration, and a hybrid memory system (vector DB + SQL). The system is built using an event-driven, microservices architecture.

## 👥 Agent Team Roles & Protocols

**GOVERNING PRINCIPLE:** All agents operate under the A2A (Agent-to-Agent) protocol for structured communication. Decisions must be documented in the shared memory layer.

### **Strategic Layer**
*   **CEO_Agent**: Final arbitrator for strategic decisions. Must consult the CFO on budget and department heads on feasibility.
*   **CFO_Agent**: Manages the monthly budget pool, evaluates ROI on proposals, and runs the budget arbitration engine.

### **R&D Department** (Managed by R&D_Director_Agent)
*   **Researcher_Agent**: Conducts research, formulates hypotheses.
*   **Architect_Agent**: Designs system architecture and data flows.
*   **Engineer_Agent**: Implements performance-critical Rust code.

### **Operational Layer**
*   **DevOps_Agent**: Manages containerization, deployment (Kubernetes/Docker), and cloud infrastructure.
*   **QA_Agent**: Creates comprehensive test suites, and validates system integrity and performance.

## 🔧 Technical Stack

*   **Primary Language:** Rust (Tokio for async, Axum for HTTP, SQLx for PostgreSQL)
*   **Memory:**
    *   **Vector DB:** Qdrant (for agent conversation embeddings and semantic search)
    *   **SQL DB:** PostgreSQL (for structured agent state, decision logs, and budget ledger)
    *   **Cache/Message Bus:** Redis
*   **Communication:** 
    *   **A2A Message Broker:** PostgreSQL-based with priority queuing and delivery guarantees
    *   **Protocol:** A2A (Agent-to-Agent) protocol format
    *   **Transport:** HTTP/REST and WebSocket for real-time communication
*   **Observability:** Structured logs and metrics (using `tracing` and OpenTelemetry)

## 📁 Project Structure

```
constellation/
├── crates/              # Core Rust microservices (orchestrator, memory, agents)
├── deployment/          # Dockerfiles, Kubernetes manifests, Terraform configs
├── docs/architecture/   # Architecture Decision Records (ADRs) and diagrams
└── openspec/           # OpenSpec specifications for spec-driven development
```

## ⚠️ Security & Safety Rules

*   NEVER commit API keys, passwords, or `.env` files
*   All external API calls must have proper error handling and timeouts
*   Validate and sanitize all simulated "budget" and "decision" inputs

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🚀 Getting Started

### Message Broker Quick Start

1. **Setup Database:**
```bash
createdb constellation
psql constellation < migrations/001_initial_message_broker_schema.sql
```

2. **Run Examples:**
```bash
# Basic message broker example
cargo run --example message_broker_example

# Full HTTP/WebSocket server
DATABASE_URL=postgresql://localhost/constellation cargo run --example message_broker_server
```

3. **Explore Documentation:**
- [Message Broker Guide](docs/MESSAGE_BROKER_GUIDE.md)
- [A2A Protocol Spec](openspec/changes/add-a2a-message-broker/specs/agent-a2a-protocol/spec.md)
- [Database Schema](migrations/001_initial_message_broker_schema.sql)

## 📚 Documentation

- [Message Broker Guide](docs/MESSAGE_BROKER_GUIDE.md) - Complete guide to A2A message broker
- [API Documentation](docs/API.md) - REST and WebSocket API reference
- [Deployment Guide](docs/DEPLOYMENT_GUIDE.md) - Production deployment instructions
- [Research Paper](docs/RESEARCH_PAPER.md) - Academic foundations and design principles