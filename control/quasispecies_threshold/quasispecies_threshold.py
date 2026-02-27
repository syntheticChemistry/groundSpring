#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring Experiment 017 — Quasispecies Error Threshold

At what mutation rate does noise (copying errors) destroy signal
(heritable information)?  Eigen's error threshold (1971) defines
the boundary: below it, the master sequence maintains a stable
subpopulation; above it, population randomizes to uniform noise.

This is the most fundamental formulation of groundSpring's central
question applied to self-replicating systems.

Method:
  - Single-peak fitness landscape: master fitness sigma, mutant fitness 1
  - Wright-Fisher selection + independent per-base mutation
  - Track master sequence frequency across generations
  - Compare to analytical: x_m = (sigma*Q - 1)/(sigma - 1), Q = (1-mu)^L
  - Error threshold: mu_c = 1 - sigma^(-1/L)

Reference:
  Dolson et al. (2023) J R Soc Interface 20(208)
  Eigen (1971) Naturwiss 58:465-523
  Kimura (1968) Nature 217:624-626

Cross-spring: wetSpring (microbial evolution), Exp 014 (drift).
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from common import (
    check_max,
    check_min,
    check_range,
    check_true,
    print_summary,
    reset_counters,
)


def error_threshold(sigma: float, genome_length: int) -> float:
    """Analytical error threshold: mu_c = 1 - sigma^(-1/L)."""
    return 1.0 - sigma ** (-1.0 / genome_length)


def master_frequency_analytical(sigma: float, mu: float, genome_length: int) -> float:
    """Analytical steady-state master frequency.

    x_m = max(0, (sigma*Q - 1) / (sigma - 1))  where Q = (1-mu)^L
    """
    q = (1.0 - mu) ** genome_length
    x_m = (sigma * q - 1.0) / (sigma - 1.0)
    return max(0.0, x_m)


def quasispecies_simulation(
    pop_size: int,
    genome_length: int,
    sigma: float,
    mu: float,
    n_generations: int,
    seed: int,
) -> list[float]:
    """Simulate quasispecies dynamics, return master frequency per generation.

    Each individual is either 'master' (type 0) or 'mutant' (type 1).
    Selection: master has fitness sigma, mutant has fitness 1.
    Mutation: master offspring stays master with probability Q = (1-mu)^L.
    Back-mutation (mutant→master) is negligible for large L.
    """
    rng = np.random.default_rng(seed)
    q = (1.0 - mu) ** genome_length
    n_master = pop_size // 2

    freqs = []
    for _ in range(n_generations):
        freq = n_master / pop_size
        freqs.append(freq)

        n_mutant = pop_size - n_master
        fitness_total = sigma * n_master + 1.0 * n_mutant
        p_master = (sigma * n_master) / fitness_total

        n_selected_master = rng.binomial(pop_size, p_master)

        n_master = rng.binomial(n_selected_master, q)

    return freqs


def main() -> int:
    benchmark_path = Path(__file__).parent / "benchmark_quasispecies.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    reset_counters()

    model = benchmark["model"]
    exp = benchmark["expected_results"]

    pop_size = model["population_size"]
    genome_length = model["genome_length"]
    sigma = model["master_fitness"]
    mutation_rates = model["mutation_rates"]
    n_gen = model["n_generations"]
    base_seed = model["base_seed"]

    mu_c = error_threshold(sigma, genome_length)

    print("=" * 72)
    print("groundSpring Exp 017: Quasispecies Error Threshold (Dolson 2023)")
    print(f"  N={pop_size}, L={genome_length}, σ={sigma}")
    print(f"  Analytical error threshold: μ_c = {mu_c:.5f}")
    print(f"  Mutation rates tested: {mutation_rates}")
    print("  Cross-spring: wetSpring (microbial evolution), Exp 014")
    print("=" * 72)

    # ------------------------------------------------------------------
    # Part 1: Analytical predictions
    # ------------------------------------------------------------------
    print("\n--- Part 1: Analytical Master Frequency ---")

    for mu in mutation_rates:
        x_m = master_frequency_analytical(sigma, mu, genome_length)
        regime = "BELOW" if mu < mu_c else "ABOVE"
        print(f"  μ={mu:.3f} ({regime:5s} threshold): x_m = {x_m:.4f}")

    check_range(
        "Error threshold matches analytical",
        mu_c,
        exp["error_threshold_observed_range"][0],
        exp["error_threshold_observed_range"][1],
    )

    # ------------------------------------------------------------------
    # Part 2: Simulation below threshold
    # ------------------------------------------------------------------
    print("\n--- Part 2: Below Threshold (Signal Survives) ---")

    mu_below = mutation_rates[1]
    assert mu_below < mu_c, f"mu_below={mu_below} should be < mu_c={mu_c}"

    freqs_below = quasispecies_simulation(
        pop_size, genome_length, sigma, mu_below, n_gen, base_seed,
    )
    steady_state = float(np.mean(freqs_below[n_gen // 2 :]))
    x_m_theory = master_frequency_analytical(sigma, mu_below, genome_length)

    print(f"  μ={mu_below}: steady-state x_m = {steady_state:.4f} (theory {x_m_theory:.4f})")

    check_min(
        "Master survives below threshold",
        steady_state,
        exp["master_freq_below_threshold_min"],
    )

    # ------------------------------------------------------------------
    # Part 3: Simulation above threshold
    # ------------------------------------------------------------------
    print("\n--- Part 3: Above Threshold (Signal Destroyed) ---")

    mu_above = mutation_rates[5]
    assert mu_above > mu_c, f"mu_above={mu_above} should be > mu_c={mu_c}"

    freqs_above = quasispecies_simulation(
        pop_size, genome_length, sigma, mu_above, n_gen, base_seed + 1000,
    )
    steady_above = float(np.mean(freqs_above[n_gen // 2 :]))
    x_m_above_theory = master_frequency_analytical(sigma, mu_above, genome_length)

    print(f"  μ={mu_above}: steady-state x_m = {steady_above:.4f} (theory {x_m_above_theory:.4f})")

    check_max(
        "Master lost above threshold",
        steady_above,
        exp["master_freq_above_threshold_max"],
    )

    # ------------------------------------------------------------------
    # Part 4: Sweep across mutation rates
    # ------------------------------------------------------------------
    print("\n--- Part 4: Mutation Rate Sweep ---")

    steady_states = {}
    mean_fitnesses = {}
    for i, mu in enumerate(mutation_rates):
        freqs = quasispecies_simulation(
            pop_size, genome_length, sigma, mu, n_gen, base_seed + 5000 + i * 100,
        )
        ss = float(np.mean(freqs[n_gen // 2 :]))
        mf = sigma * ss + 1.0 * (1.0 - ss)
        steady_states[mu] = ss
        mean_fitnesses[mu] = mf
        regime = "SIGNAL" if mu < mu_c else "NOISE"
        print(f"  μ={mu:.3f}: x_m={ss:.4f}, fitness={mf:.3f} [{regime}]")

    below_rates = [mu for mu in mutation_rates if mu < mu_c]
    above_rates = [mu for mu in mutation_rates if mu > mu_c]

    if below_rates and above_rates:
        below_fitness = mean_fitnesses[below_rates[-1]]
        above_fitness = mean_fitnesses[above_rates[0]]
        check_true(
            "Mean fitness drops at threshold",
            below_fitness > above_fitness,
        )

    # ------------------------------------------------------------------
    # Part 5: Monotonicity
    # ------------------------------------------------------------------
    print("\n--- Part 5: Master Frequency Monotonically Decreases ---")
    ss_list = [steady_states[mu] for mu in mutation_rates]
    decreasing = all(
        ss_list[i] >= ss_list[i + 1] - 0.05
        for i in range(len(ss_list) - 1)
    )
    check_true("Master frequency decreases with μ", decreasing)

    # ------------------------------------------------------------------
    # Part 6: Determinism
    # ------------------------------------------------------------------
    print("\n--- Part 6: Determinism ---")

    f1 = quasispecies_simulation(pop_size, genome_length, sigma, 0.01, 100, 99999)
    f2 = quasispecies_simulation(pop_size, genome_length, sigma, 0.01, 100, 99999)
    check_true("Quasispecies deterministic", f1 == f2)

    # ------------------------------------------------------------------
    # Key findings
    # ------------------------------------------------------------------
    print(f"\n{'=' * 72}")
    print("KEY FINDINGS:")
    print(f"{'=' * 72}")
    print(f"\n1. Error threshold: μ_c = {mu_c:.5f} (Eigen 1971)")
    print(f"2. Below threshold (μ={mu_below}): master x_m = {steady_state:.4f} (signal survives)")
    print(f"3. Above threshold (μ={mu_above}): master x_m = {steady_above:.4f} (noise wins)")
    print(f"4. Mean fitness drops from {mean_fitnesses.get(below_rates[-1], 0):.3f} to "
          f"{mean_fitnesses.get(above_rates[0], 0):.3f} at threshold")
    print()
    print("  Dolson et al. (2023) asked: where does signal begin in a system")
    print("  that starts as pure noise? Eigen's error threshold provides the")
    print("  answer — below μ_c, information self-organizes; above, noise wins.")

    return print_summary("Exp 017: Quasispecies Error Threshold")


if __name__ == "__main__":
    sys.exit(main())
