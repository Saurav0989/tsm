/*!
 * Monitoring & Observability - Production metrics and tracing
 * 
 * Provides:
 * - Performance metrics (latency, throughput, errors)
 * - Health checks
 * - Distributed tracing
 * - Alerting
 * - Dashboards
 */

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

use crate::NodeId;

/// Metrics collector - Tracks system performance
pub struct MetricsCollector {
    /// Node ID
    node_id: NodeId,
    
    /// Metrics storage
    metrics: Arc<RwLock<Metrics>>,
    
    /// Start time
    started_at: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    /// Verification metrics
    pub verification: VerificationMetrics,
    
    /// Consensus metrics
    pub consensus: ConsensusMetrics,
    
    /// Network metrics
    pub network: NetworkMetrics,
    
    /// Storage metrics
    pub storage: StorageMetrics,
    
    /// Error metrics
    pub errors: ErrorMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMetrics {
    /// Total verifications
    pub total: u64,
    
    /// Successful verifications
    pub success: u64,
    
    /// Failed verifications
    pub failed: u64,
    
    /// Average verification time (μs)
    pub avg_time_us: f64,
    
    /// P50 latency (μs)
    pub p50_us: u64,
    
    /// P95 latency (μs)
    pub p95_us: u64,
    
    /// P99 latency (μs)
    pub p99_us: u64,
    
    /// Latency histogram
    pub histogram: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusMetrics {
    /// Total elections
    pub elections: u64,
    
    /// Elections won
    pub elections_won: u64,
    
    /// Average election time (ms)
    pub avg_election_time_ms: f64,
    
    /// Total log entries
    pub log_entries: u64,
    
    /// Committed entries
    pub committed_entries: u64,
    
    /// Current term
    pub current_term: u64,
    
    /// Heartbeats sent
    pub heartbeats_sent: u64,
    
    /// Heartbeats received
    pub heartbeats_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Messages sent
    pub messages_sent: u64,
    
    /// Messages received
    pub messages_received: u64,
    
    /// Bytes sent
    pub bytes_sent: u64,
    
    /// Bytes received
    pub bytes_received: u64,
    
    /// Average message latency (ms)
    pub avg_latency_ms: f64,
    
    /// Connection errors
    pub connection_errors: u64,
    
    /// Active connections
    pub active_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetrics {
    /// WAL writes
    pub wal_writes: u64,
    
    /// Snapshots created
    pub snapshots_created: u64,
    
    /// Total disk bytes
    pub disk_bytes: u64,
    
    /// WAL sync time (μs)
    pub wal_sync_us: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// Divergences detected
    pub divergences: u64,
    
    /// Invariant violations
    pub invariant_violations: u64,
    
    /// Network errors
    pub network_errors: u64,
    
    /// Storage errors
    pub storage_errors: u64,
    
    /// Recovery attempts
    pub recovery_attempts: u64,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            verification: VerificationMetrics {
                total: 0,
                success: 0,
                failed: 0,
                avg_time_us: 0.0,
                p50_us: 0,
                p95_us: 0,
                p99_us: 0,
                histogram: vec![0; 100],
            },
            consensus: ConsensusMetrics {
                elections: 0,
                elections_won: 0,
                avg_election_time_ms: 0.0,
                log_entries: 0,
                committed_entries: 0,
                current_term: 0,
                heartbeats_sent: 0,
                heartbeats_received: 0,
            },
            network: NetworkMetrics {
                messages_sent: 0,
                messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
                avg_latency_ms: 0.0,
                connection_errors: 0,
                active_connections: 0,
            },
            storage: StorageMetrics {
                wal_writes: 0,
                snapshots_created: 0,
                disk_bytes: 0,
                wal_sync_us: 0.0,
            },
            errors: ErrorMetrics {
                divergences: 0,
                invariant_violations: 0,
                network_errors: 0,
                storage_errors: 0,
                recovery_attempts: 0,
            },
        }
    }
}

impl MetricsCollector {
    pub fn new(node_id: NodeId) -> Self {
        MetricsCollector {
            node_id,
            metrics: Arc::new(RwLock::new(Metrics::default())),
            started_at: Instant::now(),
        }
    }
    
    /// Record verification
    pub fn record_verification(&self, duration: Duration, success: bool) {
        let mut metrics = self.metrics.write();
        
        metrics.verification.total += 1;
        if success {
            metrics.verification.success += 1;
        } else {
            metrics.verification.failed += 1;
        }
        
        let us = duration.as_micros() as u64;
        
        // Update average
        let prev_avg = metrics.verification.avg_time_us;
        let n = metrics.verification.total as f64;
        metrics.verification.avg_time_us = 
            (prev_avg * (n - 1.0) + us as f64) / n;
        
        // Update histogram
        let bucket = (us / 10).min(99) as usize;
        metrics.verification.histogram[bucket] += 1;
    }
    
    /// Record message sent
    pub fn record_message_sent(&self, bytes: usize) {
        let mut metrics = self.metrics.write();
        metrics.network.messages_sent += 1;
        metrics.network.bytes_sent += bytes as u64;
    }
    
    /// Record message received
    pub fn record_message_received(&self, bytes: usize) {
        let mut metrics = self.metrics.write();
        metrics.network.messages_received += 1;
        metrics.network.bytes_received += bytes as u64;
    }
    
    /// Record election
    pub fn record_election(&self, duration: Duration, won: bool) {
        let mut metrics = self.metrics.write();
        metrics.consensus.elections += 1;
        if won {
            metrics.consensus.elections_won += 1;
        }
        
        let ms = duration.as_millis() as f64;
        let n = metrics.consensus.elections as f64;
        let prev_avg = metrics.consensus.avg_election_time_ms;
        metrics.consensus.avg_election_time_ms = 
            (prev_avg * (n - 1.0) + ms) / n;
    }
    
    /// Record error
    pub fn record_error(&self, error_type: ErrorType) {
        let mut metrics = self.metrics.write();
        match error_type {
            ErrorType::Divergence => metrics.errors.divergences += 1,
            ErrorType::InvariantViolation => metrics.errors.invariant_violations += 1,
            ErrorType::Network => metrics.errors.network_errors += 1,
            ErrorType::Storage => metrics.errors.storage_errors += 1,
            ErrorType::Recovery => metrics.errors.recovery_attempts += 1,
        }
    }
    
    /// Get current metrics snapshot
    pub fn snapshot(&self) -> Metrics {
        self.metrics.read().clone()
    }
    
    /// Get uptime
    pub fn uptime(&self) -> Duration {
        self.started_at.elapsed()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorType {
    Divergence,
    InvariantViolation,
    Network,
    Storage,
    Recovery,
}

/// Health checker - Monitors system health
pub struct HealthChecker {
    /// Node ID
    node_id: NodeId,
    
    /// Health status
    status: Arc<RwLock<HealthStatus>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub node_id: NodeId,
    pub healthy: bool,
    pub status: String,
    pub checks: HashMap<String, CheckResult>,
    pub last_check: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub passed: bool,
    pub message: String,
    pub latency_ms: u64,
}

impl HealthChecker {
    pub fn new(node_id: NodeId) -> Self {
        HealthChecker {
            node_id,
            status: Arc::new(RwLock::new(HealthStatus {
                node_id,
                healthy: true,
                status: "OK".to_string(),
                checks: HashMap::new(),
                last_check: 0,
            })),
        }
    }
    
    /// Run all health checks
    pub async fn check_health(&self, metrics: &Metrics) -> HealthStatus {
        let mut checks = HashMap::new();
        
        // Check 1: Verification success rate
        let verify_rate = if metrics.verification.total > 0 {
            metrics.verification.success as f64 / metrics.verification.total as f64
        } else {
            1.0
        };
        
        checks.insert("verification".to_string(), CheckResult {
            passed: verify_rate > 0.95,
            message: format!("Success rate: {:.2}%", verify_rate * 100.0),
            latency_ms: 0,
        });
        
        // Check 2: Error rate
        let error_rate = metrics.errors.divergences + 
                        metrics.errors.invariant_violations;
        
        checks.insert("errors".to_string(), CheckResult {
            passed: error_rate < 10,
            message: format!("Total errors: {}", error_rate),
            latency_ms: 0,
        });
        
        // Check 3: Network connectivity
        checks.insert("network".to_string(), CheckResult {
            passed: metrics.network.connection_errors < 100,
            message: format!("Connection errors: {}", metrics.network.connection_errors),
            latency_ms: 0,
        });
        
        // Overall health
        let healthy = checks.values().all(|c| c.passed);
        let status = if healthy { "OK" } else { "DEGRADED" };
        
        HealthStatus {
            node_id: self.node_id,
            healthy,
            status: status.to_string(),
            checks,
            last_check: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    
    /// Get current health status
    pub fn get_status(&self) -> HealthStatus {
        self.status.read().clone()
    }
}

/// Alert manager - Triggers alerts on critical events
pub struct AlertManager {
    /// Alert rules
    rules: Vec<AlertRule>,
    
    /// Active alerts
    active_alerts: Arc<RwLock<Vec<Alert>>>,
}

#[derive(Debug, Clone)]
pub struct AlertRule {
    pub name: String,
    pub condition: AlertCondition,
    pub severity: Severity,
}

#[derive(Debug, Clone)]
pub enum AlertCondition {
    VerificationFailureRate(f64),
    ErrorCountExceeds(u64),
    NodeUnreachable(Duration),
    DivergenceDetected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub rule_name: String,
    pub severity: String,
    pub message: String,
    pub triggered_at: u64,
    pub node_id: NodeId,
}

impl AlertManager {
    pub fn new() -> Self {
        AlertManager {
            rules: Vec::new(),
            active_alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Add alert rule
    pub fn add_rule(&mut self, rule: AlertRule) {
        println!("[Alert] Added rule: {}", rule.name);
        self.rules.push(rule);
    }
    
    /// Add default rules
    pub fn add_default_rules(&mut self) {
        self.add_rule(AlertRule {
            name: "High verification failure rate".to_string(),
            condition: AlertCondition::VerificationFailureRate(0.1),
            severity: Severity::Warning,
        });
        
        self.add_rule(AlertRule {
            name: "Divergence detected".to_string(),
            condition: AlertCondition::DivergenceDetected,
            severity: Severity::Critical,
        });
        
        self.add_rule(AlertRule {
            name: "Too many errors".to_string(),
            condition: AlertCondition::ErrorCountExceeds(100),
            severity: Severity::Error,
        });
    }
    
    /// Check metrics against rules
    pub fn check_metrics(&self, node_id: NodeId, metrics: &Metrics) {
        for rule in &self.rules {
            if self.should_alert(&rule.condition, metrics) {
                self.trigger_alert(node_id, rule);
            }
        }
    }
    
    fn should_alert(&self, condition: &AlertCondition, metrics: &Metrics) -> bool {
        match condition {
            AlertCondition::VerificationFailureRate(threshold) => {
                if metrics.verification.total == 0 {
                    return false;
                }
                let rate = metrics.verification.failed as f64 / 
                          metrics.verification.total as f64;
                rate > *threshold
            }
            AlertCondition::ErrorCountExceeds(max) => {
                let errors = metrics.errors.divergences + 
                           metrics.errors.invariant_violations;
                errors > *max
            }
            AlertCondition::DivergenceDetected => {
                metrics.errors.divergences > 0
            }
            _ => false,
        }
    }
    
    fn trigger_alert(&self, node_id: NodeId, rule: &AlertRule) {
        let alert = Alert {
            rule_name: rule.name.clone(),
            severity: format!("{:?}", rule.severity),
            message: format!("Alert triggered: {}", rule.name),
            triggered_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            node_id,
        };
        
        println!(
            "[Alert] 🚨 {:?} - {} on node {}",
            rule.severity, rule.name, node_id
        );
        
        self.active_alerts.write().push(alert);
    }
    
    /// Get active alerts
    pub fn get_alerts(&self) -> Vec<Alert> {
        self.active_alerts.read().clone()
    }
    
    /// Clear alerts
    pub fn clear_alerts(&self) {
        self.active_alerts.write().clear();
    }
}

/// Distributed tracer - Tracks requests across nodes
pub struct DistributedTracer {
    /// Active traces
    traces: Arc<RwLock<HashMap<TraceId, Trace>>>,
}

pub type TraceId = u128;

#[derive(Debug, Clone)]
pub struct Trace {
    pub id: TraceId,
    pub started_at: Instant,
    pub spans: Vec<Span>,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub name: String,
    pub node_id: NodeId,
    pub started_at: Instant,
    pub duration: Option<Duration>,
    pub tags: HashMap<String, String>,
}

impl DistributedTracer {
    pub fn new() -> Self {
        DistributedTracer {
            traces: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Start a new trace
    pub fn start_trace(&self) -> TraceId {
        let id = rand::random();
        let trace = Trace {
            id,
            started_at: Instant::now(),
            spans: Vec::new(),
        };
        
        self.traces.write().insert(id, trace);
        id
    }
    
    /// Add span to trace
    pub fn add_span(&self, trace_id: TraceId, span: Span) {
        if let Some(trace) = self.traces.write().get_mut(&trace_id) {
            trace.spans.push(span);
        }
    }
    
    /// Get trace
    pub fn get_trace(&self, trace_id: TraceId) -> Option<Trace> {
        self.traces.read().get(&trace_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new(1);
        
        collector.record_verification(Duration::from_micros(100), true);
        collector.record_verification(Duration::from_micros(150), true);
        
        let metrics = collector.snapshot();
        assert_eq!(metrics.verification.total, 2);
        assert_eq!(metrics.verification.success, 2);
        assert!(metrics.verification.avg_time_us > 0.0);
    }
    
    #[test]
    fn test_alert_manager() {
        let mut mgr = AlertManager::new();
        mgr.add_default_rules();
        
        let mut metrics = Metrics::default();
        metrics.errors.divergences = 1;
        
        mgr.check_metrics(1, &metrics);
        
        let alerts = mgr.get_alerts();
        assert!(!alerts.is_empty());
    }
}
