"""
Continuous Autonomous Operation Mode

Runs the UDE continuously without human intervention.
"""

import time
import signal
import sys
from typing import Optional
from pathlib import Path


class ContinuousOperation:
    """Manages continuous autonomous discovery"""

    def __init__(self, engine, checkpoint_dir: str = "checkpoints"):
        self.engine = engine
        self.checkpoint_dir = Path(checkpoint_dir)
        self.checkpoint_dir.mkdir(exist_ok=True)
        self.running = False
        self.iteration = 0

        signal.signal(signal.SIGINT, self._signal_handler)
        signal.signal(signal.SIGTERM, self._signal_handler)

    def _signal_handler(self, signum, frame):
        """Handle shutdown signals gracefully"""
        print("\n[Continuous] Received shutdown signal, saving state...")
        self.running = False

    def run_continuously(
        self,
        target_theorems: int = 10000,
        checkpoint_interval: int = 100,
        max_runtime_hours: float = 24 * 7,
    ):
        """Run discovery continuously for up to max_runtime_hours"""
        self.running = True
        start_time = time.time()
        max_runtime = max_runtime_hours * 3600

        print(f"[Continuous] Starting continuous operation")
        print(f"[Continuous] Target: {target_theorems} theorems")
        print(f"[Continuous] Max runtime: {max_runtime_hours} hours")

        while self.running:
            elapsed = time.time() - start_time

            if elapsed > max_runtime:
                print(f"[Continuous] Max runtime reached ({max_runtime_hours}h)")
                break

            if self.engine.stats["theorems_proven"] >= target_theorems:
                print(f"[Continuous] Target reached!")
                break

            self.iteration += 1

            current = self.engine.stats["theorems_proven"]

            if current % checkpoint_interval == 0 and current > 0:
                self._save_checkpoint()
                self._log_status()

            if self.iteration % 10 == 0:
                self._log_status()

            time.sleep(1)

        self._final_report(start_time)

    def _save_checkpoint(self):
        """Save current state"""
        checkpoint_file = self.checkpoint_dir / f"continuous_{int(time.time())}.json"

        import json

        data = {
            "timestamp": time.time(),
            "iteration": self.iteration,
            "theorems_proven": self.engine.stats["theorems_proven"],
            "theorems_generated": self.engine.stats["theorems_generated"],
            "success_rate": self._calculate_success_rate(),
        }

        with open(checkpoint_file, "w") as f:
            json.dump(data, f, indent=2)

        print(f"[Continuous] Checkpoint saved: {checkpoint_file.name}")

    def _log_status(self):
        """Log current status"""
        stats = self.engine.stats
        elapsed = time.time() - stats.get("start_time", time.time())
        rate = stats["theorems_proven"] / elapsed if elapsed > 0 else 0

        print(
            f"[Continuous] {stats['theorems_proven']} theorems | {rate:.1f}/sec | {elapsed / 3600:.1f}h"
        )

    def _calculate_success_rate(self) -> float:
        """Calculate current success rate"""
        attempted = self.engine.stats["theorems_attempted"]
        proven = self.engine.stats["theorems_proven"]

        if attempted == 0:
            return 0.0

        return proven / attempted * 100

    def _final_report(self, start_time: float):
        """Print final report"""
        elapsed = time.time() - start_time
        stats = self.engine.stats

        print("\n" + "=" * 60)
        print("CONTINUOUS OPERATION COMPLETE")
        print("=" * 60)
        print(f"Runtime: {elapsed / 3600:.2f} hours")
        print(f"Theorems generated: {stats['theorems_generated']:,}")
        print(f"Theorems proven: {stats['theorems_proven']:,}")
        print(f"Success rate: {self._calculate_success_rate():.2f}%")
        print(f"Rate: {stats['theorems_proven'] / elapsed:.2f}/sec")
        print("=" * 60)


def run_continuous_discovery(engine, **kwargs):
    """Run the engine in continuous mode"""
    op = ContinuousOperation(engine)
    op.run_continuously(**kwargs)
