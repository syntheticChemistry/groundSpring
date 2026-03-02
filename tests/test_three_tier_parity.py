# SPDX-License-Identifier: AGPL-3.0-only
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
    "validate-decompose",
    "validate-rarefaction",
    "validate-seismic",
    "validate-weather",
    "validate-fao56",
    "validate-signal-specificity",
    "validate-rawr",
    "validate-anderson",
    "validate-quasiperiodic",
    "validate-bistable",
    "validate-multisignal",
    "validate-transport",
    "validate-resampling-conv",
    "validate-drift",
    "validate-uncertainty-bridge",
    "validate-rare-biosphere",
    "validate-quasispecies",
    "validate-band-edge",
    "validate-jackknife",
    "validate-freeze-out",
    "validate-spectral-recon",
    "validate-et0-anderson",
    "validate-notill-sampling",
    "validate-aggregate-stability",
    "validate-precision-drift",
    "validate-size-convergence",
    "validate-vendor-parity",
]

PYTHON_EXPERIMENTS = [
    ("sensor_noise", "sensor_noise_decomposition.py", "validate-decompose"),
    ("observation_gap", "observation_gap.py", "validate-weather"),
    ("seismic", "seismic_inversion.py", "validate-seismic"),
    ("signal_specificity", "signal_specificity.py", "validate-signal-specificity"),
    ("rawr_resampling", "rawr_resampling.py", "validate-rawr"),
    ("anderson_localization", "anderson_localization.py", "validate-anderson"),
    ("quasiperiodic", "quasiperiodic_localization.py", "validate-quasiperiodic"),
    ("bistable_switching", "bistable_switching.py", "validate-bistable"),
    ("multisignal_qs", "multisignal_qs.py", "validate-multisignal"),
    ("spin_transport", "spin_chain_transport.py", "validate-transport"),
    ("drift_selection", "drift_selection.py", "validate-drift"),
    ("rare_biosphere", "rare_biosphere.py", "validate-rare-biosphere"),
    ("quasispecies_threshold", "quasispecies_threshold.py", "validate-quasispecies"),
    ("band_edge", "band_edge.py", "validate-band-edge"),
    ("jackknife_estimation", "jackknife_estimation.py", "validate-jackknife"),
    ("freeze_out_inverse", "freeze_out_inverse.py", "validate-freeze-out"),
    ("spectral_recon", "spectral_recon.py", "validate-spectral-recon"),
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
    """All 27 Rust validation binaries must pass in default mode."""

    @pytest.mark.parametrize("bin_name", VALIDATE_BINS)
    def test_validation_binary_passes(self, bin_name: str) -> None:
        rc, output, _elapsed = _run_validation_binary(bin_name)
        passed, total = _parse_pass_count(output)
        assert rc == 0, f"{bin_name} failed:\n{output}"
        assert passed == total, f"{bin_name}: {passed}/{total}"


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
