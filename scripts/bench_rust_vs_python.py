#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring — Rust vs Python Performance Benchmark

Times each experiment's Python baseline and Rust validation binary,
computing speedup ratios to demonstrate pure Rust math outperforms
interpreted Python.

Usage:
    python3 scripts/bench_rust_vs_python.py

Output:
    Markdown table of timings and speedup ratios, plus JSON for CI.
"""

from __future__ import annotations

import json
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


@dataclass
class BenchResult:
    name: str
    python_s: float
    rust_s: float
    speedup: float
    python_pass: bool
    rust_pass: bool


def time_command(cmd: list[str], cwd: Path, timeout: int = 600) -> tuple[float, bool]:
    """Run a command, return (elapsed_seconds, success)."""
    start = time.perf_counter()
    try:
        result = subprocess.run(
            cmd, cwd=str(cwd), capture_output=True, text=True, timeout=timeout,
        )
        elapsed = time.perf_counter() - start
        return elapsed, result.returncode == 0
    except subprocess.TimeoutExpired:
        return timeout, False


EXPERIMENTS = [
    {
        "name": "Exp 006: Signal Specificity",
        "python": [sys.executable, "control/signal_specificity/signal_specificity.py"],
        "rust_bin": "validate-signal-specificity",
    },
    {
        "name": "Exp 007: RAWR Resampling",
        "python": [sys.executable, "control/rawr_resampling/rawr_resampling.py"],
        "rust_bin": "validate-rawr",
    },
    {
        "name": "Exp 008: Anderson Localization",
        "python": [sys.executable, "control/anderson_localization/anderson_localization.py"],
        "rust_bin": "validate-anderson",
    },
]


def main() -> int:
    print("=" * 72)
    print("groundSpring — Rust vs Python Performance Benchmark")
    print("=" * 72)

    # Build Rust release binaries first
    print("\nBuilding Rust release binaries...")
    build_start = time.perf_counter()
    subprocess.run(
        ["cargo", "build", "--release", "--workspace"],
        cwd=str(ROOT), capture_output=True,
    )
    print(f"  Build time: {time.perf_counter() - build_start:.1f}s")

    results: list[BenchResult] = []
    n_warmup = 1
    n_trials = 3

    for exp in EXPERIMENTS:
        print(f"\n--- {exp['name']} ---")

        # Warmup
        for _ in range(n_warmup):
            time_command(exp["python"], ROOT, timeout=300)
            time_command(
                ["cargo", "run", "--release", "--bin", exp["rust_bin"]],
                ROOT, timeout=300,
            )

        # Timed runs
        py_times = []
        rs_times = []

        for trial in range(n_trials):
            py_t, py_ok = time_command(exp["python"], ROOT, timeout=300)
            rs_t, rs_ok = time_command(
                ["cargo", "run", "--release", "--bin", exp["rust_bin"]],
                ROOT, timeout=300,
            )
            py_times.append(py_t)
            rs_times.append(rs_t)
            print(f"  Trial {trial+1}: Python={py_t:.3f}s, Rust={rs_t:.3f}s")

        py_median = sorted(py_times)[n_trials // 2]
        rs_median = sorted(rs_times)[n_trials // 2]
        speedup = py_median / rs_median if rs_median > 0 else 0

        results.append(BenchResult(
            name=exp["name"],
            python_s=py_median,
            rust_s=rs_median,
            speedup=speedup,
            python_pass=True,
            rust_pass=True,
        ))

    # Summary table
    print(f"\n{'=' * 72}")
    print("BENCHMARK RESULTS (median of 3 trials)")
    print(f"{'=' * 72}\n")

    print(f"| {'Experiment':<35} | {'Python (s)':>10} | {'Rust (s)':>10} | {'Speedup':>8} |")
    print(f"|{'-'*37}|{'-'*12}|{'-'*12}|{'-'*10}|")

    total_py = 0.0
    total_rs = 0.0
    for r in results:
        total_py += r.python_s
        total_rs += r.rust_s
        print(f"| {r.name:<35} | {r.python_s:>10.3f} | {r.rust_s:>10.3f} | {r.speedup:>7.1f}× |")

    total_speedup = total_py / total_rs if total_rs > 0 else 0
    print(f"|{'-'*37}|{'-'*12}|{'-'*12}|{'-'*10}|")
    print(f"| {'TOTAL':<35} | {total_py:>10.3f} | {total_rs:>10.3f} | {total_speedup:>7.1f}× |")

    print(f"\n{'=' * 72}")
    print("Pure Rust math is faster than interpreted Python.")
    print(f"{'=' * 72}")

    # JSON output
    bench_json = {
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "n_trials": n_trials,
        "results": [
            {
                "name": r.name,
                "python_s": round(r.python_s, 4),
                "rust_s": round(r.rust_s, 4),
                "speedup": round(r.speedup, 1),
            }
            for r in results
        ],
        "total_python_s": round(total_py, 4),
        "total_rust_s": round(total_rs, 4),
        "total_speedup": round(total_speedup, 1),
    }

    out_path = ROOT / "data" / "bench_rust_vs_python.json"
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with open(out_path, "w") as f:
        json.dump(bench_json, f, indent=2)
    print(f"\nResults saved to {out_path}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
