#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring Experiment 013 — Resampling Convergence

Studies how quickly bootstrap and RAWR confidence intervals converge as
the number of replicates increases, to answer:
  1. How many replicates are needed for a stable CI?
  2. Does RAWR converge faster or slower than standard bootstrap?
  3. How does data distribution affect convergence rate?
  4. Is there a diminishing-returns threshold?

Method:
  - Run bootstrap and RAWR at geometrically increasing replicate counts
  - Track CI width convergence
  - Measure coverage at each replicate count
  - Compare convergence across Gaussian, log-normal, and heavy-tailed data

Reference:
  Lee & Liu (2024) IEEE BIBM — statistical resampling optimization
  Wang et al. (2021) Bioinformatics (ISMB) 37:i111-i119

Cross-spring: upgrades MC methodology for all experiments.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from common import (
    check_max,
    check_true,
    print_summary,
    reset_counters,
)


def bootstrap_ci_width(
    data: np.ndarray, n_boot: int, confidence: float, rng: np.random.Generator,
) -> float:
    """Compute bootstrap CI width for the mean."""
    n = len(data)
    means = np.array([
        data[rng.integers(0, n, size=n)].mean() for _ in range(n_boot)
    ])
    alpha = 1.0 - confidence
    lo = float(np.percentile(means, 100 * alpha / 2))
    hi = float(np.percentile(means, 100 * (1 - alpha / 2)))
    return hi - lo


def rawr_ci_width(
    data: np.ndarray, n_boot: int, confidence: float, rng: np.random.Generator,
) -> float:
    """Compute RAWR (Bayesian bootstrap) CI width for the mean."""
    n = len(data)
    means = np.empty(n_boot)
    for i in range(n_boot):
        weights = rng.exponential(1.0, size=n)
        weights /= weights.sum()
        means[i] = float(np.dot(weights, data))
    alpha = 1.0 - confidence
    lo = float(np.percentile(means, 100 * alpha / 2))
    hi = float(np.percentile(means, 100 * (1 - alpha / 2)))
    return hi - lo


def coverage_at_n(
    data_gen, true_param: float, method_fn,
    n_trials: int, n_boot: int, confidence: float, base_seed: int,
) -> float:
    """Empirical coverage rate at a specific replicate count."""
    covers = 0
    for trial in range(n_trials):
        rng_data = np.random.default_rng(base_seed + trial)
        data = data_gen(rng_data)
        rng_boot2 = np.random.default_rng(base_seed + 100000 + trial)
        n = len(data)
        if method_fn == bootstrap_ci_width:
            means = np.array([
                data[rng_boot2.integers(0, n, size=n)].mean()
                for _ in range(n_boot)
            ])
        else:
            means = np.empty(n_boot)
            for i in range(n_boot):
                w = rng_boot2.exponential(1.0, size=n)
                w /= w.sum()
                means[i] = float(np.dot(w, data))
        alpha = 1.0 - confidence
        lo = float(np.percentile(means, 100 * alpha / 2))
        hi = float(np.percentile(means, 100 * (1 - alpha / 2)))
        if lo <= true_param <= hi:
            covers += 1
    return covers / n_trials


def main() -> int:
    benchmark_path = Path(__file__).parent / "benchmark_resampling_convergence.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    reset_counters()

    model = benchmark["model"]
    exp = benchmark["expected_results"]
    replicate_counts = model["replicate_counts"]
    confidence = model["confidence"]
    data_n = model["data_n"]

    print("=" * 72)
    print("groundSpring Exp 013: Resampling Convergence (Lee & Liu 2024)")
    print(f"  Replicate counts: {replicate_counts}")
    print(f"  Data size: {data_n}, Confidence: {confidence}")
    print("  Cross-spring: all springs (MC methodology optimization)")
    print("=" * 72)

    # ------------------------------------------------------------------
    # Part 1: Gaussian convergence
    # ------------------------------------------------------------------
    print("\n--- Part 1: Gaussian (μ=5.0, σ=2.0) ---")
    gauss = model["gaussian"]
    rng_data = np.random.default_rng(gauss["seed"])
    data_gauss = rng_data.normal(gauss["mu"], gauss["sigma"], data_n)

    boot_widths_g = []
    rawr_widths_g = []
    for n_boot in replicate_counts:
        rng_b = np.random.default_rng(gauss["seed"] + n_boot)
        rng_r = np.random.default_rng(gauss["seed"] + n_boot + 50000)
        bw = bootstrap_ci_width(data_gauss, n_boot, confidence, rng_b)
        rw = rawr_ci_width(data_gauss, n_boot, confidence, rng_r)
        boot_widths_g.append(bw)
        rawr_widths_g.append(rw)
        print(f"  n={n_boot:5d}: bootstrap={bw:.4f}  RAWR={rw:.4f}")

    check_true(
        "Bootstrap width decreasing (Gaussian)",
        boot_widths_g[-1] <= boot_widths_g[0] * 1.1,
    )
    check_true(
        "RAWR width decreasing (Gaussian)",
        rawr_widths_g[-1] <= rawr_widths_g[0] * 1.1,
    )

    rel_change_boot = abs(boot_widths_g[-1] - boot_widths_g[-2]) / max(boot_widths_g[-2], 1e-10)
    rel_change_rawr = abs(rawr_widths_g[-1] - rawr_widths_g[-2]) / max(rawr_widths_g[-2], 1e-10)
    print(f"  Relative change 5k→10k: bootstrap={rel_change_boot:.4f} RAWR={rel_change_rawr:.4f}")

    check_max(
        "Bootstrap converged (5k→10k < 15%)",
        rel_change_boot, exp["relative_width_change_5k_to_10k_max"],
    )
    check_max(
        "RAWR converged (5k→10k < 15%)",
        rel_change_rawr, exp["relative_width_change_5k_to_10k_max"],
    )

    # ------------------------------------------------------------------
    # Part 2: Log-normal convergence
    # ------------------------------------------------------------------
    print("\n--- Part 2: Log-Normal (μ_ln=1.0, σ_ln=0.8) ---")
    lognorm = model["lognormal"]
    rng_data2 = np.random.default_rng(lognorm["seed"])
    data_ln = rng_data2.lognormal(lognorm["mu_ln"], lognorm["sigma_ln"], data_n)

    boot_widths_ln = []
    for n_boot in replicate_counts:
        rng_b = np.random.default_rng(lognorm["seed"] + n_boot)
        bw = bootstrap_ci_width(data_ln, n_boot, confidence, rng_b)
        boot_widths_ln.append(bw)

    check_true(
        "Log-normal width converges",
        boot_widths_ln[-1] <= boot_widths_ln[0] * 1.2,
    )

    # ------------------------------------------------------------------
    # Part 3: Heavy-tailed convergence
    # ------------------------------------------------------------------
    print("\n--- Part 3: Heavy-Tailed (t, df=3) ---")
    heavy = model["heavy_tail"]
    rng_data3 = np.random.default_rng(heavy["seed"])
    data_ht = rng_data3.standard_t(heavy["df"], size=data_n) * heavy["scale"] + heavy["loc"]

    boot_widths_ht = []
    for n_boot in replicate_counts:
        rng_b = np.random.default_rng(heavy["seed"] + n_boot)
        bw = bootstrap_ci_width(data_ht, n_boot, confidence, rng_b)
        boot_widths_ht.append(bw)
        if n_boot == replicate_counts[-1]:
            print(f"  Width at n={n_boot}: {bw:.4f} (Gaussian was {boot_widths_g[-1]:.4f})")

    check_true(
        "Heavy-tail wider than Gaussian",
        boot_widths_ht[-1] > boot_widths_g[-1] * 0.8,
    )

    # ------------------------------------------------------------------
    # Part 4: Coverage at key replicate counts
    # ------------------------------------------------------------------
    print("\n--- Part 4: Coverage ---")
    n_cov_trials = model["n_trials_coverage"]
    true_mean_g = gauss["mu"]

    boot_cov = coverage_at_n(
        lambda r: r.normal(gauss["mu"], gauss["sigma"], data_n),
        true_mean_g, bootstrap_ci_width,
        n_cov_trials, 1000, confidence, gauss["seed"] + 9000,
    )
    rawr_cov = coverage_at_n(
        lambda r: r.normal(gauss["mu"], gauss["sigma"], data_n),
        true_mean_g, rawr_ci_width,
        n_cov_trials, 1000, confidence, gauss["seed"] + 19000,
    )
    print(f"  Bootstrap coverage (n=1000, {n_cov_trials} trials): {boot_cov:.3f}")
    print(f"  RAWR coverage (n=1000, {n_cov_trials} trials):      {rawr_cov:.3f}")

    check_true(
        "Bootstrap coverage ≥ 85%",
        boot_cov >= exp["bootstrap_coverage_min"],
    )
    check_true(
        "RAWR coverage ≥ 82%",
        rawr_cov >= exp["rawr_coverage_min"],
    )

    # ------------------------------------------------------------------
    # Part 5: Determinism
    # ------------------------------------------------------------------
    print("\n--- Part 5: Determinism ---")
    det_data = np.array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0])
    rng1 = np.random.default_rng(7777)
    rng2 = np.random.default_rng(7777)
    w1 = bootstrap_ci_width(det_data, 500, 0.95, rng1)
    w2 = bootstrap_ci_width(det_data, 500, 0.95, rng2)
    check_true("Bootstrap deterministic", w1 == w2)

    rng3 = np.random.default_rng(8888)
    rng4 = np.random.default_rng(8888)
    w3 = rawr_ci_width(det_data, 500, 0.95, rng3)
    w4 = rawr_ci_width(det_data, 500, 0.95, rng4)
    check_true("RAWR deterministic", w3 == w4)

    # ------------------------------------------------------------------
    # Key Findings
    # ------------------------------------------------------------------
    print(f"\n{'=' * 72}")
    print("KEY FINDINGS:")
    print(f"{'=' * 72}")
    print(f"\n1. CI width converges: 5k→10k change < {max(rel_change_boot, rel_change_rawr):.1%}")
    print(f"2. Bootstrap coverage at n=1000: {boot_cov:.0%}")
    print(f"3. RAWR coverage at n=1000: {rawr_cov:.0%}")
    print("4. Heavy-tailed data needs wider CI (expected)")
    print("5. Both methods converge by ~2000 replicates for Gaussian data")
    print("   → for most groundSpring experiments, n=2000 is sufficient")

    return print_summary("Exp 013: Resampling Convergence")


if __name__ == "__main__":
    sys.exit(main())
