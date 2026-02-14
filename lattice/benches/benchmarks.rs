/*!
 * Benchmarks - Performance validation
 * 
 * Measures:
 * - Verification overhead
 * - Consensus latency
 * - Throughput
 * - Scalability
 */

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use lattice::{State, Transition, VerifiedStateMachine};
use std::time::Duration;

fn benchmark_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("verification");
    
    // Benchmark single verification
    group.bench_function("single_write", |b| {
        let vsm = VerifiedStateMachine::new(true);
        let transition = Transition::Write {
            key: "test".to_string(),
            value: vec![0u8; 100],
        };
        
        b.iter(|| {
            vsm.execute(transition.clone()).unwrap();
        });
    });
    
    // Benchmark batch verification
    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let vsm = VerifiedStateMachine::new(true);
            
            b.iter(|| {
                for i in 0..size {
                    let transition = Transition::Write {
                        key: format!("k{}", i),
                        value: vec![i as u8; 100],
                    };
                    vsm.execute(transition).unwrap();
                }
            });
        });
    }
    
    group.finish();
}

fn benchmark_state_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashing");
    
    for size in [1, 10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut state = State::new();
            for i in 0..size {
                state.data.insert(format!("k{}", i), vec![0u8; 100]);
            }
            
            b.iter(|| {
                state.hash();
            });
        });
    }
    
    group.finish();
}

fn benchmark_transition_application(c: &mut Criterion) {
    let mut group = c.benchmark_group("transitions");
    
    group.bench_function("write", |b| {
        let state = State::new();
        let transition = Transition::Write {
            key: "test".to_string(),
            value: vec![0u8; 100],
        };
        
        b.iter(|| {
            transition.apply(state.clone());
        });
    });
    
    group.bench_function("delete", |b| {
        let mut state = State::new();
        state.data.insert("test".to_string(), vec![0u8; 100]);
        let transition = Transition::Delete {
            key: "test".to_string(),
        };
        
        b.iter(|| {
            transition.apply(state.clone());
        });
    });
    
    group.finish();
}

fn benchmark_wal_write(c: &mut Criterion) {
    use lattice::storage::{WAL, WALConfig, WALEntry};
    use tempfile::TempDir;
    
    let mut group = c.benchmark_group("wal");
    
    group.bench_function("append", |b| {
        let temp_dir = TempDir::new().unwrap();
        let mut wal = WAL::create(temp_dir.path(), WALConfig {
            sync_on_write: false, // For benchmark
            ..Default::default()
        }).unwrap();
        
        let entry = WALEntry::LogEntry {
            index: 1,
            term: 1,
            transition: Transition::Write {
                key: "test".to_string(),
                value: vec![0u8; 100],
            },
        };
        
        b.iter(|| {
            wal.append(entry.clone()).unwrap();
        });
    });
    
    group.bench_function("append_with_sync", |b| {
        let temp_dir = TempDir::new().unwrap();
        let mut wal = WAL::create(temp_dir.path(), WALConfig {
            sync_on_write: true,
            ..Default::default()
        }).unwrap();
        
        let entry = WALEntry::LogEntry {
            index: 1,
            term: 1,
            transition: Transition::Write {
                key: "test".to_string(),
                value: vec![0u8; 100],
            },
        };
        
        b.iter(|| {
            wal.append(entry.clone()).unwrap();
        });
    });
    
    group.finish();
}

fn benchmark_invariant_checking(c: &mut Criterion) {
    use lattice::invariants::{InvariantChecker, SafetyInvariants};
    
    let mut group = c.benchmark_group("invariants");
    
    group.bench_function("check_all", |b| {
        let mut checker = InvariantChecker::new();
        checker.add_invariant(SafetyInvariants::unique_members());
        checker.add_invariant(SafetyInvariants::monotonic_clock());
        
        let state = State::new();
        
        b.iter(|| {
            checker.check_all(&state).unwrap();
        });
    });
    
    group.finish();
}

fn benchmark_metrics_recording(c: &mut Criterion) {
    use lattice::monitoring::MetricsCollector;
    
    let mut group = c.benchmark_group("metrics");
    
    group.bench_function("record_verification", |b| {
        let collector = MetricsCollector::new(1);
        
        b.iter(|| {
            collector.record_verification(Duration::from_micros(100), true);
        });
    });
    
    group.finish();
}

fn benchmark_batching(c: &mut Criterion) {
    use lattice::optimization::BatchProcessor;
    
    let mut group = c.benchmark_group("batching");
    
    group.bench_function("add_to_batch", |b| {
        let mut batcher = BatchProcessor::new(100, Duration::from_secs(1));
        let transition = Transition::Write {
            key: "test".to_string(),
            value: vec![0u8; 100],
        };
        
        b.iter(|| {
            batcher.add(transition.clone());
        });
    });
    
    group.finish();
}

fn benchmark_caching(c: &mut Criterion) {
    use lattice::optimization::StateCache;
    use lattice::StateHash;
    
    let mut group = c.benchmark_group("caching");
    
    group.bench_function("cache_put", |b| {
        let cache = StateCache::new(1000);
        let hash = StateHash([0; 32]);
        
        b.iter(|| {
            cache.put(1, hash);
        });
    });
    
    group.bench_function("cache_get", |b| {
        let cache = StateCache::new(1000);
        let hash = StateHash([0; 32]);
        cache.put(1, hash);
        
        b.iter(|| {
            cache.get(1);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_verification,
    benchmark_state_hashing,
    benchmark_transition_application,
    benchmark_wal_write,
    benchmark_invariant_checking,
    benchmark_metrics_recording,
    benchmark_batching,
    benchmark_caching
);

criterion_main!(benches);
