"""
Universal Discovery Engine - Core Theorem Representation

This module defines the formal language for theorems and the core data structures.

We use a restricted but powerful subset of first-order logic + type theory
that can be mapped to Lean 4 for formal verification.
"""

from dataclasses import dataclass
from typing import List, Optional, Union, Set, Tuple
from enum import Enum
import hashlib


class LogicOp(Enum):
    """Logical operators"""

    AND = "∧"
    OR = "∨"
    NOT = "¬"
    IMPLIES = "→"
    IFF = "↔"
    FORALL = "∀"
    EXISTS = "∃"


class MathOp(Enum):
    """Mathematical operators"""

    PLUS = "+"
    MINUS = "-"
    MULT = "×"
    DIV = "÷"
    MOD = "mod"
    POW = "^"
    EQ = "="
    LT = "<"
    LE = "≤"
    GT = ">"
    GE = "≥"
    NEQ = "≠"


class MathFunc(Enum):
    """Mathematical functions"""

    ABS = "abs"
    MAX = "max"
    MIN = "min"
    GCD = "gcd"
    FACT = "fact"
    SQRT = "sqrt"
    SIN = "sin"
    COS = "cos"
    TAN = "tan"
    LOG = "log"
    LN = "ln"
    EXP = "exp"


@dataclass(frozen=True)
class Type:
    """Type in our type system"""

    name: str

    def __str__(self):
        return self.name

    def __hash__(self):
        return hash(self.name)


# Common types
NAT = Type("ℕ")  # Natural numbers
INT = Type("ℤ")  # Integers
REAL = Type("ℝ")  # Real numbers
BOOL = Type("Bool")  # Booleans
PROP = Type("Prop")  # Propositions


@dataclass
class Variable:
    """Typed variable"""

    name: str
    type: Type

    def __str__(self):
        return f"{self.name} : {self.type}"

    def __hash__(self):
        return hash((self.name, self.type))


@dataclass(frozen=True)
class Term:
    """Base class for all terms"""

    pass


@dataclass(frozen=True)
class Var(Term):
    """Variable reference"""

    variable: Variable

    def __str__(self):
        return self.variable.name


@dataclass(frozen=True)
class Const(Term):
    """Constant value"""

    value: Union[int, float, bool, str]
    type: Type

    def __str__(self):
        return str(self.value)


@dataclass(frozen=True)
class App(Term):
    """Function application"""

    func: Term
    args: Tuple[Term, ...]

    def __str__(self):
        args_str = " ".join(str(arg) for arg in self.args)
        return f"({self.func} {args_str})"


@dataclass(frozen=True)
class BinOp(Term):
    """Binary operation"""

    op: Union[LogicOp, MathOp]
    left: Term
    right: Term

    def __str__(self):
        return f"({self.left} {self.op.value} {self.right})"


@dataclass(frozen=True)
class UnOp(Term):
    """Unary operation"""

    op: LogicOp
    term: Term

    def __str__(self):
        return f"({self.op.value}{self.term})"


@dataclass(frozen=True)
class Quantifier(Term):
    """Quantified expression"""

    op: LogicOp  # FORALL or EXISTS
    var: Variable
    body: Term

    def __str__(self):
        return f"({self.op.value}{self.var.name}. {self.body})"


@dataclass
class Theorem:
    """A mathematical theorem"""

    name: str
    hypotheses: List[Term]  # Assumptions
    conclusion: Term  # What we're proving
    proof: Optional[str] = None  # Lean 4 proof (if proven)

    def __str__(self):
        if self.hypotheses:
            hyps = ", ".join(str(h) for h in self.hypotheses)
            return f"{self.name}: [{hyps}] ⊢ {self.conclusion}"
        else:
            return f"{self.name}: ⊢ {self.conclusion}"

    def hash(self) -> str:
        """Unique hash for this theorem"""
        content = str(self.hypotheses) + str(self.conclusion)
        return hashlib.sha256(content.encode()).hexdigest()[:16]

    def to_dict(self) -> dict:
        """Serialize theorem to dictionary"""
        return {
            "name": self.name,
            "hypotheses": [self._term_to_dict(h) for h in self.hypotheses],
            "conclusion": self._term_to_dict(self.conclusion),
            "proof": self.proof,
        }

    def _term_to_dict(self, term: Term) -> dict:
        """Convert term to dictionary for serialization"""
        if isinstance(term, Var):
            return {
                "type": "Var",
                "name": term.variable.name,
                "var_type": str(term.variable.type),
            }
        elif isinstance(term, Const):
            return {"type": "Const", "value": term.value, "const_type": str(term.type)}
        elif isinstance(term, BinOp):
            return {
                "type": "BinOp",
                "op": term.op.value,
                "left": self._term_to_dict(term.left),
                "right": self._term_to_dict(term.right),
            }
        elif isinstance(term, UnOp):
            return {
                "type": "UnOp",
                "op": term.op.value,
                "term": self._term_to_dict(term.term),
            }
        elif isinstance(term, Quantifier):
            return {
                "type": "Quantifier",
                "op": term.op.value,
                "var": {"name": term.var.name, "type": str(term.var.type)},
                "body": self._term_to_dict(term.body),
            }
        return {"type": "unknown"}

    @staticmethod
    def from_dict(data: dict) -> "Theorem":
        """Deserialize theorem from dictionary"""
        from generator.theorem import (
            Var,
            Const,
            BinOp,
            UnOp,
            Quantifier,
            MathOp,
            LogicOp,
            Variable,
            Type,
            NAT,
            INT,
            REAL,
            BOOL,
        )

        def dict_to_term(d: dict) -> Term:
            type_map = {
                "NAT": NAT,
                "INT": INT,
                "REAL": REAL,
                "BOOL": BOOL,
                "ℕ": NAT,
                "ℤ": INT,
                "ℝ": REAL,
                "Bool": BOOL,
            }

            if d.get("type") == "Var":
                var_type = type_map.get(d.get("var_type", "NAT"), NAT)
                return Var(Variable(d["name"], var_type))
            elif d.get("type") == "Const":
                const_type = type_map.get(d.get("const_type", "NAT"), NAT)
                return Const(d["value"], const_type)
            elif d.get("type") == "BinOp":
                op = d["op"]
                # Find the operator
                for m in MathOp:
                    if m.value == op:
                        return BinOp(
                            m, dict_to_term(d["left"]), dict_to_term(d["right"])
                        )
                for m in LogicOp:
                    if m.value == op:
                        return BinOp(
                            m, dict_to_term(d["left"]), dict_to_term(d["right"])
                        )
            elif d.get("type") == "UnOp":
                for m in LogicOp:
                    if m.value == d["op"]:
                        return UnOp(m, dict_to_term(d["term"]))
            elif d.get("type") == "Quantifier":
                for m in LogicOp:
                    if m.value == d["op"]:
                        var_type = type_map.get(d["var"].get("type", "NAT"), NAT)
                        return Quantifier(
                            m,
                            Variable(d["var"]["name"], var_type),
                            dict_to_term(d["body"]),
                        )
            return None

        hypotheses = [dict_to_term(h) for h in data.get("hypotheses", [])]
        conclusion = (
            dict_to_term(data["conclusion"]) if data.get("conclusion") else None
        )

        return Theorem(
            name=data["name"],
            hypotheses=hypotheses,
            conclusion=conclusion,
            proof=data.get("proof"),
        )

    def to_lean(self) -> str:
        """Convert to Lean 4 syntax"""
        return self.to_lean_simple()

    def to_lean_simple(self) -> str:
        """Convert to Lean 4 syntax (simple, no Mathlib)"""
        conclusion = self._term_to_lean_simple(self.conclusion)

        if self.hypotheses:
            hyps_str = " ".join(
                f"(h{i}: {self._term_to_lean_simple(h)})"
                for i, h in enumerate(self.hypotheses)
            )
            return f"theorem {self.name} {hyps_str} : {conclusion} :=\n  sorry"
        else:
            return f"theorem {self.name} : {conclusion} :=\n  sorry"

    def _term_to_lean_simple(self, term: Term) -> str:
        """Convert term to simple Lean syntax (no special symbols)"""
        if term is None:
            return "true"

        if isinstance(term, Var):
            return term.variable.name
        elif isinstance(term, Const):
            val = term.value
            if isinstance(val, bool):
                return "true" if val else "false"
            return str(val)
        elif isinstance(term, BinOp):
            left = self._term_to_lean_simple(term.left)
            right = self._term_to_lean_simple(term.right)

            # Simple ASCII operators
            op_map = {
                MathOp.PLUS: "+",
                MathOp.MINUS: "-",
                MathOp.MULT: "*",
                MathOp.DIV: "/",
                MathOp.MOD: "%",
                MathOp.EQ: "=",
                MathOp.LT: "<",
                MathOp.LE: "<=",
                MathOp.GT: ">",
                MathOp.GE: ">=",
                MathOp.NEQ: "!=",
                LogicOp.AND: "/\\",
                LogicOp.OR: "\\/",
                LogicOp.IMPLIES: "->",
                LogicOp.NOT: "!",
            }

            op = op_map.get(term.op, str(term.op.value))
            return f"({left} {op} {right})"
        elif isinstance(term, UnOp):
            inner = self._term_to_lean_simple(term.term)
            if term.op == LogicOp.NOT:
                return f"(!{inner})"
            return inner
        elif isinstance(term, Quantifier):
            var = term.var.name
            body = self._term_to_lean_simple(term.body)
            if term.op == LogicOp.FORALL:
                return f"(forall (x : Nat), {body})"
            else:
                return f"(exists (x : Nat), {body})"

        return "true"

    def _term_to_lean(self, term: Term) -> str:
        """Convert term to Lean syntax"""
        if isinstance(term, Var):
            return term.variable.name
        elif isinstance(term, Const):
            return str(term.value)
        elif isinstance(term, BinOp):
            left = self._term_to_lean(term.left)
            right = self._term_to_lean(term.right)

            # Map operators to Lean syntax
            op_map = {
                LogicOp.AND: "∧",
                LogicOp.OR: "∨",
                LogicOp.IMPLIES: "→",
                LogicOp.IFF: "↔",
                MathOp.EQ: "=",
                MathOp.LT: "<",
                MathOp.LE: "≤",
                MathOp.PLUS: "+",
                MathOp.MULT: "*",
            }

            op_str = op_map.get(term.op, term.op.value)
            return f"({left} {op_str} {right})"
        elif isinstance(term, UnOp):
            inner = self._term_to_lean(term.term)
            return f"¬{inner}"
        elif isinstance(term, Quantifier):
            var_str = f"{term.var.name} : {term.var.type}"
            body = self._term_to_lean(term.body)
            if term.op == LogicOp.FORALL:
                return f"∀ {var_str}, {body}"
            else:
                return f"∃ {var_str}, {body}"
        else:
            return str(term)


@dataclass
class ProofResult:
    """Result of a proof attempt"""

    success: bool
    theorem: Theorem
    proof: Optional[str] = None  # Lean proof term
    time_seconds: float = 0.0
    verification_status: bool = False
    error_message: Optional[str] = None


@dataclass
class TheoremSpace:
    """Defines a space of theorems to explore"""

    axioms: List[Theorem]  # Starting axioms
    types: Set[Type]  # Available types
    constants: Set[Const]  # Available constants
    max_term_depth: int = 5  # Maximum nesting depth
    max_quantifiers: int = 2  # Maximum quantifier alternations

    def __str__(self):
        return f"TheoremSpace(axioms={len(self.axioms)}, types={len(self.types)})"


# Example: Peano Arithmetic axioms
PEANO_AXIOMS = [
    Theorem(
        name="zero_nat",
        hypotheses=[],
        conclusion=BinOp(MathOp.EQ, Const(0, NAT), Const(0, NAT)),
    ),
    Theorem(
        name="succ_injective",
        hypotheses=[
            BinOp(
                MathOp.EQ,
                App(Var(Variable("S", Type("ℕ → ℕ"))), [Var(Variable("n", NAT))]),
                App(Var(Variable("S", Type("ℕ → ℕ"))), [Var(Variable("m", NAT))]),
            )
        ],
        conclusion=BinOp(MathOp.EQ, Var(Variable("n", NAT)), Var(Variable("m", NAT))),
    ),
]


# Example: Create a simple theorem
def example_commutativity():
    """Example: Commutativity of addition"""
    n = Variable("n", NAT)
    m = Variable("m", NAT)

    left = BinOp(MathOp.PLUS, Var(n), Var(m))
    right = BinOp(MathOp.PLUS, Var(m), Var(n))

    theorem = Theorem(
        name="add_comm",
        hypotheses=[],
        conclusion=Quantifier(
            LogicOp.FORALL,
            n,
            Quantifier(LogicOp.FORALL, m, BinOp(MathOp.EQ, left, right)),
        ),
    )

    return theorem


if __name__ == "__main__":
    # Test theorem representation
    thm = example_commutativity()
    print("Theorem:", thm)
    print("\nLean 4 syntax:")
    print(thm.to_lean())
    print("\nHash:", thm.hash())
