"""
Propositional Logic Domain for UDE

Enables discovery of logical equivalences and tautologies.
"""

from typing import List, Dict
from dataclasses import dataclass
from enum import Enum


class LogicAxiom(Enum):
    IDEMPOTENT = "idempotent"
    COMMUTATIVE = "commutative"
    ASSOCIATIVE = "associative"
    DISTRIBUTIVE = "distributive"
    ABSORPTION = "absorption"
    DE_MORGAN = "de_morgan"
    IDENTITY = "identity"
    ANNIHILATOR = "annihilator"


LOGIC_LAWS = {
    "and_idempotent": ("p ∧ p ↔ p", 1),
    "or_idempotent": ("p ∨ p ↔ p", 1),
    "and_comm": ("p ∧ q ↔ q ∧ p", 1),
    "or_comm": ("p ∨ q ↔ q ∨ p", 1),
    "and_assoc": ("(p ∧ q) ∧ r ↔ p ∧ (q ∧ r)", 1),
    "or_assoc": ("(p ∨ q) ∨ r ↔ p ∨ (q ∨ r)", 1),
    "and_or_dist": ("p ∧ (q ∨ r) ↔ (p ∧ q) ∨ (p ∧ r)", 2),
    "or_and_dist": ("p ∨ (q ∧ r) ↔ (p ∨ q) ∧ (p ∨ r)", 2),
    "and_absorb": ("p ∧ (p ∨ r) ↔ p", 2),
    "or_absorb": ("p ∨ (p ∧ r) ↔ p", 2),
    "de_morgan_and": ("¬(p ∧ q) ↔ ¬p ∨ ¬q", 2),
    "de_morgan_or": ("¬(p ∨ q) ↔ ¬p ∧ ¬q", 2),
    "and_true": ("p ∧ True ↔ p", 1),
    "or_false": ("p ∨ False ↔ p", 1),
    "and_false": ("p ∧ False ↔ False", 1),
    "or_true": ("p ∨ True ↔ True", 1),
    "implication": ("p → q ↔ ¬p ∨ q", 2),
    "contrapositive": ("p → q ↔ ¬q → ¬p", 2),
    "export_law": ("p ∧ (q ∧ r) ↔ (p ∧ q) ∧ r", 2),
    "import_law": ("p → (q → r) ↔ (p ∧ q) → r", 3),
}


@dataclass
class LogicTheoremGenerator:
    """Generates propositional logic theorems"""

    def __init__(self):
        self.generated_count = 0

    def generate_theorems(self, num: int = 30) -> List[Dict]:
        theorems = []

        for name, (formula, steps) in LOGIC_LAWS.items():
            self.generated_count += 1
            theorems.append(
                {
                    "name": f"logic_{name}",
                    "hypotheses": [],
                    "conclusion": formula,
                    "proof_steps": steps,
                    "domain": "propositional_logic",
                    "difficulty": "easy" if steps <= 2 else "medium",
                }
            )

            if len(theorems) >= num:
                break

        return theorems

    def generate_derived_theorems(self) -> List[Dict]:
        """Generate derived theorems from base laws"""
        derived = []

        derived.append(
            {
                "name": "logic_biconditional_intro",
                "hypotheses": ["p → q", "q → p"],
                "conclusion": "p ↔ q",
                "proof_steps": 2,
                "domain": "propositional_logic",
                "difficulty": "medium",
            }
        )

        derived.append(
            {
                "name": "logic_biconditional_elim",
                "hypotheses": ["p ↔ q"],
                "conclusion": "(p → q) ∧ (q → p)",
                "proof_steps": 2,
                "domain": "propositional_logic",
                "difficulty": "medium",
            }
        )

        derived.append(
            {
                "name": "logic_explosion",
                "hypotheses": ["p", "¬p"],
                "conclusion": "False",
                "proof_steps": 1,
                "domain": "propositional_logic",
                "difficulty": "easy",
            }
        )

        derived.append(
            {
                "name": "logic_excluded_middle",
                "hypotheses": [],
                "conclusion": "p ∨ ¬p",
                "proof_steps": 1,
                "domain": "propositional_logic",
                "difficulty": "easy",
            }
        )

        derived.append(
            {
                "name": "logic_double_negation",
                "hypotheses": [],
                "conclusion": "¬¬p ↔ p",
                "proof_steps": 2,
                "domain": "propositional_logic",
                "difficulty": "medium",
            }
        )

        return derived


def get_logic_theorems() -> List[Dict]:
    """Entry point for propositional logic theorems"""
    generator = LogicTheoremGenerator()
    theorems = generator.generate_theorems(20)
    theorems.extend(generator.generate_derived_theorems())
    return theorems
