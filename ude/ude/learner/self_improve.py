"""
Universal Discovery Engine - Self-Improvement System

Implements Tier 3 & 7: Self-improvement and autonomous goal generation.

The system learns from:
1. Which tactics work for which theorem patterns
2. Which axioms lead to provable theorems
3. Which hypothesis patterns are productive
"""

import json
import time
from typing import Dict, List, Any, Optional
from dataclasses import dataclass, field
from collections import defaultdict
from pathlib import Path
import random


@dataclass
class ProofAttempt:
    """Record of a proof attempt"""

    theorem_hash: str
    theorem_structure: str  # Simplified pattern
    tactics_used: List[str]
    success: bool
    time_taken: float
    error: Optional[str] = None


@dataclass
class TacticPerformance:
    """Tracks how well each tactic performs"""

    tactic_name: str
    attempts: int = 0
    successes: int = 0
    avg_time: float = 0.0
    patterns_successful: Dict[str, int] = field(default_factory=dict)


class SelfImprovementEngine:
    """
    Implements autonomous learning and improvement.

    Learns from proof attempts to:
    1. Prioritize effective tactics
    2. Generate better hypotheses
    3. Identify productive axiom combinations
    4. Create its own research goals
    """

    def __init__(self, data_dir: str = "learning_data"):
        self.data_dir = Path(data_dir)
        self.data_dir.mkdir(exist_ok=True)

        # Track tactic performance
        self.tactic_performance: Dict[str, TacticPerformance] = {}

        # Track theorem patterns
        self.theorem_patterns: Dict[str, int] = defaultdict(int)
        self.productive_patterns: Dict[str, int] = defaultdict(int)

        # Track axioms
        self.axiom_effectiveness: Dict[str, int] = defaultdict(int)

        # Research goals
        self.research_goals: List[Dict] = []
        self.completed_goals: List[Dict] = []

        # Statistics
        self.total_attempts = 0
        self.total_successes = 0
        self.learning_rate = 0.0

        # Load previous learning
        self._load_learning()

    def record_proof_attempt(
        self,
        theorem_hash: str,
        theorem_structure: str,
        tactics_used: List[str],
        success: bool,
        time_taken: float,
        axioms_used: List[str] = None,
        error: str = None,
    ):
        """Record a proof attempt for learning"""

        self.total_attempts += 1

        # Update tactic performance
        for tactic in tactics_used:
            if tactic not in self.tactic_performance:
                self.tactic_performance[tactic] = TacticPerformance(tactic_name=tactic)

            perf = self.tactic_performance[tactic]
            perf.attempts += 1

            if success:
                perf.successes += 1
                # Track which patterns this tactic works for
                if theorem_structure not in perf.patterns_successful:
                    perf.patterns_successful[theorem_structure] = 0
                perf.patterns_successful[theorem_structure] += 1

            # Update average time
            perf.avg_time = (
                perf.avg_time * (perf.attempts - 1) + time_taken
            ) / perf.attempts

        # Update theorem patterns
        self.theorem_patterns[theorem_structure] += 1
        if success:
            self.productive_patterns[theorem_structure] += 1

        # Update axiom effectiveness
        if axioms_used:
            for axiom in axioms_used:
                self.axiom_effectiveness[axiom] += 1

        # Update success rate
        self.total_successes = sum(
            p.successes for p in self.tactic_performance.values()
        )

        # Save learning
        if self.total_attempts % 100 == 0:
            self._save_learning()

    def get_best_tactics(self, theorem_structure: str, top_k: int = 5) -> List[str]:
        """Get the best tactics for a given theorem pattern"""

        scored_tactics = []

        for tactic, perf in self.tactic_performance.items():
            if perf.attempts == 0:
                continue

            # Score based on success rate and speed
            success_rate = perf.successes / perf.attempts

            # Bonus for working on this pattern
            pattern_bonus = 0.0
            if theorem_structure in perf.patterns_successful:
                pattern_successes = perf.patterns_successful[theorem_structure]
                pattern_bonus = pattern_successes / perf.attempts * 0.5

            # Penalty for slow tactics
            speed_factor = 1.0 / (1.0 + perf.avg_time)

            score = success_rate * 0.5 + pattern_bonus * 0.3 + speed_factor * 0.2
            scored_tactics.append((tactic, score))

        # Sort by score
        scored_tactics.sort(key=lambda x: x[1], reverse=True)

        return [t for t, _ in scored_tactics[:top_k]]

    def get_productive_patterns(self, min_success_rate: float = 0.1) -> List[str]:
        """Get theorem patterns that lead to successful proofs"""

        productive = []

        for pattern, attempts in self.theorem_patterns.items():
            if attempts < 5:  # Need minimum data
                continue

            successes = self.productive_patterns.get(pattern, 0)
            rate = successes / attempts

            if rate >= min_success_rate:
                productive.append((pattern, rate))

        productive.sort(key=lambda x: x[1], reverse=True)
        return [p for p, _ in productive]

    def get_effective_axioms(self) -> List[str]:
        """Get axioms that lead to provable theorems"""

        sorted_axioms = sorted(
            self.axiom_effectiveness.items(), key=lambda x: x[1], reverse=True
        )

        return [a for a, _ in sorted_axioms[:10]]

    def generate_research_goal(self) -> Dict[str, Any]:
        """Generate an autonomous research goal (Tier 7)"""

        # Analyze what we've learned
        productive = self.get_productive_patterns()
        effective_axioms = self.get_effective_axioms()

        # Identify gaps - patterns we haven't explored enough
        all_patterns = list(self.theorem_patterns.keys())

        # Pick a direction based on what we've learned
        if random.random() < 0.3 and productive:
            # Explore productive patterns further
            goal = {
                "type": "explore_pattern",
                "pattern": random.choice(productive[:3]),
                "description": f"Explore theorems similar to {productive[0]}",
                "created_at": time.time(),
            }
        elif effective_axioms:
            # Try new axiom combinations
            goal = {
                "type": "axiom_exploration",
                "axioms": random.sample(
                    effective_axioms, min(3, len(effective_axioms))
                ),
                "description": f"Explore theorems using axioms: {effective_axioms[:3]}",
                "created_at": time.time(),
            }
        elif all_patterns:
            # General exploration
            goal = {
                "type": "general_exploration",
                "pattern": random.choice(all_patterns),
                "description": "General theorem exploration",
                "created_at": time.time(),
            }
        else:
            # Initial goal
            goal = {
                "type": "initial",
                "description": "Begin autonomous discovery",
                "created_at": time.time(),
            }

        self.research_goals.append(goal)
        return goal

    def complete_goal(self, goal: Dict, results: Dict):
        """Mark a research goal as completed"""
        goal["completed_at"] = time.time()
        goal["results"] = results
        self.completed_goals.append(goal)

        # Remove from active goals
        if goal in self.research_goals:
            self.research_goals.remove(goal)

        self._save_learning()

    def get_improvement_metrics(self) -> Dict[str, Any]:
        """Get metrics showing self-improvement (Tier 3)"""

        # Calculate improvement over time
        recent_window = 100
        older_window = 200

        recent_successes = 0
        recent_attempts = 0

        # This is simplified - real implementation would track time windows
        total_successes = sum(p.successes for p in self.tactic_performance.values())
        total_attempts = sum(p.attempts for p in self.tactic_performance.values())

        current_rate = total_successes / total_attempts if total_attempts > 0 else 0

        # Estimate improvement
        # In a real system, we'd track this over time
        improvement_trend = "unknown"

        if total_attempts > 500:
            # We have enough data to estimate
            improvement_trend = "improving" if current_rate > 0.1 else "stable"

        return {
            "total_attempts": total_attempts,
            "total_successes": total_successes,
            "current_success_rate": current_rate,
            "improvement_trend": improvement_trend,
            "tactics_learned": len(self.tactic_performance),
            "patterns_discovered": len(self.theorem_patterns),
            "goals_completed": len(self.completed_goals),
            "goals_active": len(self.research_goals),
        }

    def generate_adaptive_tactics(self, theorem_structure: str) -> str:
        """Generate optimized tactic sequence based on learning"""

        best_tactics = self.get_best_tactics(theorem_structure)

        if not best_tactics:
            # Default tactics
            return """
try { rfl }
try { simp }
try { omega }
try { ring }
"""

        # Build optimized tactic sequence
        tactics = []

        for tactic in best_tactics[:5]:
            tactics.append(f"try {{ {tactic} }}")

        return "\n".join(tactics) + "\n"

    def _save_learning(self):
        """Save learned data to disk"""

        data = {
            "tactics": {
                name: {
                    "attempts": perf.attempts,
                    "successes": perf.successes,
                    "avg_time": perf.avg_time,
                    "patterns": perf.patterns_successful,
                }
                for name, perf in self.tactic_performance.items()
            },
            "patterns": dict(self.theorem_patterns),
            "productive_patterns": dict(self.productive_patterns),
            "axioms": dict(self.axiom_effectiveness),
            "completed_goals": self.completed_goals[-100:],  # Keep last 100
            "stats": {
                "total_attempts": self.total_attempts,
                "total_successes": self.total_successes,
            },
        }

        with open(self.data_dir / "learning.json", "w") as f:
            json.dump(data, f, indent=2)

    def _load_learning(self):
        """Load previous learning from disk"""

        learning_file = self.data_dir / "learning.json"

        if not learning_file.exists():
            return

        try:
            with open(learning_file, "r") as f:
                data = json.load(f)

            # Restore tactic performance
            for name, perf_data in data.get("tactics", {}).items():
                perf = TacticPerformance(
                    tactic_name=name,
                    attempts=perf_data["attempts"],
                    successes=perf_data["successes"],
                    avg_time=perf_data["avg_time"],
                    patterns_successful=perf_data.get("patterns", {}),
                )
                self.tactic_performance[name] = perf

            # Restore patterns
            self.theorem_patterns = defaultdict(int, data.get("patterns", {}))
            self.productive_patterns = defaultdict(
                int, data.get("productive_patterns", {})
            )
            self.axiom_effectiveness = defaultdict(int, data.get("axioms", {}))

            # Restore stats
            self.total_attempts = data.get("stats", {}).get("total_attempts", 0)
            self.total_successes = data.get("stats", {}).get("total_successes", 0)

            print(f"[Learner] Loaded {self.total_attempts} previous proof attempts")

        except Exception as e:
            print(f"[Learner] Failed to load learning: {e}")


class AutonomousDiscoveryLoop:
    """
    Implements the full autonomous discovery loop (Tier 2).

    generate → test → prove → verify → store → learn → repeat
    """

    def __init__(self):
        self.learner = SelfImprovementEngine()
        self.running = False
        self.iteration = 0

    def run_continuous(self, max_iterations: int = None):
        """Run the discovery loop continuously"""

        self.running = True
        self.iteration = 0

        print("[AutoLoop] Starting autonomous discovery loop...")

        while self.running:
            self.iteration += 1

            # Generate research goal
            goal = self.learner.generate_research_goal()
            print(f"[AutoLoop] Iteration {self.iteration}: {goal['description']}")

            # Generate hypotheses based on goal
            # (In full implementation, this would use the goal to guide generation)

            # Attempt proof
            # (In full implementation, this would run actual proofs)

            # Learn from results
            # (This updates the learner's model)

            # Check improvement
            metrics = self.learner.get_improvement_metrics()

            if self.iteration % 10 == 0:
                print(f"[AutoLoop] Progress: {metrics}")

            # Check stopping conditions
            if max_iterations and self.iteration >= max_iterations:
                break

        print("[AutoLoop] Discovery loop stopped")

    def stop(self):
        """Stop the loop"""
        self.running = False


if __name__ == "__main__":
    # Test the learning system
    print("Testing self-improvement system...")

    learner = SelfImprovementEngine()

    # Simulate some proof attempts
    for i in range(50):
        learner.record_proof_attempt(
            theorem_hash=f"thm_{i}",
            theorem_structure="binary_operation",
            tactics_used=["rfl", "simp", "omega"],
            success=i < 20,  # 40% success rate
            time_taken=random.uniform(0.1, 1.0),
        )

    # Get metrics
    metrics = learner.get_improvement_metrics()
    print(f"\nMetrics: {metrics}")

    # Get best tactics
    best = learner.get_best_tactics("binary_operation")
    print(f"\nBest tactics: {best}")

    # Generate research goal
    goal = learner.generate_research_goal()
    print(f"\nResearch goal: {goal}")
