"""
Self-Improvement Metrics Tracker

Tracks and reports on system improvement over time.
"""

import json
import time
from pathlib import Path
from typing import Dict, List, Optional
from dataclasses import dataclass, asdict
from collections import deque


@dataclass
class MetricsSnapshot:
    """A point-in-time snapshot of system metrics"""

    timestamp: float
    theorems_generated: int
    theorems_proven: int
    success_rate: float
    avg_proof_time: float
    domain_breakdown: Dict[str, int]


class ImprovementTracker:
    """Tracks improvement metrics over time"""

    def __init__(self, data_dir: str = "learning_data"):
        self.data_dir = Path(data_dir)
        self.data_dir.mkdir(exist_ok=True)
        self.metrics_file = self.data_dir / "improvement_metrics.json"

        self.history: deque = deque(maxlen=1000)
        self.start_time = time.time()

        self.load_history()

    def load_history(self):
        """Load previous metrics"""
        if self.metrics_file.exists():
            with open(self.metrics_file) as f:
                data = json.load(f)
                self.history = deque(data.get("history", []), maxlen=1000)
                self.start_time = data.get("start_time", time.time())

    def save_history(self):
        """Save metrics to disk"""
        data = {"start_time": self.start_time, "history": list(self.history)}
        with open(self.metrics_file, "w") as f:
            json.dump(data, f, indent=2)

    def record_snapshot(
        self,
        theorems_generated: int,
        theorems_proven: int,
        total_attempts: int,
        total_time: float,
        domain_breakdown: Dict[str, int],
    ):
        """Record a metrics snapshot"""
        success_rate = theorems_proven / total_attempts if total_attempts > 0 else 0
        avg_proof_time = total_time / theorems_proven if theorems_proven > 0 else 0

        snapshot = MetricsSnapshot(
            timestamp=time.time(),
            theorems_generated=theorems_generated,
            theorems_proven=theorems_proven,
            success_rate=success_rate,
            avg_proof_time=avg_proof_time,
            domain_breakdown=domain_breakdown,
        )

        self.history.append(asdict(snapshot))

        if len(self.history) % 10 == 0:
            self.save_history()

    def get_improvement_report(self) -> Dict:
        """Generate improvement report comparing early vs recent performance"""
        if len(self.history) < 10:
            return {
                "status": "insufficient_data",
                "message": "Need at least 10 snapshots",
            }

        early = list(self.history)[: len(self.history) // 4]
        recent = list(self.history)[-len(self.history) // 4 :]

        early_success = sum(s["success_rate"] for s in early) / len(early)
        recent_success = sum(s["success_rate"] for s in recent) / len(recent)

        early_time = sum(s["avg_proof_time"] for s in early) / len(early)
        recent_time = sum(s["avg_proof_time"] for s in recent) / len(recent)

        success_improvement = (
            ((recent_success - early_success) / early_success * 100)
            if early_success > 0
            else 0
        )
        time_improvement = (
            ((early_time - recent_time) / early_time * 100) if early_time > 0 else 0
        )

        return {
            "status": "analyzed",
            "total_snapshots": len(self.history),
            "runtime_hours": (time.time() - self.start_time) / 3600,
            "early_period": {
                "success_rate": early_success,
                "avg_proof_time": early_time,
            },
            "recent_period": {
                "success_rate": recent_success,
                "avg_proof_time": recent_time,
            },
            "improvements": {
                "success_rate_change_percent": success_improvement,
                "proof_speed_change_percent": time_improvement,
            },
            "is_improving": success_improvement > 5 or time_improvement > 5,
        }

    def get_domain_trends(self) -> Dict:
        """Analyze which domains are improving"""
        if len(self.history) < 5:
            return {"status": "insufficient_data"}

        early_domains = self.history[0].get("domain_breakdown", {})
        recent_domains = self.history[-1].get("domain_breakdown", {})

        trends = {}
        all_domains = set(early_domains.keys()) | set(recent_domains.keys())

        for domain in all_domains:
            early_count = early_domains.get(domain, 0)
            recent_count = recent_domains.get(domain, 0)
            if early_count > 0:
                trends[domain] = {
                    "change": recent_count - early_count,
                    "percent_change": (recent_count - early_count) / early_count * 100,
                }

        return {
            "status": "analyzed",
            "domain_trends": trends,
            "most_improved": max(trends.items(), key=lambda x: x[1]["change"])
            if trends
            else None,
        }


def create_tracker(data_dir: str = "learning_data") -> ImprovementTracker:
    return ImprovementTracker(data_dir)
