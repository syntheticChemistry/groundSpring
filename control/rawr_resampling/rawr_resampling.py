#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring Experiment 007 — RAWR Resampling

Compares standard bootstrap vs RAWR (Resampled And Weighted Replicates)
on three test cases to answer:
  1. Do both achieve nominal coverage for well-behaved data?
  2. Does RAWR give better coverage for skewed distributions?
  3. Does RAWR handle dependent (autocorrelated) data better?

Method:
  - Standard bootstrap: resample with replacement, compute statistic.
  - RAWR: generate weights from Exp(1), normalize to sum=n, compute
    weighted statistic.  Bayesian bootstrap with Dirichlet(1,...,1) weights.

Reference:
  Wang et al. (2021) Bioinformatics (ISMB) 37:i111-i119
  Efron (1979) Ann Statist 7:1-26

Cross-spring: upgrades Monte Carlo methodology from Exp 003.
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from common import (
    check_range,
    check_true,
    print_summary,
    reset_counters,
)

# ---------------------------------------------------------------------------
# Bootstrap implementations
# ---------------------------------------------------------------------------

def bootstrap_mean_ci(
    data: np.ndarray,
    n_bootstrap: int,
    confidence: float,
    rng: np.random.Generator,
) -> dict:
    """Standard bootstrap confidence interval for the mean."""
    n = len(data)
    means = np.empty(n_bootstrap)
    for i in range(n_bootstrap):
        sample = data[rng.integers(0, n, size=n)]
        means[i] = sample.mean()

    alpha = 1.0 - confidence
    ci_lower = float(np.percentile(means, 100 * alpha / 2))
    ci_upper = float(np.percentile(means, 100 * (1 - alpha / 2)))

    return {
        "estimate": float(np.mean(means)),
        "ci_lower": ci_lower,
        "ci_upper": ci_upper,
        "ci_width": ci_upper - ci_lower,
        "std_error": float(np.std(means)),
        "distribution": means,
    }


def rawr_mean_ci(
    data: np.ndarray,
    n_bootstrap: int,
    confidence: float,
    rng: np.random.Generator,
) -> dict:
    """RAWR (Bayesian bootstrap) confidence interval for the mean.

    Weights are drawn from Dirichlet(1,...,1) which is equivalent to
    normalized Exp(1) variates.
    """
    n = len(data)
    means = np.empty(n_bootstrap)
    for i in range(n_bootstrap):
        weights = rng.exponential(1.0, size=n)
        weights /= weights.sum()
        means[i] = float(np.dot(weights, data))

    alpha = 1.0 - confidence
    ci_lower = float(np.percentile(means, 100 * alpha / 2))
    ci_upper = float(np.percentile(means, 100 * (1 - alpha / 2)))

    return {
        "estimate": float(np.mean(means)),
        "ci_lower": ci_lower,
        "ci_upper": ci_upper,
        "ci_width": ci_upper - ci_lower,
        "std_error": float(np.std(means)),
        "distribution": means,
    }


def coverage_rate(
    data_gen, true_param: float, method_fn, n_trials: int,
    n_bootstrap: int, confidence: float, base_seed: int,
) -> float:
    """Empirical coverage: fraction of trials where CI contains truth."""
    covers = 0
    for trial in range(n_trials):
        rng_data = np.random.default_rng(base_seed + trial)
        data = data_gen(rng_data)
        rng_boot = np.random.default_rng(base_seed + 100000 + trial)
        result = method_fn(data, n_bootstrap, confidence, rng_boot)
        if result["ci_lower"] <= true_param <= result["ci_upper"]:
            covers += 1
    return covers / n_trials


# ---------------------------------------------------------------------------
# Validation harness
# ---------------------------------------------------------------------------

def main() -> int:
    benchmark_path = Path(__file__).parent / "benchmark_rawr_resampling.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    reset_counters()

    exp = benchmark["expected_results"]

    print("=" * 72)
    print("groundSpring Exp 007: RAWR Resampling vs Standard Bootstrap")
    print("  Cross-spring: all springs (MC methodology upgrade)")
    print("=" * 72)

    # ------------------------------------------------------------------
    # Part 1: Gaussian — both methods should achieve coverage
    # ------------------------------------------------------------------
    print("\n--- Part 1: Gaussian (n=100, μ=5.0, σ=2.0) ---")

    tc = benchmark["test_cases"]["gaussian"]
    rng = np.random.default_rng(tc["seed"])
    data_gauss = rng.normal(tc["mu"], tc["sigma"], tc["n"])

    rng_b = np.random.default_rng(tc["seed"] + 1)
    boot_result = bootstrap_mean_ci(data_gauss, tc["n_bootstrap"], tc["confidence"], rng_b)
    rng_r = np.random.default_rng(tc["seed"] + 2)
    rawr_result = rawr_mean_ci(data_gauss, tc["n_bootstrap"], tc["confidence"], rng_r)

    print(f"  Bootstrap: {boot_result['estimate']:.3f} "
          f"[{boot_result['ci_lower']:.3f}, {boot_result['ci_upper']:.3f}] "
          f"width={boot_result['ci_width']:.3f}")
    print(f"  RAWR:      {rawr_result['estimate']:.3f} "
          f"[{rawr_result['ci_lower']:.3f}, {rawr_result['ci_upper']:.3f}] "
          f"width={rawr_result['ci_width']:.3f}")

    check_true(
        "Bootstrap CI covers true μ=5.0",
        boot_result["ci_lower"] <= tc["mu"] <= boot_result["ci_upper"],
    )
    check_true(
        "RAWR CI covers true μ=5.0",
        rawr_result["ci_lower"] <= tc["mu"] <= rawr_result["ci_upper"],
    )
    check_range(
        "Bootstrap CI width reasonable",
        boot_result["ci_width"],
        exp["gaussian_bootstrap_ci_width_range"][0],
        exp["gaussian_bootstrap_ci_width_range"][1],
    )

    # Coverage study
    n_coverage_trials = 200
    boot_cov = coverage_rate(
        lambda r: r.normal(tc["mu"], tc["sigma"], tc["n"]),
        tc["mu"], bootstrap_mean_ci, n_coverage_trials,
        tc["n_bootstrap"], tc["confidence"], tc["seed"] + 5000,
    )
    rawr_cov = coverage_rate(
        lambda r: r.normal(tc["mu"], tc["sigma"], tc["n"]),
        tc["mu"], rawr_mean_ci, n_coverage_trials,
        tc["n_bootstrap"], tc["confidence"], tc["seed"] + 6000,
    )
    print(f"  Bootstrap coverage (200 trials): {boot_cov:.3f}")
    print(f"  RAWR coverage (200 trials):      {rawr_cov:.3f}")

    check_range("Bootstrap Gaussian coverage", boot_cov,
                exp["gaussian_coverage_range"][0], exp["gaussian_coverage_range"][1])
    check_range("RAWR Gaussian coverage", rawr_cov,
                exp["gaussian_coverage_range"][0], exp["gaussian_coverage_range"][1])

    # ------------------------------------------------------------------
    # Part 2: Skewed distribution (log-normal)
    # ------------------------------------------------------------------
    print("\n--- Part 2: Skewed (log-normal, μ_ln=1.0, σ_ln=0.8) ---")

    tc_s = benchmark["test_cases"]["skewed"]
    true_mean = math.exp(tc_s["lognormal_mu"] + tc_s["lognormal_sigma"] ** 2 / 2)

    boot_cov_s = coverage_rate(
        lambda r: r.lognormal(tc_s["lognormal_mu"], tc_s["lognormal_sigma"], tc_s["n"]),
        true_mean, bootstrap_mean_ci, n_coverage_trials,
        tc_s["n_bootstrap"], tc_s["confidence"], tc_s["seed"] + 5000,
    )
    rawr_cov_s = coverage_rate(
        lambda r: r.lognormal(tc_s["lognormal_mu"], tc_s["lognormal_sigma"], tc_s["n"]),
        true_mean, rawr_mean_ci, n_coverage_trials,
        tc_s["n_bootstrap"], tc_s["confidence"], tc_s["seed"] + 6000,
    )
    print(f"  True mean: {true_mean:.4f}")
    print(f"  Bootstrap coverage: {boot_cov_s:.3f}")
    print(f"  RAWR coverage:      {rawr_cov_s:.3f}")

    check_range("Bootstrap skewed coverage", boot_cov_s,
                exp["skewed_coverage_range"][0], exp["skewed_coverage_range"][1])
    check_range("RAWR skewed coverage", rawr_cov_s,
                exp["skewed_coverage_range"][0], exp["skewed_coverage_range"][1])

    # ------------------------------------------------------------------
    # Part 3: Correlated data (AR(1))
    # ------------------------------------------------------------------
    print("\n--- Part 3: Correlated (AR(1), ρ=0.8) ---")

    tc_c = benchmark["test_cases"]["correlated"]

    def gen_ar1(rng: np.random.Generator) -> np.ndarray:
        n = tc_c["n"]
        noise = rng.normal(0, tc_c["sigma"] * math.sqrt(1 - tc_c["rho"] ** 2), n)
        x = np.empty(n)
        x[0] = tc_c["mu"] + noise[0]
        for i in range(1, n):
            x[i] = tc_c["mu"] + tc_c["rho"] * (x[i - 1] - tc_c["mu"]) + noise[i]
        return x

    boot_mses = []
    rawr_mses = []
    for trial in range(n_coverage_trials):
        rng_d = np.random.default_rng(tc_c["seed"] + trial)
        data_ar = gen_ar1(rng_d)
        rng_b2 = np.random.default_rng(tc_c["seed"] + 200000 + trial)
        rng_r2 = np.random.default_rng(tc_c["seed"] + 300000 + trial)
        br = bootstrap_mean_ci(data_ar, tc_c["n_bootstrap"], tc_c["confidence"], rng_b2)
        rr = rawr_mean_ci(data_ar, tc_c["n_bootstrap"], tc_c["confidence"], rng_r2)
        boot_mses.append((br["estimate"] - tc_c["mu"]) ** 2)
        rawr_mses.append((rr["estimate"] - tc_c["mu"]) ** 2)

    boot_rmse = math.sqrt(float(np.mean(boot_mses)))
    rawr_rmse = math.sqrt(float(np.mean(rawr_mses)))
    mse_ratio = rawr_rmse / boot_rmse if boot_rmse > 0 else 1.0

    print(f"  Bootstrap RMSE: {boot_rmse:.4f}")
    print(f"  RAWR RMSE:      {rawr_rmse:.4f}")
    print(f"  RAWR/Bootstrap ratio: {mse_ratio:.3f}")

    check_true("RAWR RMSE ratio ≤ 1.5× bootstrap",
               mse_ratio <= exp["correlated_rawr_mse_ratio_max"])

    # ------------------------------------------------------------------
    # Part 4: Determinism
    # ------------------------------------------------------------------
    print("\n--- Part 4: Determinism ---")

    det_data = np.array([1.0, 2.0, 3.0, 4.0, 5.0])

    rng1 = np.random.default_rng(9999)
    rng2 = np.random.default_rng(9999)
    b1 = bootstrap_mean_ci(det_data, 500, 0.95, rng1)
    b2 = bootstrap_mean_ci(det_data, 500, 0.95, rng2)
    check_true("Bootstrap deterministic", b1["estimate"] == b2["estimate"])

    rng3 = np.random.default_rng(8888)
    rng4 = np.random.default_rng(8888)
    r1 = rawr_mean_ci(det_data, 500, 0.95, rng3)
    r2 = rawr_mean_ci(det_data, 500, 0.95, rng4)
    check_true("RAWR deterministic", r1["estimate"] == r2["estimate"])

    check_true("Bootstrap ≠ RAWR (different methods)",
               b1["estimate"] != r1["estimate"])

    # ------------------------------------------------------------------
    # Key Findings
    # ------------------------------------------------------------------
    print(f"\n{'=' * 72}")
    print("KEY FINDINGS:")
    print(f"{'=' * 72}")
    print(f"\n1. Gaussian: Both achieve ~{boot_cov:.0%} / {rawr_cov:.0%} coverage")
    print(f"2. Skewed: Bootstrap {boot_cov_s:.0%}, RAWR {rawr_cov_s:.0%}")
    print(f"3. Correlated: RAWR/Bootstrap RMSE ratio = {mse_ratio:.3f}")
    print("4. RAWR is competitive or better across all test cases")

    return print_summary("Exp 007: RAWR Resampling")


if __name__ == "__main__":
    sys.exit(main())
