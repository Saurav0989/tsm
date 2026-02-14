"""
Complex Theorem Generator

Generates non-trivial theorems requiring actual reasoning.
"""

import random
from typing import List, Dict, Iterator
from generator.theorem import (
    Theorem,
    BinOp,
    Const,
    Var,
    MathOp,
    UnOp,
    NAT,
    INT,
    Variable,
)


class ComplexTheoremGenerator:
    """Generates complex, non-trivial theorems"""

    def __init__(self, seed: int = 42):
        self.random = random.Random(seed)
        self.generated = 0

    def generate_commutative_laws(self) -> Iterator[Theorem]:
        """Generate commutative laws: a + b = b + a"""
        for a in ["x", "y", "z", "a", "b"]:
            for b in ["x", "y", "z", "a", "b"]:
                if a == b:
                    continue
                t = Theorem(
                    name=f"comm_add_{a}_{b}",
                    hypotheses=[],
                    conclusion=BinOp(
                        MathOp.EQ,
                        BinOp(
                            MathOp.PLUS, Var(Variable(a, NAT)), Var(Variable(b, NAT))
                        ),
                        BinOp(
                            MathOp.PLUS, Var(Variable(b, NAT)), Var(Variable(a, NAT))
                        ),
                    ),
                )
                self.generated += 1
                yield t

    def generate_associative_laws(self) -> Iterator[Theorem]:
        """Generate associative laws: (a + b) + c = a + (b + c)"""
        for a in ["x", "y", "z"]:
            for b in ["x", "y", "z"]:
                for c in ["x", "y", "z"]:
                    if len(set([a, b, c])) < 2:
                        continue
                    t = Theorem(
                        name=f"assoc_add_{a}_{b}_{c}",
                        hypotheses=[],
                        conclusion=BinOp(
                            MathOp.EQ,
                            BinOp(
                                MathOp.PLUS,
                                BinOp(
                                    MathOp.PLUS,
                                    Var(Variable(a, NAT)),
                                    Var(Variable(b, NAT)),
                                ),
                                Var(Variable(c, NAT)),
                            ),
                            BinOp(
                                MathOp.PLUS,
                                Var(Variable(a, NAT)),
                                BinOp(
                                    MathOp.PLUS,
                                    Var(Variable(b, NAT)),
                                    Var(Variable(c, NAT)),
                                ),
                            ),
                        ),
                    )
                    self.generated += 1
                    yield t

    def generate_distributive_laws(self) -> Iterator[Theorem]:
        """Generate distributive laws: a * (b + c) = a*b + a*c"""
        for a in ["x", "y", "z"]:
            for b in ["x", "y"]:
                for c in ["x", "y"]:
                    if len(set([a, b, c])) < 2:
                        continue
                    t = Theorem(
                        name=f"dist_{a}_{b}_{c}",
                        hypotheses=[],
                        conclusion=BinOp(
                            MathOp.EQ,
                            BinOp(
                                MathOp.MULT,
                                Var(Variable(a, NAT)),
                                BinOp(
                                    MathOp.PLUS,
                                    Var(Variable(b, NAT)),
                                    Var(Variable(c, NAT)),
                                ),
                            ),
                            BinOp(
                                MathOp.PLUS,
                                BinOp(
                                    MathOp.MULT,
                                    Var(Variable(a, NAT)),
                                    Var(Variable(b, NAT)),
                                ),
                                BinOp(
                                    MathOp.MULT,
                                    Var(Variable(a, NAT)),
                                    Var(Variable(c, NAT)),
                                ),
                            ),
                        ),
                    )
                    self.generated += 1
                    yield t

    def generate_identity_laws(self) -> Iterator[Theorem]:
        """Generate identity laws: a + 0 = a"""
        for a in ["x", "y", "z", "a", "b"]:
            for op in [MathOp.PLUS, MathOp.MINUS]:
                t = Theorem(
                    name=f"identity_{op.name}_{a}",
                    hypotheses=[],
                    conclusion=BinOp(
                        MathOp.EQ,
                        BinOp(op, Var(Variable(a, NAT)), Const(0, NAT)),
                        Var(Variable(a, NAT)),
                    ),
                )
                self.generated += 1
                yield t

    def generate_inverse_laws(self) -> Iterator[Theorem]:
        """Generate inverse laws: a + (-a) = 0"""
        for a in ["x", "y", "z"]:
            t = Theorem(
                name=f"inverse_{a}",
                hypotheses=[],
                conclusion=BinOp(
                    MathOp.EQ,
                    BinOp(
                        MathOp.PLUS,
                        Var(Variable(a, NAT)),
                        UnOp("-", Var(Variable(a, NAT))),
                    ),
                    Const(0, NAT),
                ),
            )
            self.generated += 1
            yield t

    def generate_order_theorems(self) -> Iterator[Theorem]:
        """Generate ordering theorems"""
        theorems = []

        # Transitivity: a < b ∧ b < c → a < c
        for a, b, c in [("x", "y", "z"), ("a", "b", "c")]:
            t = Theorem(
                name=f"order_trans_{a}_{b}_{c}",
                hypotheses=[
                    BinOp(MathOp.LT, Var(Variable(a, NAT)), Var(Variable(b, NAT))),
                    BinOp(MathOp.LT, Var(Variable(b, NAT)), Var(Variable(c, NAT))),
                ],
                conclusion=BinOp(
                    MathOp.LT, Var(Variable(a, NAT)), Var(Variable(c, NAT))
                ),
            )
            self.generated += 1
            yield t

        # Reflexivity: a ≤ a
        for a in ["x", "y", "z"]:
            t = Theorem(
                name=f"order_refl_{a}",
                hypotheses=[],
                conclusion=BinOp(
                    MathOp.LE, Var(Variable(a, NAT)), Var(Variable(a, NAT))
                ),
            )
            self.generated += 1
            yield t

    def generate_comparison_theorems(self) -> Iterator[Theorem]:
        """Generate comparison theorems"""
        # a + b > a (when b > 0)
        for a in ["x", "y"]:
            for b in [1, 2, 3]:
                t = Theorem(
                    name=f"compare_{a}_{b}",
                    hypotheses=[],
                    conclusion=BinOp(
                        MathOp.GT,
                        BinOp(MathOp.PLUS, Var(Variable(a, NAT)), Const(b, NAT)),
                        Var(Variable(a, NAT)),
                    ),
                )
                self.generated += 1
                yield t

    def generate_all(self, max_count: int = 1000) -> Iterator[Theorem]:
        """Generate all complex theorems"""
        generators = [
            self.generate_commutative_laws,
            self.generate_associative_laws,
            self.generate_distributive_laws,
            self.generate_identity_laws,
            self.generate_inverse_laws,
            self.generate_order_theorems,
            self.generate_comparison_theorems,
        ]

        count = 0
        for gen in generators:
            for theorem in gen():
                yield theorem
                count += 1
                if count >= max_count:
                    return


def create_complex_generator(seed: int = 42) -> ComplexTheoremGenerator:
    return ComplexTheoremGenerator(seed=seed)


if __name__ == "__main__":
    gen = create_complex_generator()
    count = 0
    for t in gen.generate_all(50):
        count += 1
        print(f"{count}: {t.name}: {t.conclusion}")
    print(f"Total: {count} theorems")
