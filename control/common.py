# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring shared statistical primitives and validation harness.

All experiments share a common statistical framework rooted in
bias-variance error decomposition. This module provides the canonical
implementations so each experiment script delegates to a single source
of truth rather than carrying its own copy.
"""

from __future__ import annotations

import json
import math
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import TYPE_CHECKING

import numpy as np

if TYPE_CHECKING:
    pass


# ---------------------------------------------------------------------------
# Provenance — reproducible benchmark generation
# ---------------------------------------------------------------------------

def provenance_metadata(script_path: str, notes: str = "") -> dict:
    """Collect git commit, date, and command for benchmark provenance."""
    try:
        commit = subprocess.check_output(
            ["git", "rev-parse", "HEAD"],
            stderr=subprocess.DEVNULL,
        ).decode().strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        commit = "unknown"

    return {
        "baseline_date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
        "baseline_commit": commit,
        "validation_script": script_path,
        "command": f"python3 {script_path}",
        "notes": notes,
    }


def write_benchmark(
    data: dict,
    output_path: str | Path,
    *,
    script_path: str,
    notes: str = "",
) -> None:
    """Write a benchmark JSON with embedded provenance metadata.

    Merges a fresh ``_provenance`` block into *data* and writes
    pretty-printed JSON to *output_path*.
    """
    data["_provenance"] = provenance_metadata(script_path, notes)
    path = Path(output_path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    print(f"  Wrote benchmark: {path}")


# ---------------------------------------------------------------------------
# Error decomposition (Pillar 1: Signal vs Noise)
# ---------------------------------------------------------------------------

def decompose_error(mbe: float, rmse: float) -> dict[str, float]:
    """Decompose total RMSE into bias and random noise components.

    RMSE² = MBE² + Var(error)

    where MBE (Mean Bias Error) is the correctable systematic component
    and Var(error) is the irreducible random noise.

    Returns a dict with bias, random_std, bias_fraction, and noise_fraction.
    """
    bias_sq = mbe ** 2
    total_sq = rmse ** 2
    variance = max(0.0, total_sq - bias_sq)
    random_std = math.sqrt(variance)
    bias_fraction = bias_sq / total_sq if total_sq > 0 else 0.0

    return {
        "bias": mbe,
        "bias_abs": abs(mbe),
        "random_std": random_std,
        "total_rmse": rmse,
        "bias_sq": bias_sq,
        "variance": variance,
        "bias_fraction": bias_fraction,
        "noise_fraction": 1.0 - bias_fraction,
    }


def noise_floor_reduction(
    factory_rmse: float, corrected_rmse: float
) -> dict[str, float]:
    """Quantify removable (systematic) vs irreducible (noise) error.

    After soil-specific correction, the corrected RMSE represents
    the irreducible noise floor for that sensor–soil combination.
    """
    if factory_rmse ** 2 > corrected_rmse ** 2:
        removed = math.sqrt(factory_rmse ** 2 - corrected_rmse ** 2)
    else:
        removed = 0.0

    reduction_pct = (
        (1.0 - corrected_rmse / factory_rmse) * 100.0 if factory_rmse > 0 else 0.0
    )

    return {
        "factory_rmse": factory_rmse,
        "corrected_rmse": corrected_rmse,
        "removed_error": removed,
        "noise_floor": corrected_rmse,
        "reduction_pct": reduction_pct,
    }


# ---------------------------------------------------------------------------
# Core statistical metrics
# ---------------------------------------------------------------------------

def compute_rmse(observed: np.ndarray, modeled: np.ndarray) -> float:
    """Root Mean Square Error."""
    return float(np.sqrt(np.mean((observed - modeled) ** 2)))


def compute_mbe(observed: np.ndarray, modeled: np.ndarray) -> float:
    """Mean Bias Error (modeled − observed)."""
    return float(np.mean(modeled - observed))


def compute_r2(observed: np.ndarray, modeled: np.ndarray) -> float:
    """Coefficient of determination (R²)."""
    ss_res = np.sum((observed - modeled) ** 2)
    ss_tot = np.sum((observed - np.mean(observed)) ** 2)
    if ss_tot == 0:
        return 0.0
    return float(1.0 - ss_res / ss_tot)


def compute_ia(observed: np.ndarray, modeled: np.ndarray) -> float:
    """Index of Agreement (Willmott 1981)."""
    o_bar = np.mean(observed)
    num = np.sum((observed - modeled) ** 2)
    den = np.sum((np.abs(modeled - o_bar) + np.abs(observed - o_bar)) ** 2)
    if den == 0:
        return 0.0
    return float(1.0 - num / den)


def bias_variance_decompose(
    obs: np.ndarray, mod: np.ndarray
) -> dict[str, float]:
    """Decompose model-observation gap into bias and variance components."""
    mbe = compute_mbe(obs, mod)
    rmse = compute_rmse(obs, mod)
    return decompose_error(mbe, rmse)


# ---------------------------------------------------------------------------
# Validation harness — canonical check functions
# ---------------------------------------------------------------------------

_PASS_COUNT = 0
_FAIL_COUNT = 0


def reset_counters() -> None:
    """Reset global pass/fail counters for a new validation run."""
    global _PASS_COUNT, _FAIL_COUNT  # noqa: PLW0603
    _PASS_COUNT = 0
    _FAIL_COUNT = 0


def pass_count() -> int:
    return _PASS_COUNT


def fail_count() -> int:
    return _FAIL_COUNT


def total_count() -> int:
    return _PASS_COUNT + _FAIL_COUNT


def _record(passed: bool) -> bool:
    global _PASS_COUNT, _FAIL_COUNT  # noqa: PLW0603
    if passed:
        _PASS_COUNT += 1
    else:
        _FAIL_COUNT += 1
    return passed


def check_approx(
    label: str, computed: float, expected: float, tol: float
) -> bool:
    """Check that *computed* is within *tol* of *expected*."""
    diff = abs(computed - expected)
    ok = diff <= tol
    status = "PASS" if ok else "FAIL"
    print(
        f"  [{status}] {label}: {computed:.4f} "
        f"(expected {expected:.4f}, tol {tol:.4f}, diff {diff:.4f})"
    )
    return _record(ok)


def check_range(
    label: str, computed: float, low: float, high: float
) -> bool:
    """Check that *computed* falls within [*low*, *high*]."""
    ok = low <= computed <= high
    status = "PASS" if ok else "FAIL"
    print(
        f"  [{status}] {label}: {computed:.4f} "
        f"(expected [{low:.4f}, {high:.4f}])"
    )
    return _record(ok)


def check_min(label: str, computed: float, minimum: float) -> bool:
    """Check that *computed* >= *minimum*."""
    ok = computed >= minimum
    status = "PASS" if ok else "FAIL"
    print(f"  [{status}] {label}: {computed:.4f} (minimum {minimum:.4f})")
    return _record(ok)


def check_max(label: str, computed: float, maximum: float) -> bool:
    """Check that *computed* <= *maximum*."""
    ok = computed <= maximum
    status = "PASS" if ok else "FAIL"
    print(f"  [{status}] {label}: {computed:.4f} (max {maximum:.4f})")
    return _record(ok)


def check_true(label: str, condition: bool) -> bool:
    """Check that *condition* is True."""
    status = "PASS" if condition else "FAIL"
    print(f"  [{status}] {label}")
    return _record(condition)


def print_summary(experiment_name: str = "") -> int:
    """Print final PASS/FAIL summary and return exit code (0 or 1)."""
    total = total_count()
    p = pass_count()
    f = fail_count()
    print(f"\n{'=' * 72}")
    if experiment_name:
        print(f"{experiment_name}")
    print(f"TOTAL: {p}/{total} PASS, {f}/{total} FAIL")
    print(f"{'=' * 72}")
    return 0 if f == 0 else 1
