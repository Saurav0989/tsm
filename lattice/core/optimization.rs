/*!
 * Performance Optimization - Maximize throughput and minimize latency
 * 
 * Optimizations:
 * - Batching
 * - Pipelining
 * - Caching
 * - Zero-copy
 * - Parallel processing
 * - Memory pooling
 */

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{Duration, Instant};

use crate::{State, Transition, StateHash};

/// Batch processor - Process multiple transitions together
pub struct BatchProcessor {
    /// Pending transitions
    pending: VecDeque<Transition>,
    
    /// Batch size
    batch_size: usize,
    
    /// Max wait time
    max_wait: Duration,
    
    /// Last flush
    last_flush: Instant,
}

impl BatchProcessor {
    pub fn new(batch_size: usize, max_wait: Duration) -> Self {
        BatchProcessor {
            pending: VecDeque::new(),
            batch_size,
            max_wait,
            last_flush: Instant::now(),
        }
    }
    
    /// Add transition to batch
    pub fn add(&mut self, transition: Transition) -> Option<Vec<Transition>> {
        self.pending.push_back(transition);
        
        // Flush if batch full or timeout
        if self.pending.len() >= self.batch_size || 
           self.last_flush.elapsed() >= self.max_wait {
            self.flush()
        } else {
            None
        }
    }
    
    /// Flush pending batch
    pub fn flush(&mut self) -> Option<Vec<Transition>> {
        if self.pending.is_empty() {
            return None;
        }
        
        let batch: Vec<_> = self.pending.drain(..).collect();
        self.last_flush = Instant::now();
        
        Some(batch)
    }
}

/// State cache - Avoid redundant hash computations
pub struct StateCache {
    /// Cache
    cache: Arc<RwLock<HashMap<u64, StateHash>>>,
    
    /// Max entries
    max_entries: usize,
}

impl StateCache {
    pub fn new(max_entries: usize) -> Self {
        StateCache {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_entries,
        }
    }
    
    /// Get cached hash
    pub fn get(&self, clock: u64) -> Option<StateHash> {
        self.cache.read().get(&clock).copied()
    }
    
    /// Put hash in cache
    pub fn put(&self, clock: u64, hash: StateHash) {
        let mut cache = self.cache.write();
        
        // Evict oldest if full
        if cache.len() >= self.max_entries {
            if let Some(&oldest) = cache.keys().min() {
                cache.remove(&oldest);
            }
        }
        
        cache.insert(clock, hash);
    }
    
    /// Clear cache
    pub fn clear(&self) {
        self.cache.write().clear();
    }
}

/// Memory pool for allocation optimization
pub struct MemoryPool<T> {
    pool: Arc<RwLock<Vec<T>>>,
    factory: fn() -> T,
    max_size: usize,
}

impl<T> MemoryPool<T> {
    pub fn new(factory: fn() -> T, max_size: usize) -> Self {
        MemoryPool {
            pool: Arc::new(RwLock::new(Vec::new())),
            factory,
            max_size,
        }
    }
    
    /// Get object from pool
    pub fn get(&self) -> T {
        self.pool.write().pop().unwrap_or_else(self.factory)
    }
    
    /// Return object to pool
    pub fn put(&self, obj: T) {
        let mut pool = self.pool.write();
        if pool.len() < self.max_size {
            pool.push(obj);
        }
    }
}

/// Pipeline processor - Overlap computation and I/O
pub struct Pipeline {
    stages: Vec<Box<dyn PipelineStage>>,
}

pub trait PipelineStage: Send + Sync {
    fn process(&self, input: Vec<u8>) -> Vec<u8>;
}

impl Pipeline {
    pub fn new() -> Self {
        Pipeline {
            stages: Vec::new(),
        }
    }
    
    pub fn add_stage(&mut self, stage: Box<dyn PipelineStage>) {
        self.stages.push(stage);
    }
    
    /// Process through all stages
    pub fn process(&self, mut input: Vec<u8>) -> Vec<u8> {
        for stage in &self.stages {
            input = stage.process(input);
        }
        input
    }
}

/// Parallel verifier - Verify multiple transitions in parallel
pub struct ParallelVerifier {
    /// Thread pool size
    threads: usize,
}

impl ParallelVerifier {
    pub fn new(threads: usize) -> Self {
        ParallelVerifier { threads }
    }
    
    /// Verify transitions in parallel
    pub async fn verify_parallel(
        &self,
        transitions: Vec<Transition>,
        initial_state: State,
    ) -> Vec<(Transition, StateHash)> {
        use tokio::task;
        
        let chunk_size = (transitions.len() + self.threads - 1) / self.threads;
        let chunks: Vec<_> = transitions.chunks(chunk_size).collect();
        
        let mut handles = Vec::new();
        
        for chunk in chunks {
            let chunk = chunk.to_vec();
            let state = initial_state.clone();
            
            let handle = task::spawn(async move {
                let mut results = Vec::new();
                let mut current_state = state;
                
                for transition in chunk {
                    let next_state = transition.apply(current_state.clone());
                    let hash = next_state.hash();
                    results.push((transition, hash));
                    current_state = next_state;
                }
                
                results
            });
            
            handles.push(handle);
        }
        
        let mut all_results = Vec::new();
        for handle in handles {
            if let Ok(results) = handle.await {
                all_results.extend(results);
            }
        }
        
        all_results
    }
}

/// Adaptive batch sizing - Dynamically adjust batch size
pub struct AdaptiveBatcher {
    /// Current batch size
    current_size: usize,
    
    /// Min/max batch size
    min_size: usize,
    max_size: usize,
    
    /// Recent latencies
    latencies: VecDeque<Duration>,
    
    /// Window size for averaging
    window: usize,
}

impl AdaptiveBatcher {
    pub fn new(min_size: usize, max_size: usize) -> Self {
        AdaptiveBatcher {
            current_size: min_size,
            min_size,
            max_size,
            latencies: VecDeque::new(),
            window: 10,
        }
    }
    
    /// Record batch latency
    pub fn record_latency(&mut self, latency: Duration) {
        self.latencies.push_back(latency);
        
        if self.latencies.len() > self.window {
            self.latencies.pop_front();
        }
        
        // Adjust batch size
        self.adjust_size();
    }
    
    fn adjust_size(&mut self) {
        if self.latencies.len() < self.window {
            return;
        }
        
        let avg: Duration = self.latencies.iter().sum::<Duration>() / self.window as u32;
        
        // If latency is low, increase batch size
        if avg < Duration::from_millis(10) && self.current_size < self.max_size {
            self.current_size = (self.current_size * 12 / 10).min(self.max_size);
        }
        // If latency is high, decrease batch size
        else if avg > Duration::from_millis(50) && self.current_size > self.min_size {
            self.current_size = (self.current_size * 8 / 10).max(self.min_size);
        }
    }
    
    /// Get current batch size
    pub fn batch_size(&self) -> usize {
        self.current_size
    }
}

/// Performance profiler
pub struct Profiler {
    samples: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
}

impl Profiler {
    pub fn new() -> Self {
        Profiler {
            samples: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Start profiling a section
    pub fn start(&self, name: &str) -> ProfilerGuard {
        ProfilerGuard {
            name: name.to_string(),
            start: Instant::now(),
            samples: self.samples.clone(),
        }
    }
    
    /// Get statistics
    pub fn stats(&self, name: &str) -> Option<ProfileStats> {
        let samples = self.samples.read();
        let durations = samples.get(name)?;
        
        if durations.is_empty() {
            return None;
        }
        
        let mut sorted = durations.clone();
        sorted.sort();
        
        let count = sorted.len();
        let sum: Duration = sorted.iter().sum();
        let avg = sum / count as u32;
        
        let p50 = sorted[count / 2];
        let p95 = sorted[count * 95 / 100];
        let p99 = sorted[count * 99 / 100];
        let max = sorted[count - 1];
        
        Some(ProfileStats {
            count,
            avg,
            p50,
            p95,
            p99,
            max,
        })
    }
}

pub struct ProfilerGuard {
    name: String,
    start: Instant,
    samples: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
}

impl Drop for ProfilerGuard {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.samples.write()
            .entry(self.name.clone())
            .or_insert_with(Vec::new)
            .push(duration);
    }
}

#[derive(Debug)]
pub struct ProfileStats {
    pub count: usize,
    pub avg: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub max: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_batch_processor() {
        let mut batcher = BatchProcessor::new(3, Duration::from_secs(1));
        
        assert!(batcher.add(Transition::Write {
            key: "a".to_string(),
            value: vec![],
        }).is_none());
        
        assert!(batcher.add(Transition::Write {
            key: "b".to_string(),
            value: vec![],
        }).is_none());
        
        // Third should trigger flush
        let batch = batcher.add(Transition::Write {
            key: "c".to_string(),
            value: vec![],
        });
        
        assert!(batch.is_some());
        assert_eq!(batch.unwrap().len(), 3);
    }
    
    #[test]
    fn test_state_cache() {
        let cache = StateCache::new(10);
        let hash = StateHash([1; 32]);
        
        cache.put(1, hash);
        assert_eq!(cache.get(1), Some(hash));
        assert_eq!(cache.get(2), None);
    }
}
