#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""Exp 020: Freeze-Out Inverse Problem (Bazavov 2016 Phys Rev D 93, 014512).

Chi-squared fitting inverse problem: recover freeze-out curve parameters
(T0, kappa2) from noisy observations of T_f(mu_B) using 2D grid search.
Validates polynomial forward model, chi-squared statistic, grid-search
recovery, and noise degradation of precision.

Cross-spring: extends Exp 005 (seismic grid-search inversion) with
chi-squared fitting on polynomial models.
"""

import json
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from common import (
    check_max,
    check_min,
    check_true,
    print_summary,
    reset_counters,
)


def freeze_out_curve(t0, kappa2, mu_b):
    """T_f(mu_B) = T0 * (1 - kappa2 * (mu_B / T0)^2)."""
    return t0 * (1.0 - kappa2 * (mu_b / t0) ** 2)


def chi_squared(obs, pred, sigma):
    """Chi-squared statistic with uniform errors."""
    return np.sum(((obs - pred) / sigma) ** 2)


def grid_search_2d(obs, mu_b, sigma, t0_range, t0_step, k2_range, k2_step):
    """2D grid search minimizing chi-squared over (T0, kappa2)."""
    best_chi2 = np.inf
    best_t0 = t0_range[0]
    best_k2 = k2_range[0]

    t0_vals = np.arange(t0_range[0], t0_range[1] + t0_step / 2, t0_step)
    k2_vals = np.arange(k2_range[0], k2_range[1] + k2_step / 2, k2_step)

    for t0 in t0_vals:
        for k2 in k2_vals:
            pred = np.array([freeze_out_curve(t0, k2, m) for m in mu_b])
            c2 = chi_squared(obs, pred, sigma)
            if c2 < best_chi2:
                best_chi2 = c2
                best_t0 = t0
                best_k2 = k2

    return best_t0, best_k2, best_chi2


def main():
    reset_counters()

    bench_path = Path(__file__).parent / "benchmark_freeze_out.json"
    with open(bench_path) as f:
        benchmark = json.load(f)

    model = benchmark["model"]
    grid = benchmark["grid"]
    exp = benchmark["expected_results"]

    true_t0 = model["true_t0"]
    true_k2 = model["true_kappa2"]
    mu_b = np.array(model["mu_b_values"])
    noise_std = model["noise_std"]
    seed = model["seed"]
    n_rep = model["n_replicates"]

    print("1. Forward model correctness")
    t_at_0 = freeze_out_curve(true_t0, true_k2, 0.0)
    check_true("T_f(0) = T0", abs(t_at_0 - true_t0) < 1e-12)

    t_at_400 = freeze_out_curve(true_t0, true_k2, 400.0)
    expected_400 = true_t0 * (1.0 - true_k2 * (400.0 / true_t0) ** 2)
    check_true("T_f(400) matches formula", abs(t_at_400 - expected_400) < 1e-12)

    print("\n2. Chi-squared at truth")
    rng = np.random.default_rng(seed)
    true_curve = np.array([freeze_out_curve(true_t0, true_k2, m) for m in mu_b])
    noise = rng.normal(0, noise_std, len(mu_b))
    obs = true_curve + noise
    chi2_truth = chi_squared(obs, true_curve, noise_std)
    n_dof = len(mu_b) - 2
    chi2_per_dof = chi2_truth / n_dof
    check_max("Chi2/dof at truth reasonable", chi2_per_dof, exp["chi2_per_dof_max"])

    print("\n3. Grid search recovery (single realization)")
    fit_t0, fit_k2, _fit_chi2 = grid_search_2d(
        obs,
        mu_b,
        noise_std,
        grid["t0_range"],
        grid["t0_step"],
        grid["kappa2_range"],
        grid["kappa2_step"],
    )
    check_max("T0 recovery error", abs(fit_t0 - true_t0), exp["t0_recovery_tol"])
    check_max("kappa2 recovery error", abs(fit_k2 - true_k2), exp["kappa2_recovery_tol"])

    print("\n4. Replicate coverage")
    coverage = 0
    for i in range(n_rep):
        rng_i = np.random.default_rng(seed + i + 1)
        noise_i = rng_i.normal(0, noise_std, len(mu_b))
        obs_i = true_curve + noise_i
        t0_i, k2_i, _ = grid_search_2d(
            obs_i,
            mu_b,
            noise_std,
            grid["t0_range"],
            grid["t0_step"],
            grid["kappa2_range"],
            grid["kappa2_step"],
        )
        if abs(t0_i - true_t0) <= exp["t0_recovery_tol"] and abs(k2_i - true_k2) <= exp["kappa2_recovery_tol"]:
            coverage += 1
    frac = coverage / n_rep
    check_min("Replicate coverage", frac, exp["replicate_coverage_min"])

    print("\n5. Noise degrades precision")
    rng_low = np.random.default_rng(seed + 999)
    obs_low_noise = true_curve + rng_low.normal(0, noise_std * 0.1, len(mu_b))
    t0_low, k2_low, _ = grid_search_2d(
        obs_low_noise,
        mu_b,
        noise_std * 0.1,
        grid["t0_range"],
        grid["t0_step"],
        grid["kappa2_range"],
        grid["kappa2_step"],
    )
    err_high_noise = abs(fit_t0 - true_t0) + abs(fit_k2 - true_k2)
    err_low_noise = abs(t0_low - true_t0) + abs(k2_low - true_k2)
    check_true("Lower noise improves recovery", err_low_noise <= err_high_noise + 0.5)

    print("\n6. Determinism")
    rng_d1 = np.random.default_rng(seed)
    obs_d1 = true_curve + rng_d1.normal(0, noise_std, len(mu_b))
    rng_d2 = np.random.default_rng(seed)
    obs_d2 = true_curve + rng_d2.normal(0, noise_std, len(mu_b))
    check_true("Observations deterministic", np.array_equal(obs_d1, obs_d2))

    return print_summary("Exp 020: Freeze-Out Inverse Problem")


if __name__ == "__main__":
    sys.exit(main())
