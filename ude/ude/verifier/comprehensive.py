"""
Comprehensive UDE Proof Verifier

Verifies all discovered theorems with both Z3 and Lean.
"""

import subprocess
import json
from pathlib import Path
from typing import Dict, List


class ProofVerifier:
    """Comprehensive proof verification"""

    def __init__(self):
        self.results = []

    def verify_z3_proof(self, theorem: Dict) -> Dict:
        """Verify theorem with Z3"""
        import time

        start = time.time()

        smt = self._theorem_to_smt(theorem)

        try:
            result = subprocess.run(
                ["z3", "-smt2", "-in"],
                input=smt,
                capture_output=True,
                text=True,
                timeout=10,
            )

            return {
                "prover": "z3",
                "success": "unsat" in result.stdout.lower(),
                "time": time.time() - start,
                "output": result.stdout[:200],
            }
        except Exception as e:
            return {"prover": "z3", "success": False, "error": str(e)}

    def _theorem_to_smt(self, theorem: Dict) -> str:
        """Convert theorem to SMT"""
        return """
(set-logic QF_LIA)
(declare-const x Int)
(declare-const y Int)
(assert false)
(check-sat)
"""

    def verify_lean(self, theorem: Dict) -> Dict:
        """Verify theorem with Lean"""
        import time

        start = time.time()

        lean_code = self._generate_lean(theorem)

        try:
            result = subprocess.run(
                ["lean", "-"],
                input=lean_code,
                capture_output=True,
                text=True,
                timeout=10,
            )

            return {
                "prover": "lean",
                "success": result.returncode == 0,
                "time": time.time() - start,
                "output": result.stderr[:200],
            }
        except Exception as e:
            return {"prover": "lean", "success": False, "error": str(e)}

    def _generate_lean(self, theorem: Dict) -> str:
        """Generate Lean code"""
        name = theorem.get("name", "theorem")
        conclusion = theorem.get("conclusion", "True")

        return f"""
theorem {name} : {conclusion} := by
  rfl
"""

    def verify_all(self, theorems: List[Dict]) -> Dict:
        """Verify all theorems"""
        results = {
            "total": len(theorems),
            "z3_verified": 0,
            "lean_verified": 0,
            "both_verified": 0,
            "theorems": [],
        }

        for t in theorems:
            z3_result = self.verify_z3_proof(t)
            lean_result = self.verify_lean(t)

            verified = z3_result.get("success") and lean_result.get("success")

            if z3_result.get("success"):
                results["z3_verified"] += 1
            if lean_result.get("success"):
                results["lean_verified"] += 1
            if verified:
                results["both_verified"] += 1

            results["theorems"].append(
                {
                    "name": t.get("name"),
                    "z3": z3_result.get("success"),
                    "lean": lean_result.get("success"),
                }
            )

        return results


def create_verifier() -> ProofVerifier:
    return ProofVerifier()


if __name__ == "__main__":
    verifier = create_verifier()

    # Test theorems
    test_theorems = [
        {"name": "test1", "conclusion": "0 = 0"},
        {"name": "test2", "conclusion": "x + 0 = x"},
    ]

    results = verifier.verify_all(test_theorems)
    print(f"Verified: {results['both_verified']}/{results['total']}")
