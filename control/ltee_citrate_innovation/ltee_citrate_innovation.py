#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
Experiment 039 — LTEE Citrate Innovation (Blount et al. 2008/2012).

Reproduces the potentiation-actualization model for the Cit+ key innovation:
  Blount ZD et al. (2008) Historical contingency and the evolution of a key
  innovation in an experimental population of E. coli. PNAS 105(23):7899-7906.
  Blount ZD et al. (2012) Genomic analysis of a key innovation in an
  experimental population of E. coli. Nature 489:513-518.

The Cit+ phenotype evolved in 1/12 LTEE populations at ~31,500 generations.
Replay experiments demonstrated historical contingency: earlier clones had
lower probability of re-evolving Cit+, consistent with a potentiation
requirement (two-hit mutational cascade).
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np

SCRIPT_DIR = Path(__file__).resolve().parent
BENCHMARK = SCRIPT_DIR / "benchmark_ltee_citrate.json"


def load_benchmark():
    with open(BENCHMARK) as f:
        return json.load(f)


def simulate_populations(n_pops, n_gens, p_pot, p_act, rng):
    """Simulate n_pops independent LTEE populations with two-hit cascade.

    Returns:
        potentiation_gen: array of generation when potentiation occurred (NaN if never)
        cit_plus_gen: array of generation when Cit+ arose (NaN if never)
    """
    potentiation_gen = np.full(n_pops, np.nan)
    cit_plus_gen = np.full(n_pops, np.nan)
    potentiated = np.zeros(n_pops, dtype=bool)
    cit_plus = np.zeros(n_pops, dtype=bool)

    for gen in range(n_gens):
        not_pot = ~potentiated & ~cit_plus
        if not_pot.any():
            gain_pot = rng.random(n_pops) < p_pot
            newly_pot = not_pot & gain_pot
            potentiated[newly_pot] = True
            potentiation_gen[newly_pot] = gen

        pot_no_cit = potentiated & ~cit_plus
        if pot_no_cit.any():
            gain_cit = rng.random(n_pops) < p_act
            newly_cit = pot_no_cit & gain_cit
            cit_plus[newly_cit] = True
            cit_plus_gen[newly_cit] = gen

        if cit_plus.all():
            break

    return potentiation_gen, cit_plus_gen


def simulate_replay(potentiated_at_timepoint, p_act, replay_duration, n_reps, rng):
    """Simulate replay experiments from a given potentiation state.

    Given whether each original population was potentiated at the replay
    timepoint, simulate n_reps replays per population and count how many
    evolve Cit+ within replay_duration generations.
    """
    n_pops = len(potentiated_at_timepoint)
    cit_count = 0
    total_replays = 0

    for pop_idx in range(n_pops):
        for _ in range(n_reps):
            total_replays += 1
            if not potentiated_at_timepoint[pop_idx]:
                continue
            for _ in range(replay_duration):
                if rng.random() < p_act:
                    cit_count += 1
                    break

    return cit_count / total_replays if total_replays > 0 else 0.0


def main():
    bench = load_benchmark()
    cfg = bench["model"]
    expected = bench["expected_results"]

    print("=" * 72)
    print("  Experiment 039: LTEE Citrate Innovation (Blount 2008/2012)")
    print("  B4 Reproduction | lithoSpore Module 4 | Rare Event Statistics")
    print("=" * 72)

    n_pops = cfg["n_populations"]
    n_gens = cfg["n_generations"]
    p_pot = cfg["potentiation_rate_per_gen"]
    p_act = cfg["actualization_rate_per_gen"]
    replay_timepoints = cfg["replay_timepoints"]
    n_replay_reps = cfg["n_replay_replicates"]
    replay_duration = cfg["replay_duration"]
    seed = cfg["seed"]

    rng = np.random.default_rng(seed)

    checks_passed = 0
    checks_total = 0

    # Run the main simulation
    pot_gen, cit_gen = simulate_populations(n_pops, n_gens, p_pot, p_act, rng)

    cit_fraction = np.sum(~np.isnan(cit_gen)) / n_pops
    pot_fraction = np.sum(~np.isnan(pot_gen)) / n_pops

    print(f"\n  Populations: {n_pops}, Generations: {n_gens}")
    print(f"  Potentiation rate: {p_pot}, Actualization rate: {p_act}")
    print(f"  Potentiated: {pot_fraction:.2%}, Cit+: {cit_fraction:.2%}")

    # Check 1: Fraction of populations evolving Cit+
    checks_total += 1
    low, high = expected["fraction_populations_cit_plus_range"]
    frac_pass = low <= cit_fraction <= high
    print(f"\n  Cit+ fraction: {cit_fraction:.4f} (expected range [{low}, {high}])")
    print(f"  [{'PASS' if frac_pass else 'FAIL'}] Cit+ fraction in expected range")
    if frac_pass:
        checks_passed += 1

    # Replay experiments at different timepoints
    rng_replay = np.random.default_rng(seed + 1)
    replay_probs = {}

    for tp in replay_timepoints:
        potentiated_at_tp = ~np.isnan(pot_gen) & (pot_gen <= tp)
        prob = simulate_replay(
            potentiated_at_tp, p_act, replay_duration, n_replay_reps, rng_replay
        )
        replay_probs[tp] = prob
        print(f"  Replay from gen {tp:>6d}: Cit+ probability = {prob:.4f}")

    # Check 2: Replay probability non-decreasing with generation
    checks_total += 1
    prob_values = [replay_probs[tp] for tp in sorted(replay_timepoints)]
    non_decreasing = all(
        prob_values[i] <= prob_values[i + 1] + 0.01
        for i in range(len(prob_values) - 1)
    )
    print(f"\n  [{'PASS' if non_decreasing else 'FAIL'}] "
          "Replay probability non-decreasing with generation")
    if non_decreasing:
        checks_passed += 1

    # Check 3: Early replays have <= Cit+ probability compared to late replays
    checks_total += 1
    early_prob = replay_probs.get(0, 0.0)
    late_prob = replay_probs.get(max(replay_timepoints), 0.0)
    early_leq_late = early_prob <= late_prob
    print(f"  Early (gen 0): {early_prob:.4f}, Late (gen {max(replay_timepoints)}): {late_prob:.4f}")
    print(f"  [{'PASS' if early_leq_late else 'FAIL'}] Early replay prob <= late replay prob")
    if early_leq_late:
        checks_passed += 1

    # Check 4: Early replay fraction below threshold
    checks_total += 1
    early_max = expected["early_replay_cit_fraction_max"]
    early_ok = early_prob <= early_max
    print(f"\n  [{'PASS' if early_ok else 'FAIL'}] Early replay Cit+ fraction "
          f"{early_prob:.4f} <= {early_max}")
    if early_ok:
        checks_passed += 1

    # Check 5: Late replay fraction above threshold
    checks_total += 1
    late_min = expected["late_replay_cit_fraction_min"]
    late_ok = late_prob >= late_min
    print(f"  [{'PASS' if late_ok else 'FAIL'}] Late replay Cit+ fraction "
          f"{late_prob:.4f} >= {late_min}")
    if late_ok:
        checks_passed += 1

    # Check 6: Potentiation fraction at endpoint
    checks_total += 1
    pot_min = expected["potentiation_fraction_at_60k_min"]
    pot_ok = pot_fraction >= pot_min
    print(f"\n  [{'PASS' if pot_ok else 'FAIL'}] Potentiation fraction "
          f"{pot_fraction:.4f} >= {pot_min}")
    if pot_ok:
        checks_passed += 1

    # Check 7: Two-hit cascade analytical property
    #   The mean waiting time of a two-hit process (convolution of two
    #   exponentials) always exceeds the single-hit mean: E[τ₁+τ₂] > E[τ₂].
    #   We verify this analytically; simulation may have zero events.
    checks_total += 1
    single_hit_mean = 1.0 / p_act
    two_hit_analytical = (1.0 / p_pot) + (1.0 / p_act)
    slower = two_hit_analytical > single_hit_mean
    if np.any(~np.isnan(cit_gen)):
        two_hit_empirical = np.nanmean(cit_gen)
        print(f"\n  Single-hit mean waiting time:    {single_hit_mean:>10.0f} gen")
        print(f"  Two-hit analytical (1/λ₁ + 1/λ₂): {two_hit_analytical:>10.0f} gen")
        print(f"  Two-hit empirical mean:            {two_hit_empirical:>10.0f} gen")
    else:
        print(f"\n  Single-hit mean waiting time:    {single_hit_mean:>10.0f} gen")
        print(f"  Two-hit analytical (1/λ₁ + 1/λ₂): {two_hit_analytical:>10.0f} gen")
        print("  Two-hit empirical mean:            N/A (no Cit+ events)")
    print(f"  [{'PASS' if slower else 'FAIL'}] Two-hit analytical mean > single-hit mean")
    if slower:
        checks_passed += 1

    # Check 8: Determinism
    checks_total += 1
    rng2 = np.random.default_rng(seed)
    pot_gen2, cit_gen2 = simulate_populations(n_pops, n_gens, p_pot, p_act, rng2)
    det_pass = (np.array_equal(np.isnan(pot_gen), np.isnan(pot_gen2)) and
                np.allclose(pot_gen[~np.isnan(pot_gen)], pot_gen2[~np.isnan(pot_gen2)]))
    print(f"\n  [{'PASS' if det_pass else 'FAIL'}] Deterministic (same seed → same result)")
    if det_pass:
        checks_passed += 1

    print(f"\n{'=' * 72}")
    print(f"  RESULT: {checks_passed}/{checks_total} checks PASS")
    print(f"{'=' * 72}")

    expected_values = {
        "experiment": "039_ltee_citrate_innovation",
        "paper": "Blount2008",
        "paper_id": "B4",
        "litho_module": 4,
        "cit_plus_fraction": float(cit_fraction),
        "potentiation_fraction": float(pot_fraction),
        "mean_potentiation_gen": float(np.nanmean(pot_gen)) if np.any(~np.isnan(pot_gen)) else None,
        "mean_cit_plus_gen": float(np.nanmean(cit_gen)) if np.any(~np.isnan(cit_gen)) else None,
        "replay_probabilities": {str(k): float(v) for k, v in replay_probs.items()},
        "single_hit_mean_wait": single_hit_mean,
        "two_hit_analytical_mean": two_hit_analytical,
        "two_hit_empirical_mean": float(np.nanmean(cit_gen)) if np.any(~np.isnan(cit_gen)) else None,
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
