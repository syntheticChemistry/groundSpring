#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring Experiment 009 — Quasiperiodic Localization (Almost-Mathieu)

Numerically verifies the Aubry-André metal-insulator transition in the
Almost-Mathieu operator to answer:
  1. Does structured (quasiperiodic) noise produce the same localization
     as random disorder?
  2. Can we predict the EXACT transition point (λ=2)?
  3. Does Herman's formula γ = ln(λ/2) hold for λ > 2?
  4. Do level statistics distinguish extended vs localized phases?

Method:
  - 1D Almost-Mathieu: H ψ(n) = ψ(n+1) + ψ(n-1) + 2λ cos(2παn + θ) ψ(n)
  - α = golden ratio (maximally irrational → no resonances)
  - Lyapunov exponent via transfer matrix (same as Exp 008)
  - Level spacing ratio for finite-size eigenvalue statistics

Reference:
  Aubry & André (1980) Ann Israel Phys Soc 3:133
  Herman (1983) Commentarii Math Helv 58:453
  Jitomirskaya & Kachkovskiy (2018) JEMS 21:777-795
  Bourgain & Kachkovskiy (2018) GAFA 29:3-43

Cross-spring with hotSpring: spectral theory, Hofstadter butterfly.
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

# ---------------------------------------------------------------------------
# Almost-Mathieu model
# ---------------------------------------------------------------------------

def almost_mathieu_potential(
    n: int, coupling: float, alpha: float, theta: float
) -> np.ndarray:
    """Generate the quasiperiodic potential V(i) = λ cos(2παi + θ).

    Standard convention: coupling λ multiplies cos directly, so the
    Aubry-André transition is at λ=2 and Herman's formula gives
    γ = ln(λ/2) for λ > 2.

    Unlike Anderson's random disorder, this potential is fully deterministic
    but incommensurate with the lattice when α is irrational.
    """
    indices = np.arange(n, dtype=np.float64)
    return np.asarray(coupling * np.cos(2.0 * np.pi * alpha * indices + theta))


def lyapunov_exponent(potential: np.ndarray, energy: float) -> float:
    """Compute Lyapunov exponent via transfer matrix method.

    Identical to Exp 008 (Anderson): T_n = [[E - V(n), -1], [1, 0]].
    """
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


def level_spacing_ratio(eigenvalues: np.ndarray) -> float:
    """Mean level spacing ratio <r> for a sorted eigenvalue sequence.

    r_n = min(δ_n, δ_{n+1}) / max(δ_n, δ_{n+1})
    <r> ≈ 0.5307 for GOE (extended), ≈ 0.3863 for Poisson (localized).
    """
    es = np.sort(eigenvalues)
    gaps = np.diff(es)
    if len(gaps) < 2:
        return 0.0
    ratios = []
    for i in range(len(gaps) - 1):
        small = min(gaps[i], gaps[i + 1])
        large = max(gaps[i], gaps[i + 1])
        if large > 0:
            ratios.append(small / large)
    return float(np.mean(ratios)) if ratios else 0.0


def build_hamiltonian(n: int, coupling: float, alpha: float, theta: float) -> np.ndarray:
    """Build the n×n Almost-Mathieu Hamiltonian matrix (dense)."""
    h = np.zeros((n, n))
    pot = almost_mathieu_potential(n, coupling, alpha, theta)
    for i in range(n):
        h[i, i] = pot[i]
        if i + 1 < n:
            h[i, i + 1] = 1.0
            h[i + 1, i] = 1.0
    return h


# ---------------------------------------------------------------------------
# Validation harness
# ---------------------------------------------------------------------------

def main() -> int:
    benchmark_path = Path(__file__).parent / "benchmark_quasiperiodic.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    reset_counters()

    model = benchmark["model"]
    pred = benchmark["analytical_predictions"]
    exp = benchmark["expected_results"]

    n_sites = model["n_sites"]
    energy = model["energy"]
    alpha = model["alpha"]
    theta = model["theta"]
    couplings = model["coupling_strengths"]
    n_eig = model["n_eigenvalues"]

    print("=" * 72)
    print("groundSpring Exp 009: Quasiperiodic Localization (Almost-Mathieu)")
    print(f"  Model: 1D Almost-Mathieu, {n_sites} sites, α = golden ratio")
    print("  Cross-spring: hotSpring (spectral theory, Hofstadter butterfly)")
    print("=" * 72)

    # ------------------------------------------------------------------
    # Part 1: Clean system (λ=0)
    # ------------------------------------------------------------------
    print("\n--- Part 1: Clean System (λ=0) ---")

    pot_clean = almost_mathieu_potential(n_sites, 0.0, alpha, theta)
    gamma_clean = lyapunov_exponent(pot_clean, energy)
    print(f"  Lyapunov exponent (λ=0): {gamma_clean:.6f}")

    check_approx(
        "Clean system γ ≈ 0",
        gamma_clean, pred["clean_lyapunov"], exp["clean_lyapunov_tol"],
    )

    # ------------------------------------------------------------------
    # Part 2: Coupling sweep — Lyapunov exponents
    # ------------------------------------------------------------------
    print("\n--- Part 2: Coupling Sweep ---")

    gammas = {}
    for lam in couplings:
        pot = almost_mathieu_potential(n_sites, lam, alpha, theta)
        g = lyapunov_exponent(pot, energy)
        gammas[lam] = g
        print(f"  λ={lam:.1f}: γ={g:.6f}")

    check_max(
        "Extended regime (λ=1) γ < threshold",
        gammas[1.0], exp["extended_regime_lyapunov_max"],
    )

    check_approx(
        "Herman's formula at λ=3: γ ≈ ln(3/2)",
        gammas[3.0], pred["herman_lambda_3"], exp["herman_tol_lambda_3"],
    )

    check_approx(
        "Herman's formula at λ=4: γ ≈ ln(2)",
        gammas[4.0], pred["herman_lambda_4"], exp["herman_tol_lambda_4"],
    )

    # ------------------------------------------------------------------
    # Part 3: Critical point (λ=2)
    # ------------------------------------------------------------------
    print("\n--- Part 3: Critical Point (λ=2) ---")

    print(f"  Lyapunov at critical coupling (λ=2): {gammas[2.0]:.6f}")
    check_approx(
        "Critical point γ ≈ 0 (Aubry-André)",
        gammas[2.0], 0.0, exp["critical_lyapunov_tol"],
    )

    # ------------------------------------------------------------------
    # Part 4: Monotonicity above critical point
    # ------------------------------------------------------------------
    print("\n--- Part 4: Monotonicity ---")

    above_critical = [lam for lam in couplings if lam >= 2.0]
    above_gammas = [gammas[lam] for lam in above_critical]

    check_true(
        "γ monotonically increasing for λ ≥ 2",
        all(
            above_gammas[i] <= above_gammas[i + 1]
            for i in range(len(above_gammas) - 1)
        ),
    )

    # ------------------------------------------------------------------
    # Part 5: Level spacing ratio (finite-size eigenvalue statistics)
    # ------------------------------------------------------------------
    # The Almost-Mathieu model is deterministic (not a random ensemble),
    # so we average over many θ values to recover ensemble statistics.
    # We also restrict to the bulk (middle 50%) of eigenvalues to avoid
    # edge effects.
    print("\n--- Part 5: Level Spacing Statistics ---")

    n_theta = 50
    thetas = np.linspace(0, 2 * np.pi, n_theta, endpoint=False)

    ratios_ext = []
    ratios_loc = []
    for th in thetas:
        h_ext = build_hamiltonian(n_eig, 1.0, alpha, th)
        eigs_ext = np.sort(np.linalg.eigvalsh(h_ext))
        n_bulk = len(eigs_ext)
        lo, hi = n_bulk // 4, 3 * n_bulk // 4
        bulk = eigs_ext[lo:hi]
        gaps = np.diff(bulk)
        for j in range(len(gaps) - 1):
            s, b = min(gaps[j], gaps[j + 1]), max(gaps[j], gaps[j + 1])
            if b > 0:
                ratios_ext.append(s / b)

        h_loc = build_hamiltonian(n_eig, 4.0, alpha, th)
        eigs_loc = np.sort(np.linalg.eigvalsh(h_loc))
        bulk_loc = eigs_loc[lo:hi]
        gaps_loc = np.diff(bulk_loc)
        for j in range(len(gaps_loc) - 1):
            s, b = min(gaps_loc[j], gaps_loc[j + 1]), max(gaps_loc[j], gaps_loc[j + 1])
            if b > 0:
                ratios_loc.append(s / b)

    r_ext = float(np.mean(ratios_ext)) if ratios_ext else 0.0
    r_loc = float(np.mean(ratios_loc)) if ratios_loc else 0.0

    print(f"  λ=1 (extended): <r> = {r_ext:.4f}  (GOE ≈ {pred['goe_r']})")
    print(f"  λ=4 (localized): <r> = {r_loc:.4f}  (Poisson ≈ {pred['poisson_r']})")

    check_range(
        "Level spacing λ=1 ~ GOE",
        r_ext,
        exp["level_spacing_extended_range"][0],
        exp["level_spacing_extended_range"][1],
    )

    check_range(
        "Level spacing λ=4 ~ Poisson",
        r_loc,
        exp["level_spacing_localized_range"][0],
        exp["level_spacing_localized_range"][1],
    )

    # ------------------------------------------------------------------
    # Key Findings
    # ------------------------------------------------------------------
    print(f"\n{'=' * 72}")
    print("KEY FINDINGS:")
    print(f"{'=' * 72}")
    print(f"\n1. Clean system (λ=0): γ = {gamma_clean:.6f} ≈ 0 (free particle)")
    print(f"2. Extended regime (λ=1): γ = {gammas[1.0]:.6f} ≈ 0 (delocalized)")
    print(f"3. Critical point (λ=2): γ = {gammas[2.0]:.6f} ≈ 0 (Aubry-André)")
    print(f"4. Localized (λ=3): γ = {gammas[3.0]:.6f} ≈ ln(3/2) = 0.4055")
    print(f"5. Localized (λ=4): γ = {gammas[4.0]:.6f} ≈ ln(2) = 0.6931")
    print("6. Structured noise (quasiperiodic) shows a SHARP transition vs")
    print("   Anderson's gradual onset — the key groundSpring insight for")
    print("   periodic environmental signals (tides, seasons, diurnal cycles)")

    return print_summary("Exp 009: Quasiperiodic Localization")


if __name__ == "__main__":
    sys.exit(main())
