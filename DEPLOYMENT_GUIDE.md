# Constellation Deployment Guide

This guide covers deployment options for the Constellation A2A Message Broker with Apache Iggy.

## Table of Contents
1. [Quick Start with Docker Compose](#quick-start-with-docker-compose)
2. [Manual Deployment](#manual-deployment)
3. [Kubernetes Deployment](#kubernetes-deployment)
4. [Production Considerations](#production-considerations)
5. [Monitoring and Observability](#monitoring-and-observability)
6. [Backup and Recovery](#backup-and-recovery)

## Quick Start with Docker Compose

The easiest way to get started is using Docker Compose:

```bash
# Clone the repository
git clone <repository-url>
cd constellation

# Start all services
docker-compose up -d

# Check service status
docker-compose ps

# View logs
docker-compose logs -f constellation-app
```

Services started:
- **Iggy Server**: `http://localhost:8090` (message broker)
- **Constellation App**: `http://localhost:8080` (A2A API)
- **Prometheus**: `http://localhost:9090` (metrics)
- **Grafana**: `http://localhost:3001` (dashboards)

## Manual Deployment

### Prerequisites
- Rust 1.80+ and Cargo
- Iggy server running (or use the provided Docker image)

### Step 1: Install Iggy Server

```bash
# Using Docker
docker run -d \
  -p 8090:8090 \
  -p 3000:3000 \
  -v iggy-data:/data \
  iggyrs/iggy:latest

# Or build from source
cargo install iggy-server
iggy-server
```

### Step 2: Build and Run Constellation

```bash
# Clone and build
git clone <repository-url>
cd constellation
cargo build --release

# Run examples
./target/release/examples/iggy_message_broker_example
./target/release/examples/a2a_request_response_example
```

### Step 3: Configure Environment

Create a `.env` file:
```env
IGGY_SERVER_ADDRESS=127.0.0.1:8090
IGGY_USERNAME=guest
IGGY_PASSWORD=guest
RUST_LOG=info
CONSTELLATION_ENV=production
```

## Kubernetes Deployment

### Prerequisites
- Kubernetes cluster (minikube, EKS, GKE, AKS)
- kubectl configured
- Helm (optional)

### Deployment Manifests

Create `kubernetes/` directory with the following files:

#### 1. Namespace
```yaml
# kubernetes/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: constellation
```

#### 2. ConfigMap for Configuration
```yaml
# kubernetes/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: constellation-config
  namespace: constellation
data:
  iggy-server-address: "iggy-server.constellation.svc.cluster.local:8090"
  log-level: "info"
```

#### 3. Iggy Server Deployment
```yaml
# kubernetes/iggy-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: iggy-server
  namespace: constellation
spec:
  replicas: 3
  selector:
    matchLabels:
      app: iggy-server
  template:
    metadata:
      labels:
        app: iggy-server
    spec:
      containers:
      - name: iggy-server
        image: iggyrs/iggy:latest
        ports:
        - containerPort: 8090
          name: tcp
        - containerPort: 3000
          name: http
        env:
        - name: IGGY_SERVER_ADDRESS
          value: "0.0.0.0:8090"
        - name: IGGY_HTTP_ENABLED
          value: "true"
        - name: IGGY_HTTP_ADDRESS
          value: "0.0.0.0:3000"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        volumeMounts:
        - name: iggy-data
          mountPath: /data
      volumes:
      - name: iggy-data
        emptyDir: {}
---
apiVersion: v1
kind: Service
metadata:
  name: iggy-server
  namespace: constellation
spec:
  selector:
    app: iggy-server
  ports:
  - port: 8090
    targetPort: 8090
    name: tcp
  - port: 3000
    targetPort: 3000
    name: http
  type: ClusterIP
```

#### 4. Constellation Application Deployment
```yaml
# kubernetes/constellation-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: constellation-app
  namespace: constellation
spec:
  replicas: 2
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
        image: constellation:latest
        ports:
        - containerPort: 8080
        env:
        - name: IGGY_SERVER_ADDRESS
          valueFrom:
            configMapKeyRef:
              name: constellation-config
              key: iggy-server-address
        - name: RUST_LOG
          valueFrom:
            configMapKeyRef:
              name: constellation-config
              key: log-level
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "200m"
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
---
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
  type: LoadBalancer
```

#### 5. Horizontal Pod Autoscaler
```yaml
# kubernetes/hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: constellation-hpa
  namespace: constellation
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: constellation-app
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### Deploy to Kubernetes

```bash
# Apply all manifests
kubectl apply -f kubernetes/

# Check deployment status
kubectl get all -n constellation

# View logs
kubectl logs -f deployment/constellation-app -n constellation

# Get service URL
kubectl get svc constellation -n constellation
```

## Production Considerations

### 1. High Availability
- Deploy Iggy server with 3+ replicas
- Use persistent storage for Iggy data
- Implement proper health checks and readiness probes
- Use multiple availability zones

### 2. Security
- Use TLS/SSL for all communications
- Implement proper authentication and authorization
- Use secrets management (Hashicorp Vault, AWS Secrets Manager)
- Regular security updates and patches

### 3. Performance
- Monitor message throughput and latency
- Adjust Iggy configuration based on workload
- Implement connection pooling
- Use appropriate resource limits

### 4. Storage
- Use persistent volumes for Iggy data
- Implement backup strategy
- Monitor disk usage
- Consider SSD storage for high throughput

## Monitoring and Observability

### Metrics to Monitor
1. **Message Throughput**: Messages per second
2. **Latency**: End-to-end message delivery time
3. **Error Rates**: Failed messages, authentication errors
4. **Resource Usage**: CPU, memory, disk I/O
5. **Connection Count**: Active agent connections

### Logging
- Structured logging with JSON format
- Log aggregation (ELK stack, Loki)
- Log retention policy
- Alerting on critical errors

### Tracing
- Distributed tracing with OpenTelemetry
- Trace sampling for production
- Integration with monitoring tools

## Backup and Recovery

### Backup Strategy
1. **Iggy Data**: Regular backups of message store
2. **Configuration**: Version control for all configs
3. **Secrets**: Secure backup of encryption keys

### Recovery Procedures
1. **Data Loss**: Restore from latest backup
2. **Service Outage**: Failover to standby cluster
3. **Corruption**: Validate and repair data stores

### Disaster Recovery
- Multi-region deployment
- Regular disaster recovery drills
- Documented recovery procedures
- Automated recovery scripts

## Troubleshooting

### Common Issues

1. **Connection Issues**
   - Check Iggy server status
   - Verify network connectivity
   - Check firewall rules

2. **Performance Issues**
   - Monitor resource usage
   - Check message queue depth
   - Review configuration settings

3. **Authentication Failures**
   - Verify JWT tokens
   - Check MCP crypto configuration
   - Validate agent registration

### Debug Commands
```bash
# Check service health
curl http://localhost:8080/health

# View metrics
curl http://localhost:8080/metrics

# Check Iggy status
curl http://localhost:3000/ping

# View logs
docker-compose logs -f constellation-app
```

## Support

For issues and questions:
1. Check the [Troubleshooting Guide](./TROUBLESHOOTING.md)
2. Review [API Documentation](./API.md)
3. Open an issue on GitHub
4. Contact the development team