"""
Universal Discovery Engine - Knowledge Archive

Stores all proven theorems and enables efficient querying.

This is the "memory" of the discovery system.
As theorems are proven, they become available for:
- Guided generation
- Pattern learning
- Building on previous work
"""

import json
import sqlite3
from typing import List, Optional, Set, Dict
from dataclasses import asdict
from pathlib import Path

from generator.theorem import Theorem, ProofResult


class TheoremArchive:
    """
    Persistent storage for proven theorems.

    Uses SQLite for reliability and querying capabilities.
    """

    def __init__(self, db_path: str = "theorems.db"):
        self.db_path = db_path
        self.conn = sqlite3.connect(db_path)
        self._create_tables()

        # In-memory cache for fast access
        self.proven_hashes: Set[str] = set()
        self._load_proven_hashes()

    def _create_tables(self):
        """Create database schema"""
        cursor = self.conn.cursor()

        cursor.execute("""
            CREATE TABLE IF NOT EXISTS theorems (
                hash TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                hypotheses TEXT NOT NULL,
                conclusion TEXT NOT NULL,
                proof TEXT,
                proof_time_seconds REAL,
                discovered_timestamp INTEGER,
                verification_status BOOLEAN
            )
        """)

        cursor.execute("""
            CREATE TABLE IF NOT EXISTS statistics (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
        """)

        cursor.execute("""
            CREATE INDEX IF NOT EXISTS idx_name ON theorems(name)
        """)

        cursor.execute("""
            CREATE INDEX IF NOT EXISTS idx_timestamp ON theorems(discovered_timestamp)
        """)

        self.conn.commit()

    def _load_proven_hashes(self):
        """Load proven theorem hashes into memory"""
        cursor = self.conn.cursor()
        cursor.execute("SELECT hash FROM theorems")
        self.proven_hashes = {row[0] for row in cursor.fetchall()}

    def add_theorem(self, result: ProofResult) -> bool:
        """
        Add proven theorem to archive.

        Returns True if added (new), False if already exists.
        """
        if not result.success:
            return False

        theorem = result.theorem
        h = theorem.hash()

        if h in self.proven_hashes:
            return False  # Already have this theorem

        cursor = self.conn.cursor()

        import time

        timestamp = int(time.time())

        # Use proper JSON serialization
        import json

        theorem_json = json.dumps(theorem.to_dict())

        cursor.execute(
            """
            INSERT INTO theorems (
                hash, name, hypotheses, conclusion, proof,
                proof_time_seconds, discovered_timestamp, verification_status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """,
            (
                h,
                theorem.name,
                theorem_json,
                theorem_json,
                result.proof,
                result.time_seconds,
                timestamp,
                result.verification_status,
            ),
        )

        self.conn.commit()
        self.proven_hashes.add(h)

        return True

    def is_proven(self, theorem: Theorem) -> bool:
        """Check if theorem is already proven"""
        return theorem.hash() in self.proven_hashes

    def get_all_proven(self, limit: Optional[int] = None) -> List["Theorem"]:
        """Get all proven theorems"""
        import json

        cursor = self.conn.cursor()

        if limit:
            cursor.execute(
                """
                SELECT name, hypotheses, conclusion, proof
                FROM theorems
                ORDER BY discovered_timestamp DESC
                LIMIT ?
            """,
                (limit,),
            )
        else:
            cursor.execute("""
                SELECT name, hypotheses, conclusion, proof
                FROM theorems
                ORDER BY discovered_timestamp DESC
            """)

        theorems = []
        for row in cursor.fetchall():
            name, theorem_json, _, proof = row

            # Deserialize theorem properly
            try:
                data = json.loads(theorem_json)
                theorem = Theorem.from_dict(data)
                theorem.proof = proof
                theorems.append(theorem)
            except Exception:
                # Fallback for old format
                from generator.theorem import Theorem as ThmClass

                theorem = ThmClass(
                    name=name,
                    hypotheses=[],
                    conclusion=None,
                    proof=proof,
                )
                theorems.append(theorem)

        return theorems

    def get_statistics(self) -> Dict[str, int]:
        """Get archive statistics"""
        cursor = self.conn.cursor()

        cursor.execute("SELECT COUNT(*) FROM theorems")
        total_theorems = cursor.fetchone()[0]

        cursor.execute("SELECT AVG(proof_time_seconds) FROM theorems")
        avg_time = cursor.fetchone()[0] or 0.0

        cursor.execute("SELECT SUM(proof_time_seconds) FROM theorems")
        total_time = cursor.fetchone()[0] or 0.0

        return {
            "total_theorems": total_theorems,
            "avg_proof_time": avg_time,
            "total_computation_time": total_time,
        }

    def export_to_lean(self, output_file: str):
        """Export all theorems as a Lean 4 library"""
        cursor = self.conn.cursor()
        cursor.execute("SELECT name, hypotheses, conclusion, proof FROM theorems")

        with open(output_file, "w") as f:
            f.write("-- Auto-generated by Universal Discovery Engine\n")
            f.write("import Mathlib.Tactic\n\n")

            for row in cursor.fetchall():
                name, _, _, proof = row
                f.write(f"theorem {name} : ... := by\n")
                f.write(f"  {proof}\n\n")

    def close(self):
        """Close database connection"""
        self.conn.close()


class DiscoveryLog:
    """
    Log of discovery process for analysis.

    Tracks:
    - Which theorems were attempted
    - Success/failure rates over time
    - Computational resources used
    """

    def __init__(self, log_file: str = "discovery_log.jsonl"):
        self.log_file = log_file
        self.file = open(log_file, "a")

    def log_attempt(self, theorem: Theorem, result: ProofResult):
        """Log a proof attempt"""
        entry = {
            "theorem_name": theorem.name,
            "theorem_hash": theorem.hash(),
            "success": result.success,
            "proof_time": result.time_seconds,
            "verification": result.verification_status,
            "error": result.error_message,
        }

        import time

        entry["timestamp"] = time.time()

        self.file.write(json.dumps(entry) + "\n")
        self.file.flush()

    def close(self):
        """Close log file"""
        self.file.close()


if __name__ == "__main__":
    # Test archive
    from theorem import example_commutativity

    archive = TheoremArchive("test_theorems.db")

    theorem = example_commutativity()

    # Simulate adding a proven theorem
    result = ProofResult(
        success=True,
        theorem=theorem,
        proof="rfl",
        time_seconds=0.15,
        verification_status=True,
    )

    added = archive.add_theorem(result)
    print(f"Added: {added}")

    # Check if proven
    print(f"Is proven: {archive.is_proven(theorem)}")

    # Get statistics
    stats = archive.get_statistics()
    print(f"Statistics: {stats}")

    archive.close()
