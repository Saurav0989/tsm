"""
Guided Theorem Generator using Learned Patterns

Uses self-improvement engine insights to guide theorem generation.
"""

from typing import List, Dict, Optional, Set
import random
from pathlib import Path


class GuidedGenerator:
    """Guides theorem generation using learned patterns"""

    def __init__(self, learner=None, base_generator=None):
        self.learner = learner
        self.base_generator = base_generator
        self.generated: Set[str] = set()

    def generate_guided_theorems(self, num: int = 50) -> List[Dict]:
        """Generate theorems guided by learned patterns"""
        results = []

        if not self.learner:
            return self._generate_random(num)

        productive_patterns = self.learner.get_productive_patterns(
            min_success_rate=0.05
        )

        if productive_patterns:
            for pattern in productive_patterns[:num]:
                theorem = self._theorem_from_pattern(pattern)
                if theorem:
                    results.append(theorem)

        remaining = num - len(results)
        if remaining > 0:
            results.extend(self._generate_random(remaining))

        return results[:num]

    def _theorem_from_pattern(self, pattern: str) -> Optional[Dict]:
        """Convert a learned pattern to a theorem"""
        if "group_theory" in pattern or "set_theory" in pattern:
            return None

        return {
            "name": f"guided_{hash(pattern) % 100000}",
            "hypotheses": [],
            "conclusion": pattern,
            "proof_steps": 2,
            "domain": "guided",
            "difficulty": "medium",
        }

    def _generate_random(self, num: int) -> List[Dict]:
        """Generate random theorems when no learning available"""
        theorems = []

        domains = ["group_theory", "set_theory", "propositional_logic"]

        for i in range(num):
            domain = random.choice(domains)
            theorems.append(
                {
                    "name": f"random_{i}",
                    "hypotheses": [],
                    "conclusion": f"Random theorem {i}",
                    "proof_steps": random.randint(1, 3),
                    "domain": domain,
                    "difficulty": "medium",
                }
            )

        return theorems

    def get_learner_stats(self) -> Dict:
        """Get statistics from learner"""
        if not self.learner:
            return {"status": "no_learner"}

        return {
            "total_attempts": self.learner.total_attempts,
            "total_successes": self.learner.total_successes,
            "productive_patterns": len(self.learner.get_productive_patterns()),
            "effective_axioms": len(self.learner.get_effective_axioms()),
        }


def create_guided_generator(learner=None) -> GuidedGenerator:
    """Factory for guided generator"""
    return GuidedGenerator(learner=learner)


class AdaptiveDomainSelector:
    """Selects domains based on success rates"""

    def __init__(self, learner=None):
        self.learner = learner
        self.domain_success: Dict[str, float] = {
            "arithmetic": 0.15,
            "group_theory": 0.30,
            "set_theory": 0.25,
            "propositional_logic": 0.35,
        }

    def select_domain(self) -> str:
        """Select domain based on weighted success rates"""
        if self.learner:
            domain_stats = self._get_domain_stats()
            if domain_stats:
                weights = list(domain_stats.values())
                domains = list(domain_stats.keys())
                return random.choices(domains, weights=weights, k=1)[0]

        return random.choices(
            list(self.domain_success.keys()),
            weights=list(self.domain_success.values()),
            k=1,
        )[0]

    def _get_domain_stats(self) -> Dict[str, float]:
        """Get domain success rates from learner"""
        return self.domain_success

    def update_domain_success(self, domain: str, success: bool):
        """Update success rate for a domain"""
        current = self.domain_success.get(domain, 0.5)
        if success:
            self.domain_success[domain] = min(0.95, current + 0.02)
        else:
            self.domain_success[domain] = max(0.05, current - 0.01)


def create_domain_selector(learner=None) -> AdaptiveDomainSelector:
    """Factory for adaptive domain selector"""
    return AdaptiveDomainSelector(learner=learner)
