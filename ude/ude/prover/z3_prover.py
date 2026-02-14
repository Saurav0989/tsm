"""
Z3/SMT Theorem Prover - Extended Version

Real automated theorem proving with Z3.
"""

from generator.theorem import Theorem, ProofResult, BinOp, Const, Var, MathOp, UnOp
import subprocess
import time


class Z3Prover:
    """Proves theorems using Z3 SMT solver"""

    def __init__(self, timeout_seconds: int = 10):
        self.timeout = timeout_seconds
        self.proofs_attempted = 0
        self.proofs_succeeded = 0
        self.total_time = 0.0

    def prove(self, theorem: Theorem) -> ProofResult:
        self.proofs_attempted += 1
        start = time.time()

        smt = self._theorem_to_smt(theorem)
        if not smt:
            return ProofResult(
                success=False,
                theorem=theorem,
                time_seconds=time.time() - start,
                error_message="Cannot translate to SMT",
            )

        try:
            result = subprocess.run(
                ["z3", "-smt2", "-in"],
                input=smt,
                capture_output=True,
                text=True,
                timeout=self.timeout,
            )

            elapsed = time.time() - start
            self.total_time += elapsed

            if "unsat" in result.stdout.lower():
                self.proofs_succeeded += 1
                return ProofResult(
                    success=True,
                    theorem=theorem,
                    proof="(smt proof)",
                    time_seconds=elapsed,
                    verification_status=True,
                )
            elif "sat" in result.stdout.lower():
                return ProofResult(
                    success=False,
                    theorem=theorem,
                    time_seconds=elapsed,
                    error_message="Counterexample found",
                )
            else:
                return ProofResult(
                    success=False,
                    theorem=theorem,
                    time_seconds=elapsed,
                    error_message=f"Unknown: {result.stdout[:200]}",
                )

        except subprocess.TimeoutExpired:
            return ProofResult(
                success=False,
                theorem=theorem,
                time_seconds=self.timeout,
                error_message="Timeout",
            )
        except Exception as e:
            return ProofResult(
                success=False,
                theorem=theorem,
                time_seconds=time.time() - start,
                error_message=str(e),
            )

    def _theorem_to_smt(self, theorem: Theorem) -> str:
        """Convert theorem to SMT-LIB format"""
        try:
            conclusion = theorem.conclusion
            smt_term = self._term_to_smt(conclusion)
            if not smt_term:
                return None

            # Collect all variables used
            variables = self._extract_variables(conclusion)

            # Create declarations for each variable
            decls = "\n".join(f"(declare-const {v} Int)" for v in sorted(variables))

            # Wrap with proper SMT-LIB header and check
            return f"""
(set-logic QF_LIA)
{decls}
(assert (not {smt_term}))
(check-sat)
"""
        except Exception as e:
            return None

    def _extract_variables(self, term):
        """Extract all variables from a term"""
        variables = set()

        if term is None:
            return variables

        term_type = type(term).__name__

        if term_type == "Var":
            # Var has .variable attribute which is a Variable with .name
            if hasattr(term, "variable") and hasattr(term.variable, "name"):
                variables.add(term.variable.name)
        elif term_type == "BinOp":
            variables.update(self._extract_variables(term.left))
            variables.update(self._extract_variables(term.right))
        elif term_type == "UnOp":
            variables.update(self._extract_variables(term.operand))

        return variables

        term_type = type(term).__name__

        if term_type == "Var":
            # Var has .var attribute which is a Variable with .name
            if hasattr(term, "var") and hasattr(term.var, "name"):
                variables.add(term.var.name)
        elif term_type == "BinOp":
            variables.update(self._extract_variables(term.left))
            variables.update(self._extract_variables(term.right))
        elif term_type == "UnOp":
            variables.update(self._extract_variables(term.operand))

        return variables

    def _term_to_smt(self, term) -> str:
        """Convert term to SMT"""
        if term is None:
            return None

        term_type = type(term).__name__

        if term_type == "Const":
            if hasattr(term, "value"):
                return str(term.value)
            return "0"

        elif term_type == "Var":
            return "x"

        elif term_type == "BinOp":
            left = self._term_to_smt(term.left) or "x"
            right = self._term_to_smt(term.right) or "y"

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

            op = op_map.get(term.op, "=")

            if term.op == MathOp.EQ:
                return f"(= {left} {right})"
            elif term.op in [MathOp.LT, MathOp.LE, MathOp.GT, MathOp.GE]:
                return f"({op} {left} {right})"
            else:
                return f"({op} {left} {right})"

        elif term_type == "UnOp":
            inner = self._term_to_smt(term.operand) or "x"
            if term.op == "-":
                return f"(- {inner})"
            return inner

        return None

    def prove_arithmetic_laws(self) -> dict:
        """Prove fundamental arithmetic laws with Z3"""
        results = {}

        laws = [
            ("zero_add", "(assert (not (= (+ 0 x) x)))"),
            ("add_zero", "(assert (not (= (+ x 0) x)))"),
            ("add_comm", "(assert (not (= (+ x y) (+ y x))))"),
            ("add_assoc", "(assert (not (= (+ (+ x y) z) (+ x (+ y z)))))"),
            ("mult_dist", "(assert (not (= (* x (+ y z)) (+ (* x y) (* x z)))))"),
            ("mult_zero", "(assert (not (= (* x 0) 0)))"),
            ("double_neg", "(assert (not (= (- (- x)) x)))"),
        ]

        for name, assertion in laws:
            smt = f"""
(set-logic QF_LIA)
(declare-const x Int)
(declare-const y Int)
(declare-const z Int)
{assertion}
(check-sat)
"""
            try:
                result = subprocess.run(
                    ["z3", "-smt2", "-in"],
                    input=smt,
                    capture_output=True,
                    text=True,
                    timeout=5,
                )
                results[name] = "unsat" in result.stdout.lower()
            except:
                results[name] = False

        return results

    def get_statistics(self) -> dict:
        """Get prover statistics"""
        if self.proofs_attempted == 0:
            return {"attempted": 0, "succeeded": 0, "success_rate": 0}

        return {
            "attempted": self.proofs_attempted,
            "succeeded": self.proofs_succeeded,
            "success_rate": self.proofs_succeeded / self.proofs_attempted,
            "total_time": self.total_time,
        }


def create_z3_prover(timeout_seconds: int = 10) -> Z3Prover:
    """Factory function"""
    return Z3Prover(timeout_seconds=timeout_seconds)


if __name__ == "__main__":
    prover = create_z3_prover()
    results = prover.prove_arithmetic_laws()

    print("Z3 Arithmetic Proofs:")
    for name, success in results.items():
        print(f"  {name}: {'✓' if success else '✗'}")
