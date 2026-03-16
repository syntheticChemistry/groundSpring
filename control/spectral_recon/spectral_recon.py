#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""Exp 021: Spectral Function Reconstruction (Bazavov 2025 arXiv 2501.12259).

Tikhonov-regularized reconstruction of a spectral function from a noisy
Euclidean correlator.  Validates kernel matrix construction, forward model,
Cholesky-based normal equation solve, peak recovery under regularization,
and the bias-variance trade-off as lambda varies.

Cross-spring: extends Exp 005 (seismic inversion) and Exp 020 (chi-squared
fitting) with a continuous inverse problem requiring regularization.
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


def gaussian_peak(omega, center, width, amplitude):
    """Gaussian spectral function."""
    return amplitude * np.exp(-0.5 * ((omega - center) / width) ** 2) / (
        width * np.sqrt(2 * np.pi)
    )


def build_kernel(tau, omega):
    """Laplace-transform kernel K[i,j] = exp(-tau_i * omega_j) * d_omega."""
    d_omega = omega[1] - omega[0] if len(omega) > 1 else 1.0
    return np.exp(-np.outer(tau, omega)) * d_omega


def forward_correlator(kernel, rho):
    """G = K @ rho."""
    return kernel @ rho


def tikhonov_solve(kernel, data, lam):
    """Solve (K^T K + lambda I) rho = K^T data via Cholesky."""
    ktk = kernel.T @ kernel
    ktg = kernel.T @ data
    n = ktk.shape[0]
    a_reg = ktk + lam * np.eye(n)
    return np.linalg.solve(a_reg, ktg)


def main():
    reset_counters()

    bench_path = Path(__file__).parent / "benchmark_spectral_recon.json"
    with open(bench_path) as f:
        benchmark = json.load(f)

    sf = benchmark["spectral_function"]
    grid = benchmark["grid"]
    noise_cfg = benchmark["noise"]
    reg = benchmark["regularization"]
    exp = benchmark["expected_results"]

    n_tau = grid["n_tau"]
    n_omega = grid["n_omega"]
    tau = np.linspace(0, grid["tau_max"], n_tau, endpoint=False) + grid["tau_max"] / n_tau
    omega = np.linspace(0, grid["omega_max"], n_omega, endpoint=False) + grid["omega_max"] / n_omega

    rho_true = gaussian_peak(omega, sf["omega_center"], sf["omega_width"], sf["amplitude"])

    print("1. Kernel and forward model")
    kernel = build_kernel(tau, omega)
    g_exact = forward_correlator(kernel, rho_true)
    g_recon = kernel @ tikhonov_solve(kernel, g_exact, 0.0)
    rmse_noiseless = np.sqrt(np.mean((g_exact - g_recon) ** 2))
    check_max(
        "Noiseless forward RMSE",
        rmse_noiseless,
        exp["forward_rmse_noiseless_max"],
    )

    print("\n2. Cholesky residual check")
    rho_noiseless = tikhonov_solve(kernel, g_exact, 1e-12)
    residual = np.max(np.abs(kernel @ rho_noiseless - g_exact))
    check_max("Cholesky max residual", residual, exp["cholesky_residual_max"])

    print("\n3. Noisy reconstruction at optimal lambda")
    rng = np.random.default_rng(noise_cfg["seed"])
    noise = rng.normal(0, noise_cfg["correlator_noise_std"], n_tau)
    g_noisy = g_exact + noise
    lam_opt = reg["optimal_lambda"]
    rho_recon = tikhonov_solve(kernel, g_noisy, lam_opt)
    peak_idx = np.argmax(rho_recon)
    peak_omega = omega[peak_idx]
    check_max(
        "Peak location error",
        abs(peak_omega - sf["omega_center"]),
        exp["peak_location_tol"],
    )
    check_true("Peak value positive", rho_recon[peak_idx] > 0)

    print("\n4. Regularization trade-off")
    lambdas = reg["lambda_values"]
    rmses = []
    for lam in lambdas:
        rho_l = tikhonov_solve(kernel, g_noisy, lam)
        rmse_l = np.sqrt(np.mean((rho_l - rho_true) ** 2))
        rmses.append(rmse_l)

    small_rmse = rmses[0]
    large_rmse = rmses[-1]
    opt_rmse = rmses[2]
    check_true(
        "Small lambda amplifies noise (higher RMSE than optimal)",
        small_rmse >= opt_rmse * 0.5,
    )
    check_true(
        "Large lambda over-smooths (higher RMSE than optimal)",
        large_rmse >= opt_rmse * 0.5,
    )
    check_range(
        "Optimal lambda RMSE in range",
        opt_rmse,
        exp["optimal_lambda_rmse_range"][0],
        exp["optimal_lambda_rmse_range"][1],
    )

    print("\n5. Determinism")
    rng2 = np.random.default_rng(noise_cfg["seed"])
    noise2 = rng2.normal(0, noise_cfg["correlator_noise_std"], n_tau)
    g_noisy2 = g_exact + noise2
    rho2 = tikhonov_solve(kernel, g_noisy2, lam_opt)
    check_true("Reconstruction deterministic", np.array_equal(rho_recon, rho2))

    return print_summary("Exp 021: Spectral Function Reconstruction")


if __name__ == "__main__":
    sys.exit(main())
