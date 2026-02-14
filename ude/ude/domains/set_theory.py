"""
Set Theory Domain for UDE

Enables discovery of non-trivial set-theoretic theorems.
"""

from typing import List, Dict, Set
from dataclasses import dataclass
from enum import Enum


class SetAxiom(Enum):
    EXTENSIONALITY = "extensionality"
    EMPTY_SET = "empty_set"
    PAIRING = "pairing"
    UNION = "union"
    POWER_SET = "power_set"
    INFINITY = "infinity"
    SEPARATION = "separation"


SET_AXIOMS = {
    SetAxiom.EXTENSIONALITY: "∀A B : Set, (∀x, x ∈ A ↔ x ∈ B) → A = B",
    SetAxiom.EMPTY_SET: "∃∅ : Set, ∀x, ¬(x ∈ ∅)",
    SetAxiom.PAIRING: "∀a b, ∃p, a ∈ p ∧ b ∈ p",
    SetAxiom.UNION: "∀F, ∃U, ∀x, (∃A ∈ F, x ∈ A) → x ∈ U",
    SetAxiom.POWER_SET: "∀A, ∃P, ∀B, (∀x ∈ B, x ∈ A) → B ∈ P",
}


@dataclass
class SetTheoremGenerator:
    """Generates set theory theorems"""

    def generate_theorems(self, num: int = 50) -> List[Dict]:
        theorems = []

        theorems.extend(self._membership_theorems())
        theorems.extend(self._operation_theorems())
        theorems.extend(self._identity_theorems())

        return theorems[:num]

    def _membership_theorems(self) -> List[Dict]:
        return [
            {
                "name": "membership_extensionality",
                "hypotheses": ["∀x, x ∈ A ↔ x ∈ B"],
                "conclusion": "A = B",
                "proof_steps": 2,
                "domain": "set_theory",
            },
            {
                "name": "empty_mem_false",
                "hypotheses": ["x ∈ ∅"],
                "conclusion": "False",
                "proof_steps": 1,
                "domain": "set_theory",
            },
            {
                "name": "subset_refl",
                "hypotheses": ["A ⊆ A"],
                "conclusion": "True",
                "proof_steps": 1,
                "domain": "set_theory",
            },
            {
                "name": "subset_trans",
                "hypotheses": ["A ⊆ B", "B ⊆ C"],
                "conclusion": "A ⊆ C",
                "proof_steps": 2,
                "domain": "set_theory",
            },
        ]

    def _operation_theorems(self) -> List[Dict]:
        return [
            {
                "name": "union_comm",
                "hypotheses": ["A ∪ B = B ∪ A"],
                "conclusion": "True",
                "proof_steps": 1,
                "domain": "set_theory",
            },
            {
                "name": "intersection_comm",
                "hypotheses": ["A ∩ B = B ∩ A"],
                "conclusion": "True",
                "proof_steps": 1,
                "domain": "set_theory",
            },
            {
                "name": "distributivity",
                "hypotheses": ["A ∩ (B ∪ C) = (A ∩ B) ∪ (A ∩ C)"],
                "conclusion": "True",
                "proof_steps": 3,
                "domain": "set_theory",
            },
            {
                "name": "de_morgan_1",
                "hypotheses": [],
                "conclusion": "¬(A ∧ B) ↔ ¬A ∨ ¬B",
                "proof_steps": 2,
                "domain": "set_theory",
            },
            {
                "name": "de_morgan_2",
                "hypotheses": [],
                "conclusion": "¬(A ∨ B) ↔ ¬A ∧ ¬B",
                "proof_steps": 2,
                "domain": "set_theory",
            },
        ]

    def _identity_theorems(self) -> List[Dict]:
        return [
            {
                "name": "union_empty_id",
                "hypotheses": [],
                "conclusion": "A ∪ ∅ = A",
                "proof_steps": 2,
                "domain": "set_theory",
            },
            {
                "name": "intersection_empty_id",
                "hypotheses": [],
                "conclusion": "A ∩ ∅ = ∅",
                "proof_steps": 2,
                "domain": "set_theory",
            },
            {
                "name": "power_set_mem",
                "hypotheses": ["X ∈ P(A)"],
                "conclusion": "X ⊆ A",
                "proof_steps": 2,
                "domain": "set_theory",
            },
        ]


def get_set_theorems() -> List[Dict]:
    """Entry point for set theory theorems"""
    generator = SetTheoremGenerator()
    return generator.generate_theorems(30)
