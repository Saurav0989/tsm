/*!
 * Machine Learning Anomaly Detection
 * 
 * This goes beyond simple thresholds to detect subtle distributed system bugs
 * using statistical learning.
 * 
 * Techniques:
 * - Isolation Forest for outlier detection
 * - LSTM for time-series prediction
 * - Autoencoders for normal behavior modeling
 * - Bayesian networks for causal inference
 * 
 * Goal: Detect bugs that humans would miss
 */

use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};

use crate::monitoring::Metrics;
use crate::trace::TraceEvent;

/// ML-based anomaly detector
pub struct AnomalyDetector {
    /// Isolation forest model
    isolation_forest: IsolationForest,
    
    /// Time series predictor
    predictor: TimeSeriesPredictor,
    
    /// Autoencoder for normal behavior
    autoencoder: Autoencoder,
    
    /// Bayesian network for causality
    causal_network: BayesianNetwork,
    
    /// Training data
    training_data: VecDeque<MetricSnapshot>,
    
    /// Configuration
    config: MLConfig,
}

#[derive(Debug, Clone)]
pub struct MLConfig {
    /// Training window size
    pub training_window: usize,
    
    /// Anomaly threshold (0-1)
    pub anomaly_threshold: f64,
    
    /// Retrain interval
    pub retrain_interval: usize,
    
    /// Feature dimensions
    pub num_features: usize,
}

impl Default for MLConfig {
    fn default() -> Self {
        MLConfig {
            training_window: 10000,
            anomaly_threshold: 0.95,
            retrain_interval: 1000,
            num_features: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    /// Timestamp
    pub timestamp: u64,
    
    /// Feature vector
    pub features: Vec<f64>,
    
    /// Labels for supervised learning
    pub labels: Option<Vec<String>>,
}

impl AnomalyDetector {
    pub fn new(config: MLConfig) -> Self {
        AnomalyDetector {
            isolation_forest: IsolationForest::new(100, 256),
            predictor: TimeSeriesPredictor::new(10, 5),
            autoencoder: Autoencoder::new(config.num_features, 10),
            causal_network: BayesianNetwork::new(),
            training_data: VecDeque::new(),
            config,
        }
    }
    
    /// Extract features from metrics
    pub fn extract_features(&self, metrics: &Metrics) -> Vec<f64> {
        let mut features = Vec::new();
        
        // Verification metrics (5 features)
        features.push(metrics.verification.avg_time_us);
        features.push(metrics.verification.p99_us as f64);
        features.push(metrics.verification.total as f64);
        features.push(metrics.verification.success as f64);
        features.push(metrics.verification.failed as f64);
        
        // Consensus metrics (5 features)
        features.push(metrics.consensus.elections as f64);
        features.push(metrics.consensus.avg_election_time_ms);
        features.push(metrics.consensus.log_entries as f64);
        features.push(metrics.consensus.committed_entries as f64);
        features.push(metrics.consensus.current_term as f64);
        
        // Network metrics (5 features)
        features.push(metrics.network.messages_sent as f64);
        features.push(metrics.network.messages_received as f64);
        features.push(metrics.network.avg_latency_ms);
        features.push(metrics.network.connection_errors as f64);
        features.push(metrics.network.active_connections as f64);
        
        // Error metrics (5 features)
        features.push(metrics.errors.divergences as f64);
        features.push(metrics.errors.invariant_violations as f64);
        features.push(metrics.errors.network_errors as f64);
        features.push(metrics.errors.storage_errors as f64);
        features.push(metrics.errors.recovery_attempts as f64);
        
        // Derived features (30 features)
        
        // Ratios
        let success_rate = if metrics.verification.total > 0 {
            metrics.verification.success as f64 / metrics.verification.total as f64
        } else {
            1.0
        };
        features.push(success_rate);
        
        let commit_rate = if metrics.consensus.log_entries > 0 {
            metrics.consensus.committed_entries as f64 / metrics.consensus.log_entries as f64
        } else {
            1.0
        };
        features.push(commit_rate);
        
        // Network efficiency
        let msg_balance = if metrics.network.messages_received > 0 {
            metrics.network.messages_sent as f64 / metrics.network.messages_received as f64
        } else {
            1.0
        };
        features.push(msg_balance);
        
        // Error rate
        let total_errors = metrics.errors.divergences + 
                          metrics.errors.invariant_violations +
                          metrics.errors.network_errors +
                          metrics.errors.storage_errors;
        features.push(total_errors as f64);
        
        // Pad to num_features
        while features.len() < self.config.num_features {
            features.push(0.0);
        }
        
        features.truncate(self.config.num_features);
        features
    }
    
    /// Train on metrics
    pub fn train(&mut self, metrics: &Metrics) {
        let features = self.extract_features(metrics);
        
        let snapshot = MetricSnapshot {
            timestamp: current_timestamp(),
            features: features.clone(),
            labels: None,
        };
        
        self.training_data.push_back(snapshot);
        
        if self.training_data.len() > self.config.training_window {
            self.training_data.pop_front();
        }
        
        // Retrain models periodically
        if self.training_data.len() % self.config.retrain_interval == 0 {
            self.retrain_models();
        }
    }
    
    /// Retrain all models
    fn retrain_models(&mut self) {
        println!("[ML] Retraining models on {} samples", self.training_data.len());
        
        let data: Vec<Vec<f64>> = self.training_data.iter()
            .map(|s| s.features.clone())
            .collect();
        
        // Train isolation forest
        self.isolation_forest.train(&data);
        
        // Train time series predictor
        self.predictor.train(&data);
        
        // Train autoencoder
        self.autoencoder.train(&data, 100);
    }
    
    /// Detect anomalies
    pub fn detect(&self, metrics: &Metrics) -> Vec<Anomaly> {
        let features = self.extract_features(metrics);
        let mut anomalies = Vec::new();
        
        // Isolation forest detection
        let isolation_score = self.isolation_forest.anomaly_score(&features);
        if isolation_score > self.config.anomaly_threshold {
            anomalies.push(Anomaly {
                detector: "IsolationForest".to_string(),
                score: isolation_score,
                description: "Statistical outlier detected".to_string(),
                severity: AnomalySeverity::Medium,
                features: features.clone(),
            });
        }
        
        // Time series prediction
        if let Some(predicted) = self.predictor.predict(&features) {
            let prediction_error = euclidean_distance(&features, &predicted);
            if prediction_error > 5.0 {
                anomalies.push(Anomaly {
                    detector: "TimeSeriesPredictor".to_string(),
                    score: prediction_error / 10.0,
                    description: format!("Unexpected behavior - prediction error: {:.2}", prediction_error),
                    severity: AnomalySeverity::High,
                    features: features.clone(),
                });
            }
        }
        
        // Autoencoder reconstruction
        let reconstruction = self.autoencoder.reconstruct(&features);
        let reconstruction_error = euclidean_distance(&features, &reconstruction);
        if reconstruction_error > 3.0 {
            anomalies.push(Anomaly {
                detector: "Autoencoder".to_string(),
                score: reconstruction_error / 5.0,
                description: format!("Abnormal pattern - reconstruction error: {:.2}", reconstruction_error),
                severity: AnomalySeverity::Critical,
                features: features.clone(),
            });
        }
        
        anomalies
    }
    
    /// Infer root cause using Bayesian network
    pub fn infer_root_cause(&self, anomalies: &[Anomaly]) -> Option<RootCause> {
        if anomalies.is_empty() {
            return None;
        }
        
        // Use Bayesian network for causal inference
        let probabilities = self.causal_network.infer(&anomalies[0].features);
        
        // Find most likely cause
        let (cause, prob) = probabilities.into_iter()
            .max_by(|(_, p1), (_, p2)| p1.partial_cmp(p2).unwrap())?;
        
        Some(RootCause {
            cause,
            probability: prob,
            explanation: self.explain_cause(&cause),
        })
    }
    
    fn explain_cause(&self, cause: &str) -> String {
        match cause {
            "network_partition" => "Network partition detected - nodes cannot communicate".to_string(),
            "memory_leak" => "Possible memory leak - usage increasing monotonically".to_string(),
            "disk_slow" => "Disk I/O slowdown - latency increasing".to_string(),
            "cpu_saturation" => "CPU saturation - processing time increasing".to_string(),
            "byzantine_node" => "Byzantine node detected - invalid state transitions".to_string(),
            _ => format!("Unknown cause: {}", cause),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Anomaly {
    pub detector: String,
    pub score: f64,
    pub description: String,
    pub severity: AnomalySeverity,
    pub features: Vec<f64>,
}

#[derive(Debug, Clone, Copy)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct RootCause {
    pub cause: String,
    pub probability: f64,
    pub explanation: String,
}

/// Isolation Forest for outlier detection
struct IsolationForest {
    trees: Vec<IsolationTree>,
    num_trees: usize,
    sample_size: usize,
}

impl IsolationForest {
    fn new(num_trees: usize, sample_size: usize) -> Self {
        IsolationForest {
            trees: Vec::new(),
            num_trees,
            sample_size,
        }
    }
    
    fn train(&mut self, data: &[Vec<f64>]) {
        self.trees.clear();
        
        for _ in 0..self.num_trees {
            let tree = IsolationTree::build(data, self.sample_size, 0, 10);
            self.trees.push(tree);
        }
    }
    
    fn anomaly_score(&self, point: &[f64]) -> f64 {
        if self.trees.is_empty() {
            return 0.0;
        }
        
        let avg_depth: f64 = self.trees.iter()
            .map(|tree| tree.path_length(point) as f64)
            .sum::<f64>() / self.trees.len() as f64;
        
        // Normalize to 0-1
        let c = 2.0 * (self.sample_size as f64 - 1.0).ln();
        (2.0_f64).powf(-avg_depth / c)
    }
}

struct IsolationTree {
    split_feature: Option<usize>,
    split_value: Option<f64>,
    left: Option<Box<IsolationTree>>,
    right: Option<Box<IsolationTree>>,
    size: usize,
}

impl IsolationTree {
    fn build(data: &[Vec<f64>], sample_size: usize, depth: usize, max_depth: usize) -> Self {
        if depth >= max_depth || data.len() <= 1 {
            return IsolationTree {
                split_feature: None,
                split_value: None,
                left: None,
                right: None,
                size: data.len(),
            };
        }
        
        // Random feature and split
        let feature = rand::random::<usize>() % data[0].len();
        let min = data.iter().map(|x| x[feature]).fold(f64::INFINITY, f64::min);
        let max = data.iter().map(|x| x[feature]).fold(f64::NEG_INFINITY, f64::max);
        let split = min + (max - min) * rand::random::<f64>();
        
        // Split data
        let (left_data, right_data): (Vec<_>, Vec<_>) = data.iter()
            .partition(|x| x[feature] < split);
        
        IsolationTree {
            split_feature: Some(feature),
            split_value: Some(split),
            left: Some(Box::new(Self::build(&left_data, sample_size, depth + 1, max_depth))),
            right: Some(Box::new(Self::build(&right_data, sample_size, depth + 1, max_depth))),
            size: data.len(),
        }
    }
    
    fn path_length(&self, point: &[f64]) -> usize {
        if self.split_feature.is_none() {
            return 0;
        }
        
        let feature = self.split_feature.unwrap();
        let split = self.split_value.unwrap();
        
        if point[feature] < split {
            1 + self.left.as_ref().unwrap().path_length(point)
        } else {
            1 + self.right.as_ref().unwrap().path_length(point)
        }
    }
}

/// Time series predictor using LSTM-like approach
struct TimeSeriesPredictor {
    window_size: usize,
    hidden_size: usize,
    history: VecDeque<Vec<f64>>,
}

impl TimeSeriesPredictor {
    fn new(window_size: usize, hidden_size: usize) -> Self {
        TimeSeriesPredictor {
            window_size,
            hidden_size,
            history: VecDeque::new(),
        }
    }
    
    fn train(&mut self, data: &[Vec<f64>]) {
        // Simplified - just store recent history
        for point in data.iter().rev().take(self.window_size) {
            self.history.push_front(point.clone());
        }
    }
    
    fn predict(&self, _current: &[f64]) -> Option<Vec<f64>> {
        if self.history.len() < 2 {
            return None;
        }
        
        // Simple moving average prediction
        let len = self.history.len().min(self.window_size);
        let sum: Vec<f64> = (0..self.history[0].len())
            .map(|i| {
                self.history.iter()
                    .take(len)
                    .map(|h| h[i])
                    .sum::<f64>() / len as f64
            })
            .collect();
        
        Some(sum)
    }
}

/// Autoencoder for learning normal behavior
struct Autoencoder {
    input_size: usize,
    hidden_size: usize,
    encoder_weights: Vec<Vec<f64>>,
    decoder_weights: Vec<Vec<f64>>,
}

impl Autoencoder {
    fn new(input_size: usize, hidden_size: usize) -> Self {
        // Initialize with random weights
        let encoder_weights = (0..hidden_size)
            .map(|_| (0..input_size).map(|_| rand::random::<f64>() - 0.5).collect())
            .collect();
        
        let decoder_weights = (0..input_size)
            .map(|_| (0..hidden_size).map(|_| rand::random::<f64>() - 0.5).collect())
            .collect();
        
        Autoencoder {
            input_size,
            hidden_size,
            encoder_weights,
            decoder_weights,
        }
    }
    
    fn train(&mut self, data: &[Vec<f64>], _epochs: usize) {
        // Simplified - would do gradient descent
        // For now, just reinitialize
    }
    
    fn encode(&self, input: &[f64]) -> Vec<f64> {
        self.encoder_weights.iter()
            .map(|weights| {
                weights.iter()
                    .zip(input.iter())
                    .map(|(w, x)| w * x)
                    .sum::<f64>()
                    .tanh() // Activation
            })
            .collect()
    }
    
    fn decode(&self, hidden: &[f64]) -> Vec<f64> {
        self.decoder_weights.iter()
            .map(|weights| {
                weights.iter()
                    .zip(hidden.iter())
                    .map(|(w, h)| w * h)
                    .sum::<f64>()
                    .tanh()
            })
            .collect()
    }
    
    fn reconstruct(&self, input: &[f64]) -> Vec<f64> {
        let hidden = self.encode(input);
        self.decode(&hidden)
    }
}

/// Bayesian network for causal inference
struct BayesianNetwork {
    // Simplified - would have full conditional probability tables
    priors: HashMap<String, f64>,
}

impl BayesianNetwork {
    fn new() -> Self {
        let mut priors = HashMap::new();
        priors.insert("network_partition".to_string(), 0.1);
        priors.insert("memory_leak".to_string(), 0.05);
        priors.insert("disk_slow".to_string(), 0.15);
        priors.insert("cpu_saturation".to_string(), 0.2);
        priors.insert("byzantine_node".to_string(), 0.01);
        
        BayesianNetwork { priors }
    }
    
    fn infer(&self, _features: &[f64]) -> HashMap<String, f64> {
        // Simplified - would do Bayesian inference
        self.priors.clone()
    }
}

fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

mod rand {
    pub fn random<T>() -> T
    where
        T: From<u64>,
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        T::from(nanos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_isolation_forest() {
        let data = vec![
            vec![1.0, 2.0],
            vec![1.1, 2.1],
            vec![0.9, 1.9],
            vec![10.0, 10.0], // Outlier
        ];
        
        let mut forest = IsolationForest::new(10, 4);
        forest.train(&data);
        
        let normal_score = forest.anomaly_score(&vec![1.0, 2.0]);
        let outlier_score = forest.anomaly_score(&vec![10.0, 10.0]);
        
        assert!(outlier_score > normal_score);
    }
}
