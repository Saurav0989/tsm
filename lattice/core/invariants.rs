/*!
 * Formal Specification - Runtime invariant checking
 * 
 * This is a simplified version of TLA+ invariants.
 * Production would integrate with actual TLA+ model checker.
 * 
 * CONCEPT: Define properties that must ALWAYS hold.
 * Check them on every state transition.
 */

use crate::{State, Transition, NodeId};

/// Invariant - A property that must always be true.
/// 
/// If an invariant is violated, the system has a bug.
pub trait Invariant: Send + Sync {
    /// Check if this invariant holds for a state.
    fn check(&self, state: &State) -> Result<(), InvariantViolation>;
    
    /// Name of this invariant (for debugging).
    fn name(&self) -> &str;
}

/// InvariantViolation - When a property is violated.
#[derive(Debug, Clone)]
pub struct InvariantViolation {
    pub invariant_name: String,
    pub message: String,
    pub state_clock: u64,
}

impl std::fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "INVARIANT VIOLATION: {}\n\
             Message: {}\n\
             State clock: {}\n\
             ACTION: System must halt - invariant broken",
            self.invariant_name,
            self.message,
            self.state_clock
        )
    }
}

impl std::error::Error for InvariantViolation {}

/// Safety Properties - Things that must never happen
pub struct SafetyInvariants;

impl SafetyInvariants {
    /// No two nodes can be leader in same term.
    /// 
    /// This is a classic distributed systems invariant.
    pub fn single_leader_per_term() -> Box<dyn Invariant> {
        Box::new(SingleLeaderPerTerm)
    }
    
    /// Clock must be monotonically increasing.
    pub fn monotonic_clock() -> Box<dyn Invariant> {
        Box::new(MonotonicClock { last_clock: 0 })
    }
    
    /// All members must be unique.
    pub fn unique_members() -> Box<dyn Invariant> {
        Box::new(UniqueMembers)
    }
}

/// Liveness Properties - Things that must eventually happen
/// 
/// NOTE: Liveness is harder to check at runtime.
/// These would require temporal logic (TLA+).
pub struct LivenessInvariants;

impl LivenessInvariants {
    /// Eventually a leader must be elected.
    /// 
    /// We can't check "eventually" at runtime, but we can
    /// check if we've been leaderless too long.
    pub fn leader_elected_eventually(timeout_ms: u64) -> Box<dyn Invariant> {
        Box::new(LeaderElectedEventually {
            timeout_ms,
            start_time: None,
        })
    }
}

// ============================================================================
// Invariant Implementations
// ============================================================================

struct SingleLeaderPerTerm;

impl Invariant for SingleLeaderPerTerm {
    fn check(&self, state: &State) -> Result<(), InvariantViolation> {
        // For now, we only track one leader per state.
        // In a real system, we'd need to track leader history per term.
        // This is simplified for demo.
        Ok(())
    }
    
    fn name(&self) -> &str {
        "SingleLeaderPerTerm"
    }
}

struct MonotonicClock {
    last_clock: u64,
}

impl Invariant for MonotonicClock {
    fn check(&self, state: &State) -> Result<(), InvariantViolation> {
        if state.clock < self.last_clock {
            return Err(InvariantViolation {
                invariant_name: self.name().to_string(),
                message: format!(
                    "Clock went backwards: {} -> {}",
                    self.last_clock,
                    state.clock
                ),
                state_clock: state.clock,
            });
        }
        Ok(())
    }
    
    fn name(&self) -> &str {
        "MonotonicClock"
    }
}

struct UniqueMembers;

impl Invariant for UniqueMembers {
    fn check(&self, state: &State) -> Result<(), InvariantViolation> {
        let mut seen = std::collections::HashSet::new();
        
        for member in &state.members {
            if !seen.insert(member) {
                return Err(InvariantViolation {
                    invariant_name: self.name().to_string(),
                    message: format!("Duplicate member: {}", member),
                    state_clock: state.clock,
                });
            }
        }
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "UniqueMembers"
    }
}

struct LeaderElectedEventually {
    timeout_ms: u64,
    start_time: Option<std::time::Instant>,
}

impl Invariant for LeaderElectedEventually {
    fn check(&self, state: &State) -> Result<(), InvariantViolation> {
        if state.leader.is_some() {
            return Ok(());
        }
        
        // If we don't have start time, this is first check
        // Real implementation would track this per-instance
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "LeaderElectedEventually"
    }
}

/// InvariantChecker - Runs all invariants on every state.
pub struct InvariantChecker {
    invariants: Vec<Box<dyn Invariant>>,
}

impl InvariantChecker {
    pub fn new() -> Self {
        InvariantChecker {
            invariants: Vec::new(),
        }
    }
    
    /// Add an invariant to check.
    pub fn add_invariant(&mut self, invariant: Box<dyn Invariant>) {
        self.invariants.push(invariant);
    }
    
    /// Check all invariants against a state.
    /// 
    /// Returns the first violation found, if any.
    pub fn check_all(&self, state: &State) -> Result<(), InvariantViolation> {
        for invariant in &self.invariants {
            invariant.check(state)?;
        }
        Ok(())
    }
    
    /// Get names of all invariants.
    pub fn invariant_names(&self) -> Vec<String> {
        self.invariants.iter().map(|i| i.name().to_string()).collect()
    }
}

/// TLA+ style specification (simplified)
/// 
/// This shows how you'd write formal specs.
/// Real TLA+ would be in a separate .tla file.
pub struct TLASpec {
    /// Initial state predicate
    /// 
    /// TLA+: Init == state.clock = 0 /\ state.data = {}
    pub fn init_predicate(state: &State) -> bool {
        state.clock == 0 && state.data.is_empty()
    }
    
    /// Next state relation
    /// 
    /// TLA+: Next == \E t \in Transitions: Apply(t, state, state')
    pub fn next_relation(before: &State, after: &State, transition: &Transition) -> bool {
        // Verify that 'after' is the result of applying transition to 'before'
        let expected = transition.apply(before.clone());
        expected == *after
    }
    
    /// Temporal properties
    /// 
    /// TLA+: []<>Leader (eventually always has a leader)
    /// 
    /// Can't check this at single state - need history.
    pub fn temporal_properties() -> Vec<&'static str> {
        vec![
            "Eventually a leader is elected",
            "Leaders don't change without term increment",
            "Data is never lost",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unique_members_invariant() {
        let invariant = UniqueMembers;
        
        let mut state = State::new();
        state.members = vec![1, 2, 3];
        
        assert!(invariant.check(&state).is_ok());
        
        // Add duplicate
        state.members.push(2);
        assert!(invariant.check(&state).is_err());
    }
    
    #[test]
    fn test_invariant_checker() {
        let mut checker = InvariantChecker::new();
        checker.add_invariant(Box::new(UniqueMembers));
        
        let mut state = State::new();
        state.members = vec![1, 1]; // Duplicate
        
        let result = checker.check_all(&state);
        assert!(result.is_err());
    }
}
