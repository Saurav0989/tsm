# Lattice Operations Manual

**Day-to-day operational procedures for production clusters**

---

## Daily Operations

### Morning Checklist

```bash
# 1. Check cluster health
lattice status

# Expected: All nodes healthy, leader elected

# 2. Check metrics
curl http://localhost:9001/metrics | grep -E "(errors|latency|throughput)"

# 3. Review logs
journalctl -u lattice@1 --since "24 hours ago" | grep -E "(ERROR|WARN|CRITICAL)"

# 4. Verify backups
ls -lh /var/backups/ | tail -5

# 5. Check disk space
df -h /var/lib/lattice
```

### Health Monitoring

**Every 5 minutes (automated):**
```bash
#!/bin/bash
# /usr/local/bin/lattice-health-check.sh

STATUS=$(lattice status 2>&1)

if echo "$STATUS" | grep -q "Healthy"; then
    echo "$(date) - Cluster healthy"
else
    echo "$(date) - ALERT: Cluster unhealthy"
    # Send alert
    curl -X POST https://webhook.site/xxx \
      -d "message=Lattice cluster unhealthy"
fi
```

---

## Routine Maintenance

### Weekly Tasks

**Sunday 2 AM (automated):**

```bash
#!/bin/bash
# /usr/local/bin/lattice-weekly-maintenance.sh

# 1. Create snapshots
for node in 1 2 3; do
    lattice admin snapshot $node
done

# 2. Compact WAL
for node in 1 2 3; do
    # Keep last 100k entries
    lattice admin compact-log $node 100000
done

# 3. Backup to offsite storage
for node in 1 2 3; do
    lattice backup $node /var/backups/weekly/node$node-$(date +%Y%m%d).tar.gz
    aws s3 cp /var/backups/weekly/node$node-$(date +%Y%m%d).tar.gz \
      s3://lattice-backups/weekly/
done

# 4. Clean old local backups (keep 30 days)
find /var/backups -name "*.tar.gz" -mtime +30 -delete
```

### Monthly Tasks

**First Sunday of month:**

```bash
# 1. Run full benchmark
lattice benchmark 300

# 2. Review performance trends
# Compare with baseline metrics

# 3. Update documentation
# Document any config changes

# 4. Security audit
# Review access logs
journalctl -u lattice@1 | grep -i "denied\|unauthorized"
```

---

## Common Operational Procedures

### Adding a New Node

```bash
# 1. Prepare new server
ssh new-server
sudo mkdir -p /var/lib/lattice/node-4

# 2. Install Lattice
sudo cp lattice /usr/local/bin/

# 3. Generate config
lattice config generate 4 /etc/lattice/node4.toml

# 4. Update cluster config on existing nodes
# Edit /etc/lattice/node*.toml and add:
# [[cluster.peers]]
# id = 4
# address = "10.0.1.13:5001"

# 5. Restart existing nodes (rolling restart)
for node in 1 2 3; do
    sudo systemctl restart lattice@$node
    sleep 30 # Wait for re-election
done

# 6. Start new node
lattice start 4 /etc/lattice/node4.toml

# 7. Verify
lattice status
# Should show 4 nodes
```

### Removing a Node

```bash
# 1. Remove from cluster config
lattice admin remove-node 3

# 2. Stop node
ssh node3
sudo systemctl stop lattice@3

# 3. Update configs on remaining nodes
# Remove node 3 from cluster.peers

# 4. Restart remaining nodes
for node in 1 2; do
    sudo systemctl restart lattice@$node
    sleep 30
done

# 5. Verify
lattice status
# Should show 2 nodes
```

### Rolling Restart

**Zero-downtime cluster update:**

```bash
#!/bin/bash
# rolling-restart.sh

NODES=(1 2 3)

for node in "${NODES[@]}"; do
    echo "Restarting node $node..."
    
    # 1. Stop node
    sudo systemctl stop lattice@$node
    
    # 2. Wait for re-election
    sleep 60
    
    # 3. Verify quorum maintained
    if ! lattice status | grep -q "Quorum"; then
        echo "ERROR: Lost quorum, aborting"
        sudo systemctl start lattice@$node
        exit 1
    fi
    
    # 4. Start node
    sudo systemctl start lattice@$node
    
    # 5. Wait for sync
    sleep 30
    
    # 6. Verify node healthy
    lattice status $node
    
    echo "Node $node restarted successfully"
done

echo "Rolling restart complete"
```

### Leadership Transfer

```bash
# Transfer leadership from node 1 to node 2
lattice admin transfer-leadership 2

# Verify
lattice status
# Should show: Node 2: ✅ Leader
```

---

## Incident Response

### Node Failure

**Symptoms:**
- Node unreachable
- No heartbeat
- Not in cluster status

**Response:**

```bash
# 1. Check if node is down
ssh node2
sudo systemctl status lattice@2

# 2. Check logs
journalctl -u lattice@2 -n 100

# 3. Attempt restart
sudo systemctl restart lattice@2

# 4. If restart fails, restore from backup
lattice restore 2 /var/backups/node2-latest.tar.gz
sudo systemctl start lattice@2

# 5. Verify cluster recovered
lattice status
```

### Split Brain

**Symptoms:**
- Two leaders
- Inconsistent state across nodes
- Partition detected

**Response:**

```bash
# 1. Identify partition
lattice status --verbose
# Look for network issues

# 2. Verify network connectivity
for node in 1 2 3; do
    ping -c 3 10.0.1.1$node
done

# 3. If network is fine, force re-election
# Stop minority partition
ssh node3
sudo systemctl stop lattice@3

# 4. Wait for majority to stabilize
sleep 60

# 5. Restart minority nodes
sudo systemctl start lattice@3

# 6. Verify single leader
lattice status
```

### High Latency

**Symptoms:**
- Slow transactions
- High P99 latency
- User complaints

**Investigation:**

```bash
# 1. Check current latency
curl http://localhost:9001/metrics | grep latency

# 2. Check system resources
top
iostat -x 1 10

# 3. Check network
iftop

# 4. Check if verification is bottleneck
journalctl -u lattice@1 | grep "verification took"

# 5. Tune configuration
# Edit /etc/lattice/node1.toml
[performance]
batch_size = 200 # Increase batching
cache_size = 20000 # Increase cache

# 6. Restart with new config
sudo systemctl restart lattice@1
```

### State Divergence

**THIS IS CRITICAL - System halted to prevent corruption**

**Symptoms:**
- Node stopped
- Log shows "STATE DIVERGENCE DETECTED"
- AI analysis output present

**Response:**

```bash
# 1. DO NOT restart node yet
# Review divergence details
journalctl -u lattice@1 | grep -A 50 "DIVERGENCE"

# 2. Examine AI analysis
cat /var/lib/lattice/node-1/divergence-report.json

# 3. Review suggested fix
# AI will suggest patch or configuration change

# 4. Apply fix (example)
# If bug in code: Apply patch and rebuild
# If memory corruption: Check hardware
# If config issue: Update config

# 5. Restore from last known good state
lattice restore 1 /var/backups/node1-pre-divergence.tar.gz

# 6. Restart node
sudo systemctl start lattice@1

# 7. Monitor closely
watch -n 1 'lattice status 1 --verbose'

# 8. Document incident
echo "$(date) - Divergence incident - Cause: XYZ - Fix: ABC" \
  >> /var/log/lattice/incidents.log
```

### Disk Full

**Response:**

```bash
# 1. Check disk usage
df -h /var/lib/lattice
du -sh /var/lib/lattice/node-1/*

# 2. Compact WAL immediately
lattice admin compact-log 1 50000

# 3. Create snapshot
lattice admin snapshot 1

# 4. Remove old snapshots
ls -lh /var/lib/lattice/node-1/snapshots/
rm /var/lib/lattice/node-1/snapshots/snapshot-00000100.snap

# 5. If still full, add disk space
# Or move to larger volume
```

---

## Performance Tuning

### Throughput Optimization

```toml
# /etc/lattice/node1.toml

[performance]
batch_size = 500          # Larger batches
batch_timeout_ms = 5      # Lower timeout
worker_threads = 8        # More parallelism
cache_size = 50000        # Larger cache

[consensus]
max_entries_per_append = 500  # Bigger replication batches
```

### Latency Optimization

```toml
[performance]
batch_size = 10           # Smaller batches
batch_timeout_ms = 1      # Lower timeout for quick flush
worker_threads = 16       # More threads for parallelism

[storage]
sync_on_write = false     # Trade durability for speed (careful!)

[network]
tcp_nodelay = true        # Disable Nagle
```

### Memory Optimization

```toml
[performance]
cache_size = 1000         # Smaller cache
batch_size = 50           # Smaller batches

[storage]
max_wal_size_mb = 32      # Smaller WAL before rotation
snapshot_interval = 5000  # More frequent snapshots
```

---

## Monitoring & Alerting

### Key Metrics to Monitor

**Critical:**
- Cluster quorum (alert if < 50%)
- Error rate (alert if > 0.1%)
- Leader election failures
- State divergences (alert immediately)

**Important:**
- Verification latency P99 (alert if > 2ms)
- Throughput (alert if < 5000 tx/sec)
- Disk usage (alert if > 80%)
- Network errors (alert if > 100/min)

**Informational:**
- Verification latency P50
- Network throughput
- Cache hit rate

### Alert Rules (Prometheus)

```yaml
# prometheus-alerts.yml

groups:
  - name: lattice
    interval: 10s
    rules:
      - alert: LatticeNodeDown
        expr: up{job="lattice"} == 0
        for: 1m
        annotations:
          summary: "Lattice node is down"
          
      - alert: LatticeLostQuorum
        expr: lattice_cluster_quorum < 0.5
        for: 30s
        annotations:
          summary: "Lattice lost quorum"
          
      - alert: LatticeStateDivergence
        expr: increase(lattice_errors_divergences[1m]) > 0
        annotations:
          summary: "STATE DIVERGENCE DETECTED - CRITICAL"
          
      - alert: LatticeHighLatency
        expr: lattice_verification_latency_p99_us > 2000
        for: 5m
        annotations:
          summary: "Verification latency is high"
```

---

## Disaster Recovery

### Scenario: Complete Cluster Loss

```bash
# 1. Restore all nodes from backup
for node in 1 2 3; do
    lattice restore $node /var/backups/latest/node$node.tar.gz
done

# 2. Start nodes
for node in 1 2 3; do
    sudo systemctl start lattice@$node
done

# 3. Verify cluster formation
lattice status

# 4. If nodes don't form cluster, manually trigger election
lattice admin transfer-leadership 1
```

### Scenario: Corrupted State

```bash
# 1. Stop affected node
sudo systemctl stop lattice@2

# 2. Clear corrupted data
rm -rf /var/lib/lattice/node-2/wal/*
rm -rf /var/lib/lattice/node-2/snapshots/*

# 3. Restore from healthy node
scp node1:/var/lib/lattice/node-1/snapshots/latest.snap \
    /var/lib/lattice/node-2/snapshots/

# 4. Start node
sudo systemctl start lattice@2

# 5. Verify sync
lattice status 2 --verbose
```

---

## Change Management

### Configuration Changes

```bash
# 1. Test in development
lattice config validate /tmp/new-config.toml

# 2. Backup current config
cp /etc/lattice/node1.toml /etc/lattice/node1.toml.backup

# 3. Apply new config
cp /tmp/new-config.toml /etc/lattice/node1.toml

# 4. Rolling restart
./rolling-restart.sh

# 5. Monitor for issues
watch -n 5 'lattice status --verbose'

# 6. Rollback if needed
cp /etc/lattice/node1.toml.backup /etc/lattice/node1.toml
./rolling-restart.sh
```

### Software Updates

```bash
# 1. Test new version in staging
./run-integration-tests.sh

# 2. Rolling upgrade
for node in 1 2 3; do
    # Stop node
    sudo systemctl stop lattice@$node
    
    # Update binary
    sudo cp lattice-v2.0 /usr/local/bin/lattice
    
    # Start node
    sudo systemctl start lattice@$node
    
    # Verify
    lattice status $node
    
    # Wait before next
    sleep 60
done

# 3. Verify cluster
lattice status
```

---

## Runbook Index

**Quick Reference:**

- Node won't start → [Troubleshooting](#troubleshooting)
- No leader elected → [Split Brain](#split-brain)
- High latency → [High Latency](#high-latency)
- Divergence detected → [State Divergence](#state-divergence)
- Disk full → [Disk Full](#disk-full)
- Add node → [Adding a New Node](#adding-a-new-node)
- Remove node → [Removing a Node](#removing-a-node)
- Update software → [Software Updates](#software-updates)
- Complete failure → [Disaster Recovery](#disaster-recovery)

---

## Contact Information

**On-call rotation:** https://pagerduty.com/lattice  
**Escalation:** ops-manager@company.com  
**Emergency:** +1-555-LATTICE
