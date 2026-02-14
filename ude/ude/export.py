"""
Universal Discovery Engine - Pattern Detection & Export

Detects patterns in discovered theorems and exports to various formats.
"""

import json
from typing import Dict, Any, List
from pathlib import Path


class PatternDetector:
    """Detects patterns and interesting structures in theorems"""

    def __init__(self, theorems: List):
        self.theorems = theorems

    def detect_patterns(self) -> Dict[str, Any]:
        """Detect all patterns in discovered theorems"""

        patterns = {
            "commutativity": [],
            "associativity": [],
            "distributivity": [],
            "identity": [],
            "inverse": [],
            "symmetry": [],
            "transitivity": [],
            "known_theorems": [],
        }

        from generator.theorem import BinOp, MathOp

        for theorem in self.theorems:
            if not theorem.conclusion:
                continue

            # Check for commutativity patterns: f(x,y) = f(y,x)
            if self._is_commutative(theorem.conclusion):
                patterns["commutativity"].append(theorem.name)

            # Check for identity patterns: f(x,0) = x
            if self._is_identity(theorem.conclusion):
                patterns["identity"].append(theorem.name)

            # Check for symmetry in equality
            if self._is_symmetric(theorem.conclusion):
                patterns["symmetry"].append(theorem.name)

            # Check for known theorems
            known = self._check_known_theorem(theorem)
            if known:
                patterns["known_theorems"].append(
                    {
                        "name": theorem.name,
                        "known_as": known,
                    }
                )

        return patterns

    def _is_commutative(self, term) -> bool:
        """Check if term represents commutativity"""
        from generator.theorem import BinOp, MathOp

        if isinstance(term, BinOp) and term.op == MathOp.EQ:
            left_str = str(term.left)
            right_str = str(term.right)
            # Check if left and right are same expression with swapped vars
            return (
                left_str.replace("x", "TEMP").replace("y", "x").replace("TEMP", "y")
                == right_str
            )
        return False

    def _is_identity(self, term) -> bool:
        """Check if term represents identity element"""
        from generator.theorem import BinOp, MathOp

        if isinstance(term, BinOp) and term.op == MathOp.EQ:
            # Look for patterns like x + 0 = x
            left = str(term.left)
            right = str(term.right)

            for op in ["+", "-", "*"]:
                for identity in ["0", "1"]:
                    if (
                        f"{op} {identity}" in left
                        and left.replace(f"{op} {identity}", "") == right.strip()
                    ):
                        return True
                    if (
                        f"{identity} {op}" in left
                        and left.replace(f"{identity} {op}", "") == right.strip()
                    ):
                        return True
        return False

    def _is_symmetric(self, term) -> bool:
        """Check if equality is symmetric"""
        from generator.theorem import BinOp, MathOp

        if isinstance(term, BinOp) and term.op == MathOp.EQ:
            left = str(term.left)
            right = str(term.right)
            # Simple symmetry: x = y vs y = x
            return left.split() == right.split()[::-1]
        return False

    def _check_known_theorem(self, theorem) -> str:
        """Check if theorem matches known mathematical theorems"""
        from generator.theorem import BinOp, MathOp

        stmt = str(theorem.conclusion)

        known = {
            "x + 0 = x": "Additive Identity",
            "0 + x = x": "Additive Identity (commuted)",
            "x * 1 = x": "Multiplicative Identity",
            "1 * x = x": "Multiplicative Identity (commuted)",
            "x + x = 2*x": "Double",
            "x = x": "Reflexivity",
        }

        for pattern, name in known.items():
            if pattern in stmt:
                return name

        return None


class TheoremExporter:
    """Exports theorems to various formats"""

    def __init__(self, theorems: List):
        self.theorems = theorems

    def to_json(self, filepath: str):
        """Export theorems to JSON"""
        data = {
            "theorems": [t.to_dict() for t in self.theorems],
            "count": len(self.theorems),
        }

        with open(filepath, "w") as f:
            json.dump(data, f, indent=2)

        print(f"Exported {len(self.theorems)} theorems to {filepath}")

    def to_latex(self, filepath: str):
        """Export theorems to LaTeX for publication"""

        latex = []
        latex.append("\\documentclass{article}")
        latex.append("\\usepackage{amsmath}")
        latex.append("\\usepackage{amssymb}")
        latex.append("\\begin{document}")
        latex.append("")
        latex.append("\\title{Discovered Theorems}")
        latex.append("\\author{Universal Discovery Engine}")
        latex.append("\\maketitle")
        latex.append("")
        latex.append("\\section{Discovered Theorems}")
        latex.append("")

        for i, theorem in enumerate(self.theorems, 1):
            latex.append(f"\\begin{{theorem}}")
            latex.append(f"\\textbf{{Theorem {i}:}} {theorem.name}")
            latex.append(f"\\\\")

            if theorem.hypotheses:
                latex.append(
                    f"\\textit{{Hypotheses:}} {', '.join(str(h) for h in theorem.hypotheses)}"
                )
                latex.append(f"\\\\")

            latex.append(
                f"\\textit{{Conclusion:}} ${self._to_math_latex(theorem.conclusion)}$"
            )
            latex.append(f"\\end{{theorem}}")
            latex.append("")

        latex.append("\\end{document}")

        with open(filepath, "w") as f:
            f.write("\n".join(latex))

        print(f"Exported {len(self.theorems)} theorems to {filepath}")

    def to_lean(self, filepath: str):
        """Export theorems to Lean 4 format"""

        lean = []
        lean.append("import Mathlib")
        lean.append("")
        lean.append("-- Automatically discovered theorems")
        lean.append("namespace UDE")
        lean.append("")

        for theorem in self.theorems:
            lean.append(f"-- Theorem: {theorem.name}")
            lean.append(f"theorem {theorem.name}")

            if theorem.hypotheses:
                hyps = " /\\ ".join(str(h) for h in theorem.hypotheses)
                lean.append(f"  ({hyps}) :")
            else:
                lean.append("  :")

            lean.append(f"    {self._to_lean_term(theorem.conclusion)} :=")
            lean.append("  by sorry")
            lean.append("")

        lean.append("end UDE")

        with open(filepath, "w") as f:
            f.write("\n".join(lean))

        print(f"Exported {len(self.theorems)} theorems to {filepath}")

    def to_csv(self, filepath: str):
        """Export theorems to CSV for analysis"""

        import csv

        with open(filepath, "w", newline="") as f:
            writer = csv.writer(f)
            writer.writerow(["name", "hash", "hypotheses", "conclusion", "proof"])

            for theorem in self.theorems:
                writer.writerow(
                    [
                        theorem.name,
                        theorem.hash(),
                        str(theorem.hypotheses),
                        str(theorem.conclusion),
                        theorem.proof or "",
                    ]
                )

        print(f"Exported {len(self.theorems)} theorems to {filepath}")

    def _to_math_latex(self, term) -> str:
        """Convert term to LaTeX math"""
        from generator.theorem import BinOp, UnOp, Const, Var, MathOp, LogicOp

        if isinstance(term, Var):
            return term.variable.name
        elif isinstance(term, Const):
            return str(term.value)
        elif isinstance(term, BinOp):
            left = self._to_math_latex(term.left)
            right = self._to_math_latex(term.right)

            op_map = {
                MathOp.PLUS: "+",
                MathOp.MINUS: "-",
                MathOp.MULT: r"\times",
                MathOp.EQ: "=",
                MathOp.LT: "<",
                MathOp.LE: r"\le",
                MathOp.GT: ">",
                MathOp.GE: r"\ge",
                LogicOp.AND: r"\land",
                LogicOp.OR: r"\lor",
                LogicOp.IMPLIES: r"\rightarrow",
            }

            op = op_map.get(term.op, str(term.op))
            return f"{left} {op} {right}"

        return str(term)

    def _to_lean_term(self, term) -> str:
        """Convert term to Lean syntax"""
        from generator.theorem import BinOp, UnOp, Const, Var, MathOp, LogicOp

        if isinstance(term, Var):
            return term.variable.name
        elif isinstance(term, Const):
            return str(term.value)
        elif isinstance(term, BinOp):
            left = self._to_lean_term(term.left)
            right = self._to_lean_term(term.right)

            op_map = {
                MathOp.PLUS: "+",
                MathOp.MINUS: "-",
                MathOp.MULT: "*",
                MathOp.EQ: "=",
                MathOp.LT: "<",
                MathOp.LE: "<=",
                MathOp.GT: ">",
                MathOp.GE: ">=",
            }

            op = op_map.get(term.op, str(term.op))
            return f"({left} {op} {right})"

        return str(term)


def export_all(formats: List[str] = None):
    """Export theorems to all formats"""
    from archive.storage import TheoremArchive

    if formats is None:
        formats = ["json", "latex", "lean", "csv"]

    print("Loading archive...")
    archive = TheoremArchive()
    theorems = archive.get_all_proven()
    archive.close()

    print(f"Found {len(theorems)} theorems")

    exporter = TheoremExporter(theorems)

    base = "theorems"

    if "json" in formats:
        exporter.to_json(f"{base}.json")

    if "latex" in formats:
        exporter.to_latex(f"{base}.tex")

    if "lean" in formats:
        exporter.to_lean(f"{base}.lean")

    if "csv" in formats:
        exporter.to_csv(f"{base}.csv")

    # Run pattern detection
    detector = PatternDetector(theorems)
    patterns = detector.detect_patterns()

    print("\nPattern Detection:")
    for pattern, theorems in patterns.items():
        if theorems:
            print(f"  {pattern}: {len(theorems)}")


if __name__ == "__main__":
    import sys

    formats = sys.argv[1:] if len(sys.argv) > 1 else None
    export_all(formats)
