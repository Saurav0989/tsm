"""
Universal Discovery Engine - Distributed Task Queue

Task queue for distributing theorem generation and proving across multiple workers.
Uses threading for parallelism (works reliably on all platforms).

Components:
- TaskQueue: Thread-safe queue for work distribution
- WorkManager: Coordinates workers and tracks progress
- SharedState: Shared state across threads
"""

import threading
import queue
from typing import Any, Optional, Dict, List, Callable
from dataclasses import dataclass, field
from enum import Enum
import time
import uuid
import os


class TaskType(Enum):
    GENERATE = "generate"
    PROVE = "prove"
    VERIFY = "verify"
    LEARN = "learn"


@dataclass
class Task:
    """A unit of work to be distributed"""

    task_id: str
    task_type: TaskType
    payload: Dict[str, Any]
    priority: int = 0
    created_at: float = field(default_factory=time.time)


class SharedState:
    """Shared state across all workers using simple locks"""

    def __init__(self):
        self._lock = threading.Lock()

        self._theorems_generated = 0
        self._theorems_attempted = 0
        self._theorems_proven = 0
        self._theorems_verified = 0

        self._start_time = time.time()
        self._total_proof_time = 0.0

        self._running = True
        self._paused = False

        self._recent_discoveries = []
        self._workers = {}
        self._proven_hashes = set()

    def increment(self, counter_name: str, amount: int = 1):
        with self._lock:
            if counter_name == "theorems_generated":
                self._theorems_generated += amount
            elif counter_name == "theorems_attempted":
                self._theorems_attempted += amount
            elif counter_name == "theorems_proven":
                self._theorems_proven += amount
            elif counter_name == "theorems_verified":
                self._theorems_verified += amount
            elif counter_name == "total_proof_time":
                self._total_proof_time += amount

    def get_stats(self) -> Dict[str, Any]:
        with self._lock:
            elapsed = time.time() - self._start_time

            stats = {
                "theorems_generated": self._theorems_generated,
                "theorems_attempted": self._theorems_attempted,
                "theorems_proven": self._theorems_proven,
                "theorems_verified": self._theorems_verified,
                "elapsed_seconds": elapsed,
                "rate_generated": self._theorems_generated / elapsed
                if elapsed > 0
                else 0,
                "rate_proven": self._theorems_proven / elapsed if elapsed > 0 else 0,
                "running": self._running,
                "paused": self._paused,
                "workers": dict(self._workers),
                "recent_discoveries": list(self._recent_discoveries)[-10:],
            }
            return stats

    def add_discovery(self, theorem_data: Dict):
        with self._lock:
            self._recent_discoveries.append(theorem_data)
            if len(self._recent_discoveries) > 100:
                self._recent_discoveries = self._recent_discoveries[-100:]

    def register_worker(self, worker_id: str, worker_type: str):
        with self._lock:
            self._workers[worker_id] = {
                "type": worker_type,
                "started_at": time.time(),
                "tasks_completed": 0,
            }

    def update_worker(self, worker_id: str, tasks_completed: int):
        with self._lock:
            if worker_id in self._workers:
                self._workers[worker_id]["tasks_completed"] = tasks_completed

    def stop(self):
        with self._lock:
            self._running = False

    def pause(self):
        with self._lock:
            self._paused = True

    def resume(self):
        with self._lock:
            self._paused = False

    def is_running(self) -> bool:
        with self._lock:
            return self._running

    def is_paused(self) -> bool:
        with self._lock:
            return self._paused

    def is_proven(self, theorem_hash: str) -> bool:
        with self._lock:
            return theorem_hash in self._proven_hashes

    def mark_proven(self, theorem_hash: str):
        with self._lock:
            self._proven_hashes.add(theorem_hash)


class TaskQueue:
    """Thread-safe task queue"""

    def __init__(self, maxsize: int = 10000):
        self.queue = queue.Queue(maxsize=maxsize)
        self._pending_count = 0
        self._completed_count = 0
        self._failed_count = 0
        self._lock = threading.Lock()

    def put(self, task: Task, block: bool = True, timeout: Optional[float] = None):
        self.queue.put(task, block=block, timeout=timeout)
        with self._lock:
            self._pending_count += 1

    def get(
        self, block: bool = True, timeout: Optional[float] = None
    ) -> Optional[Task]:
        try:
            return self.queue.get(block=block, timeout=timeout)
        except queue.Empty:
            return None

    def mark_completed(self, success: bool = True):
        with self._lock:
            self._completed_count += 1
            self._pending_count -= 1

    def mark_failed(self):
        with self._lock:
            self._failed_count += 1
            self._pending_count -= 1

    def size(self) -> int:
        return self.queue.qsize()

    def is_empty(self) -> bool:
        return self.queue.empty()

    def get_stats(self) -> Dict[str, int]:
        with self._lock:
            return {
                "pending": self._pending_count,
                "completed": self._completed_count,
                "failed": self._failed_count,
                "queue_size": self.size(),
            }


def _generator_worker_fn(worker_id: str, state: SharedState, output_queue: TaskQueue):
    """Worker that generates theorems"""
    import random
    from generator.engine import TheoremGenerator, create_default_space

    space = create_default_space()
    generator = TheoremGenerator(space, seed=random.randint(0, 1000000))

    theorems_generated = 0

    while state.is_running():
        while state.is_paused() and state.is_running():
            time.sleep(0.1)

        if not state.is_running():
            break

        for theorem in generator.generate_all_theorems(max_count=50):
            if not state.is_running():
                break

            h = theorem.hash()
            if state.is_proven(h):
                continue

            theorem_dict = theorem.to_dict()

            task = Task(
                task_id=str(uuid.uuid4()),
                task_type=TaskType.PROVE,
                payload={"theorem": theorem_dict},
            )

            try:
                output_queue.put(task, block=False)
                theorems_generated += 1
                state.increment("theorems_generated")
            except:
                pass

        time.sleep(0.01)

    state.update_worker(worker_id, theorems_generated)


def _prover_worker_fn(worker_id: str, state: SharedState, input_queue: TaskQueue):
    """Worker that proves theorems"""
    from prover.lean import MockProver
    from generator.theorem import Theorem

    prover = MockProver(success_rate=0.15)

    theorems_proven = 0
    theorems_attempted = 0

    while state.is_running():
        while state.is_paused() and state.is_running():
            time.sleep(0.1)

        if not state.is_running():
            break

        task = input_queue.get(block=True, timeout=0.5)
        if task is None:
            continue

        theorems_attempted += 1
        state.increment("theorems_attempted")

        try:
            theorem_data = task.payload["theorem"]
            theorem = Theorem.from_dict(theorem_data)
            theorem_hash = theorem.hash()

            start_time = time.time()
            result = prover.prove(theorem)
            elapsed = time.time() - start_time

            state.increment("total_proof_time", int(elapsed * 1000))

            if result.success:
                theorems_proven += 1
                state.increment("theorems_proven")
                state.mark_proven(theorem_hash)

                state.add_discovery(
                    {
                        "name": theorem.name,
                        "hash": theorem_hash,
                        "proof_time": elapsed,
                        "timestamp": time.time(),
                    }
                )

            input_queue.mark_completed(result.success)

        except Exception as err:
            print(f"[Worker {worker_id}] Error: {err}")
            input_queue.mark_failed()

    state.update_worker(worker_id, theorems_proven)


class WorkManager:
    """Manages distributed workers"""

    def __init__(self, num_generators: int = 4, num_provers: int = 4):
        self.num_generators = num_generators
        self.num_provers = num_provers

        self.state = SharedState()

        self.generation_queue = TaskQueue()
        self.proof_queue = TaskQueue()
        self.verification_queue = TaskQueue()

        self.workers: List[threading.Thread] = []

        self.on_theorem_proven: Optional[Callable] = None

    def start(self):
        print(
            f"[Manager] Starting {self.num_generators} generators and {self.num_provers} provers..."
        )

        for i in range(self.num_generators):
            worker_id = f"generator_{i}"
            t = threading.Thread(
                target=_generator_worker_fn,
                args=(worker_id, self.state, self.proof_queue),
                daemon=True,
            )
            t.start()
            self.workers.append(t)
            self.state.register_worker(worker_id, "generator")

        for i in range(self.num_provers):
            worker_id = f"prover_{i}"
            t = threading.Thread(
                target=_prover_worker_fn,
                args=(worker_id, self.state, self.proof_queue),
                daemon=True,
            )
            t.start()
            self.workers.append(t)
            self.state.register_worker(worker_id, "prover")

        print(f"[Manager] Started {len(self.workers)} workers")

    def stop(self, timeout: float = 10.0):
        print("[Manager] Stopping workers...")
        self.state.stop()

        for worker in self.workers:
            worker.join(timeout=timeout)

        self.workers.clear()
        print("[Manager] All workers stopped")

    def get_stats(self) -> Dict[str, Any]:
        stats = self.state.get_stats()
        stats["generation_queue"] = self.generation_queue.get_stats()
        stats["proof_queue"] = self.proof_queue.get_stats()
        return stats

    def pause(self):
        self.state.pause()

    def resume(self):
        self.state.resume()


def create_distributed_engine(num_cores: int = None) -> WorkManager:
    """Factory function to create a distributed discovery engine"""
    if num_cores is None:
        num_cores = os.cpu_count() or 4

    # At least 1 worker per type, scale with cores
    num_generators = max(1, min(num_cores // 2, 8))
    num_provers = max(1, min(num_cores // 2, 8))

    print(f"[Engine] Creating distributed engine with {num_cores} cores")
    print(f"[Engine] {num_generators} generators, {num_provers} provers")

    return WorkManager(
        num_generators=num_generators,
        num_provers=num_provers,
    )


if __name__ == "__main__":
    print("Testing distributed task queue...")

    manager = create_distributed_engine(num_cores=4)
    manager.start()

    print("Running for 5 seconds...")
    time.sleep(5)

    stats = manager.get_stats()
    print(f"\nStatistics after 5 seconds:")
    print(f"  Generated: {stats['theorems_generated']}")
    print(f"  Attempted: {stats['theorems_attempted']}")
    print(f"  Proven: {stats['theorems_proven']}")
    print(f"  Rate: {stats['rate_proven']:.2f}/s")

    manager.stop()
    print("Done!")
