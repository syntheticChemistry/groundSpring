# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring Experiment 014 — Drift vs Selection

Wright-Fisher simulation testing when stochastic drift dominates over
deterministic selection in finite populations, to answer:
  1. At what population size does selection (signal) overcome drift (noise)?
  2. Does the N*s > 1 threshold correctly predict the regime?
  3. How does diversity decay under pure drift vs selection?
  4. Is the fixation probability consistent with Kimura's formula?

Method:
  - Wright-Fisher model: N diploid individuals, binomial sampling each gen
  - Allele A has fitness 1+s, allele a has fitness 1
  - Track fixation probability over many trials
  - Compare to Kimura (1968) analytical predictions
  - Neutral diversity: multi-species WF under pure drift

Reference:
  Anderson (2022) mBio 13:e00354-22 — drift dominates in low-biomass habitats
  Kimura (1968) Nature 217:624-626 — neutral theory of molecular evolution
  Wright (1931) Genetics 16:97-159 — Wright-Fisher model

Cross-spring: wetSpring (microbial diversity), Exp 004 (sequencing depth).
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


def wright_fisher_fixation(
    pop_size: int,
    selection: float,
    initial_freq: float,
    seed: int,
) -> bool:
    """Run one Wright-Fisher trial until fixation or loss.

    Returns True if allele A fixes, False if lost.
    """
    rng = np.random.default_rng(seed)
    n_alleles = 2 * pop_size
    n_a = round(initial_freq * n_alleles)

    max_gens = 10 * n_alleles
    for _ in range(max_gens):
        if n_a == 0:
            return False
        if n_a == n_alleles:
            return True

        freq_a = n_a / n_alleles
        fitness_a = freq_a * (1.0 + selection)
        fitness_total = fitness_a + (1.0 - freq_a)
        prob_a = fitness_a / fitness_total

        n_a = rng.binomial(n_alleles, prob_a)

    return n_a > n_alleles // 2


def kimura_fixation_prob(
    pop_size: int, selection: float, initial_freq: float,
) -> float:
    """Analytical fixation probability (Kimura 1968).

    P_fix = (1 - exp(-4*N*s*p₀)) / (1 - exp(-4*N*s))

    For diploids with 2N alleles and selection coefficient s.
    """
    four_ns = 4.0 * pop_size * selection
    if abs(four_ns) < 1e-10:
        return initial_freq

    numerator = 1.0 - math.exp(-four_ns * initial_freq)
    denominator = 1.0 - math.exp(-four_ns)
    if abs(denominator) < 1e-15:
        return initial_freq

    return numerator / denominator


def neutral_diversity_trajectory(
    n_species: int,
    pop_size: int,
    n_generations: int,
    seed: int,
) -> list[float]:
    """Track Shannon diversity under pure neutral drift.

    Multi-species Wright-Fisher: each generation, sample from multinomial
    with equal fitnesses. Returns diversity at each generation.
    """
    rng = np.random.default_rng(seed)
    abundances = np.full(n_species, pop_size // n_species)
    remainder = pop_size - abundances.sum()
    abundances[0] += remainder

    diversities = []
    for _ in range(n_generations):
        freqs = abundances / abundances.sum()
        nonzero = freqs[freqs > 0]
        shannon = -float(np.sum(nonzero * np.log(nonzero)))
        diversities.append(shannon)

        probs = freqs
        abundances = rng.multinomial(pop_size, probs)

    return diversities


def main() -> int:
    benchmark_path = Path(__file__).parent / "benchmark_drift_selection.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    reset_counters()

    model = benchmark["model"]
    pred = benchmark["analytical_predictions"]
    exp = benchmark["expected_results"]

    pop_sizes = model["population_sizes"]
    s_coeff = model["selection_coefficient"]
    p0 = model["initial_frequency"]
    n_trials = model["n_trials"]
    base_seed = model["base_seed"]

    print("=" * 72)
    print("groundSpring Exp 014: Drift vs Selection (R. Anderson 2022)")
    print(f"  Wright-Fisher model: s={s_coeff}, p₀={p0}, {n_trials} trials")
    print(f"  Population sizes: {pop_sizes}")
    print("  Cross-spring: wetSpring (microbial diversity)")
    print("=" * 72)

    # ------------------------------------------------------------------
    # Part 1: Neutral fixation (s=0)
    # ------------------------------------------------------------------
    print("\n--- Part 1: Neutral Fixation (s=0) ---")

    n_neutral = 100
    neutral_fixes = sum(
        wright_fisher_fixation(n_neutral, 0.0, p0, base_seed + i)
        for i in range(n_trials)
    )
    neutral_fix_rate = neutral_fixes / n_trials
    print(f"  N={n_neutral}, s=0: fixation rate = {neutral_fix_rate:.3f} (expected ~{p0})")

    check_range(
        "Neutral fixation ≈ p₀",
        neutral_fix_rate,
        exp["neutral_fixation_range"][0],
        exp["neutral_fixation_range"][1],
    )

    # ------------------------------------------------------------------
    # Part 2: Selection fixation across population sizes
    # ------------------------------------------------------------------
    print("\n--- Part 2: Selection Across Population Sizes ---")

    fix_rates = {}
    for n_pop in pop_sizes:
        fixes = sum(
            wright_fisher_fixation(n_pop, s_coeff, p0, base_seed + 10000 + n_pop * 1000 + i)
            for i in range(n_trials)
        )
        fix_rate = fixes / n_trials
        kimura_pred = kimura_fixation_prob(n_pop, s_coeff, p0)
        ns_product = n_pop * s_coeff
        regime = "DRIFT" if ns_product < 1.0 else "SELECTION"
        fix_rates[n_pop] = fix_rate
        print(f"  N={n_pop:4d}, N×s={ns_product:5.2f} ({regime:9s}): "
              f"P_fix={fix_rate:.3f} (Kimura={kimura_pred:.3f})")

    # Drift regime: fixation ≈ neutral
    drift_fix = fix_rates[pop_sizes[0]]
    check_range(
        f"Drift regime (N={pop_sizes[0]}) near neutral",
        drift_fix,
        pred["fixation_prob_neutral_p0_half"] - exp["drift_regime_fixation_near_neutral_tol"],
        pred["fixation_prob_neutral_p0_half"] + exp["drift_regime_fixation_near_neutral_tol"],
    )

    # Selection regime: fixation > neutral
    sel_fix = fix_rates[pop_sizes[-1]]
    check_true(
        f"Selection regime (N={pop_sizes[-1]}) > 60%",
        sel_fix >= exp["strong_selection_fixation_min"],
    )

    # Monotonicity: fixation increases with N (for s > 0)
    rates_ordered = [fix_rates[n] for n in pop_sizes]
    check_true(
        "Fixation generally increases with N",
        rates_ordered[-1] > rates_ordered[0],
    )

    # ------------------------------------------------------------------
    # Part 3: Kimura accuracy
    # ------------------------------------------------------------------
    print("\n--- Part 3: Kimura Formula Accuracy ---")
    for n_pop in pop_sizes:
        kimura = kimura_fixation_prob(n_pop, s_coeff, p0)
        observed = fix_rates[n_pop]
        diff = abs(observed - kimura)
        status = "OK" if diff < 0.10 else "WARN"
        print(f"  N={n_pop:4d}: observed={observed:.3f}, Kimura={kimura:.3f}, "
              f"diff={diff:.3f} [{status}]")

    # ------------------------------------------------------------------
    # Part 4: Neutral diversity trajectory
    # ------------------------------------------------------------------
    print("\n--- Part 4: Neutral Diversity Decay ---")

    n_sp = model["n_species_neutral"]
    n_gen = model["n_generations_diversity"]

    diversities_small = neutral_diversity_trajectory(n_sp, 50, n_gen, base_seed + 90000)
    diversities_large = neutral_diversity_trajectory(n_sp, 500, n_gen, base_seed + 91000)

    h0_small = diversities_small[0]
    h_end_small = diversities_small[-1]
    h0_large = diversities_large[0]
    h_end_large = diversities_large[-1]

    print(f"  N=50:  H(0)={h0_small:.4f} → H({n_gen})={h_end_small:.4f}")
    print(f"  N=500: H(0)={h0_large:.4f} → H({n_gen})={h_end_large:.4f}")

    check_true(
        "Diversity declines under drift (N=50)",
        h_end_small < h0_small,
    )
    check_true(
        "Small pop loses diversity faster",
        h_end_small < h_end_large,
    )

    # ------------------------------------------------------------------
    # Part 5: Determinism
    # ------------------------------------------------------------------
    print("\n--- Part 5: Determinism ---")
    r1 = wright_fisher_fixation(100, 0.01, 0.5, 99999)
    r2 = wright_fisher_fixation(100, 0.01, 0.5, 99999)
    check_true("WF deterministic (same seed)", r1 == r2)

    # ------------------------------------------------------------------
    # Key Findings
    # ------------------------------------------------------------------
    print(f"\n{'=' * 72}")
    print("KEY FINDINGS:")
    print(f"{'=' * 72}")
    print(f"\n1. Neutral fixation: {neutral_fix_rate:.3f} ≈ p₀ = {p0}")
    print(f"2. Drift regime (N={pop_sizes[0]}, N×s={pop_sizes[0]*s_coeff:.2f}): "
          f"P_fix = {fix_rates[pop_sizes[0]]:.3f} (near neutral)")
    print(f"3. Selection regime (N={pop_sizes[-1]}, N×s={pop_sizes[-1]*s_coeff:.1f}): "
          f"P_fix = {fix_rates[pop_sizes[-1]]:.3f} (selection wins)")
    print(f"4. Small populations (N=50): diversity decays {h0_small:.2f} → {h_end_small:.2f}")
    print(f"5. Large populations (N=500): diversity preserved {h0_large:.2f} → {h_end_large:.2f}")
    print("\n  R. Anderson's insight confirmed: in small populations (low-biomass")
    print("  habitats), stochastic drift overwhelms deterministic selection.")
    print("  This is the biological N×s analog of groundSpring's SNR framework.")

    return print_summary("Exp 014: Drift vs Selection")


if __name__ == "__main__":
    sys.exit(main())
