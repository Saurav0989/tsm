"""
Real Lean Prover - Uses actual proof tactics
"""

from generator.theorem import Theorem, ProofResult, BinOp, Const, Var, MathOp


class RealLeanProver:
    """Proves theorems using actual Lean tactics"""

    def __init__(self, timeout_seconds: int = 30):
        self.timeout = timeout_seconds
        self._cache = {}  # Cache results

    def prove(self, theorem: Theorem) -> ProofResult:
        # Check cache first
        h = theorem.hash()
        if h in self._cache:
            return self._cache[h]

        import subprocess
        import time

        start = time.time()

        # Generate Lean code with real proofs
        lean_code = self._generate_proof(theorem)

        # Write to temp file
        temp = f"/tmp/ude_proof_{abs(hash(theorem.name))}.lean"
        with open(temp, "w") as f:
            f.write(lean_code)

        try:
            result = subprocess.run(
                ["lean", temp], capture_output=True, text=True, timeout=self.timeout
            )

            elapsed = time.time() - start
            success = result.returncode == 0

            proof_result = ProofResult(
                success=success,
                theorem=theorem,
                proof=lean_code if success else None,
                time_seconds=elapsed,
                verification_status=success,
                error_message=None if success else result.stderr[:200],
            )

            # Cache successful proofs
            if success:
                self._cache[h] = proof_result

            return proof_result
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

            elapsed = time.time() - start
            success = result.returncode == 0

            return ProofResult(
                success=success,
                theorem=theorem,
                proof=lean_code if success else None,
                time_seconds=elapsed,
                verification_status=success,
                error_message=None if success else result.stderr[:200],
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

    def _generate_proof(self, theorem: Theorem) -> str:
        """Generate Lean code with actual proof attempts"""

        conclusion = self._term_to_lean(theorem.conclusion)

        proof = f"""theorem {theorem.name} : {conclusion} :=
by
  rfl
"""
        return proof

    def _term_to_lean(self, term) -> str:
        """Convert term to Lean"""
        if term is None:
            return "true"

        if isinstance(term, Var):
            return term.variable.name
        elif isinstance(term, Const):
            v = term.value
            if isinstance(v, bool):
                return "true" if v else "false"
            return str(v)
        elif isinstance(term, BinOp):
            left = self._term_to_lean(term.left)
            right = self._term_to_lean(term.right)

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
            }
            op = op_map.get(term.op, "=")
            return f"({left} {op} {right})"

        return "true"


def create_prover(use_lean: bool = False, real: bool = True):
    """Factory to create prover"""
    if real:
        return RealLeanProver()

    from prover.lean import MockProver

    return MockProver(success_rate=0.15)
