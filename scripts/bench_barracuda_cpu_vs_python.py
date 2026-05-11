#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring — Python vs barraCuda CPU Delegation Benchmark

Times the same mathematical workloads in:
  - Tier 0: Python baseline (control scripts)
  - Tier 2: Rust pure (cargo run --release)
  - Tier 2b: Rust + barraCuda CPU (cargo run --release --features barracuda)

Fills the gap: bench_rust_vs_python.py compares Python vs pure Rust,
but does not measure the additional delegation overhead/speedup when
Rust delegates to barraCuda CPU implementations.

Usage:
    python3 scripts/bench_barracuda_cpu_vs_python.py

Output:
    data/barracuda_cpu_benchmark.json — machine-readable benchmark
    Markdown summary to stdout
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
class TierBench:
    name: str
    python_s: float
    rust_pure_s: float
    rust_barracuda_s: float
    python_pass: bool
    rust_pure_pass: bool
    rust_barracuda_pass: bool


def timed_run(cmd: list[str], timeout: int = 300) -> tuple[bool, float]:
    """Run a command, return (success, elapsed_seconds)."""
    try:
        t0 = time.perf_counter()
        result = subprocess.run(
            cmd, cwd=str(ROOT), capture_output=True, text=True,
            timeout=timeout, check=False,
        )
        elapsed = time.perf_counter() - t0
        return result.returncode == 0, elapsed
    except subprocess.TimeoutExpired:
        return False, timeout


EXPERIMENTS = [
    {
        "name": "Sensor Noise (Exp 001)",
        "python": [sys.executable, "control/sensor_noise/sensor_noise_decomposition.py"],
        "rust_bin": "validate_decompose",
    },
    {
        "name": "Observation Gap (Exp 002)",
        "python": [sys.executable, "control/observation_gap/observation_gap.py"],
        "rust_bin": "validate_weather",
    },
    {
        "name": "Error Propagation (Exp 003)",
        "python": [sys.executable, "control/error_propagation/error_propagation_fao56.py"],
        "rust_bin": "validate_fao56",
    },
    {
        "name": "Sequencing Noise (Exp 004)",
        "python": [sys.executable, "control/sequencing_noise/sequencing_noise.py"],
        "rust_bin": "validate_rarefaction",
    },
    {
        "name": "Seismic Inversion (Exp 005)",
        "python": [sys.executable, "control/seismic/seismic_inversion.py"],
        "rust_bin": "validate_seismic",
    },
    {
        "name": "Signal Specificity (Exp 006)",
        "python": [sys.executable, "control/signal_specificity/signal_specificity.py"],
        "rust_bin": "validate_signal_specificity",
    },
    {
        "name": "RAWR Resampling (Exp 007)",
        "python": [sys.executable, "control/rawr_resampling/rawr_resampling.py"],
        "rust_bin": "validate_rawr",
    },
    {
        "name": "Anderson Localization (Exp 008)",
        "python": [sys.executable, "control/anderson_localization/anderson_localization.py"],
        "rust_bin": "validate_anderson",
    },
    {
        "name": "Quasiperiodic (Exp 009)",
        "python": [sys.executable, "control/quasiperiodic/quasiperiodic_localization.py"],
        "rust_bin": "validate_quasiperiodic",
    },
    {
        "name": "Bistable Switching (Exp 010)",
        "python": [sys.executable, "control/bistable_switching/bistable_switching.py"],
        "rust_bin": "validate_bistable",
    },
    {
        "name": "Multi-Signal QS (Exp 011)",
        "python": [sys.executable, "control/multisignal_qs/multisignal_qs.py"],
        "rust_bin": "validate_multisignal",
    },
    {
        "name": "Spin Chain Transport (Exp 012)",
        "python": [sys.executable, "control/spin_transport/spin_chain_transport.py"],
        "rust_bin": "validate_transport",
    },
    {
        "name": "Resampling Convergence (Exp 013)",
        "python": [sys.executable, "control/resampling_convergence/resampling_convergence.py"],
        "rust_bin": "validate_resampling_conv",
    },
    {
        "name": "Drift vs Selection (Exp 014)",
        "python": [sys.executable, "control/drift_selection/drift_selection.py"],
        "rust_bin": "validate_drift",
    },
    {
        "name": "Uncertainty Bridge (Exp 015)",
        "python": [sys.executable, "control/uncertainty_bridge/uncertainty_bridge.py"],
        "rust_bin": "validate_uncertainty_bridge",
    },
    {
        "name": "Rare Biosphere (Exp 016)",
        "python": [sys.executable, "control/rare_biosphere/rare_biosphere.py"],
        "rust_bin": "validate_rare_biosphere",
    },
    {
        "name": "Quasispecies Threshold (Exp 017)",
        "python": [sys.executable, "control/quasispecies_threshold/quasispecies_threshold.py"],
        "rust_bin": "validate_quasispecies",
    },
    {
        "name": "Band Edge (Exp 018)",
        "python": [sys.executable, "control/band_edge/band_edge.py"],
        "rust_bin": "validate_band_edge",
    },
    {
        "name": "Jackknife Estimation (Exp 019)",
        "python": [sys.executable, "control/jackknife_estimation/jackknife_estimation.py"],
        "rust_bin": "validate_jackknife",
    },
    {
        "name": "Freeze Out Inverse (Exp 020)",
        "python": [sys.executable, "control/freeze_out_inverse/freeze_out_inverse.py"],
        "rust_bin": "validate_freeze_out",
    },
    {
        "name": "Spectral Recon (Exp 021)",
        "python": [sys.executable, "control/spectral_recon/spectral_recon.py"],
        "rust_bin": "validate_spectral_recon",
    },
    {
        "name": "ET₀ Anderson Propagation (Exp 022)",
        "python": [sys.executable, "control/et0_anderson_propagation/et0_anderson_propagation.py"],
        "rust_bin": "validate_et0_anderson",
    },
    {
        "name": "No-Till Sampling (Exp 023)",
        "python": [sys.executable, "control/notill_sampling/notill_sampling.py"],
        "rust_bin": "validate_notill_sampling",
    },
    {
        "name": "Aggregate Stability (Exp 024)",
        "python": [sys.executable, "control/aggregate_stability/aggregate_stability.py"],
        "rust_bin": "validate_aggregate_stability",
    },
    {
        "name": "Precision Drift (Exp 025)",
        "python": [sys.executable, "control/precision_drift/precision_drift.py"],
        "rust_bin": "validate_precision_drift",
    },
    {
        "name": "Size Convergence (Exp 026)",
        "python": [sys.executable, "control/size_convergence/size_convergence.py"],
        "rust_bin": "validate_size_convergence",
    },
    {
        "name": "Vendor Parity (Exp 027)",
        "python": [sys.executable, "control/vendor_parity/vendor_parity.py"],
        "rust_bin": "validate_vendor_parity",
    },
    {
        "name": "ET₀ Methods (Exp 035)",
        "python": [sys.executable, "control/et0_methods/et0_methods.py"],
        "rust_bin": "validate_et0_methods",
    },
]

# NPU experiment (requires /dev/akida0 + npu feature)
NPU_EXPERIMENTS = [
    {
        "name": "NPU Anderson (Exp 028)",
        "python": [sys.executable, "control/npu_anderson/npu_anderson.py"],
        "rust_bin": "validate_npu_anderson",
        "rust_features": ["npu"],
    },
]


def main() -> int:
    print("=" * 78)
    print("  groundSpring — Python vs barraCuda CPU Delegation Benchmark")
    print("=" * 78)

    print("\n  Building Rust (pure)...")
    subprocess.run(
        ["cargo", "build", "--release", "--workspace"],
        cwd=str(ROOT), capture_output=True, check=False,
    )

    print("  Building Rust (barraCuda CPU)...")
    subprocess.run(
        ["cargo", "build", "--release", "--workspace", "--features", "barracuda"],
        cwd=str(ROOT), capture_output=True, check=False,
    )

    results: list[TierBench] = []

    for exp in EXPERIMENTS:
        print(f"\n--- {exp['name']} ---")

        py_ok, py_s = timed_run(exp["python"])
        print(f"  Python:         {py_s:7.3f}s {'PASS' if py_ok else 'FAIL'}")

        rs_cmd = ["cargo", "run", "--release", "--bin", exp["rust_bin"]]
        rs_ok, rs_s = timed_run(rs_cmd)
        print(f"  Rust (pure):    {rs_s:7.3f}s {'PASS' if rs_ok else 'FAIL'}")

        bc_cmd = ["cargo", "run", "--release", "--bin", exp["rust_bin"],
                   "--features", "barracuda"]
        bc_ok, bc_s = timed_run(bc_cmd)
        print(f"  Rust (bC CPU):  {bc_s:7.3f}s {'PASS' if bc_ok else 'FAIL'}")

        results.append(TierBench(
            name=exp["name"],
            python_s=py_s,
            rust_pure_s=rs_s,
            rust_barracuda_s=bc_s,
            python_pass=py_ok,
            rust_pure_pass=rs_ok,
            rust_barracuda_pass=bc_ok,
        ))

    # Summary table
    print(f"\n{'=' * 78}")
    print("  THREE-TIER BENCHMARK SUMMARY")
    print(f"{'=' * 78}\n")

    hdr = f"| {'Experiment':<30} | {'Python':>8} | {'Rust':>8} | {'bC CPU':>8} | {'Py→Rs':>6} | {'Py→bC':>6} | {'Rs→bC':>6} |"
    sep = f"|{'-'*32}|{'-'*10}|{'-'*10}|{'-'*10}|{'-'*8}|{'-'*8}|{'-'*8}|"
    print(hdr)
    print(sep)

    for r in results:
        py_rs = r.python_s / r.rust_pure_s if r.rust_pure_s > 0 else 0
        py_bc = r.python_s / r.rust_barracuda_s if r.rust_barracuda_s > 0 else 0
        rs_bc = r.rust_pure_s / r.rust_barracuda_s if r.rust_barracuda_s > 0 else 0
        print(
            f"| {r.name:<30} "
            f"| {r.python_s:7.3f}s "
            f"| {r.rust_pure_s:7.3f}s "
            f"| {r.rust_barracuda_s:7.3f}s "
            f"| {py_rs:5.1f}× "
            f"| {py_bc:5.1f}× "
            f"| {rs_bc:5.2f}× |"
        )

    n_all_pass = sum(1 for r in results if r.python_pass and r.rust_pure_pass and r.rust_barracuda_pass)
    print(f"\n  {n_all_pass}/{len(results)} experiments: all three tiers PASS")
    print(f"{'=' * 78}")

    # NPU experiments (require /dev/akida0 hardware)
    npu_device = Path("/dev/akida0")
    if npu_device.exists():
        print(f"\n{'=' * 78}")
        print("  NPU EXPERIMENTS (hardware detected)")
        print(f"{'=' * 78}")
        for exp in NPU_EXPERIMENTS:
            print(f"\n--- {exp['name']} ---")
            py_ok, py_s = timed_run(exp["python"])
            print(f"  Python:         {py_s:7.3f}s {'PASS' if py_ok else 'FAIL'}")

            rs_cmd = ["cargo", "run", "--release", "--bin", exp["rust_bin"],
                       "--features", "npu"]
            rs_ok, rs_s = timed_run(rs_cmd)
            print(f"  Rust (NPU):     {rs_s:7.3f}s {'PASS' if rs_ok else 'FAIL'}")

            results.append(TierBench(
                name=exp["name"],
                python_s=py_s,
                rust_pure_s=rs_s,
                rust_barracuda_s=rs_s,
                python_pass=py_ok,
                rust_pure_pass=rs_ok,
                rust_barracuda_pass=rs_ok,
            ))
    else:
        print(f"\n  [SKIP] NPU experiments — /dev/akida0 not present")

    cert = {
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "title": "groundSpring Python vs barraCuda CPU Delegation Benchmark",
        "tiers": {
            "tier_0": "Python baseline (control scripts)",
            "tier_2": "Rust pure (no barracuda feature)",
            "tier_2b": "Rust + barraCuda CPU delegation (--features barracuda)",
        },
        "experiments": [
            {
                "name": r.name,
                "python_s": round(r.python_s, 4),
                "rust_pure_s": round(r.rust_pure_s, 4),
                "rust_barracuda_s": round(r.rust_barracuda_s, 4),
                "speedup_py_rs": round(r.python_s / max(r.rust_pure_s, 0.001), 2),
                "speedup_py_bc": round(r.python_s / max(r.rust_barracuda_s, 0.001), 2),
                "speedup_rs_bc": round(r.rust_pure_s / max(r.rust_barracuda_s, 0.001), 3),
                "all_pass": r.python_pass and r.rust_pure_pass and r.rust_barracuda_pass,
            }
            for r in results
        ],
        "summary": {
            "all_pass": n_all_pass,
            "total": len(results),
        },
    }

    out_path = ROOT / "data" / "barracuda_cpu_benchmark.json"
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with open(out_path, "w") as f:
        json.dump(cert, f, indent=2)
    print(f"\nCertificate saved to {out_path}")

    return 0 if n_all_pass == len(results) else 1


if __name__ == "__main__":
    sys.exit(main())
