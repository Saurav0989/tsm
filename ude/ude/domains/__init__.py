"""
Multi-Domain Theorem Generator

Integrates multiple mathematical domains for diverse theorem discovery.
"""

from typing import List, Dict, Iterator, Optional
import random
from dataclasses import dataclass

try:
    from domains.group_theory import GroupTheoremGenerator, get_group_theorems
    from domains.set_theory import SetTheoremGenerator, get_set_theorems
    from domains.propositional_logic import LogicTheoremGenerator, get_logic_theorems

    DOMAINS_AVAILABLE = True
except ImportError:
    DOMAINS_AVAILABLE = False
    get_group_theorems = None
    get_set_theorems = None
    get_logic_theorems = None


@dataclass
class DomainConfig:
    name: str
    weight: float  # Probability weight for selection
    enabled: bool = True


class MultiDomainGenerator:
    """Generates theorems across multiple mathematical domains"""

    DOMAINS = [
        DomainConfig("arithmetic", weight=0.2),
        DomainConfig("group_theory", weight=0.3),
        DomainConfig("set_theory", weight=0.25),
        DomainConfig("propositional_logic", weight=0.25),
    ]

    def __init__(self, seed: int = 42):
        self.random = random.Random(seed)
        self.domain_generators = {}

        if DOMAINS_AVAILABLE:
            self._init_domain_generators()

    def _init_domain_generators(self):
        """Initialize generators for each domain"""
        self.domain_generators = {
            "group_theory": get_group_theorems() if get_group_theorems else [],
            "set_theory": get_set_theorems() if get_set_theorems else [],
            "propositional_logic": get_logic_theorems() if get_logic_theorems else [],
        }

    def get_theorems_by_domain(self, domain: str, num: int = 10) -> List[Dict]:
        """Get theorems from a specific domain"""
        if domain == "group_theory" and get_group_theorems:
            return get_group_theorems()[:num]
        elif domain == "set_theory" and get_set_theorems:
            return get_set_theorems()[:num]
        elif domain == "propositional_logic" and get_logic_theorems:
            return get_logic_theorems()[:num]
        else:
            return []

    def sample_domain(self) -> str:
        """Sample a domain based on weights"""
        weights = [d.weight for d in self.DOMAINS if d.enabled]
        domains = [d.name for d in self.DOMAINS if d.enabled]
        return self.random.choices(domains, weights=weights, k=1)[0]

    def generate_cross_domain_theorems(self, num: int = 20) -> List[Dict]:
        """Generate theorems from multiple domains"""
        results = []

        for _ in range(num):
            domain = self.sample_domain()
            theorems = self.get_theorems_by_domain(domain, 1)
            if theorems:
                results.extend(theorems)

        return results

    def get_domain_stats(self) -> Dict:
        """Get statistics about domain coverage"""
        stats = {
            "total_domains": len(self.DOMAINS),
            "enabled_domains": sum(1 for d in self.DOMAINS if d.enabled),
            "domains": {},
        }

        for domain in self.DOMAINS:
            stats["domains"][domain.name] = {
                "enabled": domain.enabled,
                "weight": domain.weight,
                "theorems_available": len(self.domain_generators.get(domain.name, [])),
            }

        return stats


def create_multi_domain_generator(seed: int = 42) -> MultiDomainGenerator:
    """Factory function for multi-domain generator"""
    return MultiDomainGenerator(seed=seed)
