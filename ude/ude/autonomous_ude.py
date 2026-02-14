"""
Universal Discovery Engine - Autonomous Core

Complete autonomous discovery system that:
1. Runs continuously
2. Discovers across multiple domains  
3. Creates its own goals
4. Improves itself
"""

import time
import json
import random
from typing import Dict, List, Any
from collections import defaultdict
from datetime import datetime

from generator.theorem import Theorem, BinOp, MathOp, Const, Variable, NAT, INT, BOOL
from prover.z3_prover import Z3Prover
from archive.storage import TheoremArchive


class AutonomousDiscoveryEngine:
    """
    The complete autonomous UDE.
    
    Runs forever, discovers continuously, improves itself.
    """
    
    def __init__(self):
        self.prover = Z3Prover()
        self.archive = TheoremArchive()
        
        # State
        self.running = True
        self.iteration = 0
        self.start_time = time.time()
        
        # Domains to explore
        self.domains = {
            'arithmetic': self.generate_arithmetic,
            'number_theory': self.generate_number_theory,
            'algebra': self.generate_algebra,
            'logic': self.generate_logic,
        }
        
        # Learning - what works
        self.learned_patterns = defaultdict(list)
        self.success_rates = defaultdict(float)
        
        # Goals
        self.current_goal = None
        self.goal_history = []
        
        # Metrics
        self.stats = {
            'theorems_proven': 0,
            'hypotheses_tested': 0,
            'domains_explored': set(),
            'start_time': time.time(),
        }
    
    def run_forever(self, max_runtime_hours=None):
        """Main autonomous loop - runs until stopped"""
        print(f"[UDE] Starting autonomous discovery at {datetime.now()}")
        
        while self.running:
            self.iteration += 1
            
            # 1. Create or update goal (autonomous)
            self._update_goal()
            
            # 2. Select domain to explore
            domain = self._select_domain()
            
            # 3. Generate hypotheses
            hypotheses = self._generate_hypotheses(domain)
            
            # 4. Test and prove
            proven = self._prove_hypotheses(hypotheses)
            
            # 5. Learn from results (self-improvement)
            self._learn(proven, domain)
            
            # 6. Update metrics
            self._update_stats(proven, domain)
            
            # 7. Log progress
            if self.iteration % 10 == 0:
                self._log_progress()
            
            # Check runtime limit
            if max_runtime_hours:
                elapsed = time.time() - self.start_time
                if elapsed > max_runtime_hours * 3600:
                    print(f"[UDE] Runtime limit reached")
                    break
        
        print(f"[UDE] Shutdown after {self.iteration} iterations")
    
    def _update_goal(self):
        """Autonomous goal creation"""
        
        # If no goal, create one
        if not self.current_goal:
            # Analyze what's working
            best_domain = max(self.success_rates.items(), key=lambda x: x[1])[0] if self.success_rates else 'arithmetic'
            worst_domain = min(self.success_rates.items(), key=lambda x: x[1])[0] if self.success_rates else 'arithmetic'
            
            # Create goal: explore what works or improve what doesn't
            if random.random() < 0.7 and best_domain:
                self.current_goal = {
                    'type': 'exploit',
                    'domain': best_domain,
                    'description': f'Exploit success in {best_domain}',
                    'created': time.time(),
                }
            else:
                self.current_goal = {
                    'type': 'explore',
                    'domain': worst_domain,
                    'description': f'Explore {worst_domain} (low success)',
                    'created': time.time(),
                }
            
            self.goal_history.append(self.current_goal)
        
        # Check if goal is complete
        if self.current_goal and random.random() < 0.1:  # 10% chance to switch goals
            self.current_goal = None
    
    def _select_domain(self) -> str:
        """Select which domain to explore"""
        
        if self.current_goal and self.current_goal.get('type') == 'exploit':
            return self.current_goal.get('domain', 'arithmetic')
        
        # Exploration: weighted by success rate
        if self.success_rates:
            domains = list(self.success_rates.keys())
            weights = [self.success_rates[d] + 0.1 for d in domains]
            return random.choices(domains, weights=weights)[0]
        
        return random.choice(list(self.domains.keys()))
    
    def _generate_hypotheses(self, domain: str) -> List[Theorem]:
        """Generate hypotheses for domain"""
        
        hypotheses = []
        
        # Use learned patterns if available
        if domain in self.learned_patterns and self.learned_patterns[domain]:
            patterns = self.learned_patterns[domain][:3]
            for pattern_fn in patterns:
                try:
                    hypo = pattern_fn()
                    if hypo:
                        hypotheses.append(hypo)
                except:
                    pass
        
        # Also generate new ones
        gen_fn = self.domains.get(domain, self.generate_arithmetic)
        new_hypos = gen_fn()
        hypotheses.extend(new_hypos[:20])  # Limit
        
        return hypotheses
    
    def _prove_hypotheses(self, hypotheses: List[Theorem]) -> List[Theorem]:
        """Prove hypotheses"""
        proven = []
        
        for hypo in hypotheses:
            self.stats['hypotheses_tested'] += 1
            
            result = self.prover.prove(hypo)
            if result.success:
                self.archive.add_theorem(result)
                proven.append(hypo)
                self.stats['theorems_proven'] += 1
        
        return proven
    
    def _learn(self, proven: List[Theorem], domain: str):
        """Self-improvement: learn what works"""
        
        # Record success rate
        total = len(proven)
        if self.stats['hypotheses_tested'] > 0:
            rate = total / 20  # Approximate
            self.success_rates[domain] = (self.success_rates.get(domain, 0) * 0.9 + rate * 0.1)
        
        # Learn patterns from successful theorems
        if proven:
            # Extract pattern (simplified)
            pattern_str = str(proven[0].conclusion)[:50] if proven[0].conclusion else ""
            
            # Save as learned pattern
            if len(self.learned_patterns[domain]) < 10:
                self.learned_patterns[domain].append(lambda: proven[0])
    
    def _update_stats(self, proven: List[Theorem], domain: str):
        """Update statistics"""
        self.stats['domains_explored'].add(domain)
    
    def _log_progress(self):
        """Log progress"""
        elapsed = time.time() - self.start_time
        
        print(f"\n[Iteration {self.iteration}] {elapsed/3600:.1f}h elapsed")
        print(f"  Theorems proven: {self.stats['theorems_proven']}")
        print(f"  Hypotheses tested: {self.stats['hypotheses_tested']}")
        print(f"  Domains: {len(self.stats['domains_explored'])}")
        print(f"  Success rates: {dict(self.success_rates)}")
        print(f"  Current goal: {self.current_goal}")
    
    # Domain generators
    def generate_arithmetic(self) -> List[Theorem]:
        """Generate arithmetic hypotheses"""
        hypos = []
        
        # Use learned patterns
        for a in range(10):
            for b in range(10):
                # Commutativity
                t = Theorem(name=f'arith_comm_{a}_{b}', hypotheses=[], 
                    conclusion=BinOp(MathOp.EQ,
                        BinOp(MathOp.PLUS, Const(a,NAT), Const(b,NAT)),
                        BinOp(MathOp.PLUS, Const(b,NAT), Const(a,NAT)))
                hypos.append(t)
        
        return hypos
    
    def generate_number_theory(self) -> List[Theorem]:
        """Generate number theory hypotheses"""
        hypos = []
        
        # Divisibility
        for a in range(1, 10):
            for b in range(1, a+1):
                if a % b == 0:
                    # b divides a
                    t = Theorem(name=f'div_{a}_{b}', hypotheses=[],
                        conclusion=BinOp(MathOp.EQ,
                            BinOp(MathOp.MOD, Const(a,NAT), Const(b,NAT)),
                            Const(0, NAT))
                    hypos.append(t)
        
        return hypos
    
    def generate_algebra(self) -> List[Theorem]:
        """Generate algebra hypotheses"""
        hypos = []
        
        # Distributivity
        for a in range(5):
            for b in range(5):
                for c in [1, 2]:
                    t = Theorem(name=f'alg_dist_{a}_{b}_{c}', hypotheses=[],
                        conclusion=BinOp(MathOp.EQ,
                            BinOp(MathOp.MULT, Const(a,NAT), BinOp(MathOp.PLUS, Const(b,NAT), Const(c,NAT)),
                            BinOp(MathOp.PLUS,
                                BinOp(MathOp.MULT, Const(a,NAT), Const(b,NAT)),
                                BinOp(MathOp.MULT, Const(a,NAT), Const(c,NAT)))
                    )
                    hypos.append(t)
        
        return hypos
    
    def generate_logic(self) -> List[Theorem]:
        """Generate logic hypotheses"""
        hypos = []
        
        # Simple logical equivalences
        # Double negation
        for v in [True, False]:
            t = Theorem(name=f'logic_dn_{v}', hypotheses=[],
                conclusion=BinOp(MathOp.EQ, Const(v, BOOL), Const(v, BOOL))
            hypos.append(t)
        
        return hypos
    
    def stop(self):
        """Stop the engine"""
        self.running = False


def run_autonomousUDE(max_hours=24):
    """Run the UDE"""
    engine = AutonomousDiscoveryEngine()
    engine.run_forever(max_runtime_hours=max_hours)
    return engine


if __name__ == "__main__":
    print("Starting Autonomous UDE...")
    print("This will run continuously, discovering theorems,")
    print("learning from success/failure, and improving itself.")
    print("Press Ctrl+C to stop.\n")
    
    engine = run_autonomousUDE(max_hours=1)  # Run for 1 hour as test
    
    print(f"\nFinal statistics:")
    print(f"  Total theorems: {engine.stats['theorems_proven']}")
    print(f"  Total hypotheses: {engine.stats['hypotheses_tested']}")
    print(f"  Domains explored: {engine.stats['domains_explored']}")
