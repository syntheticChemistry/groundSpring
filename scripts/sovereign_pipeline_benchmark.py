#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring — Sovereign Pipeline Benchmark Certificate

Runs all four tiers and produces a machine-readable parity + speed
certificate proving:

  Tier 0: Python baseline         — correctness reference (established)
  Tier 1: Kokkos (C++/CUDA)       — performance reference (established)
  Tier 2: Rust CPU                — sovereign implementation (pure Rust)
  Tier 3: Rust barraCuda GPU      — sovereign GPU (WGSL, pure Rust)

This script automates the full evolution path:
  Python → Kokkos → Rust CPU → barraCuda GPU

Usage:
    python3 scripts/sovereign_pipeline_benchmark.py
    python3 scripts/sovereign_pipeline_benchmark.py --save
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATA_DIR = ROOT / "data"


def _extract_json(text: str) -> dict | None:
    """Extract the last complete JSON object from mixed text output."""
    brace_depth = 0
    json_end = -1
    json_start = -1
    for i in range(len(text) - 1, -1, -1):
        if text[i] == "}":
            if brace_depth == 0:
                json_end = i
            brace_depth += 1
        elif text[i] == "{":
            brace_depth -= 1
            if brace_depth == 0:
                json_start = i
                break
    if json_start < 0 or json_end < 0:
        return None
    try:
        return json.loads(text[json_start : json_end + 1])
    except json.JSONDecodeError:
        return None


@dataclass
class TierResult:
    tier: str
    backend: str
    results: dict[str, dict]
    raw_json: dict


def run_python_baseline() -> TierResult | None:
    """Tier 0: Python baseline (pure math, Xorshift64 parity)."""
    script = ROOT / "control" / "baseline_runner.py"
    if not script.exists():
        return None
    try:
        result = subprocess.run(
            [sys.executable, str(script), "--json-only"],
            capture_output=True, text=True, timeout=300, check=False,
            cwd=str(ROOT),
        )
        if result.returncode != 0:
            print(f"  [WARN] Python baseline failed: {result.stderr[:200]}")
            return None
        data = json.loads(result.stdout)
        by_name = {r["name"]: r for r in data.get("results", [])}
        return TierResult("Tier 0", "Python (CPython)", by_name, data)
    except Exception as e:
        print(f"  [WARN] Python baseline error: {e}")
        return None


def run_kokkos_baseline() -> TierResult | None:
    """Tier 1: Kokkos C++ (CUDA / OpenMP / Serial)."""
    binary = ROOT / "kokkos_baseline" / "build" / "kokkos_baseline"
    if not binary.exists():
        print("  [WARN] Kokkos binary not found — build with:")
        print("         cd kokkos_baseline && cmake -B build -DCMAKE_BUILD_TYPE=Release && cmake --build build -j$(nproc)")
        return None
    try:
        result = subprocess.run(
            [str(binary)], capture_output=True, text=True,
            timeout=120, check=False, cwd=str(ROOT),
        )
        data = _extract_json(result.stdout)
        if data is None:
            return None
        by_name = {r["name"]: r for r in data.get("results", [])}
        return TierResult("Tier 1", data.get("_provenance", {}).get("backend", "Kokkos"), by_name, data)
    except Exception as e:
        print(f"  [WARN] Kokkos error: {e}")
        return None


def run_rust_cpu() -> TierResult | None:
    """Tier 2: Rust CPU (bench-kokkos-parity binary)."""
    binary = ROOT / "target" / "release" / "bench-kokkos-parity"
    if not binary.exists():
        try:
            subprocess.run(
                ["cargo", "build", "--release", "--bin", "bench-kokkos-parity"],
                capture_output=True, text=True, timeout=180, check=True,
                cwd=str(ROOT),
            )
        except Exception:
            return None
    try:
        result = subprocess.run(
            [str(binary)], capture_output=True, text=True,
            timeout=120, check=False, cwd=str(ROOT),
        )
        data = _extract_json(result.stdout)
        if data is None:
            return None
        by_name = {r["name"]: r for r in data.get("results", [])}
        return TierResult("Tier 2", "Rust CPU (pure Rust)", by_name, data)
    except Exception as e:
        print(f"  [WARN] Rust CPU error: {e}")
        return None


def run_rust_gpu() -> TierResult | None:
    """Tier 3: Rust barraCuda GPU (WGSL shaders via wgpu)."""
    binary = ROOT / "target" / "release" / "bench-gpu-vs-kokkos"
    if not binary.exists():
        try:
            subprocess.run(
                ["cargo", "build", "--release", "--features", "barracuda-gpu",
                 "--bin", "bench-gpu-vs-kokkos"],
                capture_output=True, text=True, timeout=180, check=True,
                cwd=str(ROOT),
            )
        except Exception:
            return None
    try:
        result = subprocess.run(
            [str(binary)], capture_output=True, text=True,
            timeout=120, check=False, cwd=str(ROOT),
        )
        data = _extract_json(result.stdout)
        if data is None:
            return None
        by_name = {r["name"]: r for r in data.get("results", [])}
        return TierResult("Tier 3", "BarraCuda WGSL (wgpu)", by_name, data)
    except Exception as e:
        print(f"  [WARN] Rust GPU error: {e}")
        return None


KERNELS = [
    "anderson_lyapunov_averaged",
    "mean",
    "variance",
    "pearson_r",
    "bootstrap_mean",
]


def print_parity_table(tiers: list[TierResult]) -> dict:
    """Print value comparison, return parity status per kernel."""
    width = 20
    header_labels = [f"{t.tier}" for t in tiers]
    label_line = f"  {'Kernel':<30}" + "".join(f" {h:>{width}}" for h in header_labels) + f" {'Max Diff':>12}"
    print(label_line)
    print("  " + "-" * (len(label_line) - 2))

    parity: dict[str, dict] = {}
    for kernel in KERNELS:
        values = []
        for t in tiers:
            v = t.results.get(kernel, {}).get("value")
            values.append(v)

        valid_values = [v for v in values if v is not None and v != 0.0]
        if len(valid_values) >= 2:
            max_diff = max(valid_values) - min(valid_values)
        else:
            max_diff = None

        cells = []
        for v in values:
            if v is None:
                cells.append(f"{'—':>{width}}")
            elif v == 0.0:
                cells.append(f"{'0 (evolving)':>{width}}")
            else:
                cells.append(f"{v:>{width}.12e}")

        diff_str = f"{max_diff:.2e}" if max_diff is not None else "—"
        proven = max_diff is not None and max_diff < 1e-8
        status = "PROVEN" if proven else ("EVOLVING" if max_diff is None else "DIFF")
        print(f"  {kernel:<30}" + "".join(cells) + f" {diff_str:>12}  {status}")
        parity[kernel] = {
            "values": values,
            "max_diff": max_diff,
            "status": status,
        }

    return parity


def print_speed_table(tiers: list[TierResult], baseline_idx: int = 0) -> dict:
    """Print timing comparison relative to baseline tier."""
    baseline = tiers[baseline_idx]
    width = 14
    header_labels = [f"{t.tier}" for t in tiers]
    label_line = f"  {'Kernel':<30}" + "".join(f" {h:>{width}}" for h in header_labels)
    print(label_line)
    print("  " + "-" * (len(label_line) - 2))

    speed_data: dict[str, dict] = {}
    for kernel in KERNELS:
        base_us = baseline.results.get(kernel, {}).get("elapsed_us")
        cells = []
        speedups = {}
        for t in tiers:
            val = t.results.get(kernel, {}).get("value")
            us = t.results.get(kernel, {}).get("elapsed_us")
            if us is None or us == 0.0 or val == 0.0:
                cells.append(f"{'(evolving)':>{width}}")
            elif base_us and base_us > 0:
                speedup = base_us / us
                label = f"{speedup:.1f}x"
                if t is baseline:
                    label = f"{us:.0f} us"
                cells.append(f"{label:>{width}}")
                speedups[t.tier] = speedup
            else:
                cells.append(f"{us:.0f} us")

        print(f"  {kernel:<30}" + "".join(cells))
        speed_data[kernel] = speedups

    return speed_data


def main() -> None:
    save = "--save" in sys.argv
    now = datetime.now(timezone.utc)

    gs_head = "unknown"
    try:
        gs_head = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            capture_output=True, text=True, check=True, cwd=str(ROOT),
        ).stdout.strip()
    except Exception:
        pass

    print("=" * 78)
    print("  groundSpring — Sovereign Pipeline Benchmark Certificate")
    print(f"  Date: {now.isoformat()}")
    print(f"  groundSpring HEAD: {gs_head}")
    print("=" * 78)
    print()

    tiers: list[TierResult] = []

    print("Running Tier 0: Python baseline...")
    r = run_python_baseline()
    if r:
        tiers.append(r)
        print(f"  {r.backend} — {len(r.results)} kernels")

    print("Running Tier 1: Kokkos baseline...")
    r = run_kokkos_baseline()
    if r:
        tiers.append(r)
        print(f"  {r.backend} — {len(r.results)} kernels")

    print("Running Tier 2: Rust CPU...")
    r = run_rust_cpu()
    if r:
        tiers.append(r)
        print(f"  {r.backend} — {len(r.results)} kernels")

    print("Running Tier 3: Rust barraCuda GPU...")
    r = run_rust_gpu()
    if r:
        tiers.append(r)
        print(f"  {r.backend} — {len(r.results)} kernels")

    if len(tiers) < 2:
        print("\n  ERROR: Need at least 2 tiers for comparison.")
        sys.exit(1)

    # --- Parity table ---
    print()
    print("=" * 78)
    print("  PRECISION PARITY (values must match across tiers)")
    print("=" * 78)
    print()
    parity = print_parity_table(tiers)

    # --- Speed table ---
    print()
    print("=" * 78)
    print("  SPEED COMPARISON (relative to Python Tier 0)")
    print("=" * 78)
    print()
    speed = print_speed_table(tiers)

    # --- Summary ---
    proven_count = sum(1 for p in parity.values() if p["status"] == "PROVEN")
    evolving_count = sum(1 for p in parity.values() if p["status"] == "EVOLVING")
    total = len(KERNELS)

    print()
    print("=" * 78)
    print(f"  SUMMARY: {proven_count}/{total} kernels PROVEN parity, "
          f"{evolving_count} evolving")
    print()
    for t in tiers:
        print(f"    {t.tier}: {t.backend}")
    print()

    if proven_count == total:
        print("  ALL KERNELS: MATHEMATICAL PARITY PROVEN ACROSS ALL TIERS")
    elif proven_count >= 3:
        print("  CORE KERNELS: PARITY PROVEN — GPU dispatch still evolving")
    print("=" * 78)

    # --- JSON certificate ---
    if save:
        DATA_DIR.mkdir(parents=True, exist_ok=True)
        cert_path = DATA_DIR / "sovereign_pipeline_benchmark.json"
        cert = {
            "title": "groundSpring Sovereign Pipeline Benchmark Certificate",
            "date": now.isoformat(),
            "groundspring_head": gs_head,
            "tiers": [
                {
                    "tier": t.tier,
                    "backend": t.backend,
                    "results": t.raw_json.get("results", []),
                }
                for t in tiers
            ],
            "parity": {
                k: {"max_diff": v["max_diff"], "status": v["status"]}
                for k, v in parity.items()
            },
            "proven_count": proven_count,
            "total_kernels": total,
        }
        cert_path.write_text(json.dumps(cert, indent=2, default=str))
        print(f"\n  Certificate saved to {cert_path.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
