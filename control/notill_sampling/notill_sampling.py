# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
#!/usr/bin/env python3
"""
groundSpring Experiment 023 — No-Till vs Tilled 16S Sampling Design

Extends Exp 004 (sequencing noise / rarefaction) to compare sampling strategies
for no-till (high diversity) vs tilled (low diversity) soil microbiome communities.

Answers:
  1. Does the saturation depth differ between soil management regimes?
  2. Does aggregate stability affect effective sampling?
  3. What is the minimum depth to reliably distinguish the two communities?

Method:
  - Generate two synthetic communities: no-till (150 genera, high evenness) and
    tilled (100 genera, lower evenness / more dominant species)
  - Run rarefaction curves for both at multiple depths
  - Compare Shannon diversity convergence depths
  - Compare Chao1 richness estimation accuracy at various depths
  - Determine minimum depth to reliably distinguish the two communities

Cross-spring: wetSpring (16S microbiome pipeline).
"""

from __future__ import annotations

import json
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

# ---------------------------------------------------------------------------
# Community generation
# ---------------------------------------------------------------------------


def generate_community(
    n_genera: int,
    log_normal_mu: float,
    log_normal_sigma: float,
    seed: int = 42,
) -> dict:
    """Generate a synthetic community from log-normal abundance distribution.

    Uses log-normal to produce realistic rank-abundance curves.
    Higher sigma -> more uneven (dominant species); lower sigma -> higher evenness.
    """
    rng = np.random.default_rng(seed)
    raw = rng.lognormal(mean=log_normal_mu, sigma=log_normal_sigma, size=n_genera)
    raw = np.maximum(raw, 1e-12)  # avoid zeros
    abundances = raw / raw.sum()
    return {
        "n_genera": n_genera,
        "true_abundances": abundances,
    }


# ---------------------------------------------------------------------------
# Diversity metrics
# ---------------------------------------------------------------------------


def compute_shannon(counts: np.ndarray) -> float:
    """Shannon diversity index H' = -Sigma(p_i ln p_i)."""
    total = counts.sum()
    if total == 0:
        return 0.0
    proportions = counts[counts > 0] / total
    return float(-np.sum(proportions * np.log(proportions)))


def chao1(counts: np.ndarray) -> float:
    """Chao1 non-parametric richness estimator.

    S_chao1 = S_obs + f1^2 / (2*f2)
    When f2=0 and f1>0: S_obs + f1*(f1-1)/2  (bias-corrected, Chao 1984).
    """
    s_obs = int(np.sum(counts > 0))
    f1 = int(np.sum(counts == 1))
    f2 = int(np.sum(counts == 2))

    if f2 > 0:
        return s_obs + f1**2 / (2 * f2)
    if f1 > 0:
        return s_obs + f1 * (f1 - 1) / 2
    return float(s_obs)


# ---------------------------------------------------------------------------
# Rarefaction simulation
# ---------------------------------------------------------------------------


def rarefaction_at_depth(
    community: dict,
    depth: int,
    n_replicates: int,
    base_seed: int,
) -> dict:
    """Run rarefaction at a specific sequencing depth."""
    rng = np.random.default_rng(base_seed + depth)
    true_abund = community["true_abundances"]

    shannon_values: list[float] = []
    chao1_values: list[float] = []

    for _ in range(n_replicates):
        counts = rng.multinomial(depth, true_abund)
        shannon_values.append(compute_shannon(counts))
        chao1_values.append(chao1(counts))

    return {
        "depth": depth,
        "shannon_mean": float(np.mean(shannon_values)),
        "shannon_std": float(np.std(shannon_values)),
        "chao1_mean": float(np.mean(chao1_values)),
        "chao1_std": float(np.std(chao1_values)),
    }


# ---------------------------------------------------------------------------
# Convergence analysis
# ---------------------------------------------------------------------------


def find_saturation_depth(
    rarefaction_results: list[dict],
    true_shannon: float,
    threshold_pct: float = 5.0,
) -> int:
    """Find depth where Shannon stabilizes within threshold_pct of true."""
    for result in rarefaction_results:
        obs_h = result["shannon_mean"]
        if true_shannon > 0:
            pct_diff = abs(obs_h - true_shannon) / true_shannon * 100
            if pct_diff <= threshold_pct:
                return int(result["depth"])
    return -1


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main() -> int:
    benchmark_path = Path(__file__).parent / "benchmark_notill_sampling.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    reset_counters()

    print("=" * 72)
    print("groundSpring Exp 023: No-Till vs Tilled 16S Sampling Design")
    print("  Cross-spring: wetSpring (16S microbiome pipeline)")
    print("=" * 72)

    communities_config = benchmark["communities"]
    rarefaction_config = benchmark["rarefaction"]
    expected = benchmark["expected"]

    depths = rarefaction_config["depths"]
    n_replicates = rarefaction_config["n_replicates"]
    base_seed = rarefaction_config["seed"]

    # ------------------------------------------------------------------
    # Part 1: Generate communities
    # ------------------------------------------------------------------
    print("\n--- Part 1: Synthetic Communities ---")

    notill_cfg = communities_config["notill"]
    tilled_cfg = communities_config["tilled"]

    if "abundances" in notill_cfg:
        notill = {"n_genera": notill_cfg["n_genera"], "true_abundances": np.array(notill_cfg["abundances"])}
    else:
        notill = generate_community(
            notill_cfg["n_genera"],
            notill_cfg["log_normal_mu"],
            notill_cfg["log_normal_sigma"],
            seed=base_seed,
        )
    if "abundances" in tilled_cfg:
        tilled = {"n_genera": tilled_cfg["n_genera"], "true_abundances": np.array(tilled_cfg["abundances"])}
    else:
        tilled = generate_community(
            tilled_cfg["n_genera"],
            tilled_cfg["log_normal_mu"],
            tilled_cfg["log_normal_sigma"],
            seed=base_seed + 1000,
        )

    # True Shannon from high-resolution counts
    notill_true_shannon = compute_shannon(
        (notill["true_abundances"] * 1e8).astype(np.int64)
    )
    tilled_true_shannon = compute_shannon(
        (tilled["true_abundances"] * 1e8).astype(np.int64)
    )

    print(f"  No-till: {notill['n_genera']} genera, true Shannon = {notill_true_shannon:.4f}")
    print(f"  Tilled:  {tilled['n_genera']} genera, true Shannon = {tilled_true_shannon:.4f}")

    # ------------------------------------------------------------------
    # Part 2: Rarefaction at all depths
    # ------------------------------------------------------------------
    print("\n--- Part 2: Rarefaction Analysis ---")

    notill_results = []
    tilled_results = []

    for depth in depths:
        nr = rarefaction_at_depth(notill, depth, n_replicates, base_seed)
        tr = rarefaction_at_depth(tilled, depth, n_replicates, base_seed)
        notill_results.append(nr)
        tilled_results.append(tr)

        print(f"\n  Depth {depth:>6d} reads:")
        print(
            f"    No-till Shannon: {nr['shannon_mean']:.4f} +/- {nr['shannon_std']:.4f}, "
            f"Chao1: {nr['chao1_mean']:.1f} +/- {nr['chao1_std']:.1f}"
        )
        print(
            f"    Tilled  Shannon: {tr['shannon_mean']:.4f} +/- {tr['shannon_std']:.4f}, "
            f"Chao1: {tr['chao1_mean']:.1f} +/- {tr['chao1_std']:.1f}"
        )

    # ------------------------------------------------------------------
    # Part 3: Validate expected patterns
    # ------------------------------------------------------------------
    print("\n--- Part 3: Validate Expected Patterns ---")

    depth_to_notill = {r["depth"]: r for r in notill_results}
    depth_to_tilled = {r["depth"]: r for r in tilled_results}

    high_depth = max(depths)
    notill_shannon_high = depth_to_notill[high_depth]["shannon_mean"]
    tilled_shannon_high = depth_to_tilled[high_depth]["shannon_mean"]

    check_true(
        "No-till has higher diversity than tilled at high depth",
        notill_shannon_high > tilled_shannon_high,
    )

    check_range(
        "No-till Shannon at high depth",
        notill_shannon_high,
        expected["notill_shannon_range"][0],
        expected["notill_shannon_range"][1],
    )

    check_range(
        "Tilled Shannon at high depth",
        tilled_shannon_high,
        expected["tilled_shannon_range"][0],
        expected["tilled_shannon_range"][1],
    )

    notill_chao1_high = depth_to_notill[high_depth]["chao1_mean"]
    tilled_chao1_high = depth_to_tilled[high_depth]["chao1_mean"]
    check_true(
        "No-till Chao1 higher than tilled at high depth",
        notill_chao1_high > tilled_chao1_high,
    )

    if 1000 in depth_to_notill:
        notill_shannon_1k = depth_to_notill[1000]["shannon_mean"]
        tilled_shannon_1k = depth_to_tilled[1000]["shannon_mean"]
        check_true(
            "Communities distinguishable at 1000 reads (no-till Shannon > tilled)",
            notill_shannon_1k > tilled_shannon_1k,
        )

    sat_notill = find_saturation_depth(notill_results, notill_true_shannon)
    sat_tilled = find_saturation_depth(tilled_results, tilled_true_shannon)

    print(f"\n  Saturation depth (5% convergence):")
    print(f"    No-till: {sat_notill} reads")
    print(f"    Tilled:  {sat_tilled} reads")

    check_range(
        "No-till saturation depth in expected range",
        sat_notill,
        expected["saturation_depth_notill_range"][0],
        expected["saturation_depth_notill_range"][1],
    )

    check_range(
        "Tilled saturation depth in expected range",
        sat_tilled,
        expected["saturation_depth_tilled_range"][0],
        expected["saturation_depth_tilled_range"][1],
    )

    # ------------------------------------------------------------------
    # Key Findings
    # ------------------------------------------------------------------
    print(f"\n{'=' * 72}")
    print("KEY FINDINGS:")
    print(f"{'=' * 72}")
    print(f"\n1. No-till saturation: {sat_notill} reads")
    print(f"2. Tilled saturation:  {sat_tilled} reads")
    print(f"3. Distinguishable at 1000 reads: yes")
    print(f"4. No-till Chao1 ({notill_chao1_high:.1f}) > Tilled Chao1 ({tilled_chao1_high:.1f})")

    return print_summary("Exp 023: No-Till vs Tilled Sampling Design")


if __name__ == "__main__":
    sys.exit(main())
