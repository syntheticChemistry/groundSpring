#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring Experiment 012 — Spin Chain Transport

Wavepacket dynamics in the 1D Almost-Mathieu (quasiperiodic) tight-binding
model to answer:
  1. Does a wavepacket injected at one site spread ballistically (β≈1)
     in the extended phase (λ<2)?
  2. Does the wavepacket remain localized (β≈0) in the localized phase (λ>2)?
  3. What happens at the critical point (λ=2)?
  4. Does the Lyapunov exponent correctly predict the transport regime?

Method:
  - Build tridiagonal Hamiltonian: H_{ij} = δ_{i,j±1} + λ cos(2παi+θ) δ_{ij}
  - Eigendecompose: H = U Λ U^T
  - Time-evolve: ψ_j(t) = Σ_k U_{j,k} U_{n₀,k} exp(-i E_k t)
  - Compute MSD: σ²(t) = Σ_j (j - n₀)² |ψ_j(t)|²
  - Extract transport exponent: fit log σ(t) = β log(t) + const

Reference:
  Kachkovskiy (2016) Comm Math Phys 345:659-673
  Jitomirskaya & Kachkovskiy (2018) JEMS 21:777-795

Cross-spring: extends Exp 009 (quasiperiodic localization) with dynamics.
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from common import (
    check_approx,
    check_max,
    check_range,
    check_true,
    print_summary,
    reset_counters,
)


def build_hamiltonian(
    n: int, coupling: float, alpha: float, theta: float,
) -> np.ndarray:
    """Build the n×n Almost-Mathieu Hamiltonian (tridiagonal)."""
    h = np.zeros((n, n))
    for i in range(n):
        h[i, i] = coupling * math.cos(2.0 * math.pi * alpha * i + theta)
        if i + 1 < n:
            h[i, i + 1] = 1.0
            h[i + 1, i] = 1.0
    return h


def wavepacket_msd(
    eigenvalues: np.ndarray,
    eigenvectors: np.ndarray,
    init_site: int,
    time: float,
) -> tuple[float, float]:
    """Compute MSD and normalization for a wavepacket at a given time.

    Returns (msd, normalization) where normalization should be ≈ 1.0.
    """
    n = len(eigenvalues)
    coeffs = eigenvectors[init_site, :]

    phases_cos = np.cos(eigenvalues * time)
    phases_sin = np.sin(eigenvalues * time)

    psi_real = eigenvectors @ (coeffs * phases_cos)
    psi_imag = -eigenvectors @ (coeffs * phases_sin)

    prob = psi_real**2 + psi_imag**2
    normalization = prob.sum()

    positions = np.arange(n, dtype=np.float64) - init_site
    msd = (positions**2 * prob).sum()

    return float(msd), float(normalization)


def transport_exponent(times: np.ndarray, msds: np.ndarray) -> float:
    """Fit transport exponent β from log(σ) = β log(t) + const.

    Uses log(√MSD) = β log(t) since σ = √MSD and MSD ~ t^{2β}.
    Only uses points where MSD > 0.
    """
    mask = msds > 1e-20
    if mask.sum() < 2:
        return 0.0

    log_t = np.log(times[mask])
    log_sigma = 0.5 * np.log(msds[mask])

    n = len(log_t)
    sx = log_t.sum()
    sy = log_sigma.sum()
    sxx = (log_t**2).sum()
    sxy = (log_t * log_sigma).sum()

    denom = n * sxx - sx * sx
    if abs(denom) < 1e-30:
        return 0.0

    return float((n * sxy - sx * sy) / denom)


def lyapunov_exponent(potential: np.ndarray, energy: float) -> float:
    """Compute Lyapunov exponent via transfer matrix method (from Exp 009)."""
    n = len(potential)
    if n == 0:
        return 0.0

    log_growth = 0.0
    vec = np.array([1.0, 0.0])

    for i in range(n):
        new_0 = (energy - potential[i]) * vec[0] - vec[1]
        new_1 = vec[0]
        vec[0] = new_0
        vec[1] = new_1

        norm = math.sqrt(vec[0] ** 2 + vec[1] ** 2)
        if norm > 0:
            log_growth += math.log(norm)
            vec[0] /= norm
            vec[1] /= norm

    return log_growth / n


def main() -> int:
    benchmark_path = Path(__file__).parent / "benchmark_spin_transport.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    reset_counters()

    model = benchmark["model"]
    exp = benchmark["expected_results"]

    n_sites = model["n_sites"]
    alpha = model["alpha"]
    theta = model["theta"]
    init_site = model["init_site"]
    couplings = model["coupling_strengths"]
    times = np.array(model["times"])
    lyap_n = model["lyapunov_n_sites"]
    lyap_e = model["lyapunov_energy"]

    print("=" * 72)
    print("groundSpring Exp 012: Spin Chain Transport (Kachkovskiy 2016)")
    print(f"  Model: 1D Almost-Mathieu, {n_sites} sites, α = golden ratio")
    print(f"  Wavepacket: δ_{{{init_site}}}, times: {list(times)}")
    print("  Cross-spring: hotSpring (spectral), wetSpring (porous transport)")
    print("=" * 72)

    betas = {}
    final_msds = {}

    for lam in couplings:
        print(f"\n--- Coupling λ = {lam:.1f} ---")

        h = build_hamiltonian(n_sites, lam, alpha, theta)
        eigenvalues, eigenvectors = np.linalg.eigh(h)

        msds_at_t = []
        for t in times:
            msd, norm = wavepacket_msd(eigenvalues, eigenvectors, init_site, t)
            msds_at_t.append(msd)
            if t == times[0]:
                check_approx(
                    f"Normalization λ={lam:.1f} t={t:.0f}",
                    norm, 1.0, exp["normalization_tolerance"],
                )
            if t == times[-1]:
                norm_check = norm
                check_approx(
                    f"Normalization λ={lam:.1f} t={t:.0f}",
                    norm_check, 1.0, exp["normalization_tolerance"],
                )

        msds_arr = np.array(msds_at_t)
        beta = transport_exponent(times, msds_arr)
        betas[lam] = beta
        final_msds[lam] = msds_at_t[-1]

        sigma_final = math.sqrt(msds_at_t[-1]) if msds_at_t[-1] > 0 else 0
        print(f"  MSD(t={times[-1]:.0f}) = {msds_at_t[-1]:.4f}, σ = {sigma_final:.4f}")
        print(f"  Transport exponent β = {beta:.4f}")

    # ------------------------------------------------------------------
    # Part 1: Ballistic transport (λ < 2)
    # ------------------------------------------------------------------
    print("\n--- Validation: Transport Exponents ---")

    check_range(
        "Ballistic transport β (λ=0.5)",
        betas[0.5],
        exp["ballistic_beta_range"][0],
        exp["ballistic_beta_range"][1],
    )

    check_range(
        "Ballistic transport β (λ=1.0)",
        betas[1.0],
        exp["ballistic_beta_range"][0],
        exp["ballistic_beta_range"][1],
    )

    # ------------------------------------------------------------------
    # Part 2: Localized transport (λ > 2)
    # ------------------------------------------------------------------
    check_max(
        "Localized transport β (λ=4.0)",
        betas[4.0],
        exp["localized_beta_max"],
    )

    check_max(
        "Localized MSD bounded (λ=4.0)",
        final_msds[4.0],
        exp["msd_localized_bounded_max"],
    )

    # ------------------------------------------------------------------
    # Part 3: Critical point (λ = 2)
    # ------------------------------------------------------------------
    check_range(
        "Critical transport β (λ=2.0)",
        betas[2.0],
        exp["critical_beta_range"][0],
        exp["critical_beta_range"][1],
    )

    # ------------------------------------------------------------------
    # Part 4: Lyapunov exponent cross-check
    # ------------------------------------------------------------------
    print("\n--- Lyapunov Cross-Check ---")

    pot_ext = np.array([
        1.0 * math.cos(2.0 * math.pi * alpha * i + theta)
        for i in range(lyap_n)
    ])
    gamma_ext = lyapunov_exponent(pot_ext, lyap_e)
    print(f"  Lyapunov γ (λ=1.0): {gamma_ext:.6f}")

    check_max("Lyapunov extended (λ=1.0) γ ≈ 0", gamma_ext, exp["lyapunov_extended_max"])

    pot_loc = np.array([
        4.0 * math.cos(2.0 * math.pi * alpha * i + theta)
        for i in range(lyap_n)
    ])
    gamma_loc = lyapunov_exponent(pot_loc, lyap_e)
    print(f"  Lyapunov γ (λ=4.0): {gamma_loc:.6f}")

    check_true(
        "Lyapunov localized (λ=4.0) γ > threshold",
        gamma_loc > exp["lyapunov_localized_min"],
    )

    # ------------------------------------------------------------------
    # Part 5: Transport monotonicity
    # ------------------------------------------------------------------
    print("\n--- Monotonicity ---")

    check_true(
        "β decreases with increasing λ",
        all(
            betas[couplings[i]] >= betas[couplings[i + 1]] - 0.15
            for i in range(len(couplings) - 1)
        ),
    )

    # ------------------------------------------------------------------
    # Key Findings
    # ------------------------------------------------------------------
    print(f"\n{'=' * 72}")
    print("KEY FINDINGS:")
    print(f"{'=' * 72}")
    for lam in couplings:
        regime = "BALLISTIC" if betas[lam] > 0.5 else "LOCALIZED" if betas[lam] < 0.15 else "ANOMALOUS"
        print(f"  λ={lam:.1f}: β={betas[lam]:.4f} → {regime}")
    print("\n  Signal injected at one site: in the extended phase (λ<2), energy")
    print("  propagates ballistically (β≈1). In the localized phase (λ>2),")
    print("  the wavepacket stays trapped — noise wins over signal.")
    print("  This is the spectral theory foundation of groundSpring's central")
    print("  question: when does signal propagate through a noisy medium?")

    return print_summary("Exp 012: Spin Chain Transport")


if __name__ == "__main__":
    sys.exit(main())
