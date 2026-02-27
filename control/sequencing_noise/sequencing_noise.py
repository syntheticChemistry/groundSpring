#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring Experiment 004 — Sequencing Depth and Taxonomic Noise

Simulates rarefaction from a reference soil microbiome community to answer:
  1. At what sequencing depth does taxonomy become stable?
  2. How does Shannon diversity converge with increasing reads?
  3. What is the noise floor for genus-level assignments?
  4. When does more sequencing stop improving the answer?

Cross-spring with wetSpring: informs the DADA2 pipeline's minimum depth
requirements.

Method:
  - Generate a synthetic reference community (150 genera, 8 phyla)
  - Multinomial sampling at increasing read depths
  - Track genera detected, Shannon diversity, phylum completeness
  - N replicates per depth for confidence intervals
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

# ---------------------------------------------------------------------------
# Community generation
# ---------------------------------------------------------------------------

def generate_reference_community(config: dict, seed: int = 42) -> dict:
    """Generate a synthetic soil microbiome community.

    Uses log-normal abundance distribution within each phylum,
    producing a realistic rank-abundance curve.
    """
    rng = np.random.default_rng(seed)

    genera: list[str] = []
    phylum_assignments: list[str] = []
    true_abundances: list[float] = []

    for phylum in config["dominant_phyla"]:
        n_gen = phylum["n_genera"]
        total_abund = phylum["relative_abundance"]

        raw = rng.lognormal(mean=0, sigma=1.5, size=n_gen)
        raw = raw / raw.sum() * total_abund

        for i in range(n_gen):
            genera.append(f"{phylum['name']}_genus_{i+1:03d}")
            phylum_assignments.append(phylum["name"])
            true_abundances.append(raw[i])

    arr = np.array(true_abundances)
    arr = arr / arr.sum()

    return {
        "genera": genera,
        "phylum_assignments": phylum_assignments,
        "true_abundances": arr,
        "n_genera": len(genera),
        "n_phyla": len(set(phylum_assignments)),
    }


# ---------------------------------------------------------------------------
# Rarefaction simulation
# ---------------------------------------------------------------------------

def simulate_sampling(
    true_abundances: np.ndarray, depth: int, rng: np.random.Generator
) -> np.ndarray:
    """Simulate sequencing at a given depth via multinomial sampling."""
    return rng.multinomial(depth, true_abundances)


def compute_shannon(counts: np.ndarray) -> float:
    """Shannon diversity index H' = −Σ(pᵢ ln pᵢ)."""
    total = counts.sum()
    if total == 0:
        return 0.0
    proportions = counts[counts > 0] / total
    return float(-np.sum(proportions * np.log(proportions)))


def compute_evenness(shannon: float, n_species: int) -> float:
    """Pielou's evenness J' = H' / ln(S)."""
    if n_species <= 1:
        return 1.0
    return shannon / math.log(n_species)


def rarefaction_at_depth(
    community: dict,
    depth: int,
    n_replicates: int = 50,
    seed: int = 42,
) -> dict:
    """Run rarefaction analysis at a specific sequencing depth."""
    rng = np.random.default_rng(seed + depth)
    true_abund = community["true_abundances"]

    genera_detected: list[int] = []
    phyla_detected: list[int] = []
    shannon_values: list[float] = []

    phylum_array = np.array(community["phylum_assignments"])

    for _ in range(n_replicates):
        counts = simulate_sampling(true_abund, depth, rng)

        genera_detected.append(int(np.sum(counts > 0)))

        detected_mask = counts > 0
        detected_phyla = set(phylum_array[detected_mask])
        phyla_detected.append(len(detected_phyla))

        shannon_values.append(compute_shannon(counts))

    return {
        "depth": depth,
        "n_replicates": n_replicates,
        "genera_detected": {
            "mean": float(np.mean(genera_detected)),
            "std": float(np.std(genera_detected)),
            "min": int(np.min(genera_detected)),
            "max": int(np.max(genera_detected)),
        },
        "phyla_detected": {
            "mean": float(np.mean(phyla_detected)),
            "std": float(np.std(phyla_detected)),
            "all_detected_pct": float(
                np.mean(
                    [p == community["n_phyla"] for p in phyla_detected]
                )
                * 100
            ),
        },
        "shannon": {
            "mean": float(np.mean(shannon_values)),
            "std": float(np.std(shannon_values)),
        },
    }


# ---------------------------------------------------------------------------
# Convergence analysis
# ---------------------------------------------------------------------------

def find_convergence_depth(
    rarefaction_results: list,
    true_shannon: float,
    threshold_pct: float = 5.0,
) -> int:
    """Find depth where Shannon stabilizes within threshold_pct of true."""
    for result in rarefaction_results:
        obs_h = result["shannon"]["mean"]
        pct_diff = abs(obs_h - true_shannon) / true_shannon * 100
        if pct_diff <= threshold_pct:
            return int(result["depth"])
    return -1


def find_genus_saturation_depth(rarefaction_results: list) -> int:
    """Find depth where genus count yields < 5% new genera per doubling."""
    for i in range(1, len(rarefaction_results)):
        prev = rarefaction_results[i - 1]
        curr = rarefaction_results[i]

        if prev["genera_detected"]["mean"] > 0:
            pct_increase = (
                (curr["genera_detected"]["mean"] - prev["genera_detected"]["mean"])
                / prev["genera_detected"]["mean"]
                * 100
            )
            depth_ratio = curr["depth"] / prev["depth"]

            pct_per_doubling = pct_increase / math.log2(depth_ratio) if depth_ratio > 1 else 0

            if pct_per_doubling < 5.0:
                return int(curr["depth"])

    return -1


def find_phylum_completeness_depth(
    rarefaction_results: list, completeness_pct: float = 95.0
) -> int:
    """Find depth where all phyla detected in >= completeness_pct of replicates."""
    for result in rarefaction_results:
        if result["phyla_detected"]["all_detected_pct"] >= completeness_pct:
            return int(result["depth"])
    return -1


# ---------------------------------------------------------------------------
# Validation harness
# ---------------------------------------------------------------------------

def main() -> int:
    benchmark_path = Path(__file__).parent / "benchmark_sequencing_noise.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    reset_counters()

    print("=" * 72)
    print("groundSpring Exp 004: Sequencing Depth and Taxonomic Noise")
    print("  Cross-spring: wetSpring (16S microbiome pipeline)")
    print("=" * 72)

    # ------------------------------------------------------------------
    # Part 1: Generate reference community
    # ------------------------------------------------------------------
    print("\n--- Part 1: Reference Community ---")
    community = generate_reference_community(benchmark["reference_community"])

    print(f"  Genera: {community['n_genera']}")
    print(f"  Phyla:  {community['n_phyla']}")

    true_shannon = compute_shannon(
        (community["true_abundances"] * 1e8).astype(int)
    )
    print(f"  True Shannon H': {true_shannon:.4f}")

    check_true(
        "Abundances sum to 1.0",
        abs(community["true_abundances"].sum() - 1.0) < 1e-10,
    )
    check_true(
        "Correct number of genera",
        community["n_genera"] == benchmark["reference_community"]["n_genera"],
    )

    # ------------------------------------------------------------------
    # Part 2: Rarefaction at all depths
    # ------------------------------------------------------------------
    print("\n--- Part 2: Rarefaction Analysis ---")
    depths = benchmark["rarefaction_depths"]
    results = []

    for depth in depths:
        r = rarefaction_at_depth(community, depth, n_replicates=50)
        results.append(r)

        print(f"\n  Depth {depth:>7d} reads:")
        print(
            f"    Genera detected: {r['genera_detected']['mean']:.1f} "
            f"± {r['genera_detected']['std']:.1f} "
            f"(range {r['genera_detected']['min']}-{r['genera_detected']['max']})"
        )
        print(
            f"    Phyla detected:  {r['phyla_detected']['mean']:.1f} "
            f"({r['phyla_detected']['all_detected_pct']:.0f}% complete)"
        )
        print(
            f"    Shannon H':      {r['shannon']['mean']:.4f} "
            f"± {r['shannon']['std']:.4f}"
        )

    # ------------------------------------------------------------------
    # Part 3: Validate expected patterns
    # ------------------------------------------------------------------
    print("\n--- Part 3: Validate Expected Patterns ---")
    expected = benchmark["expected_results"]
    depth_to_result = {r["depth"]: r for r in results}

    for depth_key, exp in expected.items():
        depth_val = int(depth_key.split("_")[1])
        if depth_val not in depth_to_result:
            continue

        r = depth_to_result[depth_val]

        if "genera_detected_range" in exp:
            check_range(
                f"Genera at {depth_val} reads",
                r["genera_detected"]["mean"],
                exp["genera_detected_range"][0],
                exp["genera_detected_range"][1],
            )

        if "shannon_range" in exp:
            check_range(
                f"Shannon at {depth_val} reads",
                r["shannon"]["mean"],
                exp["shannon_range"][0],
                exp["shannon_range"][1],
            )

    # ------------------------------------------------------------------
    # Part 4: Convergence Analysis
    # ------------------------------------------------------------------
    print("\n--- Part 4: Convergence Analysis ---")

    targets = benchmark["analysis_targets"]

    convergence_depth = find_convergence_depth(
        results,
        true_shannon,
        targets["shannon_convergence"]["convergence_threshold_pct"],
    )
    print(f"  Shannon converges at: {convergence_depth} reads")
    exp_conv = targets["shannon_convergence"]["expected_convergence_depth"]
    # Slack 1.5x (tightened from 2x): stochastic convergence should
    # not deviate beyond 50% of expected depth with 50 replicates.
    check_true(
        f"Shannon converges by ~{convergence_depth} reads",
        0 < convergence_depth <= exp_conv * 1.5,
    )

    saturation_depth = find_genus_saturation_depth(results)
    print(f"  Genus saturation at:  {saturation_depth} reads")
    exp_sat = targets["genus_saturation"]["expected_saturation_depth"]
    check_true(
        f"Genus saturation by ~{saturation_depth} reads",
        0 < saturation_depth <= exp_sat * 1.5,
    )

    phylum_depth = find_phylum_completeness_depth(results)
    print(f"  All phyla detected at: {phylum_depth} reads")
    exp_phylum = targets["phylum_stability"]["expected_stable_depth"]
    tol_reads = targets["phylum_stability"]["tolerance_reads"]
    check_true(
        f"All phyla complete by {phylum_depth} reads",
        0 < phylum_depth <= exp_phylum + tol_reads,
    )

    # ------------------------------------------------------------------
    # Part 5: Noise Floor Characterization
    # ------------------------------------------------------------------
    print("\n--- Part 5: Noise Floor ---")

    high_depth_result = depth_to_result.get(100_000)
    if high_depth_result:
        print("  At 100,000 reads (near-complete sampling):")
        print(
            f"    Genera:   {high_depth_result['genera_detected']['mean']:.1f} "
            f"± {high_depth_result['genera_detected']['std']:.1f}"
        )
        print(
            f"    Shannon:  {high_depth_result['shannon']['mean']:.4f} "
            f"± {high_depth_result['shannon']['std']:.4f}"
        )

        pct_off = (
            abs(high_depth_result["shannon"]["mean"] - true_shannon)
            / true_shannon
            * 100
        )
        # 1% tolerance at 100k reads (tightened from 2%): with 100k
        # draws from 150 genera, sampling noise should be negligible.
        check_true(
            f"Shannon within 1% of true ({pct_off:.2f}%)",
            pct_off < 1.0,
        )

    genera_means = [r["genera_detected"]["mean"] for r in results]
    check_true(
        "Genera detected is monotonically increasing with depth",
        all(
            genera_means[i] <= genera_means[i + 1]
            for i in range(len(genera_means) - 1)
        ),
    )

    shannon_means = [r["shannon"]["mean"] for r in results]
    # Monotonic tolerance 0.005: sampling variance at low depth can
    # cause tiny non-monotonicity in Shannon.
    check_true(
        "Shannon diversity is monotonically increasing with depth",
        all(
            shannon_means[i] <= shannon_means[i + 1] + 0.005
            for i in range(len(shannon_means) - 1)
        ),
    )

    # ------------------------------------------------------------------
    # Key Findings
    # ------------------------------------------------------------------
    print(f"\n{'=' * 72}")
    print("KEY FINDINGS:")
    print(f"{'=' * 72}")

    print("\n1. Sequencing Depth Thresholds:")
    print(f"   All phyla detected:       {phylum_depth:>7d} reads")
    print(f"   Genus saturation:         {saturation_depth:>7d} reads")
    print(f"   Shannon convergence (5%): {convergence_depth:>7d} reads")

    if high_depth_result:
        print("\n2. Noise Floor at High Depth (100k reads):")
        print(
            f"   Genus detection:  {high_depth_result['genera_detected']['std']:.1f} genera"
        )
        print(
            f"   Shannon noise:    ±{high_depth_result['shannon']['std']:.4f}"
        )

    return print_summary("Exp 004: Sequencing Depth & Taxonomic Noise")


if __name__ == "__main__":
    sys.exit(main())
