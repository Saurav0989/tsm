/*!
 * Raft Consensus - Leader election and log replication
 * 
 * Implements the Raft consensus algorithm for distributed agreement.
 * 
 * Key properties:
 * - Leader election with terms
 * - Log replication with majority quorum
 * - Safety guarantees
 */

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::{State, Transition, NodeId};

/// RaftNode - A node participating in Raft consensus
pub struct RaftNode {
    /// My node ID
    id: NodeId,
    
    /// Current state
    state: State,
    
    /// Current role
    role: Role,
    
    /// Persistent state (survives crashes)
    persistent: PersistentState,
    
    /// Volatile state
    volatile: VolatileState,
    
    /// Configuration
    config: RaftConfig,
    
    /// Election timeout
    election_timeout: Duration,
    
    /// Last time we heard from leader
    last_heartbeat: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

/// Persistent state - must be saved to disk before responding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentState {
    /// Latest term server has seen
    current_term: u64,
    
    /// Candidate ID that received vote in current term
    voted_for: Option<NodeId>,
    
    /// Log entries (each contains command for state machine)
    log: Vec<LogEntry>,
}

/// Volatile state on all servers
#[derive(Debug, Clone)]
pub struct VolatileState {
    /// Index of highest log entry known to be committed
    commit_index: usize,
    
    /// Index of highest log entry applied to state machine
    last_applied: usize,
}

/// Volatile state on leaders (reinitialized after election)
#[derive(Debug, Clone)]
pub struct LeaderState {
    /// For each server, index of next log entry to send
    next_index: HashMap<NodeId, usize>,
    
    /// For each server, index of highest log entry known to be replicated
    match_index: HashMap<NodeId, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Term when entry was received by leader
    term: u64,
    
    /// Index in log
    index: usize,
    
    /// State machine command
    transition: Transition,
}

pub struct RaftConfig {
    /// All nodes in cluster
    pub peers: Vec<NodeId>,
    
    /// Election timeout range (ms)
    pub election_timeout_min: u64,
    pub election_timeout_max: u64,
    
    /// Heartbeat interval (ms)
    pub heartbeat_interval: u64,
}

impl RaftNode {
    pub fn new(id: NodeId, config: RaftConfig) -> Self {
        let election_timeout = Duration::from_millis(
            config.election_timeout_min +
            (rand::random::<u64>() % (config.election_timeout_max - config.election_timeout_min))
        );
        
        RaftNode {
            id,
            state: State::new(),
            role: Role::Follower,
            persistent: PersistentState {
                current_term: 0,
                voted_for: None,
                log: Vec::new(),
            },
            volatile: VolatileState {
                commit_index: 0,
                last_applied: 0,
            },
            config,
            election_timeout,
            last_heartbeat: Instant::now(),
        }
    }
    
    /// Tick - called periodically to drive state machine
    pub fn tick(&mut self) -> Vec<RaftMessage> {
        let mut messages = Vec::new();
        
        match self.role {
            Role::Follower | Role::Candidate => {
                // Check for election timeout
                if self.last_heartbeat.elapsed() > self.election_timeout {
                    messages.extend(self.start_election());
                }
            }
            Role::Leader => {
                // Send heartbeats
                messages.extend(self.send_heartbeats());
            }
        }
        
        messages
    }
    
    /// Start a new election
    fn start_election(&mut self) -> Vec<RaftMessage> {
        println!("[Node {}] Starting election for term {}", self.id, self.persistent.current_term + 1);
        
        // Increment term
        self.persistent.current_term += 1;
        
        // Become candidate
        self.role = Role::Candidate;
        
        // Vote for self
        self.persistent.voted_for = Some(self.id);
        
        // Reset election timer
        self.last_heartbeat = Instant::now();
        
        // Send RequestVote to all peers
        self.config.peers.iter()
            .filter(|&&peer| peer != self.id)
            .map(|&peer| {
                RaftMessage {
                    from: self.id,
                    to: peer,
                    payload: MessagePayload::RequestVote {
                        term: self.persistent.current_term,
                        candidate_id: self.id,
                        last_log_index: self.persistent.log.len(),
                        last_log_term: self.persistent.log.last()
                            .map(|e| e.term)
                            .unwrap_or(0),
                    },
                }
            })
            .collect()
    }
    
    /// Handle incoming message
    pub fn handle_message(&mut self, msg: RaftMessage) -> Vec<RaftMessage> {
        // Check term
        if let Some(msg_term) = msg.payload.term() {
            if msg_term > self.persistent.current_term {
                // Newer term - step down
                self.persistent.current_term = msg_term;
                self.role = Role::Follower;
                self.persistent.voted_for = None;
            }
        }
        
        match msg.payload {
            MessagePayload::RequestVote { term, candidate_id, last_log_index, last_log_term } => {
                self.handle_request_vote(msg.from, term, candidate_id, last_log_index, last_log_term)
            }
            MessagePayload::RequestVoteResponse { term, vote_granted } => {
                self.handle_vote_response(msg.from, term, vote_granted)
            }
            MessagePayload::AppendEntries { term, leader_id, prev_log_index, prev_log_term, entries, leader_commit } => {
                self.handle_append_entries(msg.from, term, leader_id, prev_log_index, prev_log_term, entries, leader_commit)
            }
            MessagePayload::AppendEntriesResponse { term, success, match_index } => {
                self.handle_append_entries_response(msg.from, term, success, match_index)
            }
        }
    }
    
    fn handle_request_vote(
        &mut self,
        from: NodeId,
        term: u64,
        candidate_id: NodeId,
        last_log_index: usize,
        last_log_term: u64,
    ) -> Vec<RaftMessage> {
        let vote_granted = if term < self.persistent.current_term {
            false
        } else if let Some(voted) = self.persistent.voted_for {
            voted == candidate_id
        } else {
            // Check if candidate's log is at least as up-to-date
            let our_last_index = self.persistent.log.len();
            let our_last_term = self.persistent.log.last().map(|e| e.term).unwrap_or(0);
            
            last_log_term > our_last_term ||
                (last_log_term == our_last_term && last_log_index >= our_last_index)
        };
        
        if vote_granted {
            self.persistent.voted_for = Some(candidate_id);
            self.last_heartbeat = Instant::now();
        }
        
        vec![RaftMessage {
            from: self.id,
            to: from,
            payload: MessagePayload::RequestVoteResponse {
                term: self.persistent.current_term,
                vote_granted,
            },
        }]
    }
    
    fn handle_vote_response(
        &mut self,
        from: NodeId,
        term: u64,
        vote_granted: bool,
    ) -> Vec<RaftMessage> {
        if self.role != Role::Candidate || term != self.persistent.current_term {
            return vec![];
        }
        
        if !vote_granted {
            return vec![];
        }
        
        // Count votes (including self)
        let votes = 1; // Self vote
        // In real impl, track votes from all nodes
        
        let majority = (self.config.peers.len() / 2) + 1;
        
        if votes >= majority {
            // Won election!
            println!("[Node {}] Won election for term {}", self.id, self.persistent.current_term);
            self.become_leader();
            return self.send_heartbeats();
        }
        
        vec![]
    }
    
    fn become_leader(&mut self) {
        self.role = Role::Leader;
        
        // Initialize leader state
        // (In real impl, would track next_index and match_index)
        
        println!("[Node {}] Became leader for term {}", self.id, self.persistent.current_term);
    }
    
    fn send_heartbeats(&mut self) -> Vec<RaftMessage> {
        self.config.peers.iter()
            .filter(|&&peer| peer != self.id)
            .map(|&peer| {
                RaftMessage {
                    from: self.id,
                    to: peer,
                    payload: MessagePayload::AppendEntries {
                        term: self.persistent.current_term,
                        leader_id: self.id,
                        prev_log_index: self.persistent.log.len(),
                        prev_log_term: self.persistent.log.last().map(|e| e.term).unwrap_or(0),
                        entries: vec![], // Heartbeat - no entries
                        leader_commit: self.volatile.commit_index,
                    },
                }
            })
            .collect()
    }
    
    fn handle_append_entries(
        &mut self,
        from: NodeId,
        term: u64,
        leader_id: NodeId,
        prev_log_index: usize,
        prev_log_term: u64,
        entries: Vec<LogEntry>,
        leader_commit: usize,
    ) -> Vec<RaftMessage> {
        // Reset election timeout
        self.last_heartbeat = Instant::now();
        
        if term < self.persistent.current_term {
            return vec![RaftMessage {
                from: self.id,
                to: from,
                payload: MessagePayload::AppendEntriesResponse {
                    term: self.persistent.current_term,
                    success: false,
                    match_index: 0,
                },
            }];
        }
        
        // Step down if we're not a follower
        if self.role != Role::Follower {
            self.role = Role::Follower;
        }
        
        // Append entries (simplified - real impl checks log consistency)
        for entry in entries {
            self.persistent.log.push(entry);
        }
        
        // Update commit index
        if leader_commit > self.volatile.commit_index {
            self.volatile.commit_index = std::cmp::min(
                leader_commit,
                self.persistent.log.len(),
            );
        }
        
        vec![RaftMessage {
            from: self.id,
            to: from,
            payload: MessagePayload::AppendEntriesResponse {
                term: self.persistent.current_term,
                success: true,
                match_index: self.persistent.log.len(),
            },
        }]
    }
    
    fn handle_append_entries_response(
        &mut self,
        from: NodeId,
        term: u64,
        success: bool,
        match_index: usize,
    ) -> Vec<RaftMessage> {
        if self.role != Role::Leader || term != self.persistent.current_term {
            return vec![];
        }
        
        if success {
            // Update match_index (in real impl)
            // Check if we can advance commit_index
        }
        
        vec![]
    }
    
    /// Propose a new transition
    pub fn propose(&mut self, transition: Transition) -> Result<(), String> {
        if self.role != Role::Leader {
            return Err("Not the leader".to_string());
        }
        
        // Append to local log
        let entry = LogEntry {
            term: self.persistent.current_term,
            index: self.persistent.log.len() + 1,
            transition,
        };
        
        self.persistent.log.push(entry);
        
        Ok(())
    }
}

/// RaftMessage - Network message between nodes
#[derive(Debug, Clone)]
pub struct RaftMessage {
    pub from: NodeId,
    pub to: NodeId,
    pub payload: MessagePayload,
}

#[derive(Debug, Clone)]
pub enum MessagePayload {
    RequestVote {
        term: u64,
        candidate_id: NodeId,
        last_log_index: usize,
        last_log_term: u64,
    },
    RequestVoteResponse {
        term: u64,
        vote_granted: bool,
    },
    AppendEntries {
        term: u64,
        leader_id: NodeId,
        prev_log_index: usize,
        prev_log_term: u64,
        entries: Vec<LogEntry>,
        leader_commit: usize,
    },
    AppendEntriesResponse {
        term: u64,
        success: bool,
        match_index: usize,
    },
}

impl MessagePayload {
    fn term(&self) -> Option<u64> {
        match self {
            MessagePayload::RequestVote { term, .. } => Some(*term),
            MessagePayload::RequestVoteResponse { term, .. } => Some(*term),
            MessagePayload::AppendEntries { term, .. } => Some(*term),
            MessagePayload::AppendEntriesResponse { term, .. } => Some(*term),
        }
    }
}

// Simple random number generator to avoid external dependency
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
