/*!
 * Recovery System - Fault tolerance and automatic recovery
 * 
 * Handles:
 * - Node crashes and restarts
 * - Network partitions
 * - Corruption detection and repair
 * - State reconstruction
 * - Automatic failover
 */

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

use crate::{State, Transition, NodeId};
use crate::storage::{WAL, Snapshot, SnapshotManager};
use crate::raft::RaftNode;

/// Recovery manager - Handles crash recovery and state reconstruction
pub struct RecoveryManager {
    /// Node ID
    node_id: NodeId,
    
    /// WAL for replay
    wal: WAL,
    
    /// Snapshot manager
    snapshots: SnapshotManager,
    
    /// Recovery state
    state: RecoveryState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryState {
    Normal,
    Recovering,
    Replaying,
    Repairing,
    Failed,
}

impl RecoveryManager {
    pub fn new(
        node_id: NodeId,
        wal: WAL,
        snapshots: SnapshotManager,
    ) -> Self {
        RecoveryManager {
            node_id,
            wal,
            snapshots,
            state: RecoveryState::Normal,
        }
    }
    
    /// Recover from crash
    pub fn recover(&mut self) -> Result<State, RecoveryError> {
        println!("[Recovery] Starting crash recovery for node {}", self.node_id);
        self.state = RecoveryState::Recovering;
        
        // Step 1: Load latest snapshot
        let mut state = match self.snapshots.load_latest()? {
            Some(snapshot) => {
                println!("[Recovery] Loaded snapshot at index {}", snapshot.last_index);
                snapshot.state
            }
            None => {
                println!("[Recovery] No snapshot found, starting from empty state");
                State::new()
            }
        };
        
        // Step 2: Replay WAL entries
        self.state = RecoveryState::Replaying;
        let entries = self.wal.read_all()?;
        
        println!("[Recovery] Replaying {} WAL entries", entries.len());
        
        for (i, entry) in entries.iter().enumerate() {
            match entry {
                crate::storage::WALEntry::LogEntry { transition, .. } => {
                    state = transition.apply(state);
                }
                crate::storage::WALEntry::Snapshot { state: snap_state, .. } => {
                    state = snap_state.clone();
                }
                _ => {} // Metadata entries don't affect state
            }
            
            if (i + 1) % 1000 == 0 {
                println!("[Recovery] Replayed {}/{} entries", i + 1, entries.len());
            }
        }
        
        println!("[Recovery] ✅ Recovery complete");
        self.state = RecoveryState::Normal;
        
        Ok(state)
    }
    
    /// Repair corrupted state
    pub fn repair(&mut self, corrupted_state: &State) -> Result<State, RecoveryError> {
        println!("[Recovery] Attempting to repair corrupted state");
        self.state = RecoveryState::Repairing;
        
        // Try to load from snapshot
        if let Some(snapshot) = self.snapshots.load_latest()? {
            println!("[Recovery] Using snapshot as repair source");
            self.state = RecoveryState::Normal;
            return Ok(snapshot.state);
        }
        
        // If no snapshot, try WAL replay from scratch
        println!("[Recovery] No snapshot available, replaying from WAL");
        self.recover()
    }
    
    /// Check if state is healthy
    pub fn health_check(&self, state: &State) -> HealthStatus {
        HealthStatus {
            node_id: self.node_id,
            recovery_state: self.state,
            clock: state.clock,
            term: state.term,
            members: state.members.len(),
            healthy: self.state == RecoveryState::Normal,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub node_id: NodeId,
    pub recovery_state: RecoveryState,
    pub clock: u64,
    pub term: u64,
    pub members: usize,
    pub healthy: bool,
}

#[derive(Debug)]
pub enum RecoveryError {
    WALError(std::io::Error),
    SnapshotError(std::io::Error),
    CorruptedData(String),
    UnrecoverableState,
}

impl From<std::io::Error> for RecoveryError {
    fn from(e: std::io::Error) -> Self {
        RecoveryError::WALError(e)
    }
}

impl std::fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecoveryError::WALError(e) => write!(f, "WAL error: {}", e),
            RecoveryError::SnapshotError(e) => write!(f, "Snapshot error: {}", e),
            RecoveryError::CorruptedData(msg) => write!(f, "Corrupted data: {}", msg),
            RecoveryError::UnrecoverableState => write!(f, "Unrecoverable state"),
        }
    }
}

impl std::error::Error for RecoveryError {}

/// Network partition detector and handler
pub struct PartitionDetector {
    /// Node ID
    node_id: NodeId,
    
    /// Last contact time with each peer
    last_contact: HashMap<NodeId, Instant>,
    
    /// Partition timeout
    timeout: Duration,
    
    /// Detected partitions
    partitions: Vec<Partition>,
}

#[derive(Debug, Clone)]
pub struct Partition {
    pub detected_at: Instant,
    pub unreachable_nodes: Vec<NodeId>,
    pub reachable_nodes: Vec<NodeId>,
}

impl PartitionDetector {
    pub fn new(node_id: NodeId, timeout: Duration) -> Self {
        PartitionDetector {
            node_id,
            last_contact: HashMap::new(),
            timeout,
            partitions: Vec::new(),
        }
    }
    
    /// Update last contact time with peer
    pub fn update_contact(&mut self, peer: NodeId) {
        self.last_contact.insert(peer, Instant::now());
    }
    
    /// Check for partitions
    pub fn detect_partitions(&mut self, all_peers: &[NodeId]) -> Option<Partition> {
        let now = Instant::now();
        let mut unreachable = Vec::new();
        let mut reachable = Vec::new();
        
        for &peer in all_peers {
            if peer == self.node_id {
                continue;
            }
            
            let last = self.last_contact.get(&peer);
            
            if let Some(last_time) = last {
                if now.duration_since(*last_time) > self.timeout {
                    unreachable.push(peer);
                } else {
                    reachable.push(peer);
                }
            } else {
                unreachable.push(peer);
            }
        }
        
        if !unreachable.is_empty() {
            let partition = Partition {
                detected_at: now,
                unreachable_nodes: unreachable.clone(),
                reachable_nodes: reachable.clone(),
            };
            
            println!(
                "[Partition] Detected partition: {} unreachable nodes",
                unreachable.len()
            );
            
            self.partitions.push(partition.clone());
            Some(partition)
        } else {
            None
        }
    }
    
    /// Check if we're in majority partition
    pub fn in_majority(&self, total_nodes: usize) -> bool {
        let reachable = self.last_contact.len() + 1; // +1 for self
        reachable > total_nodes / 2
    }
}

/// Automatic failover manager
pub struct FailoverManager {
    /// Primary node ID
    primary: Option<NodeId>,
    
    /// Backup nodes in priority order
    backups: Vec<NodeId>,
    
    /// Failover history
    history: Vec<FailoverEvent>,
}

#[derive(Debug, Clone)]
pub struct FailoverEvent {
    pub timestamp: Instant,
    pub from: Option<NodeId>,
    pub to: NodeId,
    pub reason: FailoverReason,
}

#[derive(Debug, Clone)]
pub enum FailoverReason {
    PrimaryUnreachable,
    HealthCheckFailed,
    ManualFailover,
    StateCorruption,
}

impl FailoverManager {
    pub fn new(backups: Vec<NodeId>) -> Self {
        FailoverManager {
            primary: None,
            backups,
            history: Vec::new(),
        }
    }
    
    /// Initiate failover
    pub fn failover(&mut self, reason: FailoverReason) -> Option<NodeId> {
        let old_primary = self.primary;
        
        // Select next backup
        if let Some(new_primary) = self.backups.first().copied() {
            self.primary = Some(new_primary);
            
            // Move old primary to end of backups if it exists
            if let Some(old) = old_primary {
                self.backups.retain(|&id| id != new_primary);
                self.backups.push(old);
            } else {
                self.backups.remove(0);
            }
            
            let event = FailoverEvent {
                timestamp: Instant::now(),
                from: old_primary,
                to: new_primary,
                reason,
            };
            
            println!(
                "[Failover] {:?} -> {} (reason: {:?})",
                old_primary, new_primary, event.reason
            );
            
            self.history.push(event);
            
            Some(new_primary)
        } else {
            println!("[Failover] ❌ No backup nodes available");
            None
        }
    }
    
    /// Get current primary
    pub fn current_primary(&self) -> Option<NodeId> {
        self.primary
    }
    
    /// Get failover history
    pub fn get_history(&self) -> &[FailoverEvent] {
        &self.history
    }
}

/// Circuit breaker pattern for fault isolation
pub struct CircuitBreaker {
    /// Current state
    state: CircuitState,
    
    /// Failure count
    failures: usize,
    
    /// Failure threshold
    threshold: usize,
    
    /// Timeout before retry
    timeout: Duration,
    
    /// Last failure time
    last_failure: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing if recovered
}

impl CircuitBreaker {
    pub fn new(threshold: usize, timeout: Duration) -> Self {
        CircuitBreaker {
            state: CircuitState::Closed,
            failures: 0,
            threshold,
            timeout,
            last_failure: None,
        }
    }
    
    /// Record success
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failures = 0;
            }
            CircuitState::HalfOpen => {
                // Recovered!
                println!("[Circuit] ✅ Recovered, closing circuit");
                self.state = CircuitState::Closed;
                self.failures = 0;
            }
            CircuitState::Open => {}
        }
    }
    
    /// Record failure
    pub fn record_failure(&mut self) {
        self.failures += 1;
        self.last_failure = Some(Instant::now());
        
        if self.failures >= self.threshold {
            println!("[Circuit] ❌ Threshold exceeded, opening circuit");
            self.state = CircuitState::Open;
        }
    }
    
    /// Check if request should be allowed
    pub fn allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout elapsed
                if let Some(last) = self.last_failure {
                    if Instant::now().duration_since(last) > self.timeout {
                        println!("[Circuit] Timeout elapsed, trying half-open");
                        self.state = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }
    
    /// Get current state
    pub fn state(&self) -> CircuitState {
        self.state
    }
}

/// Retry policy with exponential backoff
pub struct RetryPolicy {
    /// Initial delay
    initial_delay: Duration,
    
    /// Maximum delay
    max_delay: Duration,
    
    /// Backoff multiplier
    multiplier: f64,
    
    /// Maximum attempts
    max_attempts: usize,
}

impl RetryPolicy {
    pub fn exponential(initial_ms: u64, max_attempts: usize) -> Self {
        RetryPolicy {
            initial_delay: Duration::from_millis(initial_ms),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            max_attempts,
        }
    }
    
    /// Get delay for attempt number
    pub fn delay_for_attempt(&self, attempt: usize) -> Duration {
        if attempt >= self.max_attempts {
            return self.max_delay;
        }
        
        let delay_ms = self.initial_delay.as_millis() as f64
            * self.multiplier.powi(attempt as i32);
        
        let delay = Duration::from_millis(delay_ms as u64);
        
        if delay > self.max_delay {
            self.max_delay
        } else {
            delay
        }
    }
    
    /// Execute with retries
    pub async fn execute<F, T, E>(&self, mut f: F) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
    {
        for attempt in 0..self.max_attempts {
            match f() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt == self.max_attempts - 1 {
                        return Err(e);
                    }
                    
                    let delay = self.delay_for_attempt(attempt);
                    println!(
                        "[Retry] Attempt {} failed, retrying in {:?}",
                        attempt + 1,
                        delay
                    );
                    
                    tokio::time::sleep(delay).await;
                }
            }
        }
        
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_partition_detector() {
        let mut detector = PartitionDetector::new(1, Duration::from_secs(5));
        
        detector.update_contact(2);
        detector.update_contact(3);
        
        // No partition yet
        assert!(detector.detect_partitions(&[1, 2, 3]).is_none());
        
        // Simulate timeout
        std::thread::sleep(Duration::from_secs(6));
        
        // Now should detect partition
        assert!(detector.detect_partitions(&[1, 2, 3]).is_some());
    }
    
    #[test]
    fn test_circuit_breaker() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(1));
        
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());
        
        // Record failures
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        
        // Should open
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_request());
    }
    
    #[test]
    fn test_retry_policy() {
        let policy = RetryPolicy::exponential(100, 5);
        
        assert_eq!(policy.delay_for_attempt(0).as_millis(), 100);
        assert_eq!(policy.delay_for_attempt(1).as_millis(), 200);
        assert_eq!(policy.delay_for_attempt(2).as_millis(), 400);
    }
}
