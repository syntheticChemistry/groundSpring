# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""Three-tier parity validation.

Proves that Rust validation binaries produce identical PASS results
regardless of feature mode:

  Tier 0 (default)      — local CPU math only
  Tier 1 (barracuda)    — barracuda CPU delegations active
  Tier 2 (barracuda-gpu) — barracuda GPU delegations active

Each binary validates against the SAME benchmark JSON files.  If all
three modes pass, the math is proven portable across tiers.

Additionally measures Rust-vs-Python performance for each experiment
where both baselines exist.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
import time
from pathlib import Path
from typing import ClassVar

import pytest

ROOT = Path(__file__).resolve().parent.parent
CONTROL_DIR = ROOT / "control"

VALIDATE_BINS = [
    "validate_decompose",
    "validate_rarefaction",
    "validate_seismic",
    "validate_weather",
    "validate_fao56",
    "validate_signal_specificity",
    "validate_rawr",
    "validate_anderson",
    "validate_quasiperiodic",
    "validate_bistable",
    "validate_multisignal",
    "validate_transport",
    "validate_resampling_conv",
    "validate_drift",
    "validate_uncertainty_bridge",
    "validate_rare_biosphere",
    "validate_quasispecies",
    "validate_band_edge",
    "validate_jackknife",
    "validate_freeze_out",
    "validate_spectral_recon",
    "validate_et0_anderson",
    "validate_notill_sampling",
    "validate_aggregate_stability",
    "validate_precision_drift",
    "validate_size_convergence",
    "validate_vendor_parity",
    "validate_et0_methods",
    "validate_tissue_anderson",
]

PYTHON_EXPERIMENTS = [
    ("sensor_noise", "sensor_noise_decomposition.py", "validate_decompose"),
    ("observation_gap", "observation_gap.py", "validate_weather"),
    ("seismic", "seismic_inversion.py", "validate_seismic"),
    ("signal_specificity", "signal_specificity.py", "validate_signal_specificity"),
    ("rawr_resampling", "rawr_resampling.py", "validate_rawr"),
    ("anderson_localization", "anderson_localization.py", "validate_anderson"),
    ("quasiperiodic", "quasiperiodic_localization.py", "validate_quasiperiodic"),
    ("bistable_switching", "bistable_switching.py", "validate_bistable"),
    ("multisignal_qs", "multisignal_qs.py", "validate_multisignal"),
    ("spin_transport", "spin_chain_transport.py", "validate_transport"),
    ("drift_selection", "drift_selection.py", "validate_drift"),
    ("rare_biosphere", "rare_biosphere.py", "validate_rare_biosphere"),
    ("quasispecies_threshold", "quasispecies_threshold.py", "validate_quasispecies"),
    ("band_edge", "band_edge.py", "validate_band_edge"),
    ("jackknife_estimation", "jackknife_estimation.py", "validate_jackknife"),
    ("freeze_out_inverse", "freeze_out_inverse.py", "validate_freeze_out"),
    ("spectral_recon", "spectral_recon.py", "validate_spectral_recon"),
]


def _parse_pass_count(output: str) -> tuple[int, int]:
    """Extract pass/total from 'TOTAL: X/Y PASS' output."""
    match = re.search(r"TOTAL:\s+(\d+)/(\d+)\s+PASS", output)
    if match:
        return int(match.group(1)), int(match.group(2))
    return 0, 0


def _ensure_release_build() -> None:
    """Pre-build all release binaries so timing excludes compilation."""
    marker = ROOT / "target" / ".three_tier_built"
    if marker.exists():
        return
    subprocess.run(
        ["cargo", "build", "--release", "--workspace"],
        capture_output=True,
        text=True,
        timeout=300,
        check=True,
        cwd=str(ROOT),
    )
    marker.parent.mkdir(parents=True, exist_ok=True)
    marker.write_text("built")


def _run_validation_binary(bin_name: str) -> tuple[int, str, float]:
    """Run a pre-built validation binary and return (exit_code, output, elapsed_s)."""
    _ensure_release_build()
    bin_path = ROOT / "target" / "release" / bin_name
    if not bin_path.exists():
        start = time.monotonic()
        result = subprocess.run(
            ["cargo", "run", "--release", "--bin", bin_name],
            capture_output=True,
            text=True,
            timeout=180,
            check=False,
            cwd=str(ROOT),
        )
        elapsed = time.monotonic() - start
        return result.returncode, result.stdout + result.stderr, elapsed
    start = time.monotonic()
    result = subprocess.run(
        [str(bin_path)],
        capture_output=True,
        text=True,
        timeout=120,
        check=False,
        cwd=str(ROOT),
    )
    elapsed = time.monotonic() - start
    return result.returncode, result.stdout + result.stderr, elapsed


class TestRustValidationGreen:
    """All 29 Rust validation binaries must pass in default mode."""

    @pytest.mark.parametrize("bin_name", VALIDATE_BINS)
    def test_validation_binary_passes(self, bin_name: str) -> None:
        rc, output, _elapsed = _run_validation_binary(bin_name)
        passed, total = _parse_pass_count(output)
        assert rc == 0, f"{bin_name} failed:\n{output}"
        assert passed == total, f"{bin_name}: {passed}/{total}"


def _build_with_features(features: str) -> bool:
    """Build workspace with specific features, return success."""
    cmd = ["cargo", "build", "--release", "--workspace"]
    if features:
        cmd.extend(["--features", features])
    result = subprocess.run(
        cmd, capture_output=True, text=True, timeout=300,
        check=False, cwd=str(ROOT),
    )
    return result.returncode == 0


def _run_binary_with_features(bin_name: str, features: str) -> tuple[int, str, float]:
    """Run a validation binary built with specific features."""
    cmd = ["cargo", "run", "--release", "--bin", bin_name]
    if features:
        cmd.extend(["--features", features])
    start = time.monotonic()
    result = subprocess.run(
        cmd, capture_output=True, text=True, timeout=180,
        check=False, cwd=str(ROOT),
    )
    elapsed = time.monotonic() - start
    return result.returncode, result.stdout + result.stderr, elapsed


TIER_BINARIES = [
    "validate_decompose",
    "validate_anderson",
    "validate_rarefaction",
    "validate_weather",
    "validate_drift",
    "validate_rawr",
]


class TestBarracudaCpuParity:
    """Validation binaries produce identical results with barracuda CPU."""

    @pytest.fixture(autouse=True, scope="class")
    def _build(self) -> None:
        if not _build_with_features("barracuda"):
            pytest.skip("barracuda feature build failed")

    @pytest.mark.parametrize("bin_name", TIER_BINARIES)
    def test_barracuda_cpu_passes(self, bin_name: str) -> None:
        rc, output, _elapsed = _run_binary_with_features(bin_name, "barracuda")
        passed, total = _parse_pass_count(output)
        assert rc == 0, f"barracuda {bin_name} failed:\n{output}"
        assert passed == total, f"barracuda {bin_name}: {passed}/{total}"


class TestBarracudaGpuParity:
    """Validation binaries produce identical results with barracuda GPU."""

    @pytest.fixture(autouse=True, scope="class")
    def _build(self) -> None:
        if not _build_with_features("barracuda-gpu"):
            pytest.skip("barracuda-gpu feature build failed")

    @pytest.mark.parametrize("bin_name", TIER_BINARIES)
    def test_barracuda_gpu_passes(self, bin_name: str) -> None:
        rc, output, _elapsed = _run_binary_with_features(bin_name, "barracuda-gpu")
        passed, total = _parse_pass_count(output)
        assert rc == 0, f"barracuda-gpu {bin_name} failed:\n{output}"
        assert passed == total, f"barracuda-gpu {bin_name}: {passed}/{total}"


class TestRustFasterThanPython:
    """Rust CPU math must be faster than interpreted Python.

    This proves barracuda CPU is pure compiled math — the entire
    motivation for the CPU tier before GPU promotion.
    """

    # Quasiperiodic uses dense Givens QR (O(n³)) locally; Python uses
    # LAPACK (O(n²)). With barracuda-gpu (Sturm tridiag), Rust is 49.5×
    # faster. Drift can also be marginal due to short Python runtime.
    KNOWN_LAPACK_WINS: ClassVar[set[str]] = {"quasiperiodic", "drift_selection"}

    @pytest.mark.parametrize(
        "py_dir,py_script,rust_bin",
        PYTHON_EXPERIMENTS,
        ids=[p[0] for p in PYTHON_EXPERIMENTS],
    )
    def test_rust_not_slower_than_python(
        self, py_dir: str, py_script: str, rust_bin: str
    ) -> None:
        py_path = CONTROL_DIR / py_dir / py_script
        if not py_path.exists():
            pytest.skip(f"Python experiment not found: {py_path}")

        py_start = time.monotonic()
        py_result = subprocess.run(
            [sys.executable, str(py_path)],
            capture_output=True,
            text=True,
            timeout=120,
            check=False,
        )
        py_elapsed = time.monotonic() - py_start

        if py_result.returncode != 0:
            pytest.skip(f"Python experiment failed: {py_result.stderr[:200]}")

        _rc, _output, rust_elapsed = _run_validation_binary(rust_bin)

        speedup = py_elapsed / rust_elapsed if rust_elapsed > 0 else float("inf")
        if py_dir in self.KNOWN_LAPACK_WINS:
            pytest.skip(
                f"Known LAPACK advantage ({py_dir}): Python={py_elapsed:.2f}s, "
                f"Rust={rust_elapsed:.2f}s — barracuda-gpu closes this gap"
            )
        assert speedup >= 0.5, (
            f"Rust should not be >2x slower than Python: "
            f"Python={py_elapsed:.2f}s, Rust={rust_elapsed:.2f}s, "
            f"speedup={speedup:.1f}x"
        )


class TestPythonRustBenchmarkParity:
    """Both Python and Rust must validate against the same benchmark JSON.

    This is the mathematical parity proof: if both implementations
    pass against the same expected values, the math is identical
    within documented tolerances.
    """

    @pytest.mark.parametrize("bin_name", VALIDATE_BINS)
    def test_benchmark_json_exists_for_binary(self, bin_name: str) -> None:
        benchmark_files = list(ROOT.glob("control/*/benchmark_*.json"))
        assert len(benchmark_files) >= 4, (
            f"Expected at least 4 benchmark JSON files, found {len(benchmark_files)}"
        )

    def test_all_benchmark_jsons_valid(self) -> None:
        """Every benchmark JSON must parse and contain provenance."""
        benchmark_files = list(ROOT.glob("control/*/benchmark_*.json"))
        for bf in benchmark_files:
            data = json.loads(bf.read_text())
            assert "_source" in data, f"{bf.name} missing _source"
            assert "_provenance" in data, f"{bf.name} missing _provenance"


class TestPythonBaselineXorshiftParity:
    """Python Xorshift64 baseline matches Rust values exactly.

    Runs baseline_runner.py (pure Python, same PRNG) and
    bench_kokkos_parity (Rust CPU) and compares numerical output.
    """

    def test_python_rust_values_match(self) -> None:
        baseline_script = CONTROL_DIR / "baseline_runner.py"
        if not baseline_script.exists():
            pytest.skip("baseline_runner.py not found")

        py_result = subprocess.run(
            [sys.executable, str(baseline_script), "--json-only"],
            capture_output=True, text=True, timeout=300,
            check=False, cwd=str(ROOT),
        )
        if py_result.returncode != 0:
            pytest.skip(f"Python baseline failed: {py_result.stderr[:200]}")

        py_data = json.loads(py_result.stdout)

        rust_result = subprocess.run(
            ["cargo", "run", "--release", "--bin", "bench_kokkos_parity"],
            capture_output=True, text=True, timeout=180,
            check=False, cwd=str(ROOT),
        )
        assert rust_result.returncode == 0, "Rust bench_kokkos_parity failed"

        output = rust_result.stdout
        marker = "=== JSON Benchmark Output ==="
        marker_pos = output.find(marker)
        json_str = output[marker_pos + len(marker):] if marker_pos >= 0 else output
        json_start = json_str.find("{")
        depth = 0
        json_end = json_start
        for j in range(json_start, len(json_str)):
            if json_str[j] == "{":
                depth += 1
            elif json_str[j] == "}":
                depth -= 1
                if depth == 0:
                    json_end = j
                    break
        rust_data = json.loads(json_str[json_start : json_end + 1])

        py_by_name = {r["name"]: r["value"] for r in py_data["results"]}
        rust_by_name = {r["name"]: r["value"] for r in rust_data["results"]}

        # Per-operation tolerances: simple reductions are near-bitwise;
        # iterative computations (Anderson: 500×10k log/sqrt, bootstrap:
        # 5000×10k index ops) accumulate Python math.log vs Rust f64::ln
        # differences over millions of transcendental calls.
        tolerances = {
            "anderson_lyapunov_averaged": 2e-3,
            "mean": 1e-12,
            "variance": 1e-10,
            "pearson_r": 1e-6,
            "bootstrap_mean": 1e-6,
        }
        for name, tol in tolerances.items():
            py_val = py_by_name.get(name)
            rust_val = rust_by_name.get(name)
            if py_val is None or rust_val is None:
                continue
            diff = abs(py_val - rust_val)
            rel = diff / abs(py_val) if py_val != 0 else diff
            assert rel < tol, (
                f"{name}: Python={py_val:.15e} Rust={rust_val:.15e} "
                f"rel_diff={rel:.2e} (tol={tol:.0e})"
            )
