/*!
 * Integrated System - Complete distributed state machine
 * 
 * Combines:
 * - Runtime verification
 * - Raft consensus
 * - Network layer
 * - AI analysis
 * - Causal tracing
 */

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::Duration;

use crate::{
    State, Transition, VerifiedStateMachine, VerificationError,
    NodeId,
};
use crate::raft::{RaftNode, RaftConfig, RaftMessage};
use crate::network::{NetworkLayer, NetworkConfig, NetworkError};
use crate::trace::{CausalTrace, EventType};
use crate::analysis::{AnalysisEngine, BugAnalysis};
use crate::invariants::{InvariantChecker, SafetyInvariants};

/// DistributedNode - A complete node in the cluster
pub struct DistributedNode {
    /// Verified state machine
    state_machine: Arc<RwLock<VerifiedStateMachine>>,
    
    /// Raft consensus
    raft: Arc<RwLock<RaftNode>>,
    
    /// Network layer
    network: Arc<NetworkLayer>,
    
    /// Causal trace
    trace: Arc<RwLock<CausalTrace>>,
    
    /// Analysis engine
    analysis: Arc<AnalysisEngine>,
    
    /// Invariant checker
    invariants: Arc<RwLock<InvariantChecker>>,
    
    /// Node ID
    node_id: NodeId,
    
    /// Running flag
    running: Arc<RwLock<bool>>,
}

impl DistributedNode {
    /// Create a new distributed node
    pub fn new(
        node_id: NodeId,
        network_config: NetworkConfig,
        raft_config: RaftConfig,
    ) -> Self {
        let state_machine = Arc::new(RwLock::new(
            VerifiedStateMachine::new(true)
        ));
        
        let raft = Arc::new(RwLock::new(
            RaftNode::new(node_id, raft_config)
        ));
        
        let network = Arc::new(
            NetworkLayer::new(node_id, network_config)
        );
        
        let trace = Arc::new(RwLock::new(
            CausalTrace::new(node_id, 10000)
        ));
        
        let analysis = Arc::new(
            AnalysisEngine::new(false) // Set to true to enable LLM
        );
        
        let mut invariants = InvariantChecker::new();
        invariants.add_invariant(SafetyInvariants::unique_members());
        invariants.add_invariant(SafetyInvariants::monotonic_clock());
        let invariants = Arc::new(RwLock::new(invariants));
        
        DistributedNode {
            state_machine,
            raft,
            network,
            trace,
            analysis,
            invariants,
            node_id,
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Start the node
    pub async fn start(&self) -> Result<(), String> {
        println!("[Node {}] Starting...", self.node_id);
        
        // Start network layer
        self.network.start().await
            .map_err(|e| format!("Network error: {}", e))?;
        
        // Set running flag
        *self.running.write() = true;
        
        // Start main event loop
        self.run_event_loop().await;
        
        Ok(())
    }
    
    /// Main event loop
    async fn run_event_loop(&self) {
        let mut tick_interval = tokio::time::interval(Duration::from_millis(100));
        
        while *self.running.read() {
            tick_interval.tick().await;
            
            // Process Raft tick
            let messages = self.raft.write().tick();
            
            // Send outgoing messages
            for msg in messages {
                if let Err(e) = self.network.send(msg).await {
                    eprintln!("[Node {}] Send error: {}", self.node_id, e);
                }
            }
            
            // Process incoming messages
            let incoming = self.network.receive();
            for msg in incoming {
                // Record network receive in trace
                let event_id = self.trace.write().record(
                    EventType::NetworkReceive {
                        from: msg.from,
                        message_type: format!("{:?}", msg.payload),
                    },
                    vec![],
                );
                
                // Handle message
                let responses = self.raft.write().handle_message(msg);
                
                // Send responses
                for response in responses {
                    self.trace.write().record(
                        EventType::NetworkSend {
                            to: response.to,
                            message_type: format!("{:?}", response.payload),
                        },
                        vec![event_id],
                    );
                    
                    if let Err(e) = self.network.send(response).await {
                        eprintln!("[Node {}] Send error: {}", self.node_id, e);
                    }
                }
            }
        }
    }
    
    /// Propose a new transition
    pub async fn propose(&self, transition: Transition) -> Result<(), String> {
        // Record transition attempt
        let event_id = self.trace.write().record(
            EventType::Transition {
                transition: format!("{:?}", transition),
            },
            vec![],
        );
        
        // Get current state hash
        let state_before = self.state_machine.read().state_hash();
        
        // Execute with verification
        match self.state_machine.write().execute(transition.clone()) {
            Ok(()) => {
                let state_after = self.state_machine.read().state_hash();
                
                // Update trace with state hashes
                self.trace.write().set_state_hashes(
                    event_id,
                    state_before.0,
                    state_after.0,
                );
                
                // Check invariants
                let state = self.state_machine.read().state();
                if let Err(violation) = self.invariants.read().check_all(&state) {
                    eprintln!("[Node {}] Invariant violation: {}", self.node_id, violation);
                    return Err(format!("Invariant violation: {}", violation));
                }
                
                // Propose through Raft
                self.raft.write().propose(transition)
                    .map_err(|e| format!("Raft error: {}", e))?;
                
                Ok(())
            }
            Err(VerificationError::StateDivergence { expected, actual, transition }) => {
                eprintln!("[Node {}] VERIFICATION FAILURE", self.node_id);
                eprintln!("Expected: {:?}", expected);
                eprintln!("Actual: {:?}", actual);
                
                // CRITICAL: System has diverged
                // Trigger AI analysis
                self.handle_divergence(expected, actual, event_id).await;
                
                Err("Verification failed - system halted".to_string())
            }
        }
    }
    
    /// Handle state divergence with AI analysis
    async fn handle_divergence(
        &self,
        expected: crate::StateHash,
        actual: crate::StateHash,
        event_id: u64,
    ) {
        println!("[Node {}] ========================================", self.node_id);
        println!("[Node {}] CRITICAL: STATE DIVERGENCE DETECTED", self.node_id);
        println!("[Node {}] ========================================", self.node_id);
        
        // Get causal chain
        let causal_chain = self.trace.read().get_causal_chain(event_id);
        
        println!("[Node {}] Causal chain has {} events", self.node_id, causal_chain.len());
        
        // Get states
        let state_before = State::new(); // Would reconstruct from trace
        let state_after = self.state_machine.read().state();
        
        // Run AI analysis
        println!("[Node {}] Running AI analysis...", self.node_id);
        
        match self.analysis.analyze_divergence(
            expected,
            actual,
            causal_chain,
            &state_before,
            &state_after,
        ).await {
            Ok(analysis) => {
                self.print_analysis(&analysis);
                
                // Export trace for debugging
                let trace = self.trace.read().export();
                println!("[Node {}] Trace exported: {} events", self.node_id, trace.len());
            }
            Err(e) => {
                eprintln!("[Node {}] Analysis failed: {}", self.node_id, e);
            }
        }
        
        // HALT THE SYSTEM
        println!("[Node {}] HALTING - DO NOT PERSIST STATE", self.node_id);
        *self.running.write() = false;
    }
    
    /// Print AI analysis results
    fn print_analysis(&self, analysis: &BugAnalysis) {
        println!("\n[Node {}] ===== AI ANALYSIS RESULTS =====", self.node_id);
        println!("Root Cause: {}", analysis.root_cause);
        println!("\nBug Type: {:?}", analysis.bug_type);
        println!("\nLocation:");
        println!("  Component: {}", analysis.location.component);
        println!("  Function: {}", analysis.location.function);
        if let Some(line) = analysis.location.estimated_line {
            println!("  Line: {}", line);
        }
        println!("\nConfidence: {:.0}%", analysis.confidence * 100.0);
        println!("\nProof Sketch:");
        println!("{}", analysis.proof_sketch);
        println!("\nSuggested Patch:");
        println!("{}", analysis.suggested_patch);
        
        if !analysis.side_effects.is_empty() {
            println!("\nPotential Side Effects:");
            for effect in &analysis.side_effects {
                println!("  - {}", effect);
            }
        }
        println!("\n==========================================\n");
    }
    
    /// Get current state
    pub fn state(&self) -> State {
        self.state_machine.read().state()
    }
}

/// ClusterConfig - Configuration for entire cluster
#[derive(Debug, Clone)]
pub struct ClusterConfig {
    pub nodes: Vec<NodeConfig>,
}

#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub id: NodeId,
    pub addr: String,
}

impl ClusterConfig {
    /// Create config for a 3-node cluster
    pub fn three_node_cluster() -> Self {
        ClusterConfig {
            nodes: vec![
                NodeConfig { id: 1, addr: "127.0.0.1:5001".to_string() },
                NodeConfig { id: 2, addr: "127.0.0.1:5002".to_string() },
                NodeConfig { id: 3, addr: "127.0.0.1:5003".to_string() },
            ],
        }
    }
    
    /// Create network config for a node
    pub fn network_config(&self, node_id: NodeId) -> NetworkConfig {
        let my_node = self.nodes.iter()
            .find(|n| n.id == node_id)
            .expect("Node not found");
        
        let peers: HashMap<NodeId, String> = self.nodes.iter()
            .filter(|n| n.id != node_id)
            .map(|n| (n.id, n.addr.clone()))
            .collect();
        
        NetworkConfig {
            listen_addr: my_node.addr.clone(),
            peers,
            connect_timeout_ms: 1000,
            retry_attempts: 3,
        }
    }
    
    /// Create Raft config for a node
    pub fn raft_config(&self, node_id: NodeId) -> RaftConfig {
        let peers: Vec<NodeId> = self.nodes.iter()
            .map(|n| n.id)
            .collect();
        
        RaftConfig {
            peers,
            election_timeout_min: 150,
            election_timeout_max: 300,
            heartbeat_interval: 50,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_node_creation() {
        let cluster_config = ClusterConfig::three_node_cluster();
        let network_config = cluster_config.network_config(1);
        let raft_config = cluster_config.raft_config(1);
        
        let node = DistributedNode::new(1, network_config, raft_config);
        
        assert_eq!(node.node_id, 1);
    }
}
