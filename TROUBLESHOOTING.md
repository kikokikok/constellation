# Constellation Troubleshooting Guide

This guide helps diagnose and resolve common issues with the Constellation A2A Message Broker.

## Table of Contents
1. [Quick Diagnostics](#quick-diagnostics)
2. [Common Issues and Solutions](#common-issues-and-solutions)
3. [Log Analysis](#log-analysis)
4. [Performance Troubleshooting](#performance-troubleshooting)
5. [Network Issues](#network-issues)
6. [Security Issues](#security-issues)
7. [Recovery Procedures](#recovery-procedures)

## Quick Diagnostics

### Health Check Commands

```bash
# Check Constellation application health
curl -f http://localhost:8080/health

# Check Iggy server health
curl -f http://localhost:3000/ping

# Check Prometheus metrics
curl http://localhost:9090/metrics

# Check service status (Docker Compose)
docker-compose ps

# Check service status (Kubernetes)
kubectl get pods -n constellation
kubectl get svc -n constellation
```

### Common Error Messages

| Error Message | Likely Cause | Solution |
|---------------|--------------|----------|
| `Connection refused` | Service not running | Start the service |
| `Authentication failed` | Invalid credentials | Check JWT tokens or API keys |
| `Message queue full` | High load or slow consumers | Scale up or optimize consumers |
| `Protocol version mismatch` | Client/server version mismatch | Update client or server |
| `Database connection failed` | Database unavailable | Check database service |

## Common Issues and Solutions

### 1. Service Won't Start

**Symptoms:**
- Container exits immediately
- Application crashes on startup
- Port already in use

**Solutions:**

```bash
# Check logs for errors
docker-compose logs constellation-app
kubectl logs -n constellation deployment/constellation

# Check port conflicts
sudo lsof -i :8080
sudo lsof -i :8090

# Check resource limits
docker stats
kubectl describe pod -n constellation <pod-name>

# Verify configuration
cat config/constellation.yaml
```

### 2. Authentication Failures

**Symptoms:**
- `401 Unauthorized` responses
- JWT token validation failures
- Agent registration rejected

**Solutions:**

```bash
# Verify JWT secret configuration
echo $JWT_SECRET

# Check token expiration
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/v1/auth/validate

# Regenerate agent keys
curl -X POST http://localhost:8080/v1/agents/register \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "your_agent"}'

# Check MCP crypto setup
cargo test --test test_crypto_integration
```

### 3. Message Delivery Issues

**Symptoms:**
- Messages not reaching recipients
- High message latency
- Messages stuck in queue

**Solutions:**

```bash
# Check message queue status
curl http://localhost:8080/v1/messages/stats

# Check Iggy topic status
curl http://localhost:3000/topics

# Monitor message flow
curl http://localhost:8080/v1/metrics | grep message_queue

# Check consumer groups
curl http://localhost:3000/consumer_groups
```

### 4. Performance Problems

**Symptoms:**
- High CPU/Memory usage
- Slow message processing
- Timeout errors

**Solutions:**

```bash
# Monitor resource usage
docker stats
kubectl top pods -n constellation

# Check garbage collection
export RUST_LOG=debug
docker-compose restart constellation-app

# Optimize configuration
# Increase queue sizes:
# MAX_QUEUE_SIZE=50000
# MESSAGE_TTL_SECONDS=7200

# Scale horizontally
kubectl scale deployment/constellation --replicas=5 -n constellation
```

## Log Analysis

### Log Levels and Configuration

```bash
# Set appropriate log levels
export RUST_LOG=info,constellation_core=debug,iggy=warn

# Common log patterns to watch:

# Startup issues
grep -i "error\|panic\|failed" constellation.log

# Authentication issues
grep -i "auth\|jwt\|token\|unauthorized" constellation.log

# Performance issues
grep -i "slow\|timeout\|latency\|queue" constellation.log

# Network issues
grep -i "connection\|timeout\|refused\|reset" constellation.log
```

### Structured Log Fields

Constellation uses structured logging with these key fields:

- `agent_id`: ID of the agent involved
- `message_id`: Unique message identifier
- `correlation_id`: Request correlation ID
- `duration_ms`: Operation duration
- `error`: Error details if any
- `component`: Component name (broker, auth, etc.)

Example log query:
```bash
# Find errors for specific agent
grep '"agent_id":"agent_1"' constellation.log | grep '"error":'

# Find slow operations (>100ms)
grep '"duration_ms":[1-9][0-9][0-9]' constellation.log
```

## Performance Troubleshooting

### Performance Metrics

Key metrics to monitor:

```bash
# Message throughput
curl -s http://localhost:8080/v1/metrics | grep "messages_processed_total"

# Queue depth
curl -s http://localhost:8080/v1/metrics | grep "queue_depth"

# Latency percentiles
curl -s http://localhost:8080/v1/metrics | grep "message_latency_seconds"

# Error rates
curl -s http://localhost:8080/v1/metrics | grep "errors_total"
```

### Performance Optimization

1. **Increase batch sizes:**
```yaml
# In constellation.yaml
message_broker:
  max_batch_size: 1000
  batch_timeout_ms: 100
```

2. **Optimize database queries:**
```bash
# Enable query logging
export SQLX_LOG=debug

# Check slow queries
SELECT * FROM pg_stat_statements ORDER BY total_time DESC LIMIT 10;
```

3. **Adjust connection pooling:**
```yaml
database:
  max_connections: 50
  min_connections: 10
  connect_timeout: 30s
  idle_timeout: 10m
```

## Network Issues

### Connectivity Testing

```bash
# Test Iggy connectivity
nc -zv localhost 8090
curl -v http://localhost:3000/ping

# Test database connectivity
psql -h localhost -p 5432 -U constellation -d constellation -c "SELECT 1"

# Test Redis connectivity
redis-cli -h localhost -p 6379 PING

# Test internal service communication
curl -v http://constellation-app:8080/health
```

### Firewall and Network Policies

```bash
# Check open ports
sudo netstat -tulpn | grep -E "(8080|8090|5432|6379)"

# Check Kubernetes network policies
kubectl get networkpolicies -n constellation

# Check service endpoints
kubectl get endpoints -n constellation
```

## Security Issues

### Common Security Problems

1. **Exposed endpoints:**
```bash
# Check for unnecessary exposed ports
docker ps --format "table {{.Names}}\t{{.Ports}}"
kubectl get svc -n constellation -o wide
```

2. **Weak credentials:**
```bash
# Rotate JWT secret
export JWT_SECRET=$(openssl rand -base64 32)

# Rotate database password
export POSTGRES_PASSWORD=$(openssl rand -base64 16)

# Update in Kubernetes
kubectl create secret generic constellation-secrets -n constellation \
  --from-literal=jwt-secret=$JWT_SECRET \
  --from-literal=postgres-password=$POSTGRES_PASSWORD \
  --dry-run=client -o yaml | kubectl apply -f -
```

3. **Certificate issues:**
```bash
# Check certificate validity
openssl x509 -in cert.pem -text -noout | grep -A2 "Validity"

# Generate new certificates
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

## Recovery Procedures

### Data Corruption Recovery

1. **Database recovery:**
```bash
# Restore from backup
./backup-recovery.sh restore /var/backups/constellation/full_backup_*.tar.gz

# Repair corrupted tables
psql -h localhost -U constellation -d constellation -c "VACUUM ANALYZE;"
```

2. **Message queue recovery:**
```bash
# Reset Iggy topics (CAUTION: data loss)
curl -X DELETE http://localhost:3000/topics/agent_messages

# Recreate with proper configuration
curl -X POST http://localhost:3000/topics \
  -H "Content-Type: application/json" \
  -d '{"name": "agent_messages", "partitions": 4}'
```

3. **Configuration recovery:**
```bash
# Restore configuration from backup
tar -xzf /var/backups/constellation/config_*.tar.gz -C /

# Validate configuration
constellation-core --config /etc/constellation/constellation.yaml --validate
```

### Disaster Recovery

1. **Full cluster recovery:**
```bash
# Stop all services
docker-compose down
kubectl delete namespace constellation

# Restore from backup
./backup-recovery.sh restore latest_backup.tar.gz

# Redeploy
docker-compose up -d
# OR
helm install constellation ./helm/constellation -n constellation --create-namespace
```

2. **Partial recovery (single component):**
```bash
# Database only
./backup-recovery.sh restore --component database latest_backup.tar.gz

# Message queue only
./backup-recovery.sh restore --component iggy latest_backup.tar.gz
```

## Getting Help

If you cannot resolve an issue:

1. **Collect diagnostics:**
```bash
# Create diagnostic bundle
./scripts/create-diagnostics.sh

# Include:
# - Application logs
# - System logs
# - Configuration files
# - Metrics snapshot
# - Error messages
```

2. **Check known issues:**
- [GitHub Issues](https://github.com/constellation/constellation/issues)
- [Documentation](./README.md)
- [API Reference](./API.md)

3. **Contact support:**
- Open a GitHub issue with diagnostics
- Join the community Slack/Discord
- Email: support@constellation.example.com

## Prevention Best Practices

1. **Regular monitoring:**
   - Set up alerts for critical metrics
   - Regular health checks
   - Capacity planning

2. **Proactive maintenance:**
   - Regular backups
   - Security updates
   - Performance tuning

3. **Testing:**
   - Load testing before deployment
   - Disaster recovery drills
   - Security penetration testing