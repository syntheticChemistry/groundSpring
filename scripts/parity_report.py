#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring — Python ⇌ Rust Mathematical Parity Report

Runs each experiment's Python baseline and Rust validation binary,
both of which check against the SAME shared benchmark JSON.  If both
pass all checks, mathematical parity is proven within stated tolerances.

Usage:
    python3 scripts/parity_report.py

Output:
    data/parity_report.json — machine-readable parity certificate
    Markdown summary to stdout
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

NPU_AVAILABLE = Path("/dev/akida0").exists()


@dataclass
class ExperimentParity:
    name: str
    benchmark_json: str
    python_pass: bool
    python_checks: str
    rust_pass: bool
    rust_checks: str
    parity: bool


def run_and_capture(cmd: list[str], timeout: int = 300) -> tuple[bool, str]:
    """Run a command and return (success, stdout)."""
    try:
        result = subprocess.run(
            cmd, cwd=str(ROOT), capture_output=True, text=True, timeout=timeout,
            check=False,
        )
        return result.returncode == 0, result.stdout + result.stderr
    except subprocess.TimeoutExpired:
        return False, "TIMEOUT"


def extract_checks(output: str) -> str:
    """Pull check summary from Python or Rust output."""
    for line in reversed(output.splitlines()):
        if "PASS" in line and "/" in line:
            m = re.search(r"(\d+/\d+)", line)
            if m:
                return m.group(1)
        if "TOTAL:" in line:
            m = re.search(r"(\d+/\d+)", line)
            if m:
                return m.group(1)
    return "?"


EXPERIMENTS = [
    {
        "name": "Exp 001: Sensor Noise",
        "benchmark": "control/sensor_noise/benchmark_sensor_noise.json",
        "python": [sys.executable, "control/sensor_noise/sensor_noise_decomposition.py"],
        "rust_bin": "validate_decompose",
    },
    {
        "name": "Exp 002: Observation Gap",
        "benchmark": "control/observation_gap/benchmark_observation_gap.json",
        "python": [sys.executable, "control/observation_gap/observation_gap.py"],
        "rust_bin": "validate_weather",
    },
    {
        "name": "Exp 003: Error Propagation",
        "benchmark": "control/error_propagation/benchmark_error_propagation.json",
        "python": [sys.executable, "control/error_propagation/error_propagation_fao56.py"],
        "rust_bin": "validate_fao56",
    },
    {
        "name": "Exp 004: Sequencing Noise",
        "benchmark": "control/sequencing_noise/benchmark_sequencing_noise.json",
        "python": [sys.executable, "control/sequencing_noise/sequencing_noise.py"],
        "rust_bin": "validate_rarefaction",
    },
    {
        "name": "Exp 005: Seismic Inversion",
        "benchmark": "control/seismic/benchmark_seismic.json",
        "python": [sys.executable, "control/seismic/seismic_inversion.py"],
        "rust_bin": "validate_seismic",
    },
    {
        "name": "Exp 006: Signal Specificity",
        "benchmark": "control/signal_specificity/benchmark_signal_specificity.json",
        "python": [sys.executable, "control/signal_specificity/signal_specificity.py"],
        "rust_bin": "validate_signal_specificity",
    },
    {
        "name": "Exp 007: RAWR Resampling",
        "benchmark": "control/rawr_resampling/benchmark_rawr_resampling.json",
        "python": [sys.executable, "control/rawr_resampling/rawr_resampling.py"],
        "rust_bin": "validate_rawr",
    },
    {
        "name": "Exp 008: Anderson Localization",
        "benchmark": "control/anderson_localization/benchmark_anderson_localization.json",
        "python": [sys.executable, "control/anderson_localization/anderson_localization.py"],
        "rust_bin": "validate_anderson",
    },
    {
        "name": "Exp 009: Quasiperiodic",
        "benchmark": "control/quasiperiodic/benchmark_quasiperiodic.json",
        "python": [sys.executable, "control/quasiperiodic/quasiperiodic_localization.py"],
        "rust_bin": "validate_quasiperiodic",
    },
    {
        "name": "Exp 010: Bistable Switching",
        "benchmark": "control/bistable_switching/benchmark_bistable.json",
        "python": [sys.executable, "control/bistable_switching/bistable_switching.py"],
        "rust_bin": "validate_bistable",
    },
    {
        "name": "Exp 011: Multi-Signal QS",
        "benchmark": "control/multisignal_qs/benchmark_multisignal.json",
        "python": [sys.executable, "control/multisignal_qs/multisignal_qs.py"],
        "rust_bin": "validate_multisignal",
    },
    {
        "name": "Exp 012: Spin Chain Transport",
        "benchmark": "control/spin_transport/benchmark_spin_transport.json",
        "python": [sys.executable, "control/spin_transport/spin_chain_transport.py"],
        "rust_bin": "validate_transport",
    },
    {
        "name": "Exp 013: Resampling Convergence",
        "benchmark": "control/resampling_convergence/benchmark_resampling_convergence.json",
        "python": [sys.executable, "control/resampling_convergence/resampling_convergence.py"],
        "rust_bin": "validate_resampling_conv",
    },
    {
        "name": "Exp 014: Drift vs Selection",
        "benchmark": "control/drift_selection/benchmark_drift_selection.json",
        "python": [sys.executable, "control/drift_selection/drift_selection.py"],
        "rust_bin": "validate_drift",
    },
    {
        "name": "Exp 015: Uncertainty Bridge",
        "benchmark": "control/uncertainty_bridge/benchmark_uncertainty_bridge.json",
        "python": [sys.executable, "control/uncertainty_bridge/uncertainty_bridge.py"],
        "rust_bin": "validate_uncertainty_bridge",
    },
    {
        "name": "Exp 016: Rare Biosphere",
        "benchmark": "control/rare_biosphere/benchmark_rare_biosphere.json",
        "python": [sys.executable, "control/rare_biosphere/rare_biosphere.py"],
        "rust_bin": "validate_rare_biosphere",
    },
    {
        "name": "Exp 017: Quasispecies Threshold",
        "benchmark": "control/quasispecies_threshold/benchmark_quasispecies.json",
        "python": [sys.executable, "control/quasispecies_threshold/quasispecies_threshold.py"],
        "rust_bin": "validate_quasispecies",
    },
    {
        "name": "Exp 018: Band Edge",
        "benchmark": "control/band_edge/benchmark_band_edge.json",
        "python": [sys.executable, "control/band_edge/band_edge.py"],
        "rust_bin": "validate_band_edge",
    },
    {
        "name": "Exp 019: Jackknife Estimation",
        "benchmark": "control/jackknife_estimation/benchmark_jackknife.json",
        "python": [sys.executable, "control/jackknife_estimation/jackknife_estimation.py"],
        "rust_bin": "validate_jackknife",
    },
    {
        "name": "Exp 020: Freeze Out Inverse",
        "benchmark": "control/freeze_out_inverse/benchmark_freeze_out.json",
        "python": [sys.executable, "control/freeze_out_inverse/freeze_out_inverse.py"],
        "rust_bin": "validate_freeze_out",
    },
    {
        "name": "Exp 021: Spectral Recon",
        "benchmark": "control/spectral_recon/benchmark_spectral_recon.json",
        "python": [sys.executable, "control/spectral_recon/spectral_recon.py"],
        "rust_bin": "validate_spectral_recon",
    },
    {
        "name": "Exp 022: ET0 Anderson Propagation",
        "benchmark": "control/et0_anderson_propagation/benchmark_et0_anderson.json",
        "python": [sys.executable, "control/et0_anderson_propagation/et0_anderson_propagation.py"],
        "rust_bin": "validate_et0_anderson",
    },
    {
        "name": "Exp 023: No-Till Sampling",
        "benchmark": "control/notill_sampling/benchmark_notill_sampling.json",
        "python": [sys.executable, "control/notill_sampling/notill_sampling.py"],
        "rust_bin": "validate_notill_sampling",
    },
    {
        "name": "Exp 024: Aggregate Stability",
        "benchmark": "control/aggregate_stability/benchmark_aggregate_stability.json",
        "python": [sys.executable, "control/aggregate_stability/aggregate_stability.py"],
        "rust_bin": "validate_aggregate_stability",
    },
    {
        "name": "Exp 025: Precision Drift",
        "benchmark": "control/precision_drift/benchmark_precision_drift.json",
        "python": [sys.executable, "control/precision_drift/precision_drift.py"],
        "rust_bin": "validate_precision_drift",
    },
    {
        "name": "Exp 026: Size Convergence",
        "benchmark": "control/size_convergence/benchmark_size_convergence.json",
        "python": [sys.executable, "control/size_convergence/size_convergence.py"],
        "rust_bin": "validate_size_convergence",
    },
    {
        "name": "Exp 027: Vendor Parity",
        "benchmark": "control/vendor_parity/benchmark_vendor_parity.json",
        "python": [sys.executable, "control/vendor_parity/vendor_parity.py"],
        "rust_bin": "validate_vendor_parity",
    },
    {
        "name": "Exp 028: NPU Anderson",
        "benchmark": "control/npu_anderson/benchmark_npu_anderson.json",
        "python": [sys.executable, "control/npu_anderson/npu_anderson.py"],
        "rust_bin": "validate_npu_anderson",
        "rust_features": ["npu"],
        "npu_required": True,
    },
    {
        "name": "Exp 035: ET0 Methods",
        "benchmark": "control/et0_methods/benchmark_et0_methods.json",
        "python": [sys.executable, "control/et0_methods/et0_methods.py"],
        "rust_bin": "validate_et0_methods",
    },
    {
        "name": "Exp 036: LTEE Fitness B2",
        "benchmark": "control/ltee_fitness_dynamics/benchmark_ltee_fitness.json",
        "python": [sys.executable, "control/ltee_fitness_dynamics/ltee_fitness_dynamics.py"],
        "rust_bin": "validate_ltee_fitness",
    },
    {
        "name": "Exp 037: LTEE Neutral B1",
        "benchmark": "control/ltee_neutral_mutation/benchmark_ltee_neutral.json",
        "python": [sys.executable, "control/ltee_neutral_mutation/ltee_neutral_mutation.py"],
        "rust_bin": "validate_ltee_neutral",
    },
    {
        "name": "Exp 038: LTEE Clonal B3",
        "benchmark": "control/ltee_clonal_interference/benchmark_ltee_clonal.json",
        "python": [sys.executable, "control/ltee_clonal_interference/ltee_clonal_interference.py"],
        "rust_bin": "validate_ltee_clonal",
    },
    {
        "name": "Exp 039: LTEE Citrate B4",
        "benchmark": "control/ltee_citrate_innovation/benchmark_ltee_citrate.json",
        "python": [sys.executable, "control/ltee_citrate_innovation/ltee_citrate_innovation.py"],
        "rust_bin": "validate_ltee_citrate",
    },
    {
        "name": "Exp 040: LTEE BioBrick B6",
        "benchmark": "control/ltee_biobrick_burden/benchmark_ltee_biobrick.json",
        "python": [sys.executable, "control/ltee_biobrick_burden/ltee_biobrick_burden.py"],
        "rust_bin": "validate_ltee_biobrick",
    },
]


def main() -> int:
    print("=" * 72)
    print("groundSpring — Python ⇌ Rust Mathematical Parity Report")
    print("=" * 72)

    subprocess.run(
        ["cargo", "build", "--release", "--workspace"],
        cwd=str(ROOT), capture_output=True, check=False,
    )

    results: list[ExperimentParity] = []

    for exp in EXPERIMENTS:
        if exp.get("npu_required") and not NPU_AVAILABLE:
            print(f"\n--- {exp['name']} ---")
            print("  [SKIP] NPU hardware (/dev/akida0) not available")
            continue

        print(f"\n--- {exp['name']} ---")
        print(f"  Shared benchmark: {exp['benchmark']}")

        py_ok, py_out = run_and_capture(exp["python"])
        py_checks = extract_checks(py_out)
        print(f"  Python: {'PASS' if py_ok else 'FAIL'} ({py_checks})")

        rs_cmd = ["cargo", "run", "--release", "--bin", exp["rust_bin"]]
        if "rust_features" in exp:
            rs_cmd.extend(["--features", ",".join(exp["rust_features"])])
        rs_ok, rs_out = run_and_capture(rs_cmd)
        rs_checks = extract_checks(rs_out)
        print(f"  Rust:   {'PASS' if rs_ok else 'FAIL'} ({rs_checks})")

        parity = py_ok and rs_ok
        print(f"  Parity: {'PROVEN' if parity else 'UNPROVEN'}")

        results.append(ExperimentParity(
            name=exp["name"],
            benchmark_json=exp["benchmark"],
            python_pass=py_ok,
            python_checks=py_checks,
            rust_pass=rs_ok,
            rust_checks=rs_checks,
            parity=parity,
        ))

    # Summary
    print(f"\n{'=' * 72}")
    print("PARITY CERTIFICATE")
    print(f"{'=' * 72}\n")

    print(f"| {'Experiment':<30} | {'Benchmark JSON':<50} | {'Python':>7} | {'Rust':>7} | {'Parity':>8} |")
    print(f"|{'-'*32}|{'-'*52}|{'-'*9}|{'-'*9}|{'-'*10}|")

    n_parity = 0
    for r in results:
        p_status = f"{'PASS':>7}" if r.python_pass else f"{'FAIL':>7}"
        r_status = f"{'PASS':>7}" if r.rust_pass else f"{'FAIL':>7}"
        par = "PROVEN" if r.parity else "UNPROVEN"
        print(f"| {r.name:<30} | {r.benchmark_json:<50} | {p_status} | {r_status} | {par:>8} |")
        if r.parity:
            n_parity += 1

    print(f"\n  {n_parity}/{len(results)} experiments demonstrate mathematical parity.")
    print("  Both Python and Rust validate against the same benchmark JSON files.")
    print("  Parity is proven within the tolerances specified in each benchmark.")

    if n_parity == len(results):
        print(f"\n  ALL {n_parity} EXPERIMENTS: PARITY PROVEN")
    else:
        print(f"\n  WARNING: {len(results) - n_parity} experiment(s) lack parity.")

    print(f"{'=' * 72}")

    # JSON certificate
    cert = {
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "title": "groundSpring Python-Rust Mathematical Parity Certificate",
        "method": "Both languages validate against shared benchmark JSON expected values",
        "experiments": [
            {
                "name": r.name,
                "benchmark_json": r.benchmark_json,
                "python_pass": r.python_pass,
                "python_checks": r.python_checks,
                "rust_pass": r.rust_pass,
                "rust_checks": r.rust_checks,
                "parity_proven": r.parity,
            }
            for r in results
        ],
        "total_parity": n_parity,
        "total_experiments": len(results),
        "all_parity": n_parity == len(results),
    }

    out_path = ROOT / "data" / "parity_report.json"
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with open(out_path, "w") as f:
        json.dump(cert, f, indent=2)
    print(f"\nCertificate saved to {out_path}")

    return 0 if n_parity == len(results) else 1


if __name__ == "__main__":
    sys.exit(main())
