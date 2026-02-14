/*!
 * Causal Trace - Track the causal chain of every state transition
 * 
 * PROBLEM: When verification fails, we need to know WHY.
 * SOLUTION: Maintain a causal trace of all events leading to the divergence.
 * 
 * This is the "eBPF-style instrumentation" - zero-overhead event tracing.
 */

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// TraceEvent - A single event in the causal chain.
/// 
/// This captures everything we need to reconstruct what happened.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Unique event ID
    pub id: u64,
    
    /// Lamport timestamp - for causal ordering
    pub lamport_clock: u64,
    
    /// Wall clock time - for human debugging
    pub timestamp: u64,
    
    /// Node that generated this event
    pub node_id: u64,
    
    /// Type of event
    pub event_type: EventType,
    
    /// Parent event IDs - what caused this event
    pub parents: Vec<u64>,
    
    /// State hash before this event
    pub state_before: Option<[u8; 32]>,
    
    /// State hash after this event
    pub state_after: Option<[u8; 32]>,
}

/// EventType - What kind of event happened.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    /// State transition
    Transition {
        transition: String, // JSON serialized
    },
    
    /// Network receive
    NetworkReceive {
        from: u64,
        message_type: String,
    },
    
    /// Network send
    NetworkSend {
        to: u64,
        message_type: String,
    },
    
    /// Verification check
    Verification {
        passed: bool,
        expected_hash: [u8; 32],
        actual_hash: [u8; 32],
    },
    
    /// Timer tick
    TimerTick {
        timer_id: String,
    },
}

/// CausalTrace - Maintains the full causal history.
/// 
/// OPTIMIZATION: Use ring buffer to limit memory.
/// Keep last N events (e.g., 10,000).
pub struct CausalTrace {
    /// Ring buffer of events
    events: VecDeque<TraceEvent>,
    
    /// Maximum events to keep
    max_events: usize,
    
    /// Next event ID
    next_id: u64,
    
    /// Current Lamport clock
    lamport_clock: u64,
    
    /// Node ID for this trace
    node_id: u64,
}

impl CausalTrace {
    pub fn new(node_id: u64, max_events: usize) -> Self {
        CausalTrace {
            events: VecDeque::with_capacity(max_events),
            max_events,
            next_id: 0,
            lamport_clock: 0,
            node_id,
        }
    }
    
    /// Record an event.
    /// 
    /// PERFORMANCE: This must be fast - it's called on EVERY transition.
    /// Goal: <100ns
    pub fn record(&mut self, event_type: EventType, parents: Vec<u64>) -> u64 {
        // Increment Lamport clock
        self.lamport_clock += 1;
        
        let event = TraceEvent {
            id: self.next_id,
            lamport_clock: self.lamport_clock,
            timestamp: current_timestamp(),
            node_id: self.node_id,
            event_type,
            parents,
            state_before: None,
            state_after: None,
        };
        
        let id = self.next_id;
        self.next_id += 1;
        
        // Add to ring buffer
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        self.events.push_back(event);
        
        id
    }
    
    /// Add state hashes to an event.
    pub fn set_state_hashes(&mut self, event_id: u64, before: [u8; 32], after: [u8; 32]) {
        if let Some(event) = self.events.iter_mut().rev().find(|e| e.id == event_id) {
            event.state_before = Some(before);
            event.state_after = Some(after);
        }
    }
    
    /// Get causal chain leading to an event.
    /// 
    /// Returns all ancestor events in topological order.
    pub fn get_causal_chain(&self, event_id: u64) -> Vec<TraceEvent> {
        let mut chain = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![event_id];
        
        while let Some(id) = stack.pop() {
            if visited.contains(&id) {
                continue;
            }
            visited.insert(id);
            
            if let Some(event) = self.events.iter().find(|e| e.id == id) {
                chain.push(event.clone());
                
                // Add parents to stack
                for parent_id in &event.parents {
                    if !visited.contains(parent_id) {
                        stack.push(*parent_id);
                    }
                }
            }
        }
        
        // Sort by Lamport clock (causal order)
        chain.sort_by_key(|e| e.lamport_clock);
        chain
    }
    
    /// Find divergence point.
    /// 
    /// Given two state hashes that should be equal but aren't,
    /// find the first event where they diverged.
    pub fn find_divergence(&self, expected: [u8; 32], actual: [u8; 32]) -> Option<&TraceEvent> {
        // Walk backwards through events
        for event in self.events.iter().rev() {
            if let (Some(state_after), EventType::Verification { expected_hash, actual_hash, .. }) 
                = (&event.state_after, &event.event_type) 
            {
                if expected_hash != actual_hash {
                    return Some(event);
                }
            }
        }
        None
    }
    
    /// Export trace for analysis.
    pub fn export(&self) -> Vec<TraceEvent> {
        self.events.iter().cloned().collect()
    }
}

/// Get current timestamp in microseconds.
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_micros() as u64
}

/// DistributedTrace - Collects traces from multiple nodes.
/// 
/// This is how we debug distributed race conditions.
pub struct DistributedTrace {
    /// Traces from each node
    node_traces: std::collections::HashMap<u64, Vec<TraceEvent>>,
}

impl DistributedTrace {
    pub fn new() -> Self {
        DistributedTrace {
            node_traces: std::collections::HashMap::new(),
        }
    }
    
    /// Add a node's trace.
    pub fn add_node_trace(&mut self, node_id: u64, trace: Vec<TraceEvent>) {
        self.node_traces.insert(node_id, trace);
    }
    
    /// Merge all traces into global causal order.
    /// 
    /// Uses Lamport clocks to determine ordering.
    pub fn merge(&self) -> Vec<TraceEvent> {
        let mut all_events = Vec::new();
        
        for trace in self.node_traces.values() {
            all_events.extend(trace.iter().cloned());
        }
        
        // Sort by Lamport clock
        all_events.sort_by_key(|e| e.lamport_clock);
        all_events
    }
    
    /// Find race conditions.
    /// 
    /// Concurrent events (same Lamport clock) on different nodes
    /// are potential races.
    pub fn find_races(&self) -> Vec<(TraceEvent, TraceEvent)> {
        let merged = self.merge();
        let mut races = Vec::new();
        
        for i in 0..merged.len() {
            for j in (i+1)..merged.len() {
                let e1 = &merged[i];
                let e2 = &merged[j];
                
                // Same Lamport clock + different nodes = concurrent
                if e1.lamport_clock == e2.lamport_clock && e1.node_id != e2.node_id {
                    races.push((e1.clone(), e2.clone()));
                }
            }
        }
        
        races
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trace_record() {
        let mut trace = CausalTrace::new(1, 100);
        
        let event_id = trace.record(
            EventType::Transition {
                transition: "test".to_string(),
            },
            vec![],
        );
        
        assert_eq!(event_id, 0);
        assert_eq!(trace.lamport_clock, 1);
        assert_eq!(trace.events.len(), 1);
    }
    
    #[test]
    fn test_causal_chain() {
        let mut trace = CausalTrace::new(1, 100);
        
        let e1 = trace.record(EventType::TimerTick { timer_id: "t1".to_string() }, vec![]);
        let e2 = trace.record(EventType::TimerTick { timer_id: "t2".to_string() }, vec![e1]);
        let e3 = trace.record(EventType::TimerTick { timer_id: "t3".to_string() }, vec![e2]);
        
        let chain = trace.get_causal_chain(e3);
        
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].id, e1);
        assert_eq!(chain[1].id, e2);
        assert_eq!(chain[2].id, e3);
    }
    
    #[test]
    fn test_ring_buffer() {
        let mut trace = CausalTrace::new(1, 3);
        
        // Add 5 events (more than max)
        for i in 0..5 {
            trace.record(EventType::TimerTick { timer_id: format!("t{}", i) }, vec![]);
        }
        
        // Should only keep last 3
        assert_eq!(trace.events.len(), 3);
    }
}
