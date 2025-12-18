# Constellation Deployment Guide

**Version:** 1.0.0  
**Last Updated:** December 14, 2025

## Overview

This guide provides comprehensive instructions for deploying the Constellation platform in various environments, from local development to production clusters.

## Table of Contents

1. [Quick Start](#quick-start)
2. [System Requirements](#system-requirements)
3. [Local Development Deployment](#local-development-deployment)
4. [Docker Deployment](#docker-deployment)
5. [Kubernetes Deployment](#kubernetes-deployment)
6. [Cloud Deployment](#cloud-deployment)
7. [Configuration Reference](#configuration-reference)
8. [Monitoring and Observability](#monitoring-and-observability)
9. [Security Configuration](#security-configuration)
10. [Troubleshooting](#troubleshooting)

## Quick Start

### 1. Clone and Build

```bash
# Clone the repository
git clone https://github.com/your-org/constellation.git
cd constellation

# Build the project
cargo build --release

# Run tests
cargo test

# Run integration example
cargo run --example integration_example
```

### 2. Basic Configuration

Create `.env` file:

```env
# Database Configuration
DATABASE_URL=postgresql://localhost:5432/constellation
REDIS_URL=redis://localhost:6379

# Security Configuration
MCP_KEY_ROTATION_DAYS=90
ENCRYPTION_ALGORITHM=AES-256-GCM
SIGNATURE_ALGORITHM=Ed25519

# Agent Configuration
MAX_CONCURRENT_AGENTS=100
AGENT_HEARTBEAT_INTERVAL=30
```

### 3. Start Services

```bash
# Start PostgreSQL
docker run -d --name constellation-db \
  -e POSTGRES_PASSWORD=password \
  -p 5432:5432 \
  postgres:15

# Start Redis
docker run -d --name constellation-redis \
  -p 6379:6379 \
  redis:7

# Run Constellation
cargo run --release
```

## System Requirements

### Minimum Requirements

- **CPU**: 4 cores (8+ recommended)
- **RAM**: 8GB (16GB+ recommended)
- **Storage**: 10GB SSD
- **OS**: Linux, macOS, or Windows with WSL2
- **Rust**: 1.75+ (2024 edition)

### Recommended for Production

- **CPU**: 8+ cores
- **RAM**: 32GB+
- **Storage**: 100GB+ NVMe SSD
- **Network**: 1Gbps+
- **Database**: PostgreSQL 15+ with connection pooling
- **Cache**: Redis 7+ with persistence

## Local Development Deployment

### 1. Development Environment Setup

```bash
# Install dependencies
brew install postgresql redis  # macOS
# or
apt-get install postgresql redis  # Ubuntu

# Create development database
createdb constellation_dev
createdb constellation_test

# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install stable
rustup default stable

# Install development tools
cargo install cargo-watch
cargo install sqlx-cli
```

### 2. Development Configuration

Create `config/development.toml`:

```toml
[server]
host = "127.0.0.1"
port = 8080
workers = 4

[database]
url = "postgresql://localhost:5432/constellation_dev"
max_connections = 10

[redis]
url = "redis://localhost:6379"
connection_timeout = 5
read_timeout = 3

[security]
key_rotation_days = 90
encryption_algorithm = "AES-256-GCM"
signature_algorithm = "Ed25519"

[agents]
max_concurrent = 50
heartbeat_interval = 30
health_check_timeout = 10

[dtg]
max_nodes = 10000
cache_size_mb = 100

[autonomy]
measurement_interval = 60
history_days = 30
```

### 3. Development Commands

```bash
# Run with hot reload
cargo watch -x 'run --bin constellation'

# Run tests
cargo test -- --nocapture

# Run benchmarks
cargo bench

# Check code quality
cargo clippy --all-targets --all-features
cargo fmt --check
```

## Docker Deployment

### 1. Docker Compose Setup

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: constellation
      POSTGRES_USER: constellation
      POSTGRES_PASSWORD: ${DB_PASSWORD:-changeme}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U constellation"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

  constellation:
    build: .
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    environment:
      DATABASE_URL: postgresql://constellation:${DB_PASSWORD:-changeme}@postgres:5432/constellation
      REDIS_URL: redis://redis:6379
      RUST_LOG: info
    ports:
      - "8080:8080"
    volumes:
      - ./config:/app/config
      - ./data:/app/data
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
```

### 2. Dockerfile

Create `Dockerfile`:

```dockerfile
FROM rust:1.75-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies
RUN cargo build --release --locked

# Copy source code
COPY . .

# Build application
RUN cargo build --release --bin constellation

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 constellation

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/constellation /app/constellation

# Copy configuration
COPY config /app/config

# Set permissions
RUN chown -R constellation:constellation /app
USER constellation

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["curl", "-f", "http://localhost:8080/health"]

EXPOSE 8080

ENTRYPOINT ["/app/constellation"]
```

### 3. Docker Commands

```bash
# Build and run
docker-compose up --build

# Run in background
docker-compose up -d

# View logs
docker-compose logs -f constellation

# Scale services
docker-compose up --scale constellation=3

# Stop services
docker-compose down
```

## Kubernetes Deployment

### 1. Kubernetes Manifests

Create `k8s/namespace.yaml`:

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: constellation
  labels:
    name: constellation
```

Create `k8s/configmap.yaml`:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: constellation-config
  namespace: constellation
data:
  config.toml: |
    [server]
    host = "0.0.0.0"
    port = 8080
    
    [database]
    url = "postgresql://constellation:${DB_PASSWORD}@postgres.constellation.svc.cluster.local:5432/constellation"
    
    [redis]
    url = "redis://redis.constellation.svc.cluster.local:6379"
```

Create `k8s/secrets.yaml`:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: constellation-secrets
  namespace: constellation
type: Opaque
stringData:
  database-password: "changeme"
  encryption-key: "generated-encryption-key"
```

Create `k8s/deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: constellation
  namespace: constellation
  labels:
    app: constellation
spec:
  replicas: 3
  selector:
    matchLabels:
      app: constellation
  template:
    metadata:
      labels:
        app: constellation
    spec:
      containers:
      - name: constellation
        image: your-registry/constellation:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_PASSWORD
          valueFrom:
            secretKeyRef:
              name: constellation-secrets
              key: database-password
        - name: ENCRYPTION_KEY
          valueFrom:
            secretKeyRef:
              name: constellation-secrets
              key: encryption-key
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

Create `k8s/service.yaml`:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: constellation
  namespace: constellation
spec:
  selector:
    app: constellation
  ports:
  - port: 80
    targetPort: 8080
  type: ClusterIP
```

Create `k8s/ingress.yaml`:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: constellation
  namespace: constellation
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  rules:
  - host: constellation.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: constellation
            port:
              number: 80
```

### 2. Kubernetes Commands

```bash
# Apply all manifests
kubectl apply -f k8s/

# Check deployment status
kubectl get pods -n constellation
kubectl get svc -n constellation
kubectl get ingress -n constellation

# View logs
kubectl logs -f deployment/constellation -n constellation

# Scale deployment
kubectl scale deployment constellation --replicas=5 -n constellation

# Update deployment
kubectl set image deployment/constellation constellation=your-registry/constellation:v2.0.0 -n constellation
```

## Cloud Deployment

### 1. AWS ECS Deployment

Create `aws/ecs-task-definition.json`:

```json
{
  "family": "constellation",
  "networkMode": "awsvpc",
  "executionRoleArn": "arn:aws:iam::123456789012:role/ecsTaskExecutionRole",
  "containerDefinitions": [
    {
      "name": "constellation",
      "image": "123456789012.dkr.ecr.us-east-1.amazonaws.com/constellation:latest",
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "DATABASE_URL",
          "value": "postgresql://constellation:${DB_PASSWORD}@${DB_HOST}:5432/constellation"
        },
        {
          "name": "REDIS_URL",
          "value": "redis://${REDIS_HOST}:6379"
        }
      ],
      "secrets": [
        {
          "name": "DB_PASSWORD",
          "valueFrom": "arn:aws:secretsmanager:us-east-1:123456789012:secret:constellation-db-password"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/constellation",
          "awslogs-region": "us-east-1",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ],
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "512",
  "memory": "1024"
}
```

### 2. Google Cloud Run

Create `gcp/cloudrun.yaml`:

```yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: constellation
  namespace: default
spec:
  template:
    metadata:
      annotations:
        autoscaling.knative.dev/minScale: "1"
        autoscaling.knative.dev/maxScale: "10"
    spec:
      containers:
      - image: gcr.io/your-project/constellation:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: constellation-secrets
              key: database-url
        resources:
          limits:
            cpu: 1000m
            memory: 512Mi
          requests:
            cpu: 250m
            memory: 256Mi
```

### 3. Azure Container Instances

Create `azure/container-instance.yaml`:

```yaml
apiVersion: 2019-12-01
location: eastus
name: constellation
properties:
  containers:
  - name: constellation
    properties:
      image: yourregistry.azurecr.io/constellation:latest
      ports:
      - port: 8080
        protocol: TCP
      environmentVariables:
      - name: DATABASE_URL
        value: "Server=tcp:constellation-db.database.windows.net,1433;Database=constellation;User ID=constellation;Password=${DB_PASSWORD};Encrypt=true;TrustServerCertificate=false;Connection Timeout=30;"
      resources:
        requests:
          cpu: 1.0
          memoryInGB: 1.5
        limits:
          cpu: 2.0
          memoryInGB: 3.0
  osType: Linux
  ipAddress:
    type: Public
    ports:
    - protocol: tcp
      port: 80
  restartPolicy: Always
```

## Configuration Reference

### Server Configuration

```toml
[server]
# Network configuration
host = "0.0.0.0"
port = 8080
workers = 4

# Timeouts
request_timeout = 30
shutdown_timeout = 30

# CORS
cors_origins = ["http://localhost:3000", "https://example.com"]
cors_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
cors_headers = ["Authorization", "Content-Type"]
```

### Database Configuration

```toml
[database]
# Connection
url = "postgresql://user:password@localhost:5432/constellation"
max_connections = 20
min_connections = 5
connect_timeout = 10
idle_timeout = 300

# Pooling
pool_timeout = 30
statement_cache_size = 100

# SSL
ssl_mode = "prefer"
ssl_root_cert = "/path/to/ca.pem"
ssl_cert = "/path/to/client-cert.pem"
ssl_key = "/path/to/client-key.pem"
```

### Redis Configuration

```toml
[redis]
# Connection
url = "redis://localhost:6379"
connection_timeout = 5
read_timeout = 3
write_timeout = 3

# Pooling
max_connections = 10
min_idle_connections = 2

# TLS
tls_enabled = false
tls_cert = "/path/to/cert.pem"
tls_key = "/path/to/key.pem"
tls_ca = "/path/to/ca.pem"
```

### Security Configuration

```toml
[security]
# Key management
key_rotation_days = 90
key_backup_enabled = true
key_backup_location = "/secure/backup"

# Encryption
encryption_algorithm = "AES-256-GCM"
key_size_bits = 256
nonce_size_bytes = 12

# Signatures
signature_algorithm = "Ed25519"
signature_expiry_hours = 24

# Access control
require_authentication = true
require_authorization = true
default_role = "viewer"
```

### Agent Configuration

```toml
[agents]
# Limits
max_concurrent = 100
max_memory_mb = 4096
max_cpu_cores = 4

# Health checking
heartbeat_interval = 30
health_check_timeout = 10
unhealthy_threshold = 3

# Resource management
resource_check_interval = 60
auto_scaling = true
min_instances = 1
max_instances = 10
```

### DTG Configuration

```toml
[dtg]
# Limits
max_nodes = 10000
max_edges_per_node = 50
max_execution_depth = 100

# Caching
cache_size_mb = 100
cache_ttl_seconds = 3600

# Execution
max_concurrent_executions = 10
execution_timeout_seconds = 3600
retry_attempts = 3
retry_delay_seconds = 5
```

### Autonomy Configuration

```toml
[autonomy]
# Measurement
measurement_interval = 60
history_days = 30
snapshot_interval = 3600

# Scoring
kappa_update_threshold = 0.01
capability_weight_adjustment = 0.1
self_assessment_weight = 0.3

# Research
max_concurrent_experiments = 5
exploration_exploitation_ratio = 0.3
minimum_evidence_strength = 0.7
```

## Monitoring and Observability

### 1. Metrics Collection

Create `config/metrics.toml`:

```toml
[metrics]
# Prometheus
prometheus_enabled = true
prometheus_port = 9090
scrape_interval = 15

# Custom metrics
agent_count_enabled = true
task_queue_size_enabled = true
dtg_execution_time_enabled = true
autonomy_scores_enabled = true

# Alerting
alert_rules = "/config/alerts.yml"
```

### 2. Logging Configuration

Create `config/logging.toml`:

```toml
[logging]
# Levels
default_level = "info"
rust_log = "info"
sqlx_log = "warn"

# Output
console_enabled = true
file_enabled = true
file_path = "/var/log/constellation/app.log"
file_rotation = "daily"
file_retention = 30

# Structured logging
json_format = true
include_timestamp = true
include_level = true
include_target = true

# Filtering
filters = [
  "info",
  "warn",
  "error",
  "constellation=debug",
  "sqlx=warn"
]
```

### 3. Tracing Configuration

Create `config/tracing.toml`:

```toml
[tracing]
# OpenTelemetry
otel_enabled = true
otel_endpoint = "http://localhost:4317"
otel_service_name = "constellation"

# Sampling
sampling_rate = 0.1
max_traces_per_second = 100

# Attributes
include_http_headers = true
include_sql_queries = false
include_redis_commands = false

# Export
export_timeout = 30
export_batch_size = 512
export_batch_timeout = 5
```

### 4. Health Checks

```bash
# Basic health check
curl http://localhost:8080/health

# Detailed health check
curl http://localhost:8080/health/detailed

# Readiness check
curl http://localhost:8080/ready

# Liveness check
curl http://localhost:8080/live
```

## Security Configuration

### 1. TLS/SSL Configuration

```toml
[tls]
enabled = true
cert_file = "/path/to/cert.pem"
key_file = "/path/to/key.pem"
ca_file = "/path/to/ca.pem"

# Cipher suites
ciphers = [
  "TLS_AES_256_GCM_SHA384",
  "TLS_CHACHA20_POLY1305_SHA256",
  "TLS_AES_128_GCM_SHA256"
]

# Protocols
min_version = "TLSv1.2"
max_version = "TLSv1.3"
```

### 2. Authentication Configuration

```toml
[auth]
# JWT
jwt_secret = "your-jwt-secret"
jwt_expiry_hours = 24
jwt_issuer = "constellation"

# OAuth2
oauth2_enabled = false
oauth2_provider = "google"
oauth2_client_id = "your-client-id"
oauth2_client_secret = "your-client-secret"
oauth2_redirect_url = "https://example.com/auth/callback"

# API Keys
api_keys_enabled = true
api_key_header = "X-API-Key"
api_key_length = 32
```

### 3. Rate Limiting

```toml
[rate_limiting]
enabled = true
strategy = "token_bucket"

# Limits
requests_per_minute = 60
burst_size = 10

# Storage
storage_backend = "redis"
storage_key_prefix = "ratelimit:"

# Headers
include_headers = true
limit_header = "X-RateLimit-Limit"
remaining_header = "X-RateLimit-Remaining"
reset_header = "X-RateLimit-Reset"
```

## Troubleshooting

### Common Issues

#### 1. Database Connection Issues

```bash
# Check database connectivity
psql -h localhost -p 5432 -U constellation -d constellation

# Check connection pool
curl http://localhost:8080/health/detailed | jq '.database'

# Reset connection pool
sudo systemctl restart constellation
```

#### 2. Redis Connection Issues

```bash
# Test Redis connection
redis-cli -h localhost -p 6379 ping

# Check Redis memory usage
redis-cli info memory

# Clear Redis cache (development only)
redis-cli flushall
```

#### 3. Agent Registration Issues

```bash
# Check agent registry
curl http://localhost:8080/api/v1/agents | jq '.'

# View agent logs
journalctl -u constellation -f

# Reset agent state
curl -X POST http://localhost:8080/api/v1/agents/reset
```

#### 4. Performance Issues

```bash
# Check system resources
top -p $(pgrep constellation)

# Check memory usage
pmap -x $(pgrep constellation) | tail -1

# Profile CPU usage
perf record -p $(pgrep constellation) -g -- sleep 30
perf report
```

### Debugging Commands

```bash
# Enable debug logging
export RUST_LOG=debug
cargo run --bin constellation

# Run with profiling
cargo run --release --bin constellation -- --profile

# Generate flamegraph
cargo flamegraph --bin constellation

# Memory profiling
valgrind --leak-check=full ./target/release/constellation
```

### Recovery Procedures

#### 1. Database Recovery

```sql
-- Backup database
pg_dump -U constellation constellation > backup.sql

-- Restore database
psql -U constellation constellation < backup.sql

-- Reset autonomy measurements
UPDATE autonomy_measurements SET status = 'pending' WHERE status = 'failed';
```

#### 2. Key Rotation Recovery

```bash
# Backup keys
cp -r /var/lib/constellation/keys /backup/keys-$(date +%Y%m%d)

# Rotate keys with backup
constellation-cli keys rotate --backup /backup/keys

# Verify key rotation
constellation-cli keys verify
```

#### 3. Agent State Recovery

```bash
# Backup agent state
redis-cli --rdb /backup/redis-dump.rdb

# Restore agent state
redis-cli shutdown
cp /backup/redis-dump.rdb /var/lib/redis/dump.rdb
systemctl start redis
```

## Support

- **Documentation**: This guide and API reference
- **Community**: GitHub discussions and Discord
- **Issues**: GitHub issue tracker
- **Security**: security@constellation.example.com

---

**Happy deploying!** 🚀