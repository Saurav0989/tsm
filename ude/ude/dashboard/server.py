"""
Universal Discovery Engine - Web Dashboard

Real-time dashboard for monitoring the discovery engine.
Provides:
- Live statistics
- Recent discoveries
- Worker status
- Control buttons (start/stop/pause)

Run: python -m http.server 8080
Then open http://localhost:8080/dashboard.html
"""

import json
import time
import threading
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
from typing import Dict, Any, Optional
import socketserver

# Dashboard state (updated by main engine)
_dashboard_state: Dict[str, Any] = {
    "stats": {
        "theorems_generated": 0,
        "theorems_attempted": 0,
        "theorems_proven": 0,
        "theorems_verified": 0,
        "elapsed_seconds": 0,
        "rate_generated": 0,
        "rate_proven": 0,
    },
    "workers": {},
    "recent_discoveries": [],
    "running": False,
    "paused": False,
    "start_time": None,
}

_state_lock = threading.Lock()


def update_dashboard(stats: Dict[str, Any]):
    """Update dashboard with new stats"""
    global _dashboard_state

    with _state_lock:
        _dashboard_state["stats"] = stats.get("theorems_generated", 0)
        _dashboard_state["workers"] = stats.get("workers", {})
        _dashboard_state["recent_discoveries"] = stats.get("recent_discoveries", [])
        _dashboard_state["running"] = stats.get("running", True)
        _dashboard_state["paused"] = stats.get("paused", False)


class DashboardHandler(SimpleHTTPRequestHandler):
    """HTTP handler for dashboard"""

    def do_GET(self):
        if self.path == "/api/stats":
            self.send_response(200)
            self.send_header("Content-type", "application/json")
            self.end_headers()

            with _state_lock:
                response = json.dumps(_dashboard_state)
            self.wfile.write(response.encode())

        elif self.path == "/api/health":
            self.send_response(200)
            self.send_header("Content-type", "application/json")
            self.end_headers()
            self.wfile.write(b'{"status": "ok"}')

        else:
            # Serve static files
            return SimpleHTTPRequestHandler.do_GET(self)

    def do_POST(self):
        if self.path == "/api/control":
            content_length = int(self.headers["Content-Length"])
            post_data = self.rfile.read(content_length)
            data = json.loads(post_data)

            command = data.get("command")

            # Handle control commands
            if command == "stop":
                global _dashboard_state
                with _state_lock:
                    _dashboard_state["running"] = False
                self.send_response(200)
                self.send_header("Content-type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"status": "stopped"}).encode())

            elif command == "pause":
                with _state_lock:
                    _dashboard_state["paused"] = True
                self.send_response(200)
                self.send_header("Content-type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"status": "paused"}).encode())

            elif command == "resume":
                with _state_lock:
                    _dashboard_state["paused"] = False
                self.send_response(200)
                self.send_header("Content-type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"status": "resumed"}).encode())

            else:
                self.send_response(400)
                self.send_header("Content-type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"error": "Unknown command"}).encode())

        else:
            self.send_response(404)
            self.end_headers()

    def log_message(self, format, *args):
        pass  # Suppress logging


def start_dashboard_server(port: int = 8080, directory: str = ""):
    """Start the dashboard HTTP server"""

    if directory:
        import os

        os.chdir(directory)

    server = HTTPServer(("", port), DashboardHandler)
    print(f"[Dashboard] Server running at http://localhost:{port}")
    print(f"[Dashboard] Open http://localhost:{port}/dashboard.html in your browser")

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n[Dashboard] Shutting down...")
        server.shutdown()


def create_dashboard_html() -> str:
    """Generate the dashboard HTML"""

    return """<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Universal Discovery Engine - Dashboard</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: #e4e4e7;
            min-height: 100vh;
            padding: 20px;
        }
        
        .container {
            max-width: 1400px;
            margin: 0 auto;
        }
        
        header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 30px;
            padding-bottom: 20px;
            border-bottom: 1px solid rgba(255,255,255,0.1);
        }
        
        h1 {
            font-size: 2rem;
            background: linear-gradient(90deg, #00d4ff, #7c3aed);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }
        
        .status-badge {
            padding: 8px 16px;
            border-radius: 20px;
            font-weight: 600;
            font-size: 0.9rem;
        }
        
        .status-running {
            background: rgba(34, 197, 94, 0.2);
            color: #22c55e;
            border: 1px solid #22c55e;
        }
        
        .status-paused {
            background: rgba(234, 179, 8, 0.2);
            color: #eab308;
            border: 1px solid #eab308;
        }
        
        .status-stopped {
            background: rgba(239, 68, 68, 0.2);
            color: #ef4444;
            border: 1px solid #ef4444;
        }
        
        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }
        
        .stat-card {
            background: rgba(255,255,255,0.05);
            border-radius: 16px;
            padding: 24px;
            border: 1px solid rgba(255,255,255,0.1);
            transition: transform 0.2s, box-shadow 0.2s;
        }
        
        .stat-card:hover {
            transform: translateY(-4px);
            box-shadow: 0 20px 40px rgba(0,0,0,0.3);
        }
        
        .stat-label {
            font-size: 0.85rem;
            color: #a1a1aa;
            text-transform: uppercase;
            letter-spacing: 1px;
            margin-bottom: 8px;
        }
        
        .stat-value {
            font-size: 2.5rem;
            font-weight: 700;
            color: #fff;
        }
        
        .stat-rate {
            font-size: 0.9rem;
            color: #00d4ff;
            margin-top: 4px;
        }
        
        .section {
            background: rgba(255,255,255,0.05);
            border-radius: 16px;
            padding: 24px;
            margin-bottom: 20px;
            border: 1px solid rgba(255,255,255,0.1);
        }
        
        .section h2 {
            font-size: 1.2rem;
            margin-bottom: 16px;
            color: #fff;
        }
        
        .workers-grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
            gap: 12px;
        }
        
        .worker-card {
            background: rgba(0,0,0,0.3);
            border-radius: 8px;
            padding: 12px;
            border: 1px solid rgba(255,255,255,0.05);
        }
        
        .worker-type {
            font-size: 0.8rem;
            color: #00d4ff;
            text-transform: uppercase;
            letter-spacing: 1px;
        }
        
        .worker-id {
            font-size: 0.9rem;
            color: #fff;
            margin: 4px 0;
        }
        
        .worker-tasks {
            font-size: 0.8rem;
            color: #a1a1aa;
        }
        
        .discoveries-list {
            max-height: 400px;
            overflow-y: auto;
        }
        
        .discovery-item {
            background: rgba(0,0,0,0.3);
            border-radius: 8px;
            padding: 12px;
            margin-bottom: 8px;
            border-left: 3px solid #22c55e;
        }
        
        .discovery-name {
            font-weight: 600;
            color: #fff;
            margin-bottom: 4px;
        }
        
        .discovery-hash {
            font-family: monospace;
            font-size: 0.8rem;
            color: #71717a;
        }
        
        .discovery-time {
            font-size: 0.75rem;
            color: #a1a1aa;
            margin-top: 4px;
        }
        
        .controls {
            display: flex;
            gap: 12px;
        }
        
        button {
            padding: 10px 20px;
            border-radius: 8px;
            border: none;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.2s;
        }
        
        .btn-stop {
            background: #ef4444;
            color: white;
        }
        
        .btn-stop:hover {
            background: #dc2626;
        }
        
        .btn-pause {
            background: #eab308;
            color: #1a1a2e;
        }
        
        .btn-pause:hover {
            background: #ca8a04;
        }
        
        .btn-resume {
            background: #22c55e;
            color: white;
        }
        
        .btn-resume:hover {
            background: #16a34a;
        }
        
        .progress-bar {
            width: 100%;
            height: 8px;
            background: rgba(255,255,255,0.1);
            border-radius: 4px;
            overflow: hidden;
            margin-top: 8px;
        }
        
        .progress-fill {
            height: 100%;
            background: linear-gradient(90deg, #00d4ff, #7c3aed);
            border-radius: 4px;
            transition: width 0.5s ease;
        }
        
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }
        
        .live-indicator {
            display: inline-block;
            width: 8px;
            height: 8px;
            background: #22c55e;
            border-radius: 50%;
            margin-right: 8px;
            animation: pulse 2s infinite;
        }
        
        .empty-state {
            text-align: center;
            padding: 40px;
            color: #71717a;
        }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <div>
                <h1>Universal Discovery Engine</h1>
                <div style="margin-top: 8px;">
                    <span class="live-indicator"></span>
                    <span id="statusText" class="status-badge status-running">Running</span>
                </div>
            </div>
            <div class="controls">
                <button class="btn-pause" onclick="togglePause()">Pause</button>
                <button class="btn-stop" onclick="stopEngine()">Stop</button>
            </div>
        </header>
        
        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-label">Theorems Generated</div>
                <div class="stat-value" id="generated">0</div>
                <div class="stat-rate" id="rateGenerated">0/s</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Theorems Attempted</div>
                <div class="stat-value" id="attempted">0</div>
                <div class="stat-rate">Total</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Theorems Proven</div>
                <div class="stat-value" id="proven">0</div>
                <div class="stat-rate" id="rate Proven">0/s</div>
                <div class="progress-bar">
                    <div class="progress-fill" id="successBar" style="width: 0%"></div>
                </div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Runtime</div>
                <div class="stat-value" id="runtime">0:00:00</div>
                <div class="stat-rate">Elapsed</div>
            </div>
        </div>
        
        <div class="section">
            <h2>Workers</h2>
            <div class="workers-grid" id="workersGrid">
                <div class="empty-state">No workers active</div>
            </div>
        </div>
        
        <div class="section">
            <h2>Recent Discoveries</h2>
            <div class="discoveries-list" id="discoveriesList">
                <div class="empty-state">No discoveries yet</div>
            </div>
        </div>
    </div>
    
    <script>
        let isPaused = false;
        let lastStats = null;
        
        async function fetchStats() {
            try {
                const response = await fetch('/api/stats');
                const data = await response.json();
                lastStats = data;
                updateDashboard(data);
            } catch (error) {
                console.error('Failed to fetch stats:', error);
            }
        }
        
        function updateDashboard(data) {
            const stats = data.stats || {};
            
            document.getElementById('generated').textContent = formatNumber(stats.theorems_generated || 0);
            document.getElementById('attempted').textContent = formatNumber(stats.theorems_attempted || 0);
            document.getElementById('proven').textContent = formatNumber(stats.theorems_proven || 0);
            document.getElementById('rateGenerated').textContent = (stats.rate_generated || 0).toFixed(1) + '/s';
            document.getElementById('rate Proven').textContent = (stats.rate_proven || 0).toFixed(2) + '/s';
            
            // Runtime
            const elapsed = stats.elapsed_seconds || 0;
            document.getElementById('runtime').textContent = formatTime(elapsed);
            
            // Success rate bar
            const successRate = stats.theorems_attempted > 0 
                ? (stats.theorems_proven / stats.theorems_attempted * 100) 
                : 0;
            document.getElementById('successBar').style.width = successRate + '%';
            
            // Status
            const statusText = document.getElementById('statusText');
            if (data.paused) {
                statusText.textContent = 'Paused';
                statusText.className = 'status-badge status-paused';
            } else if (data.running) {
                statusText.textContent = 'Running';
                statusText.className = 'status-badge status-running';
            } else {
                statusText.textContent = 'Stopped';
                statusText.className = 'status-badge status-stopped';
            }
            
            // Workers
            const workersGrid = document.getElementById('workersGrid');
            const workers = data.workers || {};
            if (Object.keys(workers).length > 0) {
                workersGrid.innerHTML = Object.entries(workers).map(([id, w]) => `
                    <div class="worker-card">
                        <div class="worker-type">${w.type || 'unknown'}</div>
                        <div class="worker-id">${id}</div>
                        <div class="worker-tasks">${w.tasks_completed || 0} tasks completed</div>
                    </div>
                `).join('');
            } else {
                workersGrid.innerHTML = '<div class="empty-state">No workers active</div>';
            }
            
            // Discoveries
            const discoveriesList = document.getElementById('discoveriesList');
            const discoveries = data.recent_discoveries || [];
            if (discoveries.length > 0) {
                discoveriesList.innerHTML = discoveries.slice().reverse().map(d => `
                    <div class="discovery-item">
                        <div class="discovery-name">${d.name || 'Unknown'}</div>
                        <div class="discovery-hash">${d.hash || ''}</div>
                        <div class="discovery-time">Proof time: ${(d.proof_time || 0).toFixed(3)}s</div>
                    </div>
                `).join('');
            } else {
                discoveriesList.innerHTML = '<div class="empty-state">No discoveries yet</div>';
            }
        }
        
        function formatNumber(num) {
            if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
            if (num >= 1000) return (num / 1000).toFixed(1) + 'K';
            return num.toString();
        }
        
        function formatTime(seconds) {
            const hrs = Math.floor(seconds / 3600);
            const mins = Math.floor((seconds % 3600) / 60);
            const secs = Math.floor(seconds % 60);
            return `${hrs}:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
        }
        
        async function togglePause() {
            const command = isPaused ? 'resume' : 'pause';
            try {
                await fetch('/api/control', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({command})
                });
                isPaused = !isPaused;
                document.querySelector('.btn-pause').textContent = isPaused ? 'Resume' : 'Pause';
            } catch (error) {
                console.error('Failed to toggle pause:', error);
            }
        }
        
        async function stopEngine() {
            try {
                await fetch('/api/control', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({command: 'stop'})
                });
            } catch (error) {
                console.error('Failed to stop:', error);
            }
        }
        
        // Poll every second
        fetchStats();
        setInterval(fetchStats, 1000);
    </script>
</body>
</html>"""


def ensure_dashboard_files():
    """Ensure dashboard HTML file exists"""
    dashboard_dir = Path(__file__).parent.parent
    dashboard_file = dashboard_dir / "dashboard.html"

    if not dashboard_file.exists():
        html = create_dashboard_html()
        with open(dashboard_file, "w") as f:
            f.write(html)
        print(f"[Dashboard] Created {dashboard_file}")


if __name__ == "__main__":
    ensure_dashboard_files()
    start_dashboard_server()
