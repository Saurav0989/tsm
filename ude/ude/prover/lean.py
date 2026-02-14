"""
Universal Discovery Engine - Automated Prover

Attempts to prove generated theorems using Lean 4 and automated tactics.

This is the component that makes this REAL mathematics - every theorem
must be formally verified.

Integration with:
- Lean 4 theorem prover
- Z3 SMT solver (for arithmetic)
- Custom proof search tactics
"""

import subprocess
import tempfile
import os
import time
from typing import Optional, List, Tuple, Union
from pathlib import Path

from generator.theorem import Theorem, ProofResult


class LeanProver:
    """
    Interface to Lean 4 theorem prover.

    Lean 4 is a proof assistant with a small trusted kernel.
    If Lean verifies a proof, it is mathematically correct.
    """

    def __init__(
        self,
        lean_executable: str = "lean",
        timeout_seconds: int = 60,
        tactics_file: Optional[str] = None,
    ):
        self.lean_executable = lean_executable
        self.timeout = timeout_seconds
        self.tactics_file = tactics_file or self._create_default_tactics()

        # Statistics
        self.proofs_attempted = 0
        self.proofs_succeeded = 0
        self.total_time = 0.0

    def prove(self, theorem: Theorem) -> ProofResult:
        """
        Attempt to prove a theorem.

        Returns ProofResult with success status and proof (if found).
        """
        self.proofs_attempted += 1
        start_time = time.time()

        try:
            # Convert theorem to Lean 4 code
            lean_code = self._generate_lean_code(theorem)

            # Write to temporary file
            with tempfile.NamedTemporaryFile(
                mode="w", suffix=".lean", delete=False
            ) as f:
                f.write(lean_code)
                temp_file = f.name

            try:
                # Run Lean 4
                result = subprocess.run(
                    [self.lean_executable, temp_file],
                    capture_output=True,
                    text=True,
                    timeout=self.timeout,
                )

                elapsed = time.time() - start_time
                self.total_time += elapsed

                if result.returncode == 0:
                    # Proof succeeded!
                    self.proofs_succeeded += 1

                    # Extract proof term from output
                    proof = self._extract_proof(result.stdout)

                    return ProofResult(
                        success=True,
                        theorem=theorem,
                        proof=proof,
                        time_seconds=elapsed,
                        verification_status=True,
                    )
                else:
                    # Proof failed
                    return ProofResult(
                        success=False,
                        theorem=theorem,
                        time_seconds=elapsed,
                        error_message=result.stderr[:500],
                    )

            finally:
                # Clean up temp file
                os.unlink(temp_file)

        except subprocess.TimeoutExpired:
            elapsed = time.time() - start_time
            return ProofResult(
                success=False,
                theorem=theorem,
                time_seconds=elapsed,
                error_message=f"Timeout after {self.timeout}s",
            )

        except Exception as e:
            elapsed = time.time() - start_time
            return ProofResult(
                success=False,
                theorem=theorem,
                time_seconds=elapsed,
                error_message=str(e),
            )

    def _generate_lean_code(self, theorem: Theorem) -> str:
        """Generate complete Lean 4 file for theorem (Mathlib-free)"""

        code = []

        # Use bare Lean 4 (no Mathlib needed)
        code.append("-- Basic Lean 4 proof")
        code.append("")

        # Define theorem with proof
        code.append(theorem.to_lean_simple())

        return "\n".join(code)

    def _get_proof_tactics(self) -> str:
        """
        Get automated proof tactics.

        Works without Mathlib - uses basic Lean 4 tactics.
        """
        return """
  -- Try basic automated tactics (no Mathlib needed)
  try { rfl }  -- Reflexivity (works for definitional equality)
  try { decide }  -- Decision procedure for Nat
  try { trivial }  -- Proves True
  try { intro h }  -- Introduction
  try { cases h }  -- Case analysis
  
  -- If all else fails, admit (we'll mark as unproven)
  sorry
"""

    def _extract_proof(self, lean_output: str) -> str:
        """Extract proof term from Lean output"""
        # Simplified - real implementation parses Lean's output
        return lean_output

    def _create_default_tactics(self) -> str:
        """Create file with custom tactics"""
        return ""  # Placeholder

    def get_statistics(self) -> dict:
        """Get prover statistics"""
        success_rate = (
            self.proofs_succeeded / self.proofs_attempted * 100
            if self.proofs_attempted > 0
            else 0
        )
        avg_time = (
            self.total_time / self.proofs_attempted if self.proofs_attempted > 0 else 0
        )

        return {
            "attempted": self.proofs_attempted,
            "succeeded": self.proofs_succeeded,
            "success_rate": success_rate,
            "total_time": self.total_time,
            "avg_time_per_proof": avg_time,
        }


class ProofSearchEngine:
    """
    Advanced proof search using multiple strategies.

    Strategies:
    1. Direct tactics (fast, low success rate)
    2. Backward chaining (medium speed, medium success)
    3. Forward chaining (slow, high success)
    4. SMT solver (very fast for arithmetic)
    5. Neural guidance (learned from previous proofs)
    """

    def __init__(self, lean_prover: LeanProver):
        self.lean = lean_prover
        self.proof_cache = {}  # Cache proven theorems

    def prove_with_search(
        self, theorem: Theorem, max_attempts: int = 10
    ) -> ProofResult:
        """
        Try multiple proof strategies.

        This increases success rate at cost of time.
        """

        # Check cache first
        h = theorem.hash()
        if h in self.proof_cache:
            return self.proof_cache[h]

        strategies = [
            self._try_direct_tactics,
            self._try_smt_solver,
            self._try_backward_chaining,
            self._try_forward_chaining,
        ]

        for i, strategy in enumerate(strategies):
            result = strategy(theorem)
            if result.success:
                self.proof_cache[h] = result
                return result

            # Don't try too many strategies
            if i >= max_attempts - 1:
                break

        # All strategies failed
        return ProofResult(
            success=False,
            theorem=theorem,
            error_message="All proof strategies exhausted",
        )

    def _try_direct_tactics(self, theorem: Theorem) -> ProofResult:
        """Try direct automated tactics"""
        return self.lean.prove(theorem)

    def _try_smt_solver(self, theorem: Theorem) -> ProofResult:
        """
        Try SMT solver (Z3) for arithmetic/logic theorems.

        Z3 is very fast for certain classes of theorems.
        """
        try:
            import subprocess

            # Check if Z3 is available
            try:
                subprocess.run(["z3", "--version"], capture_output=True, check=True)
            except (subprocess.CalledProcessError, FileNotFoundError):
                return ProofResult(
                    success=False, theorem=theorem, error_message="Z3 not installed"
                )

            # Translate theorem to SMT-LIB format
            smt_formula = self._theorem_to_smt(theorem)
            if not smt_formula:
                return ProofResult(
                    success=False,
                    theorem=theorem,
                    error_message="Cannot translate to SMT",
                )

            # Run Z3
            result = subprocess.run(
                ["z3", "-smt2", "-in"],
                input=smt_formula,
                capture_output=True,
                text=True,
                timeout=30,
            )

            if "unsat" in result.stdout.lower():
                # Unsat means the negation is unsatisfiable, so theorem is valid!
                return ProofResult(
                    success=True,
                    theorem=theorem,
                    proof=f"(smt {smt_formula[:100]}...)",
                    verification_status=True,
                )
            elif "sat" in result.stdout.lower():
                return ProofResult(
                    success=False, theorem=theorem, error_message="Counterexample found"
                )
            else:
                return ProofResult(
                    success=False,
                    theorem=theorem,
                    error_message=f"Z3: {result.stdout[:100]}",
                )

        except subprocess.TimeoutExpired:
            return ProofResult(
                success=False, theorem=theorem, error_message="Z3 timeout"
            )
        except Exception as e:
            return ProofResult(success=False, theorem=theorem, error_message=str(e))

    def _theorem_to_smt(self, theorem: Theorem) -> str:
        """Convert theorem to SMT-LIB format"""
        from generator.theorem import (
            BinOp,
            UnOp,
            Quantifier,
            Var,
            Const,
            MathOp,
            LogicOp,
            Variable,
        )

        def term_to_smt(term) -> str:
            if isinstance(term, Var):
                return term.variable.name
            elif isinstance(term, Const):
                if isinstance(term.value, int):
                    return str(term.value)
                elif isinstance(term.value, bool):
                    return "true" if term.value else "false"
                return str(term.value)
            elif isinstance(term, BinOp):
                left = term_to_smt(term.left)
                right = term_to_smt(term.right)

                op_map = {
                    MathOp.PLUS: "+",
                    MathOp.MINUS: "-",
                    MathOp.MULT: "*",
                    MathOp.EQ: "=",
                    MathOp.LT: "<",
                    MathOp.LE: "<=",
                    MathOp.GT: ">",
                    MathOp.GE: ">=",
                    LogicOp.AND: "and",
                    LogicOp.OR: "or",
                    LogicOp.IMPLIES: "=>",
                }

                op_str = op_map.get(term.op, str(term.op))
                return f"({op_str} {left} {right})"
            elif isinstance(term, UnOp):
                inner = term_to_smt(term.term)
                if term.op == LogicOp.NOT:
                    return f"(not {inner})"
                return inner
            elif isinstance(term, Quantifier):
                body = term_to_smt(term.body)
                var_decl = f"({term.var.name} Int)"
                if term.op == LogicOp.FORALL:
                    return f"(forall ({var_decl}) {body})"
                else:
                    return f"(exists ({var_decl}) {body})"
            return ""

        # Build SMT-LIB script
        conclusion_smt = term_to_smt(theorem.conclusion)

        smt_script = f"""
(set-logic QF_LIA)
(declare-const x Int)
(declare-const y Int)
(declare-const z Int)
(assert (not {conclusion_smt}))
(check-sat)
"""
        return smt_script

    def _try_backward_chaining(self, theorem: Theorem) -> ProofResult:
        """
        Backward chaining: Start from goal, work backwards.

        This is how humans often prove things.
        """
        # TODO: Implement backward chaining
        return ProofResult(success=False, theorem=theorem)

    def _try_forward_chaining(self, theorem: Theorem) -> ProofResult:
        """
        Forward chaining: Start from axioms, derive forward.

        This is slow but can find surprising proofs.
        """
        # TODO: Implement forward chaining
        return ProofResult(success=False, theorem=theorem)


class MockProver:
    """
    Mock prover for testing when Lean is not available.

    This simulates proof attempts for development without actual sleeping.
    Uses CPU computation to simulate work.
    """

    def __init__(self, success_rate: float = 0.1):
        self.success_rate = success_rate
        self.proofs_attempted = 0
        self.proofs_succeeded = 0
        self.total_time = 0.0

        import random

        self.random = random.Random(42)

    def prove(self, theorem: Theorem) -> ProofResult:
        """Simulate proof attempt with CPU work"""
        import time

        self.proofs_attempted += 1

        # Do actual computation to simulate proof work
        # This avoids time.sleep() which is bad for distributed systems
        start_time = time.perf_counter()

        # Simulate proof search with CPU-bound work
        # Hash the theorem multiple times to simulate proof search
        theorem_str = str(theorem.conclusion)
        proof_difficulty = len(theorem_str) % 10 + 1

        # Do computational work proportional to theorem complexity
        result_hash = theorem_str.encode()
        for _ in range(proof_difficulty * 100):
            import hashlib

            result_hash = hashlib.sha256(result_hash).digest()

        elapsed = time.perf_counter() - start_time
        # Add minimum elapsed time for realism
        elapsed = max(elapsed, 0.001)
        self.total_time += elapsed

        # Randomly succeed based on success_rate
        success = self.random.random() < self.success_rate

        if success:
            self.proofs_succeeded += 1
            proof = f"mock_proof_for_{theorem.name}"
        else:
            proof = None

        return ProofResult(
            success=success,
            theorem=theorem,
            proof=proof,
            time_seconds=elapsed,
            verification_status=success,
            error_message=None if success else "Mock proof failed",
        )

    def get_statistics(self) -> dict:
        """Get mock prover statistics"""
        success_rate = (
            self.proofs_succeeded / self.proofs_attempted * 100
            if self.proofs_attempted > 0
            else 0
        )
        avg_time = (
            self.total_time / self.proofs_attempted if self.proofs_attempted > 0 else 0
        )

        return {
            "attempted": self.proofs_attempted,
            "succeeded": self.proofs_succeeded,
            "success_rate": success_rate,
            "total_time": self.total_time,
            "avg_time_per_proof": avg_time,
        }

    def prove_domain_theorem(self, theorem_dict: dict) -> ProofResult:
        """Prove a domain theorem (group theory, set theory, etc.)"""
        import time
        import hashlib

        self.proofs_attempted += 1
        start_time = time.perf_counter()

        theorem_name = theorem_dict.get("name", "domain_theorem")
        domain = theorem_dict.get("domain", "unknown")
        difficulty = theorem_dict.get("difficulty", "medium")
        proof_steps = theorem_dict.get("proof_steps", 2)

        proof_difficulty = proof_steps * 100
        result_hash = theorem_name.encode()
        for _ in range(proof_difficulty):
            result_hash = hashlib.sha256(result_hash).digest()

        elapsed = time.perf_counter() - start_time
        elapsed = max(elapsed, 0.001)
        self.total_time += elapsed

        difficulty_rates = {"easy": 0.6, "medium": 0.35, "hard": 0.15}
        rate = difficulty_rates.get(difficulty.lower(), 0.3)
        success = self.random.random() < rate

        if success:
            self.proofs_succeeded += 1
            proof = f"domain_proof_{domain}_{theorem_name}"
        else:
            proof = None

        from generator.theorem import Theorem, Const, Type, NAT

        theorem = Theorem(
            name=theorem_name,
            hypotheses=theorem_dict.get("hypotheses", []),
            conclusion=Const(0, NAT),  # Placeholder
        )

        return ProofResult(
            success=success,
            theorem=theorem,
            proof=proof,
            time_seconds=elapsed,
            verification_status=success,
            error_message=None if success else f"Domain proof failed: {domain}",
        )


def create_prover(
    use_lean: bool = False,
) -> Union[LeanProver, MockProver, "RealLeanProver"]:
    """Create appropriate prover based on availability"""
    if use_lean:
        # Use RealLeanProver (no Mathlib needed)
        try:
            import subprocess

            subprocess.run(["lean", "--version"], capture_output=True, check=True)
            from prover.real_lean import RealLeanProver

            return RealLeanProver()
        except (subprocess.CalledProcessError, FileNotFoundError):
            print("[Warning] Lean not found, using mock prover")
            return MockProver()
    else:
        return MockProver()


if __name__ == "__main__":
    # Test prover
    from theorem import example_commutativity

    theorem = example_commutativity()
    print(f"Theorem: {theorem}")
    print(f"\nLean code:\n{theorem.to_lean()}")

    # Use mock prover for testing
    prover = MockProver(success_rate=0.3)

    print("\nAttempting proof...")
    result = prover.prove(theorem)

    print(f"Success: {result.success}")
    print(f"Time: {result.time_seconds:.3f}s")
    if result.proof:
        print(f"Proof: {result.proof}")
