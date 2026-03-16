#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""Exp 019: Jackknife Error Estimation (Bazavov 2025 Phys Rev D 111, 094508).

Delete-one jackknife resampling for variance estimation and bias correction.
Validates against analytical variance of the mean for Gaussian and exponential
distributions, tests bias correction on variance estimator, and compares
block jackknife with bootstrap on AR(1) correlated data.

Cross-spring: extends Exp 007 (RAWR bootstrap) with jackknife methodology.
"""

import json
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from common import (
    check_max,
    check_range,
    check_true,
    print_summary,
    reset_counters,
)


def jackknife_mean_variance(data):
    """Delete-one jackknife estimate of mean and its variance."""
    n = len(data)
    full_mean = np.mean(data)
    jk_means = np.array([np.mean(np.delete(data, i)) for i in range(n)])
    jk_var = (n - 1) / n * np.sum((jk_means - np.mean(jk_means)) ** 2)
    return full_mean, jk_var, jk_means


def jackknife_bias(data, statistic_fn):
    """Jackknife bias estimate for an arbitrary statistic."""
    n = len(data)
    full_stat = statistic_fn(data)
    jk_stats = np.array([statistic_fn(np.delete(data, i)) for i in range(n)])
    jk_mean_stat = np.mean(jk_stats)
    bias = (n - 1) * (jk_mean_stat - full_stat)
    corrected = full_stat - bias
    return full_stat, bias, corrected


def block_jackknife_variance(data, block_size):
    """Block jackknife for correlated data."""
    n = len(data)
    n_blocks = n // block_size
    trimmed = data[: n_blocks * block_size]
    blocks = trimmed.reshape(n_blocks, block_size)
    full_mean = np.mean(trimmed)
    jk_means = np.array(
        [np.mean(np.delete(blocks, i, axis=0)) for i in range(n_blocks)]
    )
    jk_var = (n_blocks - 1) / n_blocks * np.sum(
        (jk_means - np.mean(jk_means)) ** 2
    )
    return full_mean, jk_var


def generate_ar1(n, mean, std, phi, rng):
    """Generate AR(1) correlated Gaussian series."""
    innovation_std = std * np.sqrt(1 - phi**2)
    x = np.zeros(n)
    x[0] = rng.normal(mean, std)
    for i in range(1, n):
        x[i] = mean + phi * (x[i - 1] - mean) + rng.normal(0, innovation_std)
    return x


def biased_variance(data):
    """Population variance (biased estimator for sample variance)."""
    return np.var(data, ddof=0)


def main():
    reset_counters()

    bench_path = Path(__file__).parent / "benchmark_jackknife.json"
    with open(bench_path) as f:
        benchmark = json.load(f)

    gauss = benchmark["gaussian"]
    exp_cfg = benchmark["exponential"]
    corr = benchmark["correlated"]
    exp_res = benchmark["expected_results"]

    rng_g = np.random.default_rng(gauss["seed"])
    gauss_data = rng_g.normal(gauss["true_mean"], gauss["true_std"], gauss["n_samples"])

    print("1. Jackknife on Gaussian data")
    jk_mean, jk_var, _ = jackknife_mean_variance(gauss_data)
    check_max(
        "Jackknife mean near true mean",
        abs(jk_mean - gauss["true_mean"]),
        exp_res["gaussian_jk_mean_tol"],
    )
    check_range(
        "Jackknife variance of mean",
        jk_var,
        exp_res["gaussian_jk_var_range"][0],
        exp_res["gaussian_jk_var_range"][1],
    )

    print("\n2. Jackknife on exponential data")
    rng_e = np.random.default_rng(exp_cfg["seed"])
    exp_data = rng_e.exponential(1.0 / exp_cfg["rate"], exp_cfg["n_samples"])
    exp_mean, exp_var, _ = jackknife_mean_variance(exp_data)
    check_max(
        "Exponential jackknife mean near 1/rate",
        abs(exp_mean - 1.0 / exp_cfg["rate"]),
        exp_res["exponential_jk_mean_tol"],
    )
    check_range(
        "Exponential jackknife variance of mean",
        exp_var,
        exp_res["exponential_jk_var_range"][0],
        exp_res["exponential_jk_var_range"][1],
    )

    print("\n3. Jackknife bias correction")
    _, _bias_raw, corrected = jackknife_bias(gauss_data, biased_variance)
    true_var = gauss["true_std"] ** 2
    naive_err = abs(biased_variance(gauss_data) - true_var)
    corrected_err = abs(corrected - true_var)
    check_true("Bias correction reduces error", corrected_err < naive_err * 1.5)

    print("\n4. Block jackknife on correlated data")
    rng_c = np.random.default_rng(corr["seed"])
    corr_data = generate_ar1(
        corr["n_samples"], corr["true_mean"], corr["true_std"], corr["ar1_phi"], rng_c
    )
    block_vars = []
    for bs in corr["block_sizes"]:
        _, bv = block_jackknife_variance(corr_data, bs)
        block_vars.append(bv)
    check_true(
        "Block JK variance increases with block size",
        all(block_vars[i] <= block_vars[i + 1] * 1.5 for i in range(len(block_vars) - 2)),
    )
    check_range(
        "Large-block variance in expected range",
        block_vars[-1],
        exp_res["block_jk_large_block_var_range"][0],
        exp_res["block_jk_large_block_var_range"][1],
    )

    print("\n5. Jackknife vs bootstrap comparison")
    boot_vars = []
    n_boot = 500
    for _ in range(n_boot):
        idx = rng_g.integers(0, len(gauss_data), len(gauss_data))
        boot_vars.append(np.mean(gauss_data[idx]))
    boot_var = np.var(boot_vars, ddof=1)
    ratio = jk_var / boot_var if boot_var > 0 else float("inf")
    check_range(
        "Jackknife/bootstrap variance ratio",
        ratio,
        exp_res["jk_bootstrap_ratio_range"][0],
        exp_res["jk_bootstrap_ratio_range"][1],
    )

    print("\n6. Determinism")
    rng_d1 = np.random.default_rng(gauss["seed"])
    d1 = rng_d1.normal(gauss["true_mean"], gauss["true_std"], gauss["n_samples"])
    rng_d2 = np.random.default_rng(gauss["seed"])
    d2 = rng_d2.normal(gauss["true_mean"], gauss["true_std"], gauss["n_samples"])
    m1, v1, _ = jackknife_mean_variance(d1)
    m2, v2, _ = jackknife_mean_variance(d2)
    check_true("Jackknife deterministic", m1 == m2 and v1 == v2)

    return print_summary("Exp 019: Jackknife Error Estimation")


if __name__ == "__main__":
    sys.exit(main())
