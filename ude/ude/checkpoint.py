"""
Checkpoint System for UDE

Enables continuous operation with state persistence.
"""

import json
import os
import time
from pathlib import Path
from typing import Dict, Any, Optional
from dataclasses import dataclass, asdict


@dataclass
class CheckpointState:
    """Saved state of the discovery engine"""

    timestamp: float
    theorems_generated: int
    theorems_attempted: int
    theorems_proven: int
    total_time: float
    generation_count: int
    stats: Dict[str, Any]
    config: Dict[str, Any]


class CheckpointManager:
    """Manages checkpoint saving and loading"""

    def __init__(self, checkpoint_dir: str = "checkpoints", interval: int = 100):
        self.checkpoint_dir = Path(checkpoint_dir)
        self.checkpoint_dir.mkdir(exist_ok=True)
        self.interval = interval
        self.last_save = 0

    def save_checkpoint(self, engine, config: Dict[str, Any]) -> str:
        """Save current state to checkpoint"""
        state = CheckpointState(
            timestamp=time.time(),
            theorems_generated=engine.stats.get("theorems_generated", 0),
            theorems_attempted=engine.stats.get("theorems_attempted", 0),
            theorems_proven=engine.stats.get("theorems_proven", 0),
            total_time=engine.stats.get("total_time", 0.0),
            generation_count=getattr(engine.generator, "generation_count", 0),
            stats=engine.stats.copy(),
            config=config,
        )

        filename = f"checkpoint_{int(state.timestamp)}.json"
        filepath = self.checkpoint_dir / filename

        with open(filepath, "w") as f:
            json.dump(asdict(state), f, indent=2)

        self.last_save = time.time()

        self._cleanup_old_checkpoints()

        return str(filepath)

    def _cleanup_old_checkpoints(self, keep: int = 5):
        """Keep only the most recent N checkpoints"""
        checkpoints = sorted(
            self.checkpoint_dir.glob("checkpoint_*.json"),
            key=lambda p: p.stat().st_mtime,
            reverse=True,
        )

        for old in checkpoints[keep:]:
            old.unlink()

    def load_latest_checkpoint(self) -> Optional[CheckpointState]:
        """Load the most recent checkpoint"""
        checkpoints = sorted(
            self.checkpoint_dir.glob("checkpoint_*.json"),
            key=lambda p: p.stat().st_mtime,
            reverse=True,
        )

        if not checkpoints:
            return None

        with open(checkpoints[0]) as f:
            data = json.load(f)

        return CheckpointState(**data)

    def should_save(self, current_count: int) -> bool:
        """Check if it's time to save a checkpoint"""
        return current_count - self.last_save >= self.interval

    def get_latest_stats(self) -> Optional[Dict]:
        """Get stats from latest checkpoint without loading full state"""
        checkpoint = self.load_latest_checkpoint()
        if checkpoint:
            return checkpoint.stats
        return None
