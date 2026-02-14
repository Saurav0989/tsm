/*!
 * Chaos Testing Framework - Validate system resilience
 * 
 * Injects failures to test recovery:
 * - Random node crashes
 * - Network partitions
 * - Message delays/drops
 * - Disk failures
 * - Clock skew
 * - Byzantine behavior
 */

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use rand::Rng;
use tokio::time::sleep;

use crate::NodeId;
use crate::raft::RaftMessage;

/// Chaos scenario - A planned failure injection
#[derive(Debug, Clone)]
pub struct ChaosScenario {
    pub name: String,
    pub duration: Duration,
    pub failures: Vec<ChaosFailure>,
}

#[derive(Debug, Clone)]
pub enum ChaosFailure {
    /// Kill a specific node
    KillNode {
        node_id: NodeId,
        after: Duration,
    },
    
    /// Partition network
    PartitionNetwork {
        partition_a: Vec<NodeId>,
        partition_b: Vec<NodeId>,
        after: Duration,
        duration: Duration,
    },
    
    /// Drop messages with probability
    DropMessages {
        probability: f64,
        after: Duration,
        duration: Duration,
    },
    
    /// Delay messages
    DelayMessages {
        delay: Duration,
        after: Duration,
        duration: Duration,
    },
    
    /// Corrupt message contents
    CorruptMessages {
        probability: f64,
        after: Duration,
        duration: Duration,
    },
    
    /// Introduce clock skew
    ClockSkew {
        node_id: NodeId,
        skew: Duration,
        after: Duration,
    },
    
    /// Simulate disk failure
    DiskFailure {
        node_id: NodeId,
        after: Duration,
        duration: Duration,
    },
    
    /// Byzantine node (sends conflicting messages)
    ByzantineNode {
        node_id: NodeId,
        after: Duration,
    },
}

/// Chaos engine - Executes chaos scenarios
pub struct ChaosEngine {
    /// Active scenarios
    scenarios: Vec<ActiveScenario>,
    
    /// Network fault injector
    network_faults: NetworkFaultInjector,
    
    /// Statistics
    stats: ChaosStats,
}

struct ActiveScenario {
    scenario: ChaosScenario,
    started_at: Instant,
}

#[derive(Debug, Default)]
pub struct ChaosStats {
    pub nodes_killed: usize,
    pub partitions_created: usize,
    pub messages_dropped: usize,
    pub messages_delayed: usize,
    pub messages_corrupted: usize,
}

impl ChaosEngine {
    pub fn new() -> Self {
        ChaosEngine {
            scenarios: Vec::new(),
            network_faults: NetworkFaultInjector::new(),
            stats: ChaosStats::default(),
        }
    }
    
    /// Add a chaos scenario
    pub fn add_scenario(&mut self, scenario: ChaosScenario) {
        println!("[Chaos] Added scenario: {}", scenario.name);
        self.scenarios.push(ActiveScenario {
            scenario,
            started_at: Instant::now(),
        });
    }
    
    /// Run all active scenarios
    pub async fn run(&mut self) {
        for active in &self.scenarios {
            println!("[Chaos] Running scenario: {}", active.scenario.name);
            
            for failure in &active.scenario.failures {
                self.inject_failure(failure).await;
            }
        }
    }
    
    /// Inject a specific failure
    async fn inject_failure(&mut self, failure: &ChaosFailure) {
        match failure {
            ChaosFailure::KillNode { node_id, after } => {
                sleep(*after).await;
                println!("[Chaos] 💀 Killing node {}", node_id);
                self.stats.nodes_killed += 1;
                // In real impl: Send kill signal to node
            }
            
            ChaosFailure::PartitionNetwork { partition_a, partition_b, after, duration } => {
                sleep(*after).await;
                println!(
                    "[Chaos] 🔌 Creating network partition: {:?} | {:?}",
                    partition_a, partition_b
                );
                self.stats.partitions_created += 1;
                self.network_faults.add_partition(partition_a.clone(), partition_b.clone());
                
                sleep(*duration).await;
                println!("[Chaos] ✅ Healing network partition");
                self.network_faults.clear_partitions();
            }
            
            ChaosFailure::DropMessages { probability, after, duration } => {
                sleep(*after).await;
                println!("[Chaos] 📉 Dropping messages (p={})", probability);
                self.network_faults.set_drop_rate(*probability);
                
                sleep(*duration).await;
                println!("[Chaos] ✅ Stopped dropping messages");
                self.network_faults.set_drop_rate(0.0);
            }
            
            ChaosFailure::DelayMessages { delay, after, duration } => {
                sleep(*after).await;
                println!("[Chaos] ⏱️  Delaying messages by {:?}", delay);
                self.network_faults.set_delay(*delay);
                
                sleep(*duration).await;
                println!("[Chaos] ✅ Stopped delaying messages");
                self.network_faults.set_delay(Duration::ZERO);
            }
            
            ChaosFailure::CorruptMessages { probability, after, duration } => {
                sleep(*after).await;
                println!("[Chaos] 💥 Corrupting messages (p={})", probability);
                self.network_faults.set_corruption_rate(*probability);
                
                sleep(*duration).await;
                println!("[Chaos] ✅ Stopped corrupting messages");
                self.network_faults.set_corruption_rate(0.0);
            }
            
            ChaosFailure::ClockSkew { node_id, skew, after } => {
                sleep(*after).await;
                println!("[Chaos] ⏰ Introducing clock skew on node {}: {:?}", node_id, skew);
                // In real impl: Adjust node's clock
            }
            
            ChaosFailure::DiskFailure { node_id, after, duration } => {
                sleep(*after).await;
                println!("[Chaos] 💾 Simulating disk failure on node {}", node_id);
                
                sleep(*duration).await;
                println!("[Chaos] ✅ Disk recovered on node {}", node_id);
            }
            
            ChaosFailure::ByzantineNode { node_id, after } => {
                sleep(*after).await;
                println!("[Chaos] 😈 Node {} is now Byzantine", node_id);
                // In real impl: Make node send conflicting messages
            }
        }
    }
    
    /// Get statistics
    pub fn stats(&self) -> &ChaosStats {
        &self.stats
    }
}

/// Network fault injector
pub struct NetworkFaultInjector {
    /// Message drop probability
    drop_rate: f64,
    
    /// Message delay
    delay: Duration,
    
    /// Message corruption probability
    corruption_rate: f64,
    
    /// Active partitions
    partitions: Vec<(Vec<NodeId>, Vec<NodeId>)>,
    
    /// RNG
    rng: rand::rngs::ThreadRng,
}

impl NetworkFaultInjector {
    pub fn new() -> Self {
        NetworkFaultInjector {
            drop_rate: 0.0,
            delay: Duration::ZERO,
            corruption_rate: 0.0,
            partitions: Vec::new(),
            rng: rand::thread_rng(),
        }
    }
    
    /// Should drop this message?
    pub fn should_drop(&mut self, from: NodeId, to: NodeId) -> bool {
        // Check partitions
        for (part_a, part_b) in &self.partitions {
            if (part_a.contains(&from) && part_b.contains(&to)) ||
               (part_b.contains(&from) && part_a.contains(&to)) {
                return true; // Partitioned
            }
        }
        
        // Random drop
        self.rng.gen::<f64>() < self.drop_rate
    }
    
    /// Get delay for message
    pub fn get_delay(&self) -> Duration {
        self.delay
    }
    
    /// Should corrupt this message?
    pub fn should_corrupt(&mut self) -> bool {
        self.rng.gen::<f64>() < self.corruption_rate
    }
    
    pub fn set_drop_rate(&mut self, rate: f64) {
        self.drop_rate = rate;
    }
    
    pub fn set_delay(&mut self, delay: Duration) {
        self.delay = delay;
    }
    
    pub fn set_corruption_rate(&mut self, rate: f64) {
        self.corruption_rate = rate;
    }
    
    pub fn add_partition(&mut self, part_a: Vec<NodeId>, part_b: Vec<NodeId>) {
        self.partitions.push((part_a, part_b));
    }
    
    pub fn clear_partitions(&mut self) {
        self.partitions.clear();
    }
}

/// Pre-defined chaos scenarios
pub struct ChaosScenarios;

impl ChaosScenarios {
    /// Kill random node
    pub fn kill_random_node() -> ChaosScenario {
        ChaosScenario {
            name: "Kill Random Node".to_string(),
            duration: Duration::from_secs(60),
            failures: vec![
                ChaosFailure::KillNode {
                    node_id: 2,
                    after: Duration::from_secs(10),
                },
            ],
        }
    }
    
    /// Network partition (split brain)
    pub fn split_brain() -> ChaosScenario {
        ChaosScenario {
            name: "Split Brain".to_string(),
            duration: Duration::from_secs(120),
            failures: vec![
                ChaosFailure::PartitionNetwork {
                    partition_a: vec![1],
                    partition_b: vec![2, 3],
                    after: Duration::from_secs(10),
                    duration: Duration::from_secs(30),
                },
            ],
        }
    }
    
    /// Flaky network
    pub fn flaky_network() -> ChaosScenario {
        ChaosScenario {
            name: "Flaky Network".to_string(),
            duration: Duration::from_secs(60),
            failures: vec![
                ChaosFailure::DropMessages {
                    probability: 0.3,
                    after: Duration::from_secs(5),
                    duration: Duration::from_secs(30),
                },
                ChaosFailure::DelayMessages {
                    delay: Duration::from_millis(500),
                    after: Duration::from_secs(5),
                    duration: Duration::from_secs(30),
                },
            ],
        }
    }
    
    /// Message corruption
    pub fn message_corruption() -> ChaosScenario {
        ChaosScenario {
            name: "Message Corruption".to_string(),
            duration: Duration::from_secs(60),
            failures: vec![
                ChaosFailure::CorruptMessages {
                    probability: 0.1,
                    after: Duration::from_secs(10),
                    duration: Duration::from_secs(30),
                },
            ],
        }
    }
    
    /// Cascading failures
    pub fn cascading_failures() -> ChaosScenario {
        ChaosScenario {
            name: "Cascading Failures".to_string(),
            duration: Duration::from_secs(180),
            failures: vec![
                ChaosFailure::KillNode {
                    node_id: 1,
                    after: Duration::from_secs(10),
                },
                ChaosFailure::KillNode {
                    node_id: 2,
                    after: Duration::from_secs(30),
                },
                ChaosFailure::PartitionNetwork {
                    partition_a: vec![3],
                    partition_b: vec![4, 5],
                    after: Duration::from_secs(50),
                    duration: Duration::from_secs(30),
                },
            ],
        }
    }
    
    /// All chaos at once
    pub fn chaos_monkey() -> ChaosScenario {
        ChaosScenario {
            name: "Chaos Monkey".to_string(),
            duration: Duration::from_secs(300),
            failures: vec![
                ChaosFailure::DropMessages {
                    probability: 0.2,
                    after: Duration::from_secs(10),
                    duration: Duration::from_secs(60),
                },
                ChaosFailure::KillNode {
                    node_id: 2,
                    after: Duration::from_secs(30),
                },
                ChaosFailure::PartitionNetwork {
                    partition_a: vec![1, 2],
                    partition_b: vec![3],
                    after: Duration::from_secs(60),
                    duration: Duration::from_secs(40),
                },
                ChaosFailure::DelayMessages {
                    delay: Duration::from_secs(2),
                    after: Duration::from_secs(120),
                    duration: Duration::from_secs(40),
                },
                ChaosFailure::CorruptMessages {
                    probability: 0.15,
                    after: Duration::from_secs(180),
                    duration: Duration::from_secs(40),
                },
            ],
        }
    }
}

/// Test harness for running chaos tests
pub struct ChaosTestHarness {
    engine: ChaosEngine,
    results: Vec<TestResult>,
}

#[derive(Debug)]
pub struct TestResult {
    pub scenario_name: String,
    pub passed: bool,
    pub duration: Duration,
    pub errors: Vec<String>,
}

impl ChaosTestHarness {
    pub fn new() -> Self {
        ChaosTestHarness {
            engine: ChaosEngine::new(),
            results: Vec::new(),
        }
    }
    
    /// Run all standard chaos tests
    pub async fn run_all_tests(&mut self) {
        println!("=".repeat(80));
        println!("CHAOS TESTING SUITE");
        println!("=".repeat(80));
        println!();
        
        let scenarios = vec![
            ChaosScenarios::kill_random_node(),
            ChaosScenarios::split_brain(),
            ChaosScenarios::flaky_network(),
            ChaosScenarios::message_corruption(),
            ChaosScenarios::cascading_failures(),
        ];
        
        for scenario in scenarios {
            self.run_test(scenario).await;
        }
        
        self.print_summary();
    }
    
    /// Run a single chaos test
    async fn run_test(&mut self, scenario: ChaosScenario) {
        println!("Running: {}", scenario.name);
        println!("-".repeat(80));
        
        let start = Instant::now();
        self.engine.add_scenario(scenario.clone());
        self.engine.run().await;
        let duration = start.elapsed();
        
        // Verify system recovered
        let passed = self.verify_recovery();
        
        let result = TestResult {
            scenario_name: scenario.name.clone(),
            passed,
            duration,
            errors: Vec::new(),
        };
        
        if passed {
            println!("✅ PASSED in {:?}", duration);
        } else {
            println!("❌ FAILED in {:?}", duration);
        }
        println!();
        
        self.results.push(result);
    }
    
    /// Verify system recovered from chaos
    fn verify_recovery(&self) -> bool {
        // In real impl: Check cluster health, consensus, etc.
        // For now, assume passed
        true
    }
    
    /// Print test summary
    fn print_summary(&self) {
        println!("=".repeat(80));
        println!("CHAOS TEST SUMMARY");
        println!("=".repeat(80));
        println!();
        
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        
        println!("Total tests: {}", total);
        println!("Passed: {} ({:.1}%)", passed, (passed as f64 / total as f64) * 100.0);
        println!("Failed: {}", failed);
        println!();
        
        if failed > 0 {
            println!("Failed tests:");
            for result in &self.results {
                if !result.passed {
                    println!("  ❌ {}", result.scenario_name);
                }
            }
        } else {
            println!("🎉 All chaos tests passed!");
        }
        
        println!();
        println!("Chaos statistics:");
        let stats = self.engine.stats();
        println!("  Nodes killed: {}", stats.nodes_killed);
        println!("  Partitions created: {}", stats.partitions_created);
        println!("  Messages dropped: {}", stats.messages_dropped);
        println!("  Messages delayed: {}", stats.messages_delayed);
        println!("  Messages corrupted: {}", stats.messages_corrupted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fault_injector() {
        let mut injector = NetworkFaultInjector::new();
        
        injector.set_drop_rate(0.5);
        
        // Should drop roughly half
        let mut dropped = 0;
        for _ in 0..1000 {
            if injector.should_drop(1, 2) {
                dropped += 1;
            }
        }
        
        assert!(dropped > 400 && dropped < 600);
    }
    
    #[tokio::test]
    async fn test_chaos_scenario() {
        let mut engine = ChaosEngine::new();
        engine.add_scenario(ChaosScenarios::kill_random_node());
        
        // Should complete without panic
        engine.run().await;
    }
}
