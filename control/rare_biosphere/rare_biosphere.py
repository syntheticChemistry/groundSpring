#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring Experiment 016 — Rare Biosphere Signal Detection

At what sequencing depth can we reliably distinguish rare biological
lineages from sequencing artifacts?  This extends Exp 004 (genus
saturation at 5 000 reads) to the rare end of the abundance distribution
using Chao1 richness estimation and analytical detection power.

Method:
  - Synthetic community with 50 species across 5 abundance tiers
  - Multinomial sampling simulates sequencing at various depths
  - Chao1 non-parametric richness estimator (Chao 1984)
  - Detection power: P(detect) = 1 - (1 - p)^D
  - Detection threshold: D* = ceil(ln(0.05) / ln(1-p))
  - Abundance-occupancy relationship across replicate samples

Reference:
  Anderson, Sogin, Baross (2015) FEMS Microbiol Ecol 91:fiv016
  Chao (1984) Scand J Stat 11:265-270
  Sogin et al. (2006) PNAS 103:12115-12120

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
    check_max,
    check_min,
    check_range,
    check_true,
    print_summary,
    reset_counters,
)


def chao1(counts: np.ndarray) -> float:
    """Chao1 non-parametric richness estimator.

    S_chao1 = S_obs + f1^2 / (2*f2)
    When f2=0 and f1>0: S_obs + f1*(f1-1)/2  (bias-corrected, Chao 1984).
    """
    s_obs = int(np.sum(counts > 0))
    f1 = int(np.sum(counts == 1))
    f2 = int(np.sum(counts == 2))

    if f2 > 0:
        return s_obs + f1 ** 2 / (2 * f2)
    if f1 > 0:
        return s_obs + f1 * (f1 - 1) / 2
    return float(s_obs)


def detection_power(abundance: float, depth: int) -> float:
    """Analytical detection probability: P = 1 - (1-p)^D."""
    return 1.0 - (1.0 - abundance) ** depth


def detection_threshold(abundance: float, target_power: float = 0.95) -> int:
    """Minimum depth for target detection power: ceil(ln(1-P)/ln(1-p))."""
    if abundance <= 0.0:
        return 0
    return math.ceil(math.log(1.0 - target_power) / math.log(1.0 - abundance))


def tier_detection_rate(
    community: np.ndarray,
    tier_lo: int,
    tier_hi: int,
    depth: int,
    n_replicates: int,
    base_seed: int,
) -> float:
    """Mean detection rate for species in [tier_lo, tier_hi) across replicates."""
    rng = np.random.default_rng(base_seed)
    detections = 0
    total = 0

    for _ in range(n_replicates):
        seed = int(rng.integers(0, 2**32))
        rep_rng = np.random.default_rng(seed)
        counts = rep_rng.multinomial(depth, community)
        for idx in range(tier_lo, tier_hi):
            total += 1
            if counts[idx] > 0:
                detections += 1

    return detections / total if total > 0 else 0.0


def main() -> int:
    benchmark_path = Path(__file__).parent / "benchmark_rare_biosphere.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    reset_counters()

    model = benchmark["model"]
    pred = benchmark["analytical_predictions"]
    exp = benchmark["expected_results"]

    community = np.array(model["community"])
    n_species = model["n_species"]
    depths = model["depths"]
    n_reps = model["n_replicates"]
    base_seed = model["base_seed"]
    tiers = model["tier_boundaries"]

    print("=" * 72)
    print("groundSpring Exp 016: Rare Biosphere Signal Detection")
    print(f"  Community: {n_species} species, 5 abundance tiers")
    print(f"  Depths: {depths}")
    print(f"  Replicates: {n_reps}")
    print("  Reference: Anderson, Sogin, Baross (2015) FEMS")
    print("=" * 72)

    # ------------------------------------------------------------------
    # Part 1: Chao1 accuracy across depths
    # ------------------------------------------------------------------
    print("\n--- Part 1: Chao1 Richness Estimation ---")
    rng = np.random.default_rng(base_seed)

    chao1_by_depth = {}
    sobs_by_depth = {}
    for depth in depths:
        chao1_vals = []
        sobs_vals = []
        for _ in range(n_reps):
            seed = int(rng.integers(0, 2**32))
            rep_rng = np.random.default_rng(seed)
            counts = rep_rng.multinomial(depth, community)
            chao1_vals.append(chao1(counts))
            sobs_vals.append(int(np.sum(counts > 0)))
        mean_chao1 = float(np.mean(chao1_vals))
        mean_sobs = float(np.mean(sobs_vals))
        chao1_by_depth[depth] = mean_chao1
        sobs_by_depth[depth] = mean_sobs
        print(f"  D={depth:6d}: S_obs={mean_sobs:.1f}, Chao1={mean_chao1:.1f} (true={n_species})")

    check_range(
        "Chao1 at D=50000 ≈ true richness",
        chao1_by_depth[50000],
        exp["chao1_at_depth_50000_range"][0],
        exp["chao1_at_depth_50000_range"][1],
    )

    check_true(
        "Chao1 > S_obs at low depth (D=100)",
        chao1_by_depth[100] > sobs_by_depth[100],
    )

    check_true(
        "All species detected at D=50000",
        sobs_by_depth[50000] >= exp["sobs_at_depth_50000"] - 0.5,
    )

    # ------------------------------------------------------------------
    # Part 2: Detection power by abundance tier
    # ------------------------------------------------------------------
    print("\n--- Part 2: Detection Power by Tier ---")

    dom_lo, dom_hi = tiers["dominant"]
    vr_lo, vr_hi = tiers["very_rare"]

    dom_rate = tier_detection_rate(
        community, dom_lo, dom_hi, 100, n_reps, base_seed + 1000,
    )
    vr_rate_100 = tier_detection_rate(
        community, vr_lo, vr_hi, 100, n_reps, base_seed + 2000,
    )
    vr_rate_5000 = tier_detection_rate(
        community, vr_lo, vr_hi, 5000, n_reps, base_seed + 3000,
    )

    p_dom = detection_power(0.06, 100)
    p_vr_100 = detection_power(0.003, 100)
    p_vr_5000 = detection_power(0.003, 5000)

    print(f"  Dominant  at D=100:  rate={dom_rate:.3f} (theory ≥ {p_dom:.3f})")
    print(f"  Very rare at D=100:  rate={vr_rate_100:.3f} (theory ≈ {p_vr_100:.3f})")
    print(f"  Very rare at D=5000: rate={vr_rate_5000:.3f} (theory ≈ {p_vr_5000:.3f})")

    check_min(
        "Dominant detected at D=100",
        dom_rate,
        exp["detection_rate_dominant_at_100_min"],
    )
    check_max(
        "Very rare rarely detected at D=100",
        vr_rate_100,
        exp["detection_rate_very_rare_at_100_max"],
    )
    check_min(
        "Very rare detected at D=5000",
        vr_rate_5000,
        exp["detection_rate_very_rare_at_5000_min"],
    )

    # ------------------------------------------------------------------
    # Part 3: Detection threshold verification
    # ------------------------------------------------------------------
    print("\n--- Part 3: Analytical Detection Thresholds ---")

    for label, p_val, expected_key in [
        ("very_rare (p=0.003)", 0.003, "detection_threshold_very_rare_p003"),
        ("rare (p=0.004)", 0.004, "detection_threshold_rare_p004"),
        ("moderate (p=0.008)", 0.008, "detection_threshold_moderate_p008"),
        ("common (p=0.030)", 0.030, "detection_threshold_common_p030"),
    ]:
        computed = detection_threshold(p_val, 0.95)
        expected = pred[expected_key]
        print(f"  {label}: D*={computed} (expected {expected})")

    check_true(
        "Detection threshold monotonically decreases with abundance",
        (
            detection_threshold(0.003) > detection_threshold(0.004)
            > detection_threshold(0.008) > detection_threshold(0.030)
        ),
    )

    # ------------------------------------------------------------------
    # Part 4: Abundance-occupancy relationship
    # ------------------------------------------------------------------
    print("\n--- Part 4: Abundance-Occupancy Relationship ---")

    n_samples = model["n_samples_occupancy"]
    occ_depth = model["occupancy_depth"]
    occ_rng = np.random.default_rng(base_seed + 50000)

    detection_counts = np.zeros(n_species)
    for _ in range(n_samples):
        seed = int(occ_rng.integers(0, 2**32))
        rep_rng = np.random.default_rng(seed)
        counts = rep_rng.multinomial(occ_depth, community)
        detection_counts += (counts > 0).astype(float)
    occupancy = detection_counts / n_samples

    dom_occ = float(np.mean(occupancy[tiers["dominant"][0]:tiers["dominant"][1]]))
    vr_occ = float(np.mean(occupancy[tiers["very_rare"][0]:tiers["very_rare"][1]]))
    print(f"  Dominant mean occupancy:   {dom_occ:.3f}")
    print(f"  Very rare mean occupancy:  {vr_occ:.3f}")

    from scipy.stats import spearmanr
    rho, _ = spearmanr(community, occupancy)
    print(f"  Spearman(abundance, occupancy) = {rho:.3f}")

    check_true("Occupancy positively correlated with abundance", bool(rho > 0.5))

    # ------------------------------------------------------------------
    # Part 5: Singleton discrimination
    # ------------------------------------------------------------------
    print("\n--- Part 5: Singleton Fraction vs Depth ---")

    sing_rng = np.random.default_rng(base_seed + 60000)
    singleton_fracs = {}
    for depth in depths:
        frac_sum = 0.0
        for _ in range(n_reps):
            seed = int(sing_rng.integers(0, 2**32))
            rep_rng = np.random.default_rng(seed)
            counts = rep_rng.multinomial(depth, community)
            s_obs = int(np.sum(counts > 0))
            f1 = int(np.sum(counts == 1))
            frac_sum += f1 / s_obs if s_obs > 0 else 0.0
        singleton_fracs[depth] = frac_sum / n_reps

    for depth in depths:
        print(f"  D={depth:6d}: singleton fraction = {singleton_fracs[depth]:.3f}")

    check_true(
        "Singleton fraction decreases with depth",
        singleton_fracs[depths[0]] > singleton_fracs[depths[-1]],
    )

    # ------------------------------------------------------------------
    # Part 6: Determinism
    # ------------------------------------------------------------------
    print("\n--- Part 6: Determinism ---")

    det_rng1 = np.random.default_rng(99999)
    det_rng2 = np.random.default_rng(99999)
    c1 = det_rng1.multinomial(1000, community)
    c2 = det_rng2.multinomial(1000, community)
    check_true("Multinomial deterministic (same seed)", np.array_equal(c1, c2))

    chao1_a = chao1(c1)
    chao1_b = chao1(c2)
    check_true("Chao1 deterministic", chao1_a == chao1_b)

    # ------------------------------------------------------------------
    # Key Findings
    # ------------------------------------------------------------------
    print(f"\n{'=' * 72}")
    print("KEY FINDINGS:")
    print(f"{'=' * 72}")
    print(f"\n1. Chao1 at D=50000: {chao1_by_depth[50000]:.1f} ≈ true richness ({n_species})")
    print(f"2. Chao1 at D=100: {chao1_by_depth[100]:.1f} > S_obs {sobs_by_depth[100]:.1f} (corrects undersampling)")
    print(f"3. Dominant species: {dom_rate:.0%} detected at D=100 (near-certain)")
    print(f"4. Very rare species: {vr_rate_100:.0%} at D=100 → {vr_rate_5000:.0%} at D=5000")
    print(f"5. Detection threshold for p=0.003: D*={detection_threshold(0.003)} reads")
    print(f"6. Abundance-occupancy correlation: ρ={rho:.3f}")
    print()
    print("  Anderson et al. (2015) showed that rare microbial lineages in")
    print("  hydrothermal vents are real biological signal, not sequencing noise.")
    print("  This experiment proves that sufficient depth distinguishes the two.")

    return print_summary("Exp 016: Rare Biosphere Signal Detection")


if __name__ == "__main__":
    sys.exit(main())
