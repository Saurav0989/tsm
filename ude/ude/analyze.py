"""
Universal Discovery Engine - Checkpoint Manager

Handles saving and resuming discovery state for long-running sessions.
"""

import json
import os
import time
from typing import Dict, Any, Optional
from pathlib import Path


class CheckpointManager:
    """Manages checkpointing for long-running discovery sessions"""

    def __init__(self, checkpoint_dir: str = "checkpoints"):
        self.checkpoint_dir = Path(checkpoint_dir)
        self.checkpoint_dir.mkdir(exist_ok=True)

        self.current_checkpoint_file = self.checkpoint_dir / "current.json"
        self.auto_save_interval = 60  # seconds

    def save_checkpoint(self, state: Dict[str, Any], stats: Dict[str, Any]) -> str:
        """Save current state to checkpoint file"""
        checkpoint = {
            "timestamp": time.time(),
            "state": state,
            "stats": stats,
        }

        # Save to temp file first, then rename (atomic)
        temp_file = self.checkpoint_dir / f"checkpoint_{int(time.time())}.tmp"
        with open(temp_file, "w") as f:
            json.dump(checkpoint, f, indent=2)

        # Rename to current
        temp_file.rename(self.current_checkpoint_file)

        return str(self.current_checkpoint_file)

    def load_checkpoint(self) -> Optional[Dict[str, Any]]:
        """Load most recent checkpoint if exists"""
        if not self.current_checkpoint_file.exists():
            return None

        try:
            with open(self.current_checkpoint_file, "r") as f:
                return json.load(f)
        except Exception as e:
            print(f"[Checkpoint] Failed to load: {e}")
            return None

    def get_checkpoint_age(self) -> float:
        """Get age of current checkpoint in seconds"""
        if not self.current_checkpoint_file.exists():
            return -1

        mtime = os.path.getmtime(self.current_checkpoint_file)
        return time.time() - mtime

    def list_checkpoints(self) -> list:
        """List all checkpoint files"""
        checkpoints = []
        for f in self.checkpoint_dir.glob("checkpoint_*.json"):
            checkpoints.append(
                {
                    "file": str(f),
                    "time": f.stat().st_mtime,
                    "size": f.stat().st_size,
                }
            )
        return sorted(checkpoints, key=lambda x: x["time"], reverse=True)

    def cleanup_old_checkpoints(self, keep: int = 5):
        """Remove old checkpoints, keeping only the most recent N"""
        checkpoints = self.list_checkpoints()

        for ckpt in checkpoints[keep:]:
            try:
                os.remove(ckpt["file"])
                print(f"[Checkpoint] Removed old checkpoint: {ckpt['file']}")
            except Exception as e:
                print(f"[Checkpoint] Failed to remove {ckpt['file']}: {e}")


class DiscoveryAnalyzer:
    """Analyzes discovered theorems to find interesting ones"""

    def __init__(self, archive):
        self.archive = archive

    def analyze_discoveries(self) -> Dict[str, Any]:
        """Analyze all discovered theorems"""
        theorems = self.archive.get_all_proven()

        if not theorems:
            return {"error": "No theorems to analyze"}

        analysis = {
            "total_theorems": len(theorems),
            "by_structure": {},
            "by_complexity": {
                "simple": 0,  # depth 1-2
                "medium": 0,  # depth 3-4
                "complex": 0,  # depth 5+
            },
            "by_quantifiers": {
                "unquantified": 0,
                "forall": 0,
                "exists": 0,
                "both": 0,
            },
            "interesting": [],
            "novel_candidates": [],
        }

        from generator.theorem import BinOp, UnOp, Quantifier, Var, Const

        for theorem in theorems:
            # Analyze structure
            if theorem.conclusion:
                struct = self._analyze_structure(theorem.conclusion)
                analysis["by_structure"][struct] = (
                    analysis["by_structure"].get(struct, 0) + 1
                )

                # Complexity
                depth = self._term_depth(theorem.conclusion)
                if depth <= 2:
                    analysis["by_complexity"]["simple"] += 1
                elif depth <= 4:
                    analysis["by_complexity"]["medium"] += 1
                else:
                    analysis["by_complexity"]["complex"] += 1

                # Quantifiers
                num_forall = self._count_quantifier(theorem.conclusion, "FORALL")
                num_exists = self._count_quantifier(theorem.conclusion, "EXISTS")

                if num_forall == 0 and num_exists == 0:
                    analysis["by_quantifiers"]["unquantified"] += 1
                elif num_forall > 0 and num_exists == 0:
                    analysis["by_quantifiers"]["forall"] += 1
                elif num_forall == 0 and num_exists > 0:
                    analysis["by_quantifiers"]["exists"] += 1
                else:
                    analysis["by_quantifiers"]["both"] += 1

                # Interesting theorems
                if self._is_interesting(theorem, depth, num_forall, num_exists):
                    analysis["interesting"].append(
                        {
                            "name": theorem.name,
                            "hash": theorem.hash(),
                            "structure": struct,
                            "depth": depth,
                        }
                    )

                # Novel candidates (complex, quantified)
                if depth >= 3 and (num_forall >= 1 or num_exists >= 1):
                    analysis["novel_candidates"].append(
                        {
                            "name": theorem.name,
                            "hash": theorem.hash(),
                        }
                    )

        return analysis

    def _analyze_structure(self, term) -> str:
        """Get structural summary of term"""
        from generator.theorem import BinOp, UnOp, Quantifier, Var, Const

        if isinstance(term, Var):
            return "Variable"
        elif isinstance(term, Const):
            return f"Const({term.value})"
        elif isinstance(term, BinOp):
            left = self._analyze_structure(term.left)
            right = self._analyze_structure(term.right)
            return f"BinOp({term.op.value}, {left}, {right})"
        elif isinstance(term, UnOp):
            return f"UnOp({term.op.value}, {self._analyze_structure(term.term)})"
        elif isinstance(term, Quantifier):
            return f"Quant({term.op.value}, {self._analyze_structure(term.body)})"
        return "Unknown"

    def _term_depth(self, term) -> int:
        """Calculate term depth"""
        from generator.theorem import BinOp, UnOp, Quantifier, Var, Const

        if isinstance(term, (Var, Const)):
            return 1
        elif isinstance(term, BinOp):
            return 1 + max(self._term_depth(term.left), self._term_depth(term.right))
        elif isinstance(term, UnOp):
            return 1 + self._term_depth(term.term)
        elif isinstance(term, Quantifier):
            return 1 + self._term_depth(term.body)
        return 1

    def _count_quantifier(self, term, qtype: str) -> int:
        """Count specific quantifier type"""
        from generator.theorem import Quantifier, BinOp, UnOp

        count = 0
        if isinstance(term, Quantifier):
            if term.op.value == qtype:
                count = 1
            count += self._count_quantifier(term.body, qtype)
        elif isinstance(term, BinOp):
            count = self._count_quantifier(term.left, qtype) + self._count_quantifier(
                term.right, qtype
            )
        elif isinstance(term, UnOp):
            count = self._count_quantifier(term.term, qtype)
        return count

    def _is_interesting(
        self, theorem, depth: int, num_forall: int, num_exists: int
    ) -> bool:
        """Heuristics for interesting theorems"""
        # Complex theorems are more interesting
        if depth >= 4:
            return True

        # Theorems with both quantifiers are interesting
        if num_forall >= 1 and num_exists >= 1:
            return True

        # Nested quantifiers
        if num_forall >= 2 or num_exists >= 2:
            return True

        return False

    def generate_report(self) -> str:
        """Generate human-readable analysis report"""
        analysis = self.analyze_discoveries()

        if "error" in analysis:
            return f"Error: {analysis['error']}"

        report = []
        report.append("=" * 60)
        report.append("DISCOVERY ANALYSIS REPORT")
        report.append("=" * 60)
        report.append("")
        report.append(f"Total Theorems: {analysis['total_theorems']}")
        report.append("")
        report.append("Complexity Distribution:")
        report.append(f"  Simple (depth 1-2): {analysis['by_complexity']['simple']}")
        report.append(f"  Medium (depth 3-4): {analysis['by_complexity']['medium']}")
        report.append(f"  Complex (depth 5+): {analysis['by_complexity']['complex']}")
        report.append("")
        report.append("Quantifier Distribution:")
        report.append(f"  Unquantified: {analysis['by_quantifiers']['unquantified']}")
        report.append(f"  FORALL only: {analysis['by_quantifiers']['forall']}")
        report.append(f"  EXISTS only: {analysis['by_quantifiers']['exists']}")
        report.append(f"  Both: {analysis['by_quantifiers']['both']}")
        report.append("")

        if analysis["interesting"]:
            report.append(f"Interesting Theorems ({len(analysis['interesting'])}):")
            for t in analysis["interesting"][:10]:
                report.append(f"  - {t['name']} (depth={t['depth']})")

        if analysis["novel_candidates"]:
            report.append(f"\nNovel Candidates ({len(analysis['novel_candidates'])}):")
            for t in analysis["novel_candidates"][:10]:
                report.append(f"  - {t['name']}")

        report.append("")
        report.append("=" * 60)

        return "\n".join(report)


def run_analysis(archive_path: str = "theorems.db"):
    """Run discovery analysis"""
    from archive.storage import TheoremArchive

    print("Loading archive...")
    archive = TheoremArchive(archive_path)

    print("Analyzing...")
    analyzer = DiscoveryAnalyzer(archive)
    report = analyzer.generate_report()
    print(report)

    archive.close()


if __name__ == "__main__":
    import sys

    archive_path = sys.argv[1] if len(sys.argv) > 1 else "theorems.db"
    run_analysis(archive_path)
