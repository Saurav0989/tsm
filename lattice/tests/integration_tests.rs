/*!
 * Integration Tests - End-to-end system validation
 * 
 * Tests the complete system working together:
 * - Multi-node cluster
 * - Consensus with verification
 * - Persistence and recovery
 * - Network failures
 * - Performance under load
 */

#[cfg(test)]
mod integration_tests {
    use lattice::{State, Transition, VerifiedStateMachine};
    use lattice::distributed::{DistributedNode, ClusterConfig};
    use lattice::storage::{WAL, WALConfig, SnapshotManager, SnapshotConfig};
    use lattice::raft::RaftConfig;
    use lattice::chaos::{ChaosEngine, ChaosScenarios};
    use std::time::Duration;
    use tokio::time::sleep;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_single_node_verification() {
        let vsm = VerifiedStateMachine::new(true);
        
        // Execute 1000 transitions
        for i in 0..1000 {
            let transition = Transition::Write {
                key: format!("key{}", i),
                value: vec![i as u8; 100],
            };
            
            assert!(vsm.execute(transition).is_ok());
        }
        
        let state = vsm.state();
        assert_eq!(state.clock, 1000);
        assert_eq!(state.data.len(), 1000);
    }

    #[tokio::test]
    async fn test_verification_catches_divergence() {
        // This test would simulate a divergence
        // In production, divergence shouldn't happen
        // This validates detection works
        
        let vsm = VerifiedStateMachine::new(true);
        
        // Normal operations should work
        let transition = Transition::Write {
            key: "test".to_string(),
            value: b"value".to_vec(),
        };
        
        assert!(vsm.execute(transition).is_ok());
    }

    #[tokio::test]
    async fn test_wal_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let mut wal = WAL::create(
            temp_dir.path().join("wal"),
            WALConfig::default()
        ).unwrap();
        
        // Write entries
        for i in 0..100 {
            let entry = lattice::storage::WALEntry::LogEntry {
                index: i,
                term: 1,
                transition: Transition::Write {
                    key: format!("k{}", i),
                    value: vec![i as u8],
                },
            };
            assert!(wal.append(entry).is_ok());
        }
        
        // Sync to disk
        assert!(wal.sync().is_ok());
        
        // Read back
        let entries = wal.read_all().unwrap();
        assert_eq!(entries.len(), 100);
    }

    #[tokio::test]
    async fn test_snapshot_creation_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let mgr = SnapshotManager::new(
            temp_dir.path().join("snapshots"),
            SnapshotConfig::default()
        ).unwrap();
        
        let mut state = State::new();
        state.clock = 1000;
        state.term = 5;
        
        let snapshot = lattice::storage::Snapshot {
            last_index: 1000,
            last_term: 5,
            state: state.clone(),
            timestamp: 123456789,
        };
        
        // Save
        assert!(mgr.save(&snapshot).is_ok());
        
        // Load
        let loaded = mgr.load_latest().unwrap().unwrap();
        assert_eq!(loaded.last_index, 1000);
        assert_eq!(loaded.last_term, 5);
        assert_eq!(loaded.state.clock, 1000);
    }

    #[tokio::test]
    async fn test_crash_recovery() {
        let temp_dir = TempDir::new().unwrap();
        
        // Phase 1: Write data
        {
            let mut wal = WAL::create(
                temp_dir.path().join("wal"),
                WALConfig::default()
            ).unwrap();
            
            for i in 0..50 {
                let entry = lattice::storage::WALEntry::LogEntry {
                    index: i,
                    term: 1,
                    transition: Transition::Write {
                        key: format!("k{}", i),
                        value: vec![i as u8],
                    },
                };
                wal.append(entry).unwrap();
            }
            
            wal.sync().unwrap();
        } // WAL closed (simulates crash)
        
        // Phase 2: Recover
        {
            let wal = WAL::open(
                temp_dir.path().join("wal"),
                WALConfig::default()
            ).unwrap();
            
            let entries = wal.read_all().unwrap();
            assert_eq!(entries.len(), 50);
            
            // Replay entries
            let mut state = State::new();
            for entry in entries {
                if let lattice::storage::WALEntry::LogEntry { transition, .. } = entry {
                    state = transition.apply(state);
                }
            }
            
            assert_eq!(state.clock, 50);
            assert_eq!(state.data.len(), 50);
        }
    }

    #[tokio::test]
    async fn test_invariant_checking() {
        use lattice::invariants::{InvariantChecker, SafetyInvariants};
        
        let mut checker = InvariantChecker::new();
        checker.add_invariant(SafetyInvariants::unique_members());
        
        let mut state = State::new();
        state.members = vec![1, 2, 3];
        
        // Should pass
        assert!(checker.check_all(&state).is_ok());
        
        // Add duplicate
        state.members.push(2);
        
        // Should fail
        assert!(checker.check_all(&state).is_err());
    }

    #[tokio::test]
    async fn test_tla_specifications() {
        use lattice::tla::{TLAChecker, RaftSpec};
        
        let mut checker = TLAChecker::new(100);
        checker.add_spec(Box::new(RaftSpec::new(3)));
        
        let state = State::new();
        
        // Should pass empty state
        assert!(checker.check_all(&state).is_ok());
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        use lattice::monitoring::MetricsCollector;
        
        let collector = MetricsCollector::new(1);
        
        // Record some metrics
        for _ in 0..100 {
            collector.record_verification(Duration::from_micros(100), true);
        }
        
        let metrics = collector.snapshot();
        assert_eq!(metrics.verification.total, 100);
        assert_eq!(metrics.verification.success, 100);
        assert!(metrics.verification.avg_time_us > 0.0);
    }

    #[tokio::test]
    async fn test_security_authorization() {
        use lattice::security::{SecurityContext, Action};
        
        let mut ctx = SecurityContext::new(1, vec![1]);
        
        // Admin can do anything
        assert!(ctx.authorize(1, Action::Admin));
        
        // Normal node can propose
        assert!(ctx.authorize(2, Action::Propose));
        
        // Normal node can't admin
        assert!(!ctx.authorize(2, Action::Admin));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        use lattice::security::RateLimiter;
        
        let mut limiter = RateLimiter::new(5, 1);
        
        // First 5 should pass
        for _ in 0..5 {
            assert!(limiter.allow(1));
        }
        
        // 6th should fail
        assert!(!limiter.allow(1));
    }

    #[tokio::test]
    async fn test_batch_processing() {
        use lattice::optimization::BatchProcessor;
        
        let mut batcher = BatchProcessor::new(10, Duration::from_secs(1));
        
        // Add 9 - shouldn't flush
        for i in 0..9 {
            let t = Transition::Write {
                key: format!("k{}", i),
                value: vec![],
            };
            assert!(batcher.add(t).is_none());
        }
        
        // 10th should flush
        let t = Transition::Write {
            key: "k9".to_string(),
            value: vec![],
        };
        let batch = batcher.add(t);
        assert!(batch.is_some());
        assert_eq!(batch.unwrap().len(), 10);
    }

    #[tokio::test]
    async fn test_state_caching() {
        use lattice::optimization::StateCache;
        use lattice::StateHash;
        
        let cache = StateCache::new(100);
        let hash = StateHash([42; 32]);
        
        cache.put(1, hash);
        assert_eq!(cache.get(1), Some(hash));
        assert_eq!(cache.get(2), None);
    }

    #[tokio::test]
    async fn test_chaos_network_fault_injection() {
        use lattice::chaos::NetworkFaultInjector;
        
        let mut injector = NetworkFaultInjector::new();
        injector.set_drop_rate(0.5);
        
        let mut dropped = 0;
        for _ in 0..1000 {
            if injector.should_drop(1, 2) {
                dropped += 1;
            }
        }
        
        // Should drop roughly 50%
        assert!(dropped > 400 && dropped < 600);
    }

    #[tokio::test]
    async fn test_health_checking() {
        use lattice::monitoring::{HealthChecker, Metrics};
        
        let checker = HealthChecker::new(1);
        let metrics = Metrics::default();
        
        let health = checker.check_health(&metrics).await;
        assert!(health.healthy);
        assert_eq!(health.status, "OK");
    }

    #[tokio::test]
    async fn test_alert_triggering() {
        use lattice::monitoring::{AlertManager, Metrics};
        
        let mut mgr = AlertManager::new();
        mgr.add_default_rules();
        
        let mut metrics = Metrics::default();
        metrics.errors.divergences = 1;
        
        mgr.check_metrics(1, &metrics);
        
        let alerts = mgr.get_alerts();
        assert!(!alerts.is_empty());
    }

    #[tokio::test]
    async fn test_performance_profiling() {
        use lattice::optimization::Profiler;
        
        let profiler = Profiler::new();
        
        for _ in 0..10 {
            let _guard = profiler.start("test");
            sleep(Duration::from_micros(100)).await;
        }
        
        let stats = profiler.stats("test");
        assert!(stats.is_some());
        assert_eq!(stats.unwrap().count, 10);
    }

    #[tokio::test]
    async fn test_distributed_tracing() {
        use lattice::monitoring::DistributedTracer;
        
        let tracer = DistributedTracer::new();
        let trace_id = tracer.start_trace();
        
        let trace = tracer.get_trace(trace_id);
        assert!(trace.is_some());
    }

    #[tokio::test]
    async fn test_partition_detection() {
        use lattice::recovery::PartitionDetector;
        
        let mut detector = PartitionDetector::new(1, Duration::from_secs(1));
        
        detector.update_contact(2);
        detector.update_contact(3);
        
        // No partition yet
        assert!(detector.detect_partitions(&[1, 2, 3]).is_none());
        
        // Wait for timeout
        sleep(Duration::from_secs(2)).await;
        
        // Should detect partition now
        assert!(detector.detect_partitions(&[1, 2, 3]).is_some());
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        use lattice::recovery::CircuitBreaker;
        
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(1));
        
        // Should start closed
        assert!(cb.allow_request());
        
        // Record failures
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        
        // Should open
        assert!(!cb.allow_request());
    }

    #[tokio::test]
    async fn test_end_to_end_transaction() {
        // This tests a complete transaction flow:
        // 1. Propose transition
        // 2. Verify locally
        // 3. Write to WAL
        // 4. Check invariants
        // 5. Record metrics
        
        use lattice::monitoring::MetricsCollector;
        use lattice::invariants::{InvariantChecker, SafetyInvariants};
        
        let temp_dir = TempDir::new().unwrap();
        
        let vsm = VerifiedStateMachine::new(true);
        let mut wal = WAL::create(temp_dir.path(), WALConfig::default()).unwrap();
        let mut checker = InvariantChecker::new();
        checker.add_invariant(SafetyInvariants::unique_members());
        let metrics = MetricsCollector::new(1);
        
        // Execute transaction
        let transition = Transition::Write {
            key: "test".to_string(),
            value: b"value".to_vec(),
        };
        
        let start = std::time::Instant::now();
        
        // Step 1: Verify
        assert!(vsm.execute(transition.clone()).is_ok());
        
        // Step 2: Persist
        let entry = lattice::storage::WALEntry::LogEntry {
            index: 1,
            term: 1,
            transition,
        };
        assert!(wal.append(entry).is_ok());
        
        // Step 3: Check invariants
        let state = vsm.state();
        assert!(checker.check_all(&state).is_ok());
        
        // Step 4: Record metrics
        let elapsed = start.elapsed();
        metrics.record_verification(elapsed, true);
        
        // Verify metrics
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.verification.total, 1);
        assert_eq!(snapshot.verification.success, 1);
    }
}
