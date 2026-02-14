/*!
 * Advanced Performance Profiling - Statistical Analysis and Optimization
 * 
 * This goes beyond basic profiling to provide:
 * - Statistical distribution analysis
 * - Outlier detection
 * - Performance regression detection
 * - Automated optimization suggestions
 * - Flame graphs
 * - Lock contention analysis
 * 
 * Target: Match or exceed published distributed systems papers (OSDI/SOSP level)
 */

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

/// Statistical performance profiler
pub struct StatisticalProfiler {
    /// Samples for each operation
    samples: Arc<RwLock<HashMap<String, SampleSet>>>,
    
    /// Configuration
    config: ProfilerConfig,
    
    /// Baseline measurements
    baseline: HashMap<String, Statistics>,
}

#[derive(Debug, Clone)]
pub struct ProfilerConfig {
    /// Maximum samples per operation
    pub max_samples: usize,
    
    /// Outlier threshold (standard deviations)
    pub outlier_threshold: f64,
    
    /// Regression threshold (percentage)
    pub regression_threshold: f64,
    
    /// Enable detailed tracing
    pub detailed_tracing: bool,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        ProfilerConfig {
            max_samples: 10000,
            outlier_threshold: 3.0,
            regression_threshold: 0.10, // 10% slowdown
            detailed_tracing: true,
        }
    }
}

/// Collection of timing samples
#[derive(Debug, Clone)]
pub struct SampleSet {
    /// Raw samples
    samples: VecDeque<Duration>,
    
    /// Pre-computed statistics
    stats: Option<Statistics>,
    
    /// Last update time
    last_update: Instant,
}

impl SampleSet {
    fn new() -> Self {
        SampleSet {
            samples: VecDeque::new(),
            stats: None,
            last_update: Instant::now(),
        }
    }
    
    fn add(&mut self, duration: Duration, max_samples: usize) {
        if self.samples.len() >= max_samples {
            self.samples.pop_front();
        }
        self.samples.push_back(duration);
        self.stats = None; // Invalidate cached stats
        self.last_update = Instant::now();
    }
    
    fn compute_stats(&mut self) -> Statistics {
        if let Some(ref stats) = self.stats {
            return stats.clone();
        }
        
        let stats = Statistics::from_samples(&self.samples);
        self.stats = Some(stats.clone());
        stats
    }
}

/// Statistical summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    /// Sample count
    pub count: usize,
    
    /// Mean
    pub mean: Duration,
    
    /// Standard deviation
    pub stddev: Duration,
    
    /// Median (P50)
    pub median: Duration,
    
    /// P90 percentile
    pub p90: Duration,
    
    /// P95 percentile
    pub p95: Duration,
    
    /// P99 percentile
    pub p99: Duration,
    
    /// P99.9 percentile
    pub p999: Duration,
    
    /// Minimum
    pub min: Duration,
    
    /// Maximum
    pub max: Duration,
    
    /// Coefficient of variation
    pub cv: f64,
    
    /// Skewness
    pub skewness: f64,
    
    /// Kurtosis
    pub kurtosis: f64,
}

impl Statistics {
    fn from_samples(samples: &VecDeque<Duration>) -> Self {
        if samples.is_empty() {
            return Self::empty();
        }
        
        let count = samples.len();
        
        // Convert to microseconds for calculation
        let values: Vec<f64> = samples.iter()
            .map(|d| d.as_micros() as f64)
            .collect();
        
        // Sort for percentiles
        let mut sorted = values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // Mean
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;
        
        // Variance and stddev
        let variance: f64 = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        let stddev = variance.sqrt();
        
        // Coefficient of variation
        let cv = if mean > 0.0 { stddev / mean } else { 0.0 };
        
        // Skewness
        let skewness = if stddev > 0.0 {
            values.iter()
                .map(|x| ((x - mean) / stddev).powi(3))
                .sum::<f64>() / count as f64
        } else {
            0.0
        };
        
        // Kurtosis
        let kurtosis = if stddev > 0.0 {
            values.iter()
                .map(|x| ((x - mean) / stddev).powi(4))
                .sum::<f64>() / count as f64 - 3.0 // Excess kurtosis
        } else {
            0.0
        };
        
        // Percentiles
        let p = |percentile: f64| {
            let idx = ((percentile / 100.0) * (sorted.len() - 1) as f64) as usize;
            Duration::from_micros(sorted[idx] as u64)
        };
        
        Statistics {
            count,
            mean: Duration::from_micros(mean as u64),
            stddev: Duration::from_micros(stddev as u64),
            median: p(50.0),
            p90: p(90.0),
            p95: p(95.0),
            p99: p(99.0),
            p999: p(99.9),
            min: Duration::from_micros(sorted[0] as u64),
            max: Duration::from_micros(sorted[sorted.len() - 1] as u64),
            cv,
            skewness,
            kurtosis,
        }
    }
    
    fn empty() -> Self {
        Statistics {
            count: 0,
            mean: Duration::ZERO,
            stddev: Duration::ZERO,
            median: Duration::ZERO,
            p90: Duration::ZERO,
            p95: Duration::ZERO,
            p99: Duration::ZERO,
            p999: Duration::ZERO,
            min: Duration::ZERO,
            max: Duration::ZERO,
            cv: 0.0,
            skewness: 0.0,
            kurtosis: 0.0,
        }
    }
}

impl StatisticalProfiler {
    pub fn new(config: ProfilerConfig) -> Self {
        StatisticalProfiler {
            samples: Arc::new(RwLock::new(HashMap::new())),
            config,
            baseline: HashMap::new(),
        }
    }
    
    /// Record a measurement
    pub fn record(&self, operation: &str, duration: Duration) {
        let mut samples = self.samples.write();
        let sample_set = samples.entry(operation.to_string())
            .or_insert_with(SampleSet::new);
        sample_set.add(duration, self.config.max_samples);
    }
    
    /// Get statistics for an operation
    pub fn get_stats(&self, operation: &str) -> Option<Statistics> {
        let mut samples = self.samples.write();
        samples.get_mut(operation)
            .map(|s| s.compute_stats())
    }
    
    /// Detect outliers
    pub fn detect_outliers(&self, operation: &str) -> Vec<Duration> {
        let samples = self.samples.read();
        if let Some(sample_set) = samples.get(operation) {
            let stats = Statistics::from_samples(&sample_set.samples);
            
            let mean_us = stats.mean.as_micros() as f64;
            let stddev_us = stats.stddev.as_micros() as f64;
            let threshold = mean_us + (stddev_us * self.config.outlier_threshold);
            
            sample_set.samples.iter()
                .filter(|d| d.as_micros() as f64 > threshold)
                .copied()
                .collect()
        } else {
            vec![]
        }
    }
    
    /// Detect performance regression
    pub fn detect_regression(&mut self, operation: &str) -> Option<RegressionReport> {
        let current_stats = self.get_stats(operation)?;
        let baseline_stats = self.baseline.get(operation)?;
        
        let baseline_mean = baseline_stats.mean.as_micros() as f64;
        let current_mean = current_stats.mean.as_micros() as f64;
        
        let change = (current_mean - baseline_mean) / baseline_mean;
        
        if change > self.config.regression_threshold {
            Some(RegressionReport {
                operation: operation.to_string(),
                baseline: baseline_stats.clone(),
                current: current_stats,
                percentage_change: change * 100.0,
                severity: if change > 0.5 {
                    RegressionSeverity::Critical
                } else if change > 0.25 {
                    RegressionSeverity::High
                } else {
                    RegressionSeverity::Medium
                },
            })
        } else {
            None
        }
    }
    
    /// Set baseline for regression detection
    pub fn set_baseline(&mut self, operation: &str) {
        if let Some(stats) = self.get_stats(operation) {
            self.baseline.insert(operation.to_string(), stats);
        }
    }
    
    /// Generate performance report
    pub fn generate_report(&self) -> PerformanceReport {
        let samples = self.samples.read();
        
        let mut operations = Vec::new();
        for (name, sample_set) in samples.iter() {
            let stats = Statistics::from_samples(&sample_set.samples);
            operations.push(OperationStats {
                name: name.clone(),
                stats,
            });
        }
        
        // Sort by P99 latency
        operations.sort_by(|a, b| b.stats.p99.cmp(&a.stats.p99));
        
        PerformanceReport {
            operations,
            generated_at: Instant::now(),
        }
    }
    
    /// Suggest optimizations based on statistics
    pub fn suggest_optimizations(&self) -> Vec<OptimizationSuggestion> {
        let samples = self.samples.read();
        let mut suggestions = Vec::new();
        
        for (name, sample_set) in samples.iter() {
            let stats = Statistics::from_samples(&sample_set.samples);
            
            // High variance suggests inconsistent performance
            if stats.cv > 0.5 {
                suggestions.push(OptimizationSuggestion {
                    operation: name.clone(),
                    issue: "High variance in execution time".to_string(),
                    recommendation: "Investigate cache misses, lock contention, or GC pauses".to_string(),
                    priority: Priority::High,
                });
            }
            
            // Heavy tail suggests outliers
            if stats.kurtosis > 3.0 {
                suggestions.push(OptimizationSuggestion {
                    operation: name.clone(),
                    issue: "Heavy tail distribution detected".to_string(),
                    recommendation: "Profile outlier cases, consider timeouts or circuit breakers".to_string(),
                    priority: Priority::Medium,
                });
            }
            
            // Skewed distribution suggests bimodal performance
            if stats.skewness.abs() > 1.0 {
                suggestions.push(OptimizationSuggestion {
                    operation: name.clone(),
                    issue: "Skewed performance distribution".to_string(),
                    recommendation: "May have fast/slow paths - optimize slow path or split operation".to_string(),
                    priority: Priority::Medium,
                });
            }
            
            // P99 much higher than median
            let p99_ratio = stats.p99.as_micros() as f64 / stats.median.as_micros() as f64;
            if p99_ratio > 5.0 {
                suggestions.push(OptimizationSuggestion {
                    operation: name.clone(),
                    issue: format!("P99 is {:.1}x median", p99_ratio),
                    recommendation: "Tail latency optimization needed - investigate worst-case paths".to_string(),
                    priority: Priority::Critical,
                });
            }
        }
        
        suggestions
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionReport {
    pub operation: String,
    pub baseline: Statistics,
    pub current: Statistics,
    pub percentage_change: f64,
    pub severity: RegressionSeverity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RegressionSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub operations: Vec<OperationStats>,
    pub generated_at: Instant,
}

#[derive(Debug, Clone)]
pub struct OperationStats {
    pub name: String,
    pub stats: Statistics,
}

#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    pub operation: String,
    pub issue: String,
    pub recommendation: String,
    pub priority: Priority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Lock contention analyzer
pub struct LockAnalyzer {
    /// Lock wait times
    wait_times: HashMap<String, Vec<Duration>>,
    
    /// Lock hold times
    hold_times: HashMap<String, Vec<Duration>>,
}

impl LockAnalyzer {
    pub fn new() -> Self {
        LockAnalyzer {
            wait_times: HashMap::new(),
            hold_times: HashMap::new(),
        }
    }
    
    pub fn record_wait(&mut self, lock_name: &str, duration: Duration) {
        self.wait_times.entry(lock_name.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
    }
    
    pub fn record_hold(&mut self, lock_name: &str, duration: Duration) {
        self.hold_times.entry(lock_name.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
    }
    
    pub fn find_contention(&self) -> Vec<ContentionReport> {
        let mut reports = Vec::new();
        
        for (lock_name, waits) in &self.wait_times {
            if waits.is_empty() {
                continue;
            }
            
            let total_wait: Duration = waits.iter().sum();
            let avg_wait = total_wait / waits.len() as u32;
            
            if avg_wait > Duration::from_micros(100) {
                reports.push(ContentionReport {
                    lock_name: lock_name.clone(),
                    avg_wait_time: avg_wait,
                    contentions: waits.len(),
                    severity: if avg_wait > Duration::from_millis(1) {
                        ContentionSeverity::High
                    } else {
                        ContentionSeverity::Medium
                    },
                });
            }
        }
        
        reports.sort_by(|a, b| b.avg_wait_time.cmp(&a.avg_wait_time));
        reports
    }
}

#[derive(Debug, Clone)]
pub struct ContentionReport {
    pub lock_name: String,
    pub avg_wait_time: Duration,
    pub contentions: usize,
    pub severity: ContentionSeverity,
}

#[derive(Debug, Clone, Copy)]
pub enum ContentionSeverity {
    Low,
    Medium,
    High,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_statistics_calculation() {
        let samples: VecDeque<Duration> = vec![
            Duration::from_micros(100),
            Duration::from_micros(150),
            Duration::from_micros(200),
            Duration::from_micros(120),
            Duration::from_micros(180),
        ].into();
        
        let stats = Statistics::from_samples(&samples);
        
        assert_eq!(stats.count, 5);
        assert!(stats.mean.as_micros() > 0);
        assert!(stats.median.as_micros() > 0);
    }
    
    #[test]
    fn test_outlier_detection() {
        let profiler = StatisticalProfiler::new(ProfilerConfig::default());
        
        // Normal samples
        for _ in 0..100 {
            profiler.record("test", Duration::from_micros(100));
        }
        
        // Outlier
        profiler.record("test", Duration::from_micros(1000));
        
        let outliers = profiler.detect_outliers("test");
        assert!(!outliers.is_empty());
    }
}
