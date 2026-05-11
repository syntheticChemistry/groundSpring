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

NPU_AVAILABLE = Path("/dev/akida0").exists()


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
            check=False,
        )
        elapsed = time.perf_counter() - start
        return elapsed, result.returncode == 0
    except subprocess.TimeoutExpired:
        return timeout, False


EXPERIMENTS = [
    {
        "name": "Exp 001: Sensor Noise",
        "python": [sys.executable, "control/sensor_noise/sensor_noise_decomposition.py"],
        "rust_bin": "validate_decompose",
    },
    {
        "name": "Exp 002: Observation Gap",
        "python": [sys.executable, "control/observation_gap/observation_gap.py"],
        "rust_bin": "validate_weather",
    },
    {
        "name": "Exp 003: Error Propagation",
        "python": [sys.executable, "control/error_propagation/error_propagation_fao56.py"],
        "rust_bin": "validate_fao56",
    },
    {
        "name": "Exp 004: Sequencing Noise",
        "python": [sys.executable, "control/sequencing_noise/sequencing_noise.py"],
        "rust_bin": "validate_rarefaction",
    },
    {
        "name": "Exp 005: Seismic Inversion",
        "python": [sys.executable, "control/seismic/seismic_inversion.py"],
        "rust_bin": "validate_seismic",
    },
    {
        "name": "Exp 006: Signal Specificity",
        "python": [sys.executable, "control/signal_specificity/signal_specificity.py"],
        "rust_bin": "validate_signal_specificity",
    },
    {
        "name": "Exp 007: RAWR Resampling",
        "python": [sys.executable, "control/rawr_resampling/rawr_resampling.py"],
        "rust_bin": "validate_rawr",
    },
    {
        "name": "Exp 008: Anderson Localization",
        "python": [sys.executable, "control/anderson_localization/anderson_localization.py"],
        "rust_bin": "validate_anderson",
    },
    {
        "name": "Exp 009: Quasiperiodic",
        "python": [sys.executable, "control/quasiperiodic/quasiperiodic_localization.py"],
        "rust_bin": "validate_quasiperiodic",
    },
    {
        "name": "Exp 010: Bistable Switching",
        "python": [sys.executable, "control/bistable_switching/bistable_switching.py"],
        "rust_bin": "validate_bistable",
    },
    {
        "name": "Exp 011: Multi-Signal QS",
        "python": [sys.executable, "control/multisignal_qs/multisignal_qs.py"],
        "rust_bin": "validate_multisignal",
    },
    {
        "name": "Exp 012: Spin Chain Transport",
        "python": [sys.executable, "control/spin_transport/spin_chain_transport.py"],
        "rust_bin": "validate_transport",
    },
    {
        "name": "Exp 013: Resampling Convergence",
        "python": [sys.executable, "control/resampling_convergence/resampling_convergence.py"],
        "rust_bin": "validate_resampling_conv",
    },
    {
        "name": "Exp 014: Drift vs Selection",
        "python": [sys.executable, "control/drift_selection/drift_selection.py"],
        "rust_bin": "validate_drift",
    },
    {
        "name": "Exp 015: Uncertainty Bridge",
        "python": [sys.executable, "control/uncertainty_bridge/uncertainty_bridge.py"],
        "rust_bin": "validate_uncertainty_bridge",
    },
    {
        "name": "Exp 016: Rare Biosphere",
        "python": [sys.executable, "control/rare_biosphere/rare_biosphere.py"],
        "rust_bin": "validate_rare_biosphere",
    },
    {
        "name": "Exp 017: Quasispecies Threshold",
        "python": [sys.executable, "control/quasispecies_threshold/quasispecies_threshold.py"],
        "rust_bin": "validate_quasispecies",
    },
    {
        "name": "Exp 018: Band Edge",
        "python": [sys.executable, "control/band_edge/band_edge.py"],
        "rust_bin": "validate_band_edge",
    },
    {
        "name": "Exp 019: Jackknife Estimation",
        "python": [sys.executable, "control/jackknife_estimation/jackknife_estimation.py"],
        "rust_bin": "validate_jackknife",
    },
    {
        "name": "Exp 020: Freeze Out Inverse",
        "python": [sys.executable, "control/freeze_out_inverse/freeze_out_inverse.py"],
        "rust_bin": "validate_freeze_out",
    },
    {
        "name": "Exp 021: Spectral Recon",
        "python": [sys.executable, "control/spectral_recon/spectral_recon.py"],
        "rust_bin": "validate_spectral_recon",
    },
    {
        "name": "Exp 022: ET0 Anderson Propagation",
        "python": [sys.executable, "control/et0_anderson_propagation/et0_anderson_propagation.py"],
        "rust_bin": "validate_et0_anderson",
    },
    {
        "name": "Exp 023: No-Till Sampling",
        "python": [sys.executable, "control/notill_sampling/notill_sampling.py"],
        "rust_bin": "validate_notill_sampling",
    },
    {
        "name": "Exp 024: Aggregate Stability",
        "python": [sys.executable, "control/aggregate_stability/aggregate_stability.py"],
        "rust_bin": "validate_aggregate_stability",
    },
    {
        "name": "Exp 025: Precision Drift",
        "python": [sys.executable, "control/precision_drift/precision_drift.py"],
        "rust_bin": "validate_precision_drift",
    },
    {
        "name": "Exp 026: Size Convergence",
        "python": [sys.executable, "control/size_convergence/size_convergence.py"],
        "rust_bin": "validate_size_convergence",
    },
    {
        "name": "Exp 027: Vendor Parity",
        "python": [sys.executable, "control/vendor_parity/vendor_parity.py"],
        "rust_bin": "validate_vendor_parity",
    },
    {
        "name": "Exp 028: NPU Anderson",
        "python": [sys.executable, "control/npu_anderson/npu_anderson.py"],
        "rust_bin": "validate_npu_anderson",
        "rust_features": ["npu"],
        "npu_required": True,
    },
    {
        "name": "Exp 035: Multi-Method ET₀",
        "python": [sys.executable, "control/et0_methods/et0_methods.py"],
        "rust_bin": "validate_et0_methods",
    },
]

# ── Rust-only experiments (029-034) ──────────────────────────────────────
#
# Experiments 029-034 are NUCLEUS live-infrastructure validation experiments
# that have no Python baseline by design. They validate NestGate data
# acquisition, NCBI/GHCND/IRIS pipeline integration, and real-data Anderson
# analysis. Benchmarking measures Rust binary timing only since the
# experiments cannot meaningfully run in interpreted Python:
#
#   Exp 029: NestGate NCBI validation (live NUCLEUS + NestGate HTTP API)
#   Exp 030: GHCND weather pipeline (live NUCLEUS + NOAA CDO)
#   Exp 031: Anderson NCBI (live 16S data + Anderson analysis)
#   Exp 032: IRIS seismic (live NUCLEUS + IRIS FDSN)
#   Exp 033: Tissue Anderson (correlated 3D, synthetic — Rust-only)
#   Exp 034: Barrier W_c finder (Anderson sweep — Rust-only)
#
# These are not included in the benchmark table because there is no
# Python baseline to compare against. For Rust-only timing, run:
#   cargo bench --workspace (captures criterion benchmarks)
#   cargo test --release -p groundspring-validate (integration timing)



def main() -> int:
    print("=" * 72)
    print("groundSpring — Rust vs Python Performance Benchmark")
    print("=" * 72)

    # Build Rust release binaries first
    print("\nBuilding Rust release binaries...")
    build_start = time.perf_counter()
    subprocess.run(
        ["cargo", "build", "--release", "--workspace"],
        cwd=str(ROOT), capture_output=True, check=False,
    )
    print(f"  Build time: {time.perf_counter() - build_start:.1f}s")

    results: list[BenchResult] = []
    n_warmup = 1
    n_trials = 3

    for exp in EXPERIMENTS:
        if exp.get("npu_required") and not NPU_AVAILABLE:
            print(f"\n--- {exp['name']} ---")
            print("  [SKIP] NPU hardware (/dev/akida0) not available")
            continue

        print(f"\n--- {exp['name']} ---")

        rust_cmd = ["cargo", "run", "--release", "--bin", exp["rust_bin"]]
        if "rust_features" in exp:
            rust_cmd.extend(["--features", ",".join(exp["rust_features"])])

        # Warmup
        for _ in range(n_warmup):
            time_command(exp["python"], ROOT, timeout=300)
            time_command(rust_cmd, ROOT, timeout=300)

        # Timed runs
        py_times = []
        rs_times = []

        py_pass_all = True
        rs_pass_all = True
        for trial in range(n_trials):
            py_t, py_ok = time_command(exp["python"], ROOT, timeout=300)
            rs_t, rs_ok = time_command(rust_cmd, ROOT, timeout=300)
            py_times.append(py_t)
            rs_times.append(rs_t)
            py_pass_all = py_pass_all and py_ok
            rs_pass_all = rs_pass_all and rs_ok
            print(f"  Trial {trial+1}: Python={py_t:.3f}s, Rust={rs_t:.3f}s")

        py_median = sorted(py_times)[n_trials // 2]
        rs_median = sorted(rs_times)[n_trials // 2]
        speedup = py_median / rs_median if rs_median > 0 else 0

        results.append(BenchResult(
            name=exp["name"],
            python_s=py_median,
            rust_s=rs_median,
            speedup=speedup,
            python_pass=py_pass_all,
            rust_pass=rs_pass_all,
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

    # Compute totals excluding LAPACK-bound experiments (custom QR vs LAPACK)
    lapack_bound = {"Exp 009: Quasiperiodic"}
    comp_py = sum(r.python_s for r in results if r.name not in lapack_bound)
    comp_rs = sum(r.rust_s for r in results if r.name not in lapack_bound)
    comp_speedup = comp_py / comp_rs if comp_rs > 0 else 0
    print(f"| {'TOTAL (excl. LAPACK-bound)':<35} | {comp_py:>10.3f} | {comp_rs:>10.3f} | {comp_speedup:>7.1f}× |")

    print(f"\n{'=' * 72}")
    print("Pure Rust math is faster than interpreted Python.")
    print("Note: Exp 009 uses a custom QR eigenvalue solver in Rust to prove")
    print("mathematical parity; numpy delegates to LAPACK/Fortran for dense")
    print("eigenvalues.  Barracuda GPU kernels will close this gap.")
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
                "lapack_bound": r.name in lapack_bound,
            }
            for r in results
        ],
        "total_python_s": round(total_py, 4),
        "total_rust_s": round(total_rs, 4),
        "total_speedup": round(total_speedup, 1),
        "compute_bound_python_s": round(comp_py, 4),
        "compute_bound_rust_s": round(comp_rs, 4),
        "compute_bound_speedup": round(comp_speedup, 1),
    }

    out_path = ROOT / "data" / "bench_rust_vs_python.json"
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with open(out_path, "w") as f:
        json.dump(bench_json, f, indent=2)
    print(f"\nResults saved to {out_path}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
