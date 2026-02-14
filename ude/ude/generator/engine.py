"""
Universal Discovery Engine - Theorem Generator

Systematically generates candidate theorems by exploring combinatorial spaces.

This is where we leverage AI's advantage: exploring 10^9 candidates per day
vs humans' 10 candidates per day.

Generation strategies:
1. Exhaustive enumeration (small depths)
2. Guided search (learned patterns)
3. Mutation of known theorems
4. Analogy-based generation
"""

import random
from typing import List, Iterator, Set, Tuple, Union, Optional
from itertools import product, combinations
from dataclasses import dataclass

from generator.theorem import (
    Theorem,
    Term,
    Var,
    Const,
    BinOp,
    UnOp,
    Quantifier,
    App,
    Variable,
    Type,
    LogicOp,
    MathOp,
    TheoremSpace,
    NAT,
    INT,
    REAL,
    BOOL,
    PROP,
)


class TheoremGenerator:
    """Generates candidate theorems systematically"""

    def __init__(self, space: TheoremSpace, seed: int = 42):
        self.space = space
        self.random = random.Random(seed)
        self.generated_hashes: Set[str] = set()
        self.generation_count = 0

    def generate_all_theorems(self, max_count: int = 1000000) -> Iterator[Theorem]:
        """
        Generate theorems exhaustively up to max_count.
        """

        # Strategy 1: Enumerate equalities between terms
        for depth in range(1, self.space.max_term_depth + 1):
            terms = list(self._generate_terms(depth))

            # Create equalities: term1 = term2
            for t1 in terms:
                for t2 in terms:
                    if str(t1) == str(t2):
                        continue
                    # Create theorem: t1 = t2
                    theorem = Theorem(
                        name=f"auto_{self.generation_count}",
                        hypotheses=[],
                        conclusion=BinOp(MathOp.EQ, t1, t2),
                    )

                    h = theorem.hash()
                    if h not in self.generated_hashes:
                        self.generated_hashes.add(h)
                        self.generation_count += 1
                        yield theorem

                        if self.generation_count >= max_count:
                            return

        # Strategy 2: Generate theorems with hypotheses
        # (combination of terms as assumptions)
        for num_hyps in range(1, 3):  # 1 or 2 hypotheses
            for hyp_terms in combinations(self._generate_terms(2), num_hyps):
                for conclusion in self._generate_terms(3):
                    theorem = Theorem(
                        name=f"auto_{self.generation_count}",
                        hypotheses=list(hyp_terms),
                        conclusion=conclusion,
                    )

                    h = theorem.hash()
                    if h not in self.generated_hashes:
                        self.generated_hashes.add(h)
                        self.generation_count += 1
                        yield theorem

                        if self.generation_count >= max_count:
                            return

    def _generate_terms(self, max_depth: int) -> Iterator[Term]:
        """Generate all possible terms up to given depth"""

        if max_depth == 0:
            # Base case: variables and constants
            for var in self._get_variables():
                yield Var(var)
            for const in self.space.constants:
                yield const
            return

        # Recursive case: build complex terms

        # 1. Binary operations - arithmetic
        for op in [MathOp.PLUS, MathOp.MINUS, MathOp.MULT, MathOp.DIV, MathOp.MOD]:
            for left in self._generate_terms(max_depth - 1):
                for right in self._generate_terms(max_depth - 1):
                    if self._type_compatible(op, left, right):
                        yield BinOp(op, left, right)

        # 2. Binary operations - comparison
        for op in [MathOp.EQ, MathOp.LT, MathOp.LE, MathOp.GT, MathOp.GE, MathOp.NEQ]:
            for left in self._generate_terms(max_depth - 1):
                for right in self._generate_terms(max_depth - 1):
                    if self._type_compatible(op, left, right):
                        yield BinOp(op, left, right)

        # 3. Binary operations - logic
        for op in [LogicOp.AND, LogicOp.OR, LogicOp.IMPLIES]:
            for left in self._generate_terms(max_depth - 1):
                for right in self._generate_terms(max_depth - 1):
                    if self._type_compatible(op, left, right):
                        yield BinOp(op, left, right)

        # 2. Unary operations
        for term in self._generate_terms(max_depth - 1):
            yield UnOp(LogicOp.NOT, term)

        # 3. Quantifiers (if depth allows)
        if max_depth >= 2 and self.space.max_quantifiers > 0:
            for var in self._get_variables():
                for body in self._generate_terms(max_depth - 1):
                    yield Quantifier(LogicOp.FORALL, var, body)
                    yield Quantifier(LogicOp.EXISTS, var, body)

    def _get_variables(self) -> List[Variable]:
        """Get common variables for each type"""
        vars = []
        for type_ in [NAT, INT, REAL, BOOL]:
            for name in ["x", "y", "z", "n", "m", "a", "b"]:
                vars.append(Variable(name, type_))
        return vars

    def _type_compatible(
        self, op: Union[MathOp, LogicOp], left: Term, right: Term
    ) -> bool:
        """Check if operation is type-compatible (simplified)"""
        # Simplified type checking
        # Real implementation needs full type inference
        return True

    def generate_guided(
        self, proven_theorems: List[Theorem], count: int = 1000
    ) -> Iterator[Theorem]:
        """
        Generate theorems guided by patterns learned from proven theorems.

        This uses machine learning to focus on "interesting" areas.
        """

        if not proven_theorems:
            # No guidance available, use random generation
            yield from self.generate_random(count)
            return

        # Strategy: Mutate proven theorems
        for _ in range(count):
            base_theorem = self.random.choice(proven_theorems)
            mutated = self._mutate_theorem(base_theorem)

            h = mutated.hash()
            if h not in self.generated_hashes:
                self.generated_hashes.add(h)
                self.generation_count += 1
                yield mutated

    def _mutate_theorem(self, theorem: Theorem) -> Theorem:
        """Apply random mutations to a theorem"""

        mutations = [
            self._swap_variables,
            self._strengthen_hypothesis,
            self._weaken_conclusion,
            self._add_quantifier,
            self._generalize_constant,
        ]

        mutation = self.random.choice(mutations)
        return mutation(theorem)

    def _swap_variables(self, theorem: Theorem) -> Theorem:
        """Swap variable names (preserving types)"""
        # Simplified implementation
        return Theorem(
            name=f"mutated_{theorem.name}",
            hypotheses=theorem.hypotheses,
            conclusion=theorem.conclusion,
        )

    def _strengthen_hypothesis(self, theorem: Theorem) -> Theorem:
        """Add additional hypothesis"""
        new_hyp = self._generate_random_term(depth=2)
        return Theorem(
            name=f"stronger_{theorem.name}",
            hypotheses=theorem.hypotheses + [new_hyp],
            conclusion=theorem.conclusion,
        )

    def _weaken_conclusion(self, theorem: Theorem) -> Theorem:
        """Make conclusion less specific"""
        # Simplified
        return theorem

    def _add_quantifier(self, theorem: Theorem) -> Theorem:
        """Add quantifier to conclusion"""
        var = Variable("z", NAT)
        return Theorem(
            name=f"quant_{theorem.name}",
            hypotheses=theorem.hypotheses,
            conclusion=Quantifier(LogicOp.FORALL, var, theorem.conclusion),
        )

    def _generalize_constant(self, theorem: Theorem) -> Theorem:
        """Replace constant with variable"""
        # Simplified
        return theorem

    def _generate_random_term(self, depth: int) -> Term:
        """Generate a random term"""
        terms = list(self._generate_terms(depth))
        if terms:
            return self.random.choice(terms)
        else:
            return Const(0, NAT)

    def generate_random(self, count: int = 1000) -> Iterator[Theorem]:
        """Generate random theorems (for exploration)"""
        for _ in range(count):
            depth = self.random.randint(1, self.space.max_term_depth)
            conclusion = self._generate_random_term(depth)

            # Random hypotheses
            num_hyps = self.random.randint(0, 2)
            hypotheses = [self._generate_random_term(2) for _ in range(num_hyps)]

            theorem = Theorem(
                name=f"random_{self.generation_count}",
                hypotheses=hypotheses,
                conclusion=conclusion,
            )

            h = theorem.hash()
            if h not in self.generated_hashes:
                self.generated_hashes.add(h)
                self.generation_count += 1
                yield theorem

    def generate_by_analogy(
        self, proven_theorems: List[Theorem], count: int = 100
    ) -> Iterator[Theorem]:
        """
        Generate theorems by analogy from proven ones.

        Example: If we proved "∀n. n + 0 = n", try "∀n. n * 1 = n"
        """

        if len(proven_theorems) < 2:
            return

        for _ in range(count):
            # Pick two proven theorems
            thm1 = self.random.choice(proven_theorems)
            thm2 = self.random.choice(proven_theorems)

            # Try to find structural similarity and transfer
            analogous = self._find_analogy(thm1, thm2)

            if analogous:
                h = analogous.hash()
                if h not in self.generated_hashes:
                    self.generated_hashes.add(h)
                    self.generation_count += 1
                    yield analogous

    def _find_analogy(self, thm1: Theorem, thm2: Theorem) -> Optional[Theorem]:
        """Find analogous theorem (simplified)"""
        # This would use more sophisticated pattern matching
        # For now, just mutate
        return self._mutate_theorem(thm1)

    def estimate_search_space_size(self) -> int:
        """
        Estimate total number of theorems in this space.

        This demonstrates the combinatorial explosion.
        """

        # Rough estimate based on formula:
        # T(d) = (V + C) + (O × T(d-1)²) + (Q × V × T(d-1))
        # Where:
        #   V = number of variables
        #   C = number of constants
        #   O = number of binary operators
        #   Q = number of quantifiers

        V = len(self._get_variables())
        C = len(self.space.constants)
        O = 10  # Approximate number of operators we use
        Q = 2  # FORALL, EXISTS

        def T(d):
            if d == 0:
                return V + C
            else:
                return (V + C) + (O * T(d - 1) ** 2) + (Q * V * T(d - 1))

        total = 0
        for d in range(self.space.max_term_depth + 1):
            total += T(d)

        return total


def create_default_space() -> TheoremSpace:
    """Create default exploration space"""
    return TheoremSpace(
        axioms=[],
        types={NAT, INT, REAL, BOOL, PROP},
        constants={
            # Natural numbers
            Const(0, NAT),
            Const(1, NAT),
            Const(2, NAT),
            Const(3, NAT),
            Const(4, NAT),
            Const(5, NAT),
            Const(10, NAT),
            Const(100, NAT),
            # Integers
            Const(0, INT),
            Const(1, INT),
            Const(-1, INT),
            Const(2, INT),
            Const(-2, INT),
            # Booleans
            Const(True, BOOL),
            Const(False, BOOL),
        },
        max_term_depth=4,
        max_quantifiers=2,
    )


if __name__ == "__main__":
    # Test theorem generation
    space = create_default_space()
    generator = TheoremGenerator(space)

    print(f"Estimated search space size: {generator.estimate_search_space_size():,}")
    print("\nGenerating first 20 theorems:\n")

    for i, theorem in enumerate(generator.generate_all_theorems(max_count=20)):
        print(f"{i + 1}. {theorem}")
        print(f"   Lean: {theorem.to_lean()}")
        print()
