# Project Context

## 🛰️ System Purpose & Architecture
Constellation is a Rust-based, multi-agent collaborative platform simulating a company with hierarchical decision-making, budget arbitration, and a hybrid memory system (vector DB + SQL). The system is built using an event-driven, microservices architecture.

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

## 🔧 Technical Stack & Conventions
*   **Primary Language:** Rust. Use Tokio for async, Axum for HTTP, SQLx for PostgreSQL.
*   **Memory:**
    *   **Vector DB:** Qdrant (for agent conversation embeddings and semantic search).
    *   **SQL DB:** PostgreSQL (for structured agent state, decision logs, and budget ledger).
    *   **Cache/Message Bus:** Redis.
*   **Communication:** Inter-agent messages use the A2A protocol format.
*   **Observability:** All services must emit structured logs and metrics (using `tracing` and OpenTelemetry).

## Project Conventions

### Code Style
* Rust code follows the official Rust style guide with `cargo fmt`
* Use `rust-analyzer` for development
* Document public APIs with Rustdoc comments

### Architecture Patterns
* Event-driven microservices architecture
* Clean separation between agent logic and communication layer
* Repository pattern for data access
* CQRS for decision logging and audit trails

### Testing Strategy
* Unit tests for all business logic
* Integration tests for agent communication
* E2E tests for multi-agent workflows
* Performance benchmarks for critical paths

### Git Workflow
* Feature branches from `main`
* PR reviews required before merge
* Conventional commits format
* Automated CI/CD pipeline

## Domain Context
* Multi-agent system simulating company decision-making
* Budget arbitration with ROI calculations
* Hybrid memory system combining vector embeddings and relational data
* Hierarchical agent structure with escalation paths

## Important Constraints
* NEVER commit API keys, passwords, or `.env` files
* All external API calls must have proper error handling and timeouts
* Validate and sanitize all simulated "budget" and "decision" inputs
* Maintain audit trail for all decisions and budget transactions

## External Dependencies
* PostgreSQL for structured data
* Qdrant for vector embeddings
* Redis for caching and message bus
* (Optional) Cloud providers (AWS/Azure/GCP) for deployment
