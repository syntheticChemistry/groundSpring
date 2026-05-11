#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
Experiment 038 — LTEE Clonal Interference (Good et al. 2017).

Reproduces the clonal interference analysis from:
  Good BH, McDonald MJ, Barrick JE, Lenski RE, Desai MM (2017)
  The dynamics of molecular evolution over 60,000 generations.
  Nature 551:45-50.

Key question: In large asexual populations, multiple beneficial mutations
arise simultaneously and compete. How does this competition (clonal
interference) reduce the effective fixation probability relative to the
single-mutation Haldane sieve prediction (p_fix ≈ 2s)?

Theory reference: Desai & Fisher (2007) Genetics 176:1759-1798.

This is B3 in the LTEE GuideStone Queue → lithoSpore module 3.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np

SCRIPT_DIR = Path(__file__).resolve().parent
BENCHMARK = SCRIPT_DIR / "benchmark_ltee_clonal.json"


def load_benchmark():
    with open(BENCHMARK) as f:
        return json.load(f)


# ── Simulation ────────────────────────────────────────────────────────

def simulate_clonal_interference(pop_size, n_gens, u_b, mean_s, rng):
    """
    Simplified Wright-Fisher with clonal interference.

    Track beneficial mutation lineages in a haploid asexual population.
    Each generation:
      1. New beneficial mutations arise (Poisson with rate N * u_b)
      2. Each lineage changes frequency via selection + drift
      3. Lineages reaching freq >= 1.0 are "fixed"; freq <= 0 are lost

    Returns (fixation_count, total_mutations, mean_fitness_trajectory).
    """
    n = pop_size
    lineages = []  # (frequency, selective_advantage)
    fixation_count = 0
    total_mutations = 0
    fitness_trajectory = np.ones(n_gens, dtype=np.float64)
    background_fitness = 1.0

    for gen in range(n_gens):
        # New beneficial mutations this generation
        n_new = rng.poisson(n * u_b)
        for _ in range(n_new):
            s_i = rng.exponential(mean_s)
            lineages.append([1.0 / n, s_i])  # start at frequency 1/N
            total_mutations += 1

        # Evolve existing lineages
        surviving = []
        for lin in lineages:
            freq, s_i = lin
            # Selection: expected frequency change
            mean_freq = freq * (1.0 + s_i) / (1.0 + freq * s_i)
            # Drift: binomial sampling
            n_copies = rng.binomial(n, min(mean_freq, 1.0))
            new_freq = n_copies / n

            if new_freq >= 1.0:
                fixation_count += 1
                background_fitness *= (1.0 + s_i)
            elif new_freq > 0:
                lin[0] = new_freq
                lin[1] = s_i
                surviving.append(lin)
            # else: lost (freq == 0)

        lineages = surviving
        fitness_trajectory[gen] = background_fitness

    return fixation_count, total_mutations, fitness_trajectory


def run_replicates(pop_size, n_gens, u_b, mean_s, n_reps, rng):
    """Run multiple replicates and collect statistics."""
    fix_counts = []
    mut_counts = []
    final_fitnesses = []
    all_trajectories = []

    for _ in range(n_reps):
        fixes, muts, trajectory = simulate_clonal_interference(
            pop_size, n_gens, u_b, mean_s, rng
        )
        fix_counts.append(fixes)
        mut_counts.append(muts)
        final_fitnesses.append(trajectory[-1])
        all_trajectories.append(trajectory)

    total_fixes = sum(fix_counts)
    total_muts = sum(mut_counts)
    fix_prob = total_fixes / total_muts if total_muts > 0 else 0.0
    neutral_prob = 1.0 / pop_size
    haldane_prob = 2.0 * mean_s

    mean_trajectory = np.mean(all_trajectories, axis=0)
    log_fitnesses = [np.log(f) for f in final_fitnesses]
    adaptation_rate = np.mean(log_fitnesses) / n_gens

    return {
        "pop_size": pop_size,
        "total_fixations": total_fixes,
        "total_mutations": total_muts,
        "fixation_probability": fix_prob,
        "neutral_probability": neutral_prob,
        "haldane_probability": haldane_prob,
        "interference_ratio": fix_prob / haldane_prob if haldane_prob > 0 else 0.0,
        "mean_final_fitness": float(np.mean(final_fitnesses)),
        "std_final_fitness": float(np.std(final_fitnesses)),
        "adaptation_rate": adaptation_rate,
        "mean_trajectory": mean_trajectory.tolist(),
    }


# ── Main ──────────────────────────────────────────────────────────────

def main():
    bench = load_benchmark()
    cfg = bench["model"]
    expected = bench["expected_results"]

    print("=" * 72)
    print("  Experiment 038: LTEE Clonal Interference — Good et al. (2017)")
    print("  B3 Reproduction | lithoSpore Module 3 | Clonal Dynamics")
    print("=" * 72)

    pop_sizes = cfg["pop_sizes"]
    n_gens = cfg["n_generations"]
    u_b = cfg["beneficial_mutation_rate"]
    mean_s = cfg["mean_selective_advantage"]
    n_reps = cfg["n_replicates"]
    seed = cfg["seed"]
    rng = np.random.default_rng(seed)

    print(f"\n  Population sizes: {pop_sizes}")
    print(f"  Generations: {n_gens}")
    print(f"  U_b: {u_b}, mean s: {mean_s}")
    print(f"  Replicates per size: {n_reps}")

    results_by_size = {}
    for n in pop_sizes:
        print(f"\n  Running N={n:,} ...")
        res = run_replicates(n, n_gens, u_b, mean_s, n_reps, rng)
        results_by_size[str(n)] = res
        print(f"    Fixations: {res['total_fixations']}/{res['total_mutations']} "
              f"(p_fix={res['fixation_probability']:.6f})")
        print(f"    Neutral: 1/N={res['neutral_probability']:.6f}, "
              f"Haldane: 2s={res['haldane_probability']:.4f}")
        print(f"    Interference ratio: {res['interference_ratio']:.4f}")
        print(f"    Mean final fitness: {res['mean_final_fitness']:.4f} "
              f"± {res['std_final_fitness']:.4f}")
        print(f"    Adaptation rate: {res['adaptation_rate']:.2e}")

    checks_passed = 0
    checks_total = 0

    # Check 1: fixation probability decreases with N
    checks_total += 1
    fix_probs = [results_by_size[str(n)]["fixation_probability"] for n in pop_sizes]
    decreasing = all(fix_probs[i] >= fix_probs[i + 1] for i in range(len(fix_probs) - 1))
    status = "PASS" if decreasing else "FAIL"
    print(f"\n  [{status}] Fixation probability decreases with N")
    print(f"    p_fix = {[f'{p:.6f}' for p in fix_probs]}")
    if decreasing:
        checks_passed += 1

    # Check 2: fixation prob > neutral (1/N) for all sizes
    checks_total += 1
    above_neutral = all(
        results_by_size[str(n)]["fixation_probability"] >
        results_by_size[str(n)]["neutral_probability"]
        for n in pop_sizes
    )
    status = "PASS" if above_neutral else "FAIL"
    print(f"  [{status}] Fixation probability > neutral (1/N) for all N")
    if above_neutral:
        checks_passed += 1

    # Check 3: clonal interference ratio between N=1000 and N=100
    checks_total += 1
    r100 = results_by_size["100"]["fixation_probability"]
    r1000 = results_by_size["1000"]["fixation_probability"]
    ci_ratio = r1000 / r100 if r100 > 0 else 0.0
    lo, hi = expected["clonal_interference_ratio_N1000_vs_N100"]
    ci_pass = lo <= ci_ratio <= hi
    status = "PASS" if ci_pass else "FAIL"
    print(f"  [{status}] CI ratio (N=1000/N=100) = {ci_ratio:.4f} "
          f"(expected: [{lo}, {hi}])")
    if ci_pass:
        checks_passed += 1

    # Check 4: mean fitness increases for all sizes
    checks_total += 1
    all_increase = all(
        results_by_size[str(n)]["mean_final_fitness"] > 1.0
        for n in pop_sizes
    )
    status = "PASS" if all_increase else "FAIL"
    print(f"  [{status}] Mean fitness increases for all population sizes")
    if all_increase:
        checks_passed += 1

    # Check 5: adaptation rate scales sublinearly with N (within interference regime)
    checks_total += 1
    rate_10000 = results_by_size["10000"]["adaptation_rate"]
    rate_100000 = results_by_size["100000"]["adaptation_rate"]
    rate_ratio = rate_100000 / rate_10000 if rate_10000 > 0 else 0.0
    lo_r, hi_r = expected["adaptation_rate_ratio_N100000_vs_N10000"]
    sub_pass = lo_r <= rate_ratio <= hi_r
    status = "PASS" if sub_pass else "FAIL"
    print(f"  [{status}] Adaptation rate ratio (N=100000/N=10000) = {rate_ratio:.2f} "
          f"(expected: [{lo_r}, {hi_r}], linear would be 10)")
    if sub_pass:
        checks_passed += 1

    # Check 6: Haldane sieve exceeded at small N (p_fix ≈ 2s when N*U_b << 1)
    checks_total += 1
    small_n_fix = results_by_size["100"]["fixation_probability"]
    haldane = 2.0 * mean_s
    haldane_close = small_n_fix > 0.5 * haldane
    status = "PASS" if haldane_close else "FAIL"
    print(f"  [{status}] Small-N fixation prob ({small_n_fix:.4f}) approaches "
          f"Haldane sieve (2s={haldane:.4f})")
    if haldane_close:
        checks_passed += 1

    # Check 7: determinism
    checks_total += 1
    rng2 = np.random.default_rng(seed)
    res2 = run_replicates(pop_sizes[0], n_gens, u_b, mean_s, n_reps, rng2)
    det_pass = (res2["total_fixations"] == results_by_size[str(pop_sizes[0])]["total_fixations"]
                and res2["total_mutations"] == results_by_size[str(pop_sizes[0])]["total_mutations"])
    status = "PASS" if det_pass else "FAIL"
    print(f"  [{status}] Deterministic (same seed → same counts)")
    if det_pass:
        checks_passed += 1

    # Summary
    print(f"\n{'=' * 72}")
    print(f"  RESULT: {checks_passed}/{checks_total} checks PASS")
    print(f"{'=' * 72}")

    # Write expected values JSON for lithoSpore absorption
    expected_values = {
        "experiment": "038_ltee_clonal_interference",
        "paper": "Good2017",
        "paper_id": "B3",
        "litho_module": 3,
        "pop_sizes": pop_sizes,
        "results_by_size": {
            k: {key: val for key, val in v.items() if key != "mean_trajectory"}
            for k, v in results_by_size.items()
        },
        "trajectories": {
            k: v["mean_trajectory"][::100]  # subsample every 100 gens
            for k, v in results_by_size.items()
        },
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
