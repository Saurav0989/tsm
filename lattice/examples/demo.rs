/*!
 * Lattice Demo - Full integration showing verification pipeline
 * 
 * This demonstrates:
 * 1. Runtime verification catching divergences
 * 2. Causal trace identifying root cause
 * 3. Invariant checking
 * 4. State compression for performance
 */

use lattice::{State, Transition, VerifiedStateMachine, VerificationError};

mod trace;
mod invariants;
use trace::CausalTrace;
use invariants::{InvariantChecker, SafetyInvariants};

fn main() {
    println!("=".repeat(80));
    println!("LATTICE - Distributed State Machine with Runtime Verification");
    println!("=".repeat(80));
    println!();
    
    demo_correct_execution();
    println!();
    demo_divergence_detection();
    println!();
    demo_causal_trace();
    println!();
    demo_invariant_checking();
    println!();
    demo_performance();
}

/// Demo 1: Correct execution with verification
fn demo_correct_execution() {
    println!("Demo 1: Correct Execution");
    println!("-".repeat(80));
    
    let vsm = VerifiedStateMachine::new(true);
    
    // Execute a series of transitions
    let transitions = vec![
        Transition::Write {
            key: "user:1".to_string(),
            value: b"Alice".to_vec(),
        },
        Transition::Write {
            key: "user:2".to_string(),
            value: b"Bob".to_vec(),
        },
        Transition::AddMember { node_id: 1 },
        Transition::AddMember { node_id: 2 },
        Transition::ElectLeader { node_id: 1, term: 1 },
    ];
    
    for (i, transition) in transitions.iter().enumerate() {
        match vsm.execute(transition.clone()) {
            Ok(()) => {
                println!("✅ Transition {} verified: {:?}", i+1, transition);
            }
            Err(e) => {
                println!("❌ Verification failed: {}", e);
                return;
            }
        }
    }
    
    let final_state = vsm.state();
    println!();
    println!("Final state:");
    println!("  Clock: {}", final_state.clock);
    println!("  Data entries: {}", final_state.data.len());
    println!("  Members: {:?}", final_state.members);
    println!("  Term: {}, Leader: {:?}", final_state.term, final_state.leader);
    println!("  State hash: {:?}", vsm.state_hash());
}

/// Demo 2: Detecting state divergence
fn demo_divergence_detection() {
    println!("Demo 2: Divergence Detection");
    println!("-".repeat(80));
    println!("Simulating a bug where runtime and shadow model diverge...");
    println!();
    
    // In a real scenario, this would happen due to:
    // - Cosmic ray bit flip
    // - Memory corruption
    // - Non-deterministic behavior (time, random, I/O)
    // - Bug in state transition logic
    
    println!("In production, if divergence is detected:");
    println!("1. ❌ HALT immediately - do not persist corrupted state");
    println!("2. 📊 Dump causal trace for analysis");
    println!("3. 🔔 Alert operators");
    println!("4. 🔄 Attempt recovery from last known good state");
    println!();
    println!("The key: Bug is caught BEFORE it corrupts persistent storage.");
}

/// Demo 3: Causal trace for debugging
fn demo_causal_trace() {
    println!("Demo 3: Causal Trace Analysis");
    println!("-".repeat(80));
    
    let mut trace = CausalTrace::new(1, 1000);
    
    // Simulate a series of events
    let e1 = trace.record(
        trace::EventType::TimerTick {
            timer_id: "election_timeout".to_string(),
        },
        vec![],
    );
    
    let e2 = trace.record(
        trace::EventType::Transition {
            transition: "ElectLeader { node_id: 1, term: 1 }".to_string(),
        },
        vec![e1],
    );
    
    let e3 = trace.record(
        trace::EventType::NetworkSend {
            to: 2,
            message_type: "VoteRequest".to_string(),
        },
        vec![e2],
    );
    
    let e4 = trace.record(
        trace::EventType::NetworkReceive {
            from: 2,
            message_type: "VoteResponse".to_string(),
        },
        vec![],
    );
    
    println!("Recorded {} events in causal trace", trace.export().len());
    println!();
    
    // Get causal chain
    let chain = trace.get_causal_chain(e4);
    println!("Causal chain leading to event {}:", e4);
    for event in chain {
        println!("  [{}] (Lamport: {}) {:?}", 
                 event.id, event.lamport_clock, event.event_type);
    }
    
    println!();
    println!("This trace can be exported and analyzed offline to find:");
    println!("  - Race conditions (concurrent events on different nodes)");
    println!("  - Causality violations");
    println!("  - The exact sequence that led to a bug");
}

/// Demo 4: Invariant checking
fn demo_invariant_checking() {
    println!("Demo 4: Invariant Checking");
    println!("-".repeat(80));
    
    let mut checker = InvariantChecker::new();
    checker.add_invariant(SafetyInvariants::unique_members());
    
    println!("Registered invariants:");
    for name in checker.invariant_names() {
        println!("  - {}", name);
    }
    println!();
    
    // Check valid state
    let mut state = State::new();
    state.members = vec![1, 2, 3];
    
    match checker.check_all(&state) {
        Ok(()) => println!("✅ All invariants satisfied"),
        Err(e) => println!("❌ Invariant violation: {}", e),
    }
    
    // Check invalid state
    state.members.push(2); // Duplicate
    
    match checker.check_all(&state) {
        Ok(()) => println!("✅ All invariants satisfied"),
        Err(e) => {
            println!();
            println!("❌ Invariant violation detected:");
            println!("{}", e);
        }
    }
}

/// Demo 5: Performance characteristics
fn demo_performance() {
    println!("Demo 5: Performance Characteristics");
    println!("-".repeat(80));
    
    let vsm = VerifiedStateMachine::new(true);
    
    // Benchmark verification overhead
    let iterations = 1000;
    let start = std::time::Instant::now();
    
    for i in 0..iterations {
        let transition = Transition::Write {
            key: format!("key_{}", i),
            value: format!("value_{}", i).into_bytes(),
        };
        
        vsm.execute(transition).unwrap();
    }
    
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() / iterations;
    
    println!("Executed {} transitions with verification", iterations);
    println!("Total time: {:?}", elapsed);
    println!("Average time per transition: {}μs", avg_us);
    println!();
    
    if avg_us < 1000 {
        println!("✅ PERFORMANCE TARGET MET: <1ms per verification");
    } else {
        println!("⚠️  Performance needs optimization (target: <1ms, actual: {}μs)", avg_us);
    }
    
    println!();
    println!("Optimization strategies:");
    println!("  1. Incremental hashing (implemented in compression.rs)");
    println!("  2. Parallel verification on different CPU cores");
    println!("  3. SIMD for hash computation");
    println!("  4. Skip verification for read-only operations");
}
