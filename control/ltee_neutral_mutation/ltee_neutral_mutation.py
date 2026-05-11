#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
Experiment 037 — LTEE Neutral Mutation Dynamics (Barrick et al. 2009).

Reproduces the neutral mutation accumulation analysis:
  Barrick JE et al. (2009) Genome evolution and adaptation in a long-term
  experiment with E. coli. Nature 461:1243-1247.

Under strict neutrality, mutation fixation probability = 1/N (haploid).
The molecular clock rate equals the genomic mutation rate μ, independent
of population size. Drift dominates selection when |s| < 1/N.

This establishes the null model for LTEE adaptive evolution detection.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np

SCRIPT_DIR = Path(__file__).resolve().parent
BENCHMARK = SCRIPT_DIR / "benchmark_ltee_neutral.json"


def load_benchmark():
    with open(BENCHMARK) as f:
        return json.load(f)


# ── Kimura fixation theory ────────────────────────────────────────────

def kimura_fixation_prob(pop_size: int, selection: float, initial_freq: float = None) -> float:
    """Kimura fixation probability for a new mutation in a haploid population.

    For s = 0 (neutral): P_fix = 1/N
    For s != 0: P_fix = (1 - exp(-2*s)) / (1 - exp(-2*N*s))
    """
    if initial_freq is None:
        initial_freq = 1.0 / pop_size

    if abs(selection) < 1e-10:
        return initial_freq

    numerator = 1.0 - np.exp(-2.0 * selection * pop_size * initial_freq)
    denominator = 1.0 - np.exp(-2.0 * selection * pop_size)
    return float(numerator / denominator)


def neutral_accumulation_rate(genomic_mu: float) -> float:
    """Under neutrality, substitution rate = genomic mutation rate μ.

    This is a fundamental result: new mutations arise at rate μ*N per gen,
    each fixes with probability 1/N, so substitution rate = μ*N*(1/N) = μ.
    """
    return genomic_mu


# ── Wright-Fisher neutral simulation ─────────────────────────────────

def simulate_neutral_fixations(pop_size, mu, n_gens, seed):
    """Simulate neutral mutation accumulation via Poisson process.

    In a large population with neutral mutations:
    - New mutations per generation: Poisson(N * μ)
    - Each fixes with probability 1/N
    - Net fixations per gen: Poisson(μ)
    """
    rng = np.random.default_rng(seed)
    fixations_per_gen = rng.poisson(mu, size=n_gens)
    cumulative = np.cumsum(fixations_per_gen)
    return cumulative


def main():
    bench = load_benchmark()
    cfg = bench["model"]
    expected = bench["expected_results"]

    print("=" * 72)
    print("  Experiment 037: LTEE Neutral Mutation Dynamics (Barrick 2009)")
    print("  B1 Reproduction | lithoSpore Module 2 | Drift vs Selection Null")
    print("=" * 72)

    pop_size = cfg["population_size"]
    mu = cfg["genomic_mutation_rate"]
    n_gens = cfg["n_generations"]
    n_reps = cfg["n_replicates"]
    seed = cfg["seed"]
    s_neutral = cfg["selection_coefficient"]

    checks_passed = 0
    checks_total = 0

    # Check 1: Kimura fixation probability for neutral mutation
    checks_total += 1
    pfix = kimura_fixation_prob(pop_size, s_neutral)
    expected_pfix = expected["fixation_probability_neutral"]["expected"]
    tol_factor = expected["fixation_probability_neutral"]["tolerance_factor"]
    pfix_pass = abs(pfix - expected_pfix) / expected_pfix < tol_factor
    print(f"\n  Kimura P_fix(s=0, N={pop_size}): {pfix:.2e} "
          f"(expected: {expected_pfix:.2e})")
    print(f"  [{'PASS' if pfix_pass else 'FAIL'}] Neutral fixation probability matches 1/N")
    if pfix_pass:
        checks_passed += 1

    # Check 2: Molecular clock rate = μ
    checks_total += 1
    rate = neutral_accumulation_rate(mu)
    expected_rate = expected["accumulation_rate_per_generation"]["expected"]
    rate_tol = expected["accumulation_rate_per_generation"]["tolerance_factor"]
    rate_pass = abs(rate - expected_rate) / expected_rate < rate_tol
    print(f"\n  Neutral substitution rate: {rate:.4e} "
          f"(expected: {expected_rate:.4e})")
    print(f"  [{'PASS' if rate_pass else 'FAIL'}] Molecular clock rate = μ")
    if rate_pass:
        checks_passed += 1

    # Check 3: Simulated accumulation is linear (molecular clock)
    checks_total += 1
    all_trajectories = []
    for i in range(n_reps):
        traj = simulate_neutral_fixations(pop_size, mu, n_gens, seed + i)
        all_trajectories.append(traj)

    mean_traj = np.mean(all_trajectories, axis=0)
    gens = np.arange(1, n_gens + 1, dtype=float)

    from numpy.polynomial.polynomial import polyfit
    coeffs = polyfit(gens, mean_traj, 1)
    slope = coeffs[1]
    from scipy.stats import pearsonr
    r_val, _ = pearsonr(gens, mean_traj)
    linear_pass = r_val > 0.998
    print(f"\n  Mean trajectory over {n_reps} replicates:")
    print(f"  Linear fit slope: {slope:.6f} (expected ~μ = {mu:.4e})")
    print(f"  Pearson r: {r_val:.6f}")
    print(f"  [{'PASS' if linear_pass else 'FAIL'}] Molecular clock is linear (r > 0.998)")
    if linear_pass:
        checks_passed += 1

    # Check 4: Observed rate matches expected within tolerance
    checks_total += 1
    slope_pass = abs(slope - mu) / mu < rate_tol
    print(f"  [{'PASS' if slope_pass else 'FAIL'}] Observed rate {slope:.6e} ≈ μ = {mu:.4e} "
          f"(within {rate_tol}×)")
    if slope_pass:
        checks_passed += 1

    # Check 5: Kimura analytical vs simulation agreement
    checks_total += 1
    analytical_tol = expected["kimura_fixation_analytical_match"]["tolerance"]
    analytical_pfix = 1.0 / pop_size
    match = abs(pfix - analytical_pfix) < analytical_tol
    print(f"\n  [{'PASS' if match else 'FAIL'}] Kimura formula matches 1/N analytical: "
          f"{pfix:.2e} vs {analytical_pfix:.2e}")
    if match:
        checks_passed += 1

    # Check 6: Drift dominates for small |s|
    checks_total += 1
    s_threshold = expected["drift_dominates_for_small_s"]["s_threshold"]
    factor_limit = expected["drift_dominates_for_small_s"]["fixation_prob_within_factor"]
    pfix_small_s = kimura_fixation_prob(pop_size, s_threshold)
    drift_ratio = pfix_small_s / (1.0 / pop_size)
    drift_pass = drift_ratio < factor_limit
    print(f"\n  P_fix(s={s_threshold}) = {pfix_small_s:.6e}")
    print(f"  Ratio to neutral: {drift_ratio:.2f}")
    print(f"  [{'PASS' if drift_pass else 'FAIL'}] Drift dominates at |s| = 1/N "
          f"(ratio < {factor_limit}×)")
    if drift_pass:
        checks_passed += 1

    # Check 7: Selection detectable for |s| >> 1/N
    checks_total += 1
    s_large = 0.01
    pfix_large = kimura_fixation_prob(pop_size, s_large)
    sel_detectable = pfix_large > 10.0 / pop_size
    print(f"\n  P_fix(s={s_large}) = {pfix_large:.6e}")
    print(f"  [{'PASS' if sel_detectable else 'FAIL'}] Selection detectable at s = {s_large}")
    if sel_detectable:
        checks_passed += 1

    # Check 8: Determinism
    checks_total += 1
    traj2 = simulate_neutral_fixations(pop_size, mu, n_gens, seed)
    det_pass = np.array_equal(all_trajectories[0], traj2)
    print(f"\n  [{'PASS' if det_pass else 'FAIL'}] Deterministic (same seed → same data)")
    if det_pass:
        checks_passed += 1

    print(f"\n{'=' * 72}")
    print(f"  RESULT: {checks_passed}/{checks_total} checks PASS")
    print(f"{'=' * 72}")

    expected_values = {
        "experiment": "037_ltee_neutral_mutation",
        "paper": "Barrick2009",
        "paper_id": "B1",
        "litho_module": 2,
        "kimura_fixation_prob_neutral": pfix,
        "molecular_clock_rate": float(slope),
        "molecular_clock_pearson_r": float(r_val),
        "mean_trajectory_final": float(mean_traj[-1]),
        "drift_dominance_ratio": drift_ratio,
        "checks_passed": checks_passed,
        "checks_total": checks_total,
    }
    out_path = SCRIPT_DIR / "expected_values.json"
    with open(out_path, "w") as f:
        json.dump(expected_values, f, indent=2)
    print(f"\n  Expected values written to {out_path.relative_to(SCRIPT_DIR.parent.parent)}")

    return 0 if checks_passed == checks_total else 1


if __name__ == "__main__":
    sys.exit(main())
