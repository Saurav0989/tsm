"""
Universal Discovery Engine - Main Entry Point

Supports both single-threaded and distributed modes.

Usage:
    # Single-threaded (default)
    python main.py

    # Distributed mode (Phase 2)
    python main.py --distributed --cores 8

    # With dashboard
    python main.py --distributed --dashboard
"""

import time
import argparse
import threading
from typing import Optional, Any
from dataclasses import dataclass

import sys
import os

# Handle path for different environments
if os.path.exists("/home/claude/ude"):
    sys.path.append("/home/claude/ude")
else:
    sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from generator.theorem import Theorem, ProofResult
from generator.engine import TheoremGenerator, create_default_space
from prover.lean import create_prover, MockProver
from prover.real_lean import RealLeanProver
from archive.storage import TheoremArchive, DiscoveryLog

try:
    from learner.self_improve import SelfImprovementEngine

    LEARNER_AVAILABLE = True
except ImportError:
    LEARNER_AVAILABLE = False
    SelfImprovementEngine = None

try:
    from domains import create_multi_domain_generator
    from checkpoint import CheckpointManager
    from generator.guided import GuidedGenerator, AdaptiveDomainSelector

    DOMAINS_AVAILABLE = True
except ImportError:
    DOMAINS_AVAILABLE = False
    create_multi_domain_generator = None
    CheckpointManager = None
    GuidedGenerator = None
    AdaptiveDomainSelector = None


@dataclass
class DiscoveryConfig:
    """Configuration for discovery process"""

    max_theorems: int = 1000000
    max_proven: int = 10000
    timeout_seconds: int = 60
    use_lean: bool = False
    batch_size: int = 100
    save_interval: int = 10
    log_interval: int = 100
    distributed: bool = False
    num_cores: int = 4
    dashboard: bool = False


class UniversalDiscoveryEngine:
    """The main discovery engine."""

    def __init__(self, config: DiscoveryConfig):
        self.config = config
        self.distributed_manager: Optional[Any] = None

        if config.distributed:
            self._init_distributed()
        else:
            self._init_standalone()

    def _init_standalone(self):
        """Initialize standalone mode"""
        print("[UDE] Initializing Universal Discovery Engine (Standalone Mode)...")

        self.space = create_default_space()
        self.generator = TheoremGenerator(self.space)
        self.prover = create_prover(use_lean=self.config.use_lean)
        self.archive = TheoremArchive()
        self.log = DiscoveryLog()

        if LEARNER_AVAILABLE:
            self.learner = SelfImprovementEngine(data_dir="learning_data")
            print("[Learner] Self-improvement engine initialized")
        else:
            self.learner = None

        if DOMAINS_AVAILABLE:
            self.domain_gen = create_multi_domain_generator()
            print("[Domains] Multi-domain generator initialized")
            print(
                f"[Domains] Available: {list(self.domain_gen.domain_generators.keys())}"
            )
        else:
            self.domain_gen = None

        if CheckpointManager:
            self.checkpoint_mgr = CheckpointManager(interval=self.config.save_interval)
            print("[Checkpoint] Manager initialized")
        else:
            self.checkpoint_mgr = None

        self.stats = {
            "theorems_generated": 0,
            "theorems_attempted": 0,
            "theorems_proven": 0,
            "total_time": 0.0,
            "start_time": time.time(),
        }

        print(f"[UDE] Initialized")
        print(
            f"[UDE] Search space size (estimated): {self.generator.estimate_search_space_size():,}"
        )

    def _init_distributed(self):
        """Initialize distributed mode"""
        print("[UDE] Initializing Universal Discovery Engine (Distributed Mode)...")

        from distributed import create_distributed_engine

        self.distributed_manager = create_distributed_engine(self.config.num_cores)

        self.stats = {
            "theorems_generated": 0,
            "theorems_attempted": 0,
            "theorems_proven": 0,
            "total_time": 0.0,
            "start_time": time.time(),
        }

        if self.config.dashboard:
            self._start_dashboard()

    def _start_dashboard(self):
        """Start the web dashboard"""
        from dashboard.server import start_dashboard_server, update_dashboard

        def stats_updater():
            while True:
                try:
                    if self.distributed_manager is not None:
                        stats = self.distributed_manager.get_stats()
                        update_dashboard(stats)
                except:
                    pass
                time.sleep(1)

        self.dashboard_thread = threading.Thread(target=stats_updater, daemon=True)
        self.dashboard_thread.start()

        dashboard_thread = threading.Thread(
            target=start_dashboard_server,
            args=(8080, os.path.dirname(os.path.abspath(__file__))),
            daemon=True,
        )
        dashboard_thread.start()
        print("[UDE] Dashboard available at http://localhost:8080/dashboard.html")

    def run(self):
        """Main discovery loop."""
        if self.config.distributed:
            self._run_distributed()
        else:
            self._run_standalone()

    def _run_standalone(self):
        """Run in standalone mode"""
        print(f"\n[UDE] Starting autonomous discovery (Standalone)...")
        print(f"[UDE] Target: {self.config.max_proven:,} proven theorems")
        print(f"[UDE] Max attempts: {self.config.max_theorems:,}")
        print()

        start_time = time.time()

        domain_theorems = []
        if self.domain_gen:
            print("[UDE] Loading domain theorems...")
            domain_theorems = self.domain_gen.generate_cross_domain_theorems(20)
            print(f"[UDE] Loaded {len(domain_theorems)} domain theorems")

        try:
            domain_index = 0

            for theorem in self.generator.generate_all_theorems(
                self.config.max_theorems
            ):
                self.stats["theorems_generated"] += 1

                if self.archive.is_proven(theorem):
                    continue

                result = self._attempt_proof(theorem)

                self.log.log_attempt(theorem, result)

                if result.success:
                    self._handle_new_theorem(result)

                if self.stats["theorems_attempted"] % self.config.log_interval == 0:
                    self._log_progress()

                if self.stats["theorems_proven"] >= self.config.max_proven:
                    print(
                        f"\n[UDE] Reached target of {self.config.max_proven} proven theorems!"
                    )
                    break

                if self.stats["theorems_attempted"] >= self.config.max_theorems:
                    print(f"\n[UDE] Reached max attempts of {self.config.max_theorems}")
                    break

            if (
                domain_theorems
                and self.stats["theorems_proven"] < self.config.max_proven
            ):
                print(f"\n[UDE] Now exploring domain theorems...")
                for dom_theorem in domain_theorems:
                    self.stats["theorems_generated"] += 1

                    result = self._attempt_domain_proof(dom_theorem)

                    if result.success:
                        self._handle_new_theorem(result)

                    if self.stats["theorems_proven"] >= self.config.max_proven:
                        break

        except KeyboardInterrupt:
            print("\n[UDE] Interrupted by user")

        finally:
            self._finalize()

    def _run_distributed(self):
        """Run in distributed mode"""
        print(f"\n[UDE] Starting autonomous discovery (Distributed)...")
        print(f"[UDE] Workers: {self.config.num_cores or 'auto'} cores")

        if self.distributed_manager is None:
            print("[ERROR] Distributed manager not initialized")
            return

        self.distributed_manager.start()

        try:
            while True:
                stats = self.distributed_manager.get_stats()

                if self.stats["theorems_attempted"] % self.config.log_interval == 0:
                    elapsed = time.time() - self.stats["start_time"]
                    print(f"\n[Progress after {elapsed:.0f}s]")
                    print(f"  Generated: {stats['theorems_generated']:,}")
                    print(f"  Attempted: {stats['theorems_attempted']:,}")
                    print(f"  Proven: {stats['theorems_proven']:,}")
                    print(f"  Rate: {stats['rate_proven']:.2f}/s")

                if stats["theorems_proven"] >= self.config.max_proven:
                    print(f"\n[UDE] Reached target!")
                    break

                if not stats["running"]:
                    print("\n[UDE] Workers stopped")
                    break

                time.sleep(5)

        except KeyboardInterrupt:
            print("\n[UDE] Interrupted by user")

        finally:
            self.distributed_manager.stop()
            self._finalize()

    def _attempt_proof(self, theorem: Theorem) -> ProofResult:
        """Attempt to prove a theorem"""
        self.stats["theorems_attempted"] += 1

        # Prove
        result = self.prover.prove(theorem)

        # Update statistics
        self.stats["total_time"] += result.time_seconds

        if self.learner is not None:
            structure = str(theorem)[:50] if theorem else ""
            self.learner.record_proof_attempt(
                theorem_hash=theorem.hash(),
                theorem_structure=structure,
                tactics_used=[],
                success=result.success,
                time_taken=result.time_seconds,
                axioms_used=[],
            )

        return result

    def _attempt_domain_proof(self, theorem_dict: dict) -> ProofResult:
        """Attempt to prove a domain theorem"""
        self.stats["theorems_attempted"] += 1

        theorem_name = theorem_dict.get("name", "domain_theorem")
        domain = theorem_dict.get("domain", "unknown")

        print(f"[Domain Proof] Attempting: {theorem_name} ({domain})")

        result = self.prover.prove_domain_theorem(theorem_dict)

        self.stats["total_time"] += result.time_seconds

        if self.learner is not None:
            self.learner.record_proof_attempt(
                theorem_hash=theorem_name,
                theorem_structure=domain,
                tactics_used=[],
                success=result.success,
                time_taken=result.time_seconds,
                axioms_used=[domain],
            )

        return result

    def _handle_new_theorem(self, result: ProofResult):
        """Handle newly proven theorem"""
        added = self.archive.add_theorem(result)

        if added:
            self.stats["theorems_proven"] += 1

            print(f"\n[DISCOVERY] Proven theorem #{self.stats['theorems_proven']}")
            print(f"  Name: {result.theorem.name}")
            print(f"  Statement: {result.theorem}")
            print(f"  Proof time: {result.time_seconds:.3f}s")

            # Periodically save
            if self.stats["theorems_proven"] % self.config.save_interval == 0:
                self._save_checkpoint()

    def _log_progress(self):
        """Log current progress"""
        elapsed = time.time() - self.stats["start_time"]

        rate_generated = (
            self.stats["theorems_generated"] / elapsed if elapsed > 0 else 0
        )
        rate_attempted = (
            self.stats["theorems_attempted"] / elapsed if elapsed > 0 else 0
        )
        rate_proven = self.stats["theorems_proven"] / elapsed if elapsed > 0 else 0

        success_rate = (
            self.stats["theorems_proven"] / self.stats["theorems_attempted"] * 100
            if self.stats["theorems_attempted"] > 0
            else 0
        )

        print(f"\n[Progress after {elapsed:.0f}s]")
        print(
            f"  Generated: {self.stats['theorems_generated']:,} ({rate_generated:.1f}/s)"
        )
        print(
            f"  Attempted: {self.stats['theorems_attempted']:,} ({rate_attempted:.1f}/s)"
        )
        print(f"  Proven: {self.stats['theorems_proven']:,} ({rate_proven:.2f}/s)")
        print(f"  Success rate: {success_rate:.2f}%")

        # Prover statistics
        if hasattr(self.prover, "get_statistics"):
            prover_stats = self.prover.get_statistics()
            print(f"  Prover avg time: {prover_stats['avg_time_per_proof']:.3f}s")

    def _save_checkpoint(self):
        """Save progress checkpoint"""
        if self.checkpoint_mgr:
            config_dict = {
                "max_theorems": self.config.max_theorems,
                "max_proven": self.config.max_proven,
                "use_lean": self.config.use_lean,
            }
            self.checkpoint_mgr.save_checkpoint(self, config_dict)
            print(f"[UDE] Checkpoint saved: {self.stats['theorems_proven']} theorems")
        else:
            print(f"[UDE] Checkpoint saved: {self.stats['theorems_proven']} theorems")

    def _finalize(self):
        """Cleanup and final statistics"""
        elapsed = time.time() - self.stats["start_time"]

        print(f"\n{'=' * 60}")
        print("FINAL STATISTICS")
        print(f"{'=' * 60}")
        print(f"Total time: {elapsed:.1f}s")

        # Get stats from distributed or standalone
        if self.config.distributed and self.distributed_manager:
            stats = self.distributed_manager.get_stats()
            print(f"Theorems generated: {stats['theorems_generated']:,}")
            print(f"Theorems attempted: {stats['theorems_attempted']:,}")
            print(f"Theorems proven: {stats['theorems_proven']:,}")
            success_rate = (
                stats["theorems_proven"] / stats["theorems_attempted"] * 100
                if stats["theorems_attempted"] > 0
                else 0
            )
        else:
            print(f"Theorems generated: {self.stats['theorems_generated']:,}")
            print(f"Theorems attempted: {self.stats['theorems_attempted']:,}")
            print(f"Theorems proven: {self.stats['theorems_proven']:,}")
            success_rate = (
                self.stats["theorems_proven"] / self.stats["theorems_attempted"] * 100
                if self.stats["theorems_attempted"] > 0
                else 0
            )

        print(f"Success rate: {success_rate:.2f}%")

        # Archive statistics (only for standalone)
        if not self.config.distributed and hasattr(self, "archive"):
            archive_stats = self.archive.get_statistics()
            print(f"\nArchive:")
            print(f"  Total theorems: {archive_stats['total_theorems']}")
            print(f"  Avg proof time: {archive_stats['avg_proof_time']:.3f}s")
            print(
                f"  Total computation: {archive_stats['total_computation_time']:.1f}s"
            )

        # Extrapolation
        if success_rate > 0 and elapsed > 60:
            theorems_per_day = (
                success_rate / 100 * self.stats["theorems_attempted"] / elapsed
            ) * 86400
            theorems_per_year = theorems_per_day * 365

            print(f"\nExtrapolation:")
            print(f"  At current rate: {theorems_per_day:.0f} theorems/day")
            print(f"  Per year: {theorems_per_year:.0f} theorems")
            print(f"  Humans (historical): ~100,000 total")
            print(f"  Time to match: {100000 / theorems_per_day:.1f} days")

        # Close resources
        if hasattr(self, "archive"):
            self.archive.close()
        if hasattr(self, "log"):
            self.log.close()

        print(f"\n[UDE] Shutdown complete")


def main():
    """Run the discovery engine"""
    parser = argparse.ArgumentParser(description="Universal Discovery Engine")
    parser.add_argument(
        "--distributed", action="store_true", help="Run in distributed mode"
    )
    parser.add_argument(
        "--cores", type=int, default=4, help="Number of cores for distributed mode"
    )
    parser.add_argument("--dashboard", action="store_true", help="Enable web dashboard")
    parser.add_argument(
        "--max-theorems", type=int, default=10000, help="Max theorems to generate"
    )
    parser.add_argument(
        "--max-proven", type=int, default=100, help="Stop after proving this many"
    )
    parser.add_argument(
        "--lean", action="store_true", help="Use real Lean prover (requires Lean 4)"
    )
    parser.add_argument(
        "--analyze", action="store_true", help="Run analysis on existing theorems"
    )
    parser.add_argument(
        "--export",
        choices=["json", "latex", "lean", "csv", "all"],
        help="Export theorems",
    )

    args = parser.parse_args()

    if args.analyze:
        from analyze import run_analysis

        run_analysis()
        return

    if args.export:
        from export import export_all

        formats = (
            [args.export] if args.export != "all" else ["json", "latex", "lean", "csv"]
        )
        export_all(formats)
        return

    config = DiscoveryConfig(
        max_theorems=args.max_theorems,
        max_proven=args.max_proven,
        use_lean=args.lean,
        distributed=args.distributed,
        num_cores=args.cores,
        dashboard=args.dashboard,
        batch_size=100,
        log_interval=50,
    )

    engine = UniversalDiscoveryEngine(config)
    engine.run()


if __name__ == "__main__":
    main()
