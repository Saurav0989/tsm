/*!
 * Multi-Region Geo-Distributed Architecture
 * 
 * For global deployments spanning continents.
 * 
 * Features:
 * - Cross-region consensus (EPaxos)
 * - WAN-optimized replication
 * - Geographic routing
 * - Conflict-free replicated data types (CRDTs)
 * - Multi-datacenter failover
 * 
 * Goal: <100ms cross-region latency, 99.999% availability
 */

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

use crate::{State, Transition, NodeId};

/// Geographic region
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Region {
    USEast,
    USWest,
    EUWest,
    APSoutheast,
    APNortheast,
    SAEast,
}

impl Region {
    /// Estimated latency between regions (ms)
    pub fn latency_to(&self, other: Region) -> u64 {
        match (self, other) {
            (a, b) if a == b => 1,
            (Region::USEast, Region::USWest) |
            (Region::USWest, Region::USEast) => 60,
            (Region::USEast, Region::EUWest) |
            (Region::EUWest, Region::USEast) => 80,
            (Region::EUWest, Region::APSoutheast) |
            (Region::APSoutheast, Region::EUWest) => 160,
            (Region::USWest, Region::APNortheast) |
            (Region::APNortheast, Region::USWest) => 100,
            _ => 200, // Worst case cross-globe
        }
    }
    
    /// Geographic proximity for routing
    pub fn proximity(&self, other: Region) -> usize {
        match (self, other) {
            (a, b) if a == b => 0,
            (Region::USEast, Region::USWest) |
            (Region::USWest, Region::USEast) => 1,
            (Region::EUWest, Region::APNortheast) => 2,
            _ => 3,
        }
    }
}

/// Multi-region cluster configuration
#[derive(Debug, Clone)]
pub struct MultiRegionConfig {
    /// Regions and their nodes
    pub regions: HashMap<Region, Vec<NodeId>>,
    
    /// Replication strategy
    pub replication: ReplicationStrategy,
    
    /// Consistency level
    pub consistency: ConsistencyLevel,
    
    /// WAN optimization enabled
    pub wan_optimization: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum ReplicationStrategy {
    /// Replicate to all regions
    Global,
    
    /// Replicate to nearest N regions
    NearestN(usize),
    
    /// Replicate to specific regions
    Custom,
}

#[derive(Debug, Clone, Copy)]
pub enum ConsistencyLevel {
    /// Strong consistency (wait for all regions)
    Strong,
    
    /// Causal consistency (happens-before ordering)
    Causal,
    
    /// Eventual consistency (best effort)
    Eventual,
}

/// EPaxos for WAN consensus
/// 
/// Unlike Raft, EPaxos doesn't require a leader,
/// reducing cross-region latency
pub struct EPaxosNode {
    id: NodeId,
    region: Region,
    state: State,
    instances: HashMap<InstanceId, Instance>,
    config: MultiRegionConfig,
}

type InstanceId = (NodeId, u64);

#[derive(Debug, Clone)]
struct Instance {
    id: InstanceId,
    command: Transition,
    status: InstanceStatus,
    deps: Vec<InstanceId>,
    seq: u64,
    ballot: Ballot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InstanceStatus {
    PreAccepted,
    Accepted,
    Committed,
    Executed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Ballot {
    epoch: u64,
    node_id: NodeId,
}

impl EPaxosNode {
    pub fn new(id: NodeId, region: Region, config: MultiRegionConfig) -> Self {
        EPaxosNode {
            id,
            region,
            state: State::new(),
            instances: HashMap::new(),
            config,
        }
    }
    
    /// Propose a command (no leader required!)
    pub fn propose(&mut self, command: Transition) -> Vec<EPaxosMessage> {
        let instance_id = (self.id, self.next_sequence());
        
        // Compute dependencies (commands that conflict)
        let deps = self.compute_dependencies(&command);
        
        let instance = Instance {
            id: instance_id,
            command: command.clone(),
            status: InstanceStatus::PreAccepted,
            deps: deps.clone(),
            seq: 0, // Will be determined during consensus
            ballot: Ballot { epoch: 0, node_id: self.id },
        };
        
        self.instances.insert(instance_id, instance);
        
        // Send PreAccept to fast quorum (nearest regions)
        let fast_quorum = self.fast_quorum();
        fast_quorum.into_iter().map(|peer| {
            EPaxosMessage {
                from: self.id,
                to: peer,
                payload: EPaxosPayload::PreAccept {
                    instance_id,
                    command: command.clone(),
                    deps: deps.clone(),
                    seq: 0,
                },
            }
        }).collect()
    }
    
    /// Compute command dependencies
    fn compute_dependencies(&self, command: &Transition) -> Vec<InstanceId> {
        self.instances.values()
            .filter(|inst| self.conflicts(command, &inst.command))
            .map(|inst| inst.id)
            .collect()
    }
    
    /// Check if two commands conflict
    fn conflicts(&self, a: &Transition, b: &Transition) -> bool {
        // Commands conflict if they access the same keys
        match (a, b) {
            (Transition::Write { key: k1, .. }, Transition::Write { key: k2, .. }) => k1 == k2,
            (Transition::Write { key: k1, .. }, Transition::Delete { key: k2 }) => k1 == k2,
            (Transition::Delete { key: k1 }, Transition::Write { key: k2, .. }) => k1 == k2,
            (Transition::Delete { key: k1 }, Transition::Delete { key: k2 }) => k1 == k2,
            _ => false,
        }
    }
    
    /// Fast quorum (nearest F+⌊(F+1)/2⌋ replicas)
    fn fast_quorum(&self) -> Vec<NodeId> {
        let mut peers: Vec<_> = self.config.regions.iter()
            .flat_map(|(region, nodes)| {
                nodes.iter().map(move |&id| (id, *region))
            })
            .filter(|&(id, _)| id != self.id)
            .collect();
        
        // Sort by proximity
        peers.sort_by_key(|(_, region)| self.region.proximity(*region));
        
        let f = self.fault_tolerance();
        let quorum_size = f + (f + 1) / 2;
        
        peers.into_iter()
            .take(quorum_size)
            .map(|(id, _)| id)
            .collect()
    }
    
    fn fault_tolerance(&self) -> usize {
        let total_nodes: usize = self.config.regions.values()
            .map(|v| v.len())
            .sum();
        (total_nodes - 1) / 2
    }
    
    fn next_sequence(&mut self) -> u64 {
        self.instances.len() as u64
    }
}

#[derive(Debug, Clone)]
pub struct EPaxosMessage {
    pub from: NodeId,
    pub to: NodeId,
    pub payload: EPaxosPayload,
}

#[derive(Debug, Clone)]
pub enum EPaxosPayload {
    PreAccept {
        instance_id: InstanceId,
        command: Transition,
        deps: Vec<InstanceId>,
        seq: u64,
    },
    PreAcceptOK {
        instance_id: InstanceId,
        deps: Vec<InstanceId>,
        seq: u64,
    },
    Accept {
        instance_id: InstanceId,
        deps: Vec<InstanceId>,
        seq: u64,
    },
    AcceptOK {
        instance_id: InstanceId,
    },
    Commit {
        instance_id: InstanceId,
        deps: Vec<InstanceId>,
        seq: u64,
    },
}

/// CRDT for conflict-free replication
/// 
/// Allows concurrent updates without coordination
pub trait CRDT {
    /// Merge two replicas
    fn merge(&mut self, other: &Self);
    
    /// Check if this dominates other
    fn dominates(&self, other: &Self) -> bool;
}

/// Last-Write-Wins Register (simple CRDT)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LWWRegister<T> {
    value: T,
    timestamp: u64,
    node_id: NodeId,
}

impl<T: Clone> LWWRegister<T> {
    pub fn new(value: T, node_id: NodeId) -> Self {
        LWWRegister {
            value,
            timestamp: current_timestamp(),
            node_id,
        }
    }
    
    pub fn update(&mut self, value: T) {
        self.value = value;
        self.timestamp = current_timestamp();
    }
    
    pub fn get(&self) -> &T {
        &self.value
    }
}

impl<T: Clone> CRDT for LWWRegister<T> {
    fn merge(&mut self, other: &Self) {
        if other.timestamp > self.timestamp ||
           (other.timestamp == self.timestamp && other.node_id > self.node_id) {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
            self.node_id = other.node_id;
        }
    }
    
    fn dominates(&self, other: &Self) -> bool {
        self.timestamp > other.timestamp ||
        (self.timestamp == other.timestamp && self.node_id > other.node_id)
    }
}

/// Geographic routing for client requests
pub struct GeoRouter {
    /// Region latencies
    latencies: HashMap<Region, HashMap<Region, u64>>,
}

impl GeoRouter {
    pub fn new() -> Self {
        GeoRouter {
            latencies: HashMap::new(),
        }
    }
    
    /// Route request to optimal region
    pub fn route(&self, client_region: Region, regions: &[Region]) -> Region {
        *regions.iter()
            .min_by_key(|&&r| client_region.latency_to(r))
            .unwrap_or(&regions[0])
    }
    
    /// Multi-region read (read from nearest replicas)
    pub fn multi_region_read(&self, client_region: Region, regions: &[Region], num_reads: usize) -> Vec<Region> {
        let mut sorted: Vec<_> = regions.iter()
            .map(|&r| (r, client_region.latency_to(r)))
            .collect();
        
        sorted.sort_by_key(|(_, lat)| *lat);
        
        sorted.into_iter()
            .take(num_reads)
            .map(|(r, _)| r)
            .collect()
    }
}

/// WAN Optimizer for cross-region traffic
pub struct WANOptimizer {
    /// Compression enabled
    compression: bool,
    
    /// Batching window
    batch_window: Duration,
    
    /// Pending messages
    pending: HashMap<Region, Vec<Vec<u8>>>,
}

impl WANOptimizer {
    pub fn new() -> Self {
        WANOptimizer {
            compression: true,
            batch_window: Duration::from_millis(10),
            pending: HashMap::new(),
        }
    }
    
    /// Optimize message for WAN transmission
    pub fn optimize(&mut self, dest_region: Region, message: Vec<u8>) -> Option<Vec<u8>> {
        // Add to batch
        self.pending.entry(dest_region)
            .or_insert_with(Vec::new)
            .push(message);
        
        // Flush batch periodically
        None // Would return batched/compressed messages
    }
    
    /// Compress messages
    fn compress(&self, messages: &[Vec<u8>]) -> Vec<u8> {
        // Would use actual compression (LZ4, Zstd)
        messages.concat()
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_region_latency() {
        assert_eq!(Region::USEast.latency_to(Region::USWest), 60);
        assert_eq!(Region::USEast.latency_to(Region::EUWest), 80);
        assert!(Region::EUWest.latency_to(Region::APSoutheast) > 100);
    }
    
    #[test]
    fn test_lww_register() {
        let mut reg1 = LWWRegister::new(42, 1);
        let mut reg2 = LWWRegister::new(100, 2);
        
        // Update reg2 later
        std::thread::sleep(std::time::Duration::from_millis(10));
        reg2.update(200);
        
        // Merge - reg2 should win
        reg1.merge(&reg2);
        assert_eq!(*reg1.get(), 200);
    }
    
    #[test]
    fn test_geo_routing() {
        let router = GeoRouter::new();
        let regions = vec![Region::USEast, Region::USWest, Region::EUWest];
        
        let best = router.route(Region::USEast, &regions);
        assert_eq!(best, Region::USEast); // Same region is best
    }
}
