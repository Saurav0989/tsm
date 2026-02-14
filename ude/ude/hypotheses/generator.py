"""
Universal Discovery Engine - Hypothesis Generator

Generates mathematical conjectures from axiom systems.
This is what creates the "hypotheses" to test in each mathematical universe.
"""

from typing import List, Iterator, Set, Dict
from dataclasses import dataclass
import itertools

# Import from existing code
import sys
import os

sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from generator.theorem import (
    Theorem,
    Term,
    BinOp,
    UnOp,
    Quantifier,
    Var,
    Const,
    MathOp,
    LogicOp,
    Variable,
    Type,
    NAT,
    INT,
    REAL,
    BOOL,
)


@dataclass
class Hypothesis:
    """A mathematical conjecture"""

    formula: str
    variables: List[Variable]
    free_variables: List[Variable]  # Variables not bound by quantifiers
    is_equation: bool
    is_inequality: bool
    complexity: int  # Term depth

    def __str__(self):
        return self.formula


class HypothesisGenerator:
    """Generates conjectures from available primitives"""

    def __init__(self, max_complexity: int = 5, max_free_vars: int = 3):
        self.max_complexity = max_complexity
        self.max_free_vars = max_free_vars

        # Define available operations
        self.operations = [
            MathOp.PLUS,
            MathOp.MINUS,
            MathOp.MULT,
            MathOp.EQ,
            MathOp.LT,
            MathOp.LE,
        ]

        self.logic_ops = [
            LogicOp.AND,
            LogicOp.OR,
            LogicOp.IMPLIES,
        ]

        # Variable pool
        self.variable_pool = []
        for name in ["x", "y", "z", "u", "v", "w", "a", "b", "c", "n", "m", "p", "q"]:
            for type_ in [NAT, INT, REAL]:
                self.variable_pool.append(Variable(name, type_))

    def generate_all(self) -> Iterator[Hypothesis]:
        """Generate all possible hypotheses up to max complexity"""

        # Generate equations
        for complexity in range(1, self.max_complexity + 1):
            # Generate terms of this complexity
            terms = list(self._generate_terms(complexity))

            # Create equations between terms
            for left, right in itertools.combinations(terms, 2):
                # Equality
                yield Hypothesis(
                    formula=f"{left} = {right}",
                    variables=list(
                        set(self._get_variables(left) + self._get_variables(right))
                    ),
                    free_variables=list(
                        set(self._get_variables(left) + self._get_variables(right))
                    ),
                    is_equation=True,
                    is_inequality=False,
                    complexity=complexity,
                )

                # Inequality
                for op in [MathOp.LT, MathOp.LE, MathOp.GT, MathOp.GE]:
                    yield Hypothesis(
                        formula=f"{left} {op.value} {right}",
                        variables=list(
                            set(self._get_variables(left) + self._get_variables(right))
                        ),
                        free_variables=list(
                            set(self._get_variables(left) + self._get_variables(right))
                        ),
                        is_equation=False,
                        is_inequality=True,
                        complexity=complexity,
                    )

            # Generate quantified hypotheses
            if complexity >= 2:
                for term in terms:
                    vars_in_term = self._get_variables(term)

                    if len(vars_in_term) >= 1:
                        # Universal quantification
                        for var in vars_in_term[: self.max_free_vars]:
                            yield Hypothesis(
                                formula=f"∀{var}: {term}",
                                variables=vars_in_term,
                                free_variables=[v for v in vars_in_term if v != var],
                                is_equation=False,
                                is_inequality=False,
                                complexity=complexity + 1,
                            )

                        # Existential quantification
                        for var in vars_in_term[: self.max_free_vars]:
                            yield Hypothesis(
                                formula=f"∃{var}: {term}",
                                variables=vars_in_term,
                                free_variables=[v for v in vars_in_term if v != var],
                                is_equation=False,
                                is_inequality=False,
                                complexity=complexity + 1,
                            )

    def _generate_terms(self, complexity: int) -> Iterator[str]:
        """Generate all terms of given complexity"""

        if complexity == 0:
            # Variables
            for var in self.variable_pool[:10]:  # Limit
                yield str(var)

            # Constants
            for val in [0, 1, 2, 3, 5, 10, 100]:
                yield str(val)

        elif complexity >= 1:
            # Binary operations
            for left in self._generate_terms(complexity - 1):
                for right in self._generate_terms(complexity - 1):
                    for op in self.operations:
                        yield f"({left} {op.value} {right})"

        # Generate more variations
        if complexity >= 2:
            # Nested operations
            for left in self._generate_terms(1):
                for right in self._generate_terms(1):
                    yield f"({left} + {right})"
                    yield f"({left} * {right})"

    def _get_variables(self, term: str) -> List[Variable]:
        """Extract variables from term string"""
        vars_found = []

        for var in self.variable_pool:
            if var.name in term:
                vars_found.append(var)

        return list(set(vars_found))

    def estimate_count(self) -> Dict[str, int]:
        """Estimate number of hypotheses to generate"""

        # Rough estimates
        terms_per_level = {
            0: 15,  # vars + consts
            1: 100,  # binary ops
            2: 1000,
            3: 10000,
            4: 100000,
            5: 1000000,
        }

        total = 0
        for level in range(1, self.max_complexity + 1):
            terms = terms_per_level.get(level, 10000)
            # Equations between terms
            equations = terms * terms // 2
            # Quantified versions
            quantified = equations * level * 2

            total += equations + quantified

        return {
            "estimated_terms": sum(terms_per_level.values()),
            "estimated_hypotheses": total,
            "max_complexity": self.max_complexity,
        }


class UniverseExplorer:
    """
    Explores a mathematical universe defined by axioms.
    Generates and tests hypotheses within that universe.
    """

    def __init__(self, axiom_system_name: str):
        self.axiom_system_name = axiom_system_name
        self.hypothesis_generator = HypothesisGenerator(max_complexity=4)
        self.theorems_found = []
        self.disproofs_found = []

    def explore(self, max_hypotheses: int = 1000000) -> Dict:
        """Explore this universe"""

        explored = 0
        theorems = 0
        disproofs = 0

        print(f"[Universe {self.axiom_system_name}] Starting exploration...")

        for hypothesis in self.hypothesis_generator.generate_all():
            if explored >= max_hypotheses:
                break

            explored += 1

            # In a real implementation, we would:
            # 1. Translate hypothesis to formal language
            # 2. Attempt proof using the axiom system
            # 3. If proven, add to theorems
            # 4. If disproven (counterexample found), add to disproofs

        return {
            "axiom_system": self.axiom_system_name,
            "hypotheses_explored": explored,
            "theorems_found": theorems,
            "disproofs_found": disproofs,
        }

    def get_statistics(self) -> Dict:
        """Get exploration statistics"""
        return {
            "theorems": len(self.theorems_found),
            "disproofs": len(self.disproofs_found),
            "total_explored": len(self.theorems_found) + len(self.disproofs_found),
        }


def estimate_total_search_space():
    """Estimate the total search space for all universes"""

    from axioms.generator import AxiomSystemGenerator

    # Get axiom systems
    axiom_gen = AxiomSystemGenerator(max_axioms=10)
    axiom_limits = axiom_gen.get_theoretical_limits()

    # Get hypothesis counts
    hyp_gen = HypothesisGenerator(max_complexity=5)
    hyp_counts = hyp_gen.estimate_count()

    print("=" * 60)
    print("TOTAL SEARCH SPACE ESTIMATION")
    print("=" * 60)

    print(f"\nAxiom Systems: {axiom_limits['total_systems']:,}")
    print(f"Hypotheses per system: ~{hyp_counts['estimated_hypotheses']:,}")

    total = axiom_limits["total_systems"] * hyp_counts["estimated_hypotheses"]

    print(f"\nTotal hypothesis-space: {total:,}")
    print(f"In scientific notation: {total:.2e}")

    print(f"\nAt 10^9 tests/second:")
    print(f"  Time to explore: {total / 1e9 / 3600 / 24 / 365:.2f} years")

    print(f"\nThis is why humans cannot do this:")
    print(f"  - Total atoms in universe: ~10^80")
    print(f"  - Our search space: {total:.2e}")
    print(f"  - We're exploring POSSIBLE UNIVERSES, not atoms!")


if __name__ == "__main__":
    # Estimate search space
    estimate_total_search_space()
