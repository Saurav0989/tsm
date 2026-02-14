"""
Universal Discovery Engine - Neural Learner

Learns patterns from proven theorems to guide future generation.

This is where AI's advantage scales exponentially:
- Humans learn from ~100 theorems
- AI can learn from millions

The learner extracts patterns like:
- Which term structures are provable
- Which tactics work for which patterns
- Analogies between theorems
"""

import numpy as np
from typing import List, Dict, Tuple, Optional
from collections import defaultdict, Counter

import sys
sys.path.append('/home/claude/ude')

from generator.theorem import Theorem, Term, BinOp, UnOp, Quantifier, Var, Const

class PatternLearner:
    """
    Learns patterns from proven theorems.
    
    Strategies:
    1. Structural patterns (what shapes of theorems are provable?)
    2. Tactic success patterns (which proofs work where?)
    3. Analogical reasoning (find similar theorems)
    """
    
    def __init__(self):
        # Pattern statistics
        self.term_structure_counts = Counter()
        self.operator_sequences = Counter()
        self.successful_patterns = []
        
        # Embeddings for similarity search
        self.theorem_embeddings = {}
        
        # Learned heuristics
        self.provability_score = {}  # Pattern -> probability of being provable
    
    def learn_from_theorems(self, proven_theorems: List[Theorem]):
        """
        Extract patterns from proven theorems.
        
        This updates the learner's knowledge base.
        """
        print(f"[Learner] Learning from {len(proven_theorems)} proven theorems...")
        
        for theorem in proven_theorems:
            # Extract structural patterns
            structure = self._extract_structure(theorem.conclusion)
            self.term_structure_counts[structure] += 1
            
            # Extract operator sequences
            ops = self._extract_operator_sequence(theorem.conclusion)
            self.operator_sequences[tuple(ops)] += 1
            
            # Store as successful pattern
            self.successful_patterns.append({
                'theorem': theorem,
                'structure': structure,
                'depth': self._term_depth(theorem.conclusion),
                'num_quantifiers': self._count_quantifiers(theorem.conclusion),
            })
        
        # Update provability scores
        self._update_provability_scores()
        
        print(f"[Learner] Learned {len(self.term_structure_counts)} structural patterns")
        print(f"[Learner] Top 5 provable structures:")
        for struct, count in self.term_structure_counts.most_common(5):
            print(f"  {struct}: {count} theorems")
    
    def _extract_structure(self, term: Term) -> str:
        """
        Extract structural pattern from a term.
        
        Example: (x + y) = (y + x) -> "BinOp(=, BinOp(+, Var, Var), BinOp(+, Var, Var))"
        """
        if isinstance(term, Var):
            return "Var"
        elif isinstance(term, Const):
            return f"Const({term.type})"
        elif isinstance(term, BinOp):
            left = self._extract_structure(term.left)
            right = self._extract_structure(term.right)
            return f"BinOp({term.op.value}, {left}, {right})"
        elif isinstance(term, UnOp):
            inner = self._extract_structure(term.term)
            return f"UnOp({term.op.value}, {inner})"
        elif isinstance(term, Quantifier):
            body = self._extract_structure(term.body)
            return f"Quant({term.op.value}, {body})"
        else:
            return "Unknown"
    
    def _extract_operator_sequence(self, term: Term) -> List[str]:
        """Extract sequence of operators"""
        ops = []
        
        if isinstance(term, BinOp):
            ops.append(term.op.value)
            ops.extend(self._extract_operator_sequence(term.left))
            ops.extend(self._extract_operator_sequence(term.right))
        elif isinstance(term, UnOp):
            ops.append(term.op.value)
            ops.extend(self._extract_operator_sequence(term.term))
        elif isinstance(term, Quantifier):
            ops.append(term.op.value)
            ops.extend(self._extract_operator_sequence(term.body))
        
        return ops
    
    def _term_depth(self, term: Term) -> int:
        """Calculate depth of term tree"""
        if isinstance(term, (Var, Const)):
            return 1
        elif isinstance(term, BinOp):
            return 1 + max(self._term_depth(term.left), self._term_depth(term.right))
        elif isinstance(term, UnOp):
            return 1 + self._term_depth(term.term)
        elif isinstance(term, Quantifier):
            return 1 + self._term_depth(term.body)
        else:
            return 1
    
    def _count_quantifiers(self, term: Term) -> int:
        """Count number of quantifiers"""
        count = 0
        
        if isinstance(term, Quantifier):
            count = 1 + self._count_quantifiers(term.body)
        elif isinstance(term, BinOp):
            count = self._count_quantifiers(term.left) + self._count_quantifiers(term.right)
        elif isinstance(term, UnOp):
            count = self._count_quantifiers(term.term)
        
        return count
    
    def _update_provability_scores(self):
        """
        Update probability scores for different patterns.
        
        Patterns that appear more often in proven theorems get higher scores.
        """
        total_patterns = sum(self.term_structure_counts.values())
        
        for pattern, count in self.term_structure_counts.items():
            # Probability estimate (with smoothing)
            self.provability_score[pattern] = (count + 1) / (total_patterns + len(self.term_structure_counts))
    
    def score_theorem(self, theorem: Theorem) -> float:
        """
        Score how likely a theorem is to be provable.
        
        Higher score = more likely to prove.
        This guides generation toward "interesting" theorems.
        """
        structure = self._extract_structure(theorem.conclusion)
        
        # Base score from pattern frequency
        base_score = self.provability_score.get(structure, 0.01)
        
        # Bonus for simple theorems
        depth = self._term_depth(theorem.conclusion)
        depth_bonus = 1.0 / (1.0 + depth * 0.2)
        
        # Penalty for too many quantifiers (harder to prove)
        num_quants = self._count_quantifiers(theorem.conclusion)
        quant_penalty = 1.0 / (1.0 + num_quants * 0.5)
        
        return base_score * depth_bonus * quant_penalty
    
    def suggest_mutations(self, theorem: Theorem, count: int = 5) -> List[Theorem]:
        """
        Suggest mutations based on learned patterns.
        
        This uses analogical reasoning from successful theorems.
        """
        suggestions = []
        
        # Find similar proven theorems
        similar = self._find_similar_theorems(theorem)
        
        # Apply transformations from similar theorems
        for similar_thm in similar[:count]:
            # TODO: Implement structural transformation
            suggestions.append(theorem)
        
        return suggestions
    
    def _find_similar_theorems(self, theorem: Theorem, top_k: int = 10) -> List[Theorem]:
        """Find theorems with similar structure"""
        target_struct = self._extract_structure(theorem.conclusion)
        
        # Find theorems with same or similar structure
        similar = []
        for pattern_info in self.successful_patterns:
            if pattern_info['structure'] == target_struct:
                similar.append(pattern_info['theorem'])
        
        return similar[:top_k]
    
    def get_statistics(self) -> Dict:
        """Get learning statistics"""
        return {
            'patterns_learned': len(self.term_structure_counts),
            'total_proven_analyzed': sum(self.term_structure_counts.values()),
            'most_common_pattern': self.term_structure_counts.most_common(1)[0] if self.term_structure_counts else None,
            'avg_depth': np.mean([p['depth'] for p in self.successful_patterns]) if self.successful_patterns else 0,
        }

class TheoremEmbedding:
    """
    Embed theorems into vector space for similarity search.
    
    This uses neural embeddings to find analogies.
    """
    
    def __init__(self, embedding_dim: int = 128):
        self.embedding_dim = embedding_dim
        self.embeddings = {}
    
    def embed(self, theorem: Theorem) -> np.ndarray:
        """
        Convert theorem to vector embedding.
        
        Similar theorems have similar embeddings.
        """
        # Simplified embedding (real implementation would use learned embeddings)
        
        # Extract features
        features = []
        
        # Structural features
        term = theorem.conclusion
        features.append(self._term_depth(term))
        features.append(self._count_operators(term))
        features.append(self._count_quantifiers(term))
        features.append(self._count_variables(term))
        
        # Operator presence (one-hot)
        ops = self._extract_operators(term)
        for op_name in ['+', '-', '*', '=', '<', '∧', '∨', '→', '∀', '∃']:
            features.append(1.0 if op_name in ops else 0.0)
        
        # Pad or truncate to embedding_dim
        while len(features) < self.embedding_dim:
            features.append(0.0)
        features = features[:self.embedding_dim]
        
        embedding = np.array(features, dtype=np.float32)
        
        # Normalize
        norm = np.linalg.norm(embedding)
        if norm > 0:
            embedding = embedding / norm
        
        return embedding
    
    def _term_depth(self, term: Term) -> float:
        """Calculate depth"""
        if isinstance(term, (Var, Const)):
            return 1.0
        elif isinstance(term, BinOp):
            return 1.0 + max(self._term_depth(term.left), self._term_depth(term.right))
        elif isinstance(term, UnOp):
            return 1.0 + self._term_depth(term.term)
        elif isinstance(term, Quantifier):
            return 1.0 + self._term_depth(term.body)
        else:
            return 1.0
    
    def _count_operators(self, term: Term) -> float:
        """Count operators"""
        count = 0.0
        if isinstance(term, (BinOp, UnOp)):
            count = 1.0
        
        if isinstance(term, BinOp):
            count += self._count_operators(term.left) + self._count_operators(term.right)
        elif isinstance(term, UnOp):
            count += self._count_operators(term.term)
        elif isinstance(term, Quantifier):
            count += self._count_operators(term.body)
        
        return count
    
    def _count_quantifiers(self, term: Term) -> float:
        """Count quantifiers"""
        count = 0.0
        if isinstance(term, Quantifier):
            count = 1.0 + self._count_quantifiers(term.body)
        elif isinstance(term, BinOp):
            count = self._count_quantifiers(term.left) + self._count_quantifiers(term.right)
        elif isinstance(term, UnOp):
            count = self._count_quantifiers(term.term)
        return count
    
    def _count_variables(self, term: Term) -> float:
        """Count variables"""
        if isinstance(term, Var):
            return 1.0
        elif isinstance(term, BinOp):
            return self._count_variables(term.left) + self._count_variables(term.right)
        elif isinstance(term, UnOp):
            return self._count_variables(term.term)
        elif isinstance(term, Quantifier):
            return self._count_variables(term.body)
        else:
            return 0.0
    
    def _extract_operators(self, term: Term) -> List[str]:
        """Extract all operators"""
        ops = []
        if isinstance(term, BinOp):
            ops.append(term.op.value)
            ops.extend(self._extract_operators(term.left))
            ops.extend(self._extract_operators(term.right))
        elif isinstance(term, UnOp):
            ops.append(term.op.value)
            ops.extend(self._extract_operators(term.term))
        elif isinstance(term, Quantifier):
            ops.append(term.op.value)
            ops.extend(self._extract_operators(term.body))
        return ops
    
    def similarity(self, emb1: np.ndarray, emb2: np.ndarray) -> float:
        """Compute cosine similarity"""
        return float(np.dot(emb1, emb2))

if __name__ == "__main__":
    # Test learner
    from generator.theorem import example_commutativity
    
    learner = PatternLearner()
    
    # Create some test theorems
    theorems = [example_commutativity() for _ in range(10)]
    
    learner.learn_from_theorems(theorems)
    
    stats = learner.get_statistics()
    print(f"\nLearning statistics: {stats}")
    
    # Score a theorem
    test_theorem = example_commutativity()
    score = learner.score_theorem(test_theorem)
    print(f"\nProvability score: {score:.4f}")
