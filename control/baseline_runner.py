#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring — Python Tier 0 Baseline Runner

Implements the same three benchmark workloads as the Kokkos and Rust
harnesses using *identical* Xorshift64 PRNG and parameters. This
ensures numerical parity is provable across all tiers:

  Tier 0: Python (this script)     — correctness reference
  Tier 1: Kokkos (C++ / CUDA)     — performance reference
  Tier 2: Rust + BarraCuda        — sovereign implementation

Output is JSON with provenance, matching the format of
kokkos_baseline and bench_kokkos_parity / bench_gpu_vs_kokkos.

Usage:
    python3 control/baseline_runner.py
    python3 control/baseline_runner.py --json-only
"""

from __future__ import annotations

import json
import math
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path


def _git_commit_hash() -> str:
    """Return the current HEAD commit hash, or 'unknown' if git is unavailable."""
    try:
        return subprocess.run(
            ["git", "rev-parse", "HEAD"],
            capture_output=True,
            text=True,
            check=True,
            cwd=Path(__file__).resolve().parent.parent,
        ).stdout.strip()
    except Exception:
        return "unknown"


# ---------------------------------------------------------------------------
# Xorshift64 — matches groundspring::prng::Xorshift64 exactly
# ---------------------------------------------------------------------------

class Xorshift64:
    """Deterministic PRNG matching Rust / Kokkos implementations."""

    def __init__(self, seed: int) -> None:
        self.state = seed if seed != 0 else 1
        self.state &= 0xFFFF_FFFF_FFFF_FFFF

    def next_u64(self) -> int:
        s = self.state
        s ^= (s << 13) & 0xFFFF_FFFF_FFFF_FFFF
        s ^= s >> 7
        s ^= (s << 17) & 0xFFFF_FFFF_FFFF_FFFF
        self.state = s
        return s

    def next_f64(self) -> float:
        return self.next_u64() / 0xFFFF_FFFF_FFFF_FFFF


# ---------------------------------------------------------------------------
# Benchmark infrastructure
# ---------------------------------------------------------------------------

@dataclass
class BenchResult:
    name: str
    value: float
    elapsed_us: float


def bench_timer() -> float:
    return time.perf_counter()


# ---------------------------------------------------------------------------
# 1. Anderson localization — Lyapunov exponent
# ---------------------------------------------------------------------------

def anderson_lyapunov(n_sites: int, disorder: float,
                      n_realizations: int, energy: float,
                      base_seed: int) -> BenchResult:
    t0 = bench_timer()

    gamma_sum = 0.0
    for r in range(n_realizations):
        rng = Xorshift64(base_seed + r)
        half_w = disorder / 2.0
        log_growth = 0.0
        v0, v1 = 1.0, 0.0

        for _ in range(n_sites):
            pot = rng.next_f64() * disorder - half_w
            new_0 = (energy - pot) * v0 - v1
            new_1 = v0
            v0, v1 = new_0, new_1
            norm = math.sqrt(v0 * v0 + v1 * v1)
            if norm > 0.0:
                log_growth += math.log(norm)
                v0 /= norm
                v1 /= norm

        gamma_sum += log_growth / n_sites

    gamma_avg = gamma_sum / n_realizations
    elapsed = (bench_timer() - t0) * 1e6

    return BenchResult("anderson_lyapunov_averaged", gamma_avg, elapsed)


# ---------------------------------------------------------------------------
# 2. Statistical reductions — mean, variance, Pearson r
# ---------------------------------------------------------------------------

def generate_stat_data(n: int, seed: int) -> tuple[list[float], list[float]]:
    data = []
    data2 = []
    for i in range(n):
        rng = Xorshift64(seed + i)
        v = rng.next_f64() * 100.0
        data.append(v)
        rng2 = Xorshift64(seed + 1_000_000 + i)
        noise = rng2.next_f64() * 10.0
        data2.append(v * 0.8 + noise + 5.0)
    return data, data2


def bench_mean(data: list[float]) -> BenchResult:
    t0 = bench_timer()
    m = sum(data) / len(data)
    return BenchResult("mean", m, (bench_timer() - t0) * 1e6)


def bench_variance(data: list[float], mean_val: float) -> BenchResult:
    t0 = bench_timer()
    ss = sum((x - mean_val) ** 2 for x in data)
    var = ss / len(data)
    return BenchResult("variance", var, (bench_timer() - t0) * 1e6)


def bench_pearson_r(x: list[float], y: list[float],
                    mx: float, my: float) -> BenchResult:
    t0 = bench_timer()
    sum_xy = sum_xx = sum_yy = 0.0
    for xi, yi in zip(x, y):
        dx = xi - mx
        dy = yi - my
        sum_xy += dx * dy
        sum_xx += dx * dx
        sum_yy += dy * dy
    denom = math.sqrt(sum_xx * sum_yy)
    r = sum_xy / denom if denom > 0.0 else 0.0
    return BenchResult("pearson_r", r, (bench_timer() - t0) * 1e6)


# ---------------------------------------------------------------------------
# 3. Bootstrap resampling — percentile CI for the mean
# ---------------------------------------------------------------------------

def bench_bootstrap_mean(data: list[float], n_replicates: int,
                         confidence: float, seed: int,
                         *, quiet: bool = False) -> BenchResult:
    t0 = bench_timer()
    n = len(data)
    replicate_means = []

    for r_idx in range(n_replicates):
        rng = Xorshift64(seed + r_idx * 997)
        s = 0.0
        for _ in range(n):
            idx = rng.next_u64() % n
            s += data[idx]
        replicate_means.append(s / n)

    replicate_means.sort()
    alpha = 1.0 - confidence
    lo_idx = int(alpha / 2.0 * n_replicates)
    hi_idx = int((1.0 - alpha / 2.0) * n_replicates)
    if hi_idx >= n_replicates:
        hi_idx = n_replicates - 1

    estimate = sum(replicate_means) / n_replicates
    elapsed = (bench_timer() - t0) * 1e6

    if not quiet:
        print(f"    bootstrap: estimate={estimate:.10f} "
              f"ci=[{replicate_means[lo_idx]:.10f}, {replicate_means[hi_idx]:.10f}]")

    return BenchResult("bootstrap_mean", estimate, elapsed)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> None:
    json_only = "--json-only" in sys.argv
    results: list[BenchResult] = []

    if not json_only:
        print("groundSpring Python Tier 0 Baseline")
        print("  Backend: CPython + pure Python math\n")

    # 1. Anderson localization
    N_SITES, DISORDER, N_REALIZATIONS, ENERGY, BASE_SEED = 10_000, 4.0, 500, 0.0, 42

    if not json_only:
        print("=== Anderson Localization (Lyapunov Exponent) ===")
        print(f"  N={N_SITES}, W={DISORDER}, realizations={N_REALIZATIONS}, E={ENERGY}")

    r = anderson_lyapunov(N_SITES, DISORDER, N_REALIZATIONS, ENERGY, BASE_SEED)
    results.append(r)

    if not json_only:
        xi = 1.0 / r.value if r.value > 0 else float("inf")
        print(f"  gamma_avg = {r.value:.10f}  (xi = {xi:.4f})")
        print(f"  Derrida-Gardner: xi ~ 96/W^2 = {96.0 / (DISORDER ** 2):.4f}")
        print(f"  elapsed: {r.elapsed_us:.0f} us\n")

    # 2. Statistical reductions
    N_STAT, SEED_STAT = 1_000_000, 12345

    if not json_only:
        print(f"=== Statistical Reductions (N={N_STAT}) ===")
        print("  generating data...")

    data, data2 = generate_stat_data(N_STAT, SEED_STAT)

    if not json_only:
        print("  data generated.")

    r_mean = bench_mean(data)
    results.append(r_mean)
    if not json_only:
        print(f"  mean = {r_mean.value:.10f} ({r_mean.elapsed_us:.0f} us)")

    r_var = bench_variance(data, r_mean.value)
    results.append(r_var)
    if not json_only:
        print(f"  variance = {r_var.value:.10f} ({r_var.elapsed_us:.0f} us)")

    my = sum(data2) / len(data2)
    r_pearson = bench_pearson_r(data, data2, r_mean.value, my)
    results.append(r_pearson)
    if not json_only:
        print(f"  pearson_r = {r_pearson.value:.10f} ({r_pearson.elapsed_us:.0f} us)\n")

    # 3. Bootstrap resampling
    N_BOOT, N_REPLICATES, CONFIDENCE, SEED_BOOT = 10_000, 5_000, 0.95, 99

    if not json_only:
        print(f"=== Bootstrap Resampling (N={N_BOOT}, B={N_REPLICATES}) ===")

    boot_data = []
    for i in range(N_BOOT):
        rng = Xorshift64(SEED_BOOT + i)
        boot_data.append(rng.next_f64() * 50.0 + 25.0)

    r_boot = bench_bootstrap_mean(boot_data, N_REPLICATES, CONFIDENCE, SEED_BOOT,
                                   quiet=json_only)
    results.append(r_boot)
    if not json_only:
        print(f"  elapsed: {r_boot.elapsed_us:.0f} us\n")

    # JSON output
    if not json_only:
        print("=== JSON Benchmark Output ===")

    output = {
        "_source": "Python Tier 0 baseline — groundSpring",
        "_provenance": {
            "baseline_date": "2026-03-06",
            "baseline_commit": _git_commit_hash(),
            "backend": "CPython + pure Python math",
            "python_version": f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}",
            "generated_by": "control/baseline_runner.py",
            "command": "python3 control/baseline_runner.py",
        },
        "results": [
            {"name": r.name, "value": r.value, "elapsed_us": r.elapsed_us}
            for r in results
        ],
    }
    print(json.dumps(output, indent=2))


if __name__ == "__main__":
    main()
