#!/usr/bin/env python3
"""
groundSpring Experiment 004 — Sequencing Depth and Taxonomic Noise

Simulates rarefaction from a reference soil microbiome community to answer:
  1. At what sequencing depth does taxonomy become stable?
  2. How does Shannon diversity converge with increasing reads?
  3. What is the noise floor for genus-level assignments?
  4. When does more sequencing stop improving the answer?

This is the biological equivalent of "how many decimal places matter?" from
Experiment 003's sensor uncertainty analysis.

Cross-spring with wetSpring: informs the DADA2 pipeline's minimum depth
requirements and helps distinguish real biological signal from sequencing
noise.

Method:
  - Generate a synthetic but realistic soil microbiome (150 genera, 8 phyla)
  - Simulate multinomial sampling at increasing read depths
  - Track: genera detected, Shannon diversity, phylum completeness
  - Repeat with N replicates per depth for confidence intervals

No external data needed — this is a computational experiment on a known
reference community.
"""

import json
import math
import sys
from pathlib import Path

import numpy as np
from scipy import stats


# ---------------------------------------------------------------------------
# Community generation
# ---------------------------------------------------------------------------

def generate_reference_community(config: dict, seed: int = 42) -> dict:
    """
    Generate a synthetic soil microbiome community.

    Uses log-normal abundance distribution within each phylum,
    resulting in a realistic rank-abundance curve.
    """
    rng = np.random.default_rng(seed)

    genera = []
    phylum_assignments = []
    true_abundances = []

    for phylum in config["dominant_phyla"]:
        n_gen = phylum["n_genera"]
        total_abund = phylum["relative_abundance"]

        # Log-normal relative abundances within phylum
        raw = rng.lognormal(mean=0, sigma=1.5, size=n_gen)
        raw = raw / raw.sum() * total_abund

        for i in range(n_gen):
            genera.append(f"{phylum['name']}_genus_{i+1:03d}")
            phylum_assignments.append(phylum["name"])
            true_abundances.append(raw[i])

    # Normalize to sum to 1.0
    true_abundances = np.array(true_abundances)
    true_abundances = true_abundances / true_abundances.sum()

    return {
        "genera": genera,
        "phylum_assignments": phylum_assignments,
        "true_abundances": true_abundances,
        "n_genera": len(genera),
        "n_phyla": len(set(phylum_assignments)),
    }


# ---------------------------------------------------------------------------
# Rarefaction simulation
# ---------------------------------------------------------------------------

def simulate_sampling(true_abundances: np.ndarray, depth: int,
                       rng: np.random.Generator) -> np.ndarray:
    """
    Simulate sequencing at a given depth via multinomial sampling.

    Each read is assigned to a genus proportional to its true abundance.
    Returns count vector.
    """
    return rng.multinomial(depth, true_abundances)


def compute_shannon(counts: np.ndarray) -> float:
    """Shannon diversity index H' = -Σ(pi * ln(pi))."""
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


def rarefaction_at_depth(community: dict, depth: int,
                          n_replicates: int = 50,
                          seed: int = 42) -> dict:
    """
    Run rarefaction analysis at a specific sequencing depth.

    Returns statistics across replicates.
    """
    rng = np.random.default_rng(seed + depth)
    true_abund = community["true_abundances"]

    genera_detected = []
    phyla_detected = []
    shannon_values = []

    phylum_array = np.array(community["phylum_assignments"])
    unique_phyla = list(set(community["phylum_assignments"]))

    for rep in range(n_replicates):
        counts = simulate_sampling(true_abund, depth, rng)

        # Genera detected
        n_detected = int(np.sum(counts > 0))
        genera_detected.append(n_detected)

        # Phyla detected
        detected_mask = counts > 0
        detected_phyla = set(phylum_array[detected_mask])
        phyla_detected.append(len(detected_phyla))

        # Shannon diversity
        h = compute_shannon(counts)
        shannon_values.append(h)

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
            "all_detected_pct": float(np.mean(
                [p == community["n_phyla"] for p in phyla_detected]
            ) * 100),
        },
        "shannon": {
            "mean": float(np.mean(shannon_values)),
            "std": float(np.std(shannon_values)),
        },
    }


# ---------------------------------------------------------------------------
# Convergence analysis
# ---------------------------------------------------------------------------

def find_convergence_depth(rarefaction_results: list,
                            true_shannon: float,
                            threshold_pct: float = 5.0) -> int:
    """
    Find the depth at which Shannon diversity stabilizes within
    threshold_pct of the true value.
    """
    for result in rarefaction_results:
        obs_h = result["shannon"]["mean"]
        pct_diff = abs(obs_h - true_shannon) / true_shannon * 100
        if pct_diff <= threshold_pct:
            return result["depth"]
    return -1  # never converged


def find_genus_saturation_depth(rarefaction_results: list) -> int:
    """
    Find depth where genus count stops increasing meaningfully
    (< 5% new genera per doubling).
    """
    for i in range(1, len(rarefaction_results)):
        prev = rarefaction_results[i - 1]
        curr = rarefaction_results[i]

        if prev["genera_detected"]["mean"] > 0:
            pct_increase = (
                (curr["genera_detected"]["mean"] - prev["genera_detected"]["mean"])
                / prev["genera_detected"]["mean"] * 100
            )
            depth_ratio = curr["depth"] / prev["depth"]

            # Normalize: % increase per doubling
            if depth_ratio > 1:
                pct_per_doubling = pct_increase / math.log2(depth_ratio)
            else:
                pct_per_doubling = 0

            if pct_per_doubling < 5.0:
                return curr["depth"]

    return -1


def find_phylum_completeness_depth(rarefaction_results: list,
                                     completeness_pct: float = 95.0) -> int:
    """Find depth where all phyla are detected in ≥ completeness_pct of replicates."""
    for result in rarefaction_results:
        if result["phyla_detected"]["all_detected_pct"] >= completeness_pct:
            return result["depth"]
    return -1


# ---------------------------------------------------------------------------
# Validation harness
# ---------------------------------------------------------------------------

def check(label: str, computed: float, low: float, high: float) -> bool:
    ok = low <= computed <= high
    status = "PASS" if ok else "FAIL"
    print(f"  [{status}] {label}: {computed:.2f} "
          f"(expected [{low:.2f}, {high:.2f}])")
    return ok


def check_min(label: str, computed: float, minimum: float) -> bool:
    ok = computed >= minimum
    status = "PASS" if ok else "FAIL"
    print(f"  [{status}] {label}: {computed:.2f} (minimum {minimum:.2f})")
    return ok


def main():
    benchmark_path = Path(__file__).parent / "benchmark_sequencing_noise.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    total_passed = 0
    total_failed = 0

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

    # True Shannon diversity
    true_shannon = compute_shannon(
        (community["true_abundances"] * 1e8).astype(int)
    )
    print(f"  True Shannon H': {true_shannon:.4f}")

    # Check community properties
    total_abund = community["true_abundances"].sum()
    if abs(total_abund - 1.0) < 1e-10:
        print(f"  [PASS] Abundances sum to 1.0")
        total_passed += 1
    else:
        print(f"  [FAIL] Abundances sum to {total_abund}")
        total_failed += 1

    if community["n_genera"] == benchmark["reference_community"]["n_genera"]:
        print(f"  [PASS] Correct number of genera")
        total_passed += 1
    else:
        print(f"  [FAIL] Expected {benchmark['reference_community']['n_genera']} genera")
        total_failed += 1

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
        print(f"    Genera detected: {r['genera_detected']['mean']:.1f} "
              f"± {r['genera_detected']['std']:.1f} "
              f"(range {r['genera_detected']['min']}-{r['genera_detected']['max']})")
        print(f"    Phyla detected:  {r['phyla_detected']['mean']:.1f} "
              f"({r['phyla_detected']['all_detected_pct']:.0f}% complete)")
        print(f"    Shannon H':      {r['shannon']['mean']:.4f} "
              f"± {r['shannon']['std']:.4f}")

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
            if check(f"Genera at {depth_val} reads",
                     r["genera_detected"]["mean"],
                     exp["genera_detected_range"][0],
                     exp["genera_detected_range"][1]):
                total_passed += 1
            else:
                total_failed += 1

        if "shannon_range" in exp:
            if check(f"Shannon at {depth_val} reads",
                     r["shannon"]["mean"],
                     exp["shannon_range"][0],
                     exp["shannon_range"][1]):
                total_passed += 1
            else:
                total_failed += 1

    # ------------------------------------------------------------------
    # Part 4: Convergence Analysis
    # ------------------------------------------------------------------
    print("\n--- Part 4: Convergence Analysis ---")

    # Shannon convergence
    targets = benchmark["analysis_targets"]
    convergence_depth = find_convergence_depth(
        results, true_shannon,
        targets["shannon_convergence"]["convergence_threshold_pct"]
    )
    print(f"  Shannon converges at: {convergence_depth} reads")
    exp_conv = targets["shannon_convergence"]["expected_convergence_depth"]
    if convergence_depth > 0 and convergence_depth <= exp_conv * 2:
        print(f"  [PASS] Shannon converges by ~{convergence_depth} reads")
        total_passed += 1
    else:
        print(f"  [FAIL] Shannon convergence unexpected (expected ~{exp_conv})")
        total_failed += 1

    # Genus saturation
    saturation_depth = find_genus_saturation_depth(results)
    print(f"  Genus saturation at:  {saturation_depth} reads")
    exp_sat = targets["genus_saturation"]["expected_saturation_depth"]
    if saturation_depth > 0 and saturation_depth <= exp_sat * 2:
        print(f"  [PASS] Genus saturation by ~{saturation_depth} reads")
        total_passed += 1
    else:
        print(f"  [FAIL] Genus saturation unexpected (expected ~{exp_sat})")
        total_failed += 1

    # Phylum completeness
    phylum_depth = find_phylum_completeness_depth(results)
    print(f"  All phyla detected at: {phylum_depth} reads")
    exp_phylum = targets["phylum_stability"]["expected_stable_depth"]
    if phylum_depth > 0 and phylum_depth <= exp_phylum + targets["phylum_stability"]["tolerance_reads"]:
        print(f"  [PASS] All phyla complete by {phylum_depth} reads")
        total_passed += 1
    else:
        print(f"  [FAIL] Phylum completeness unexpected (expected ~{exp_phylum})")
        total_failed += 1

    # ------------------------------------------------------------------
    # Part 5: Noise Floor Characterization
    # ------------------------------------------------------------------
    print("\n--- Part 5: Noise Floor ---")

    print(f"\n  The 'noise floor' is the variability that remains even at")
    print(f"  high sequencing depth (sampling noise from multinomial).")
    print()

    high_depth_result = depth_to_result.get(100000)
    if high_depth_result:
        print(f"  At 100,000 reads (near-complete sampling):")
        print(f"    Genera:   {high_depth_result['genera_detected']['mean']:.1f} "
              f"± {high_depth_result['genera_detected']['std']:.1f}")
        print(f"    Shannon:  {high_depth_result['shannon']['mean']:.4f} "
              f"± {high_depth_result['shannon']['std']:.4f}")

        # Shannon should be within 2% of true at 100k reads
        pct_off = abs(high_depth_result["shannon"]["mean"] - true_shannon) / true_shannon * 100
        if pct_off < 2.0:
            print(f"    [PASS] Shannon within 2% of true ({pct_off:.2f}%)")
            total_passed += 1
        else:
            print(f"    [FAIL] Shannon {pct_off:.2f}% off from true")
            total_failed += 1

    # Monotonicity check: genera detected should increase with depth
    genera_means = [r["genera_detected"]["mean"] for r in results]
    is_monotonic = all(genera_means[i] <= genera_means[i+1]
                        for i in range(len(genera_means) - 1))
    if is_monotonic:
        print(f"\n  [PASS] Genera detected is monotonically increasing with depth")
        total_passed += 1
    else:
        print(f"\n  [FAIL] Genera detected is not monotonic!")
        total_failed += 1

    # Shannon should also be monotonically increasing
    shannon_means = [r["shannon"]["mean"] for r in results]
    is_shannon_monotonic = all(shannon_means[i] <= shannon_means[i+1] + 0.01
                                for i in range(len(shannon_means) - 1))
    if is_shannon_monotonic:
        print(f"  [PASS] Shannon diversity is monotonically increasing with depth")
        total_passed += 1
    else:
        print(f"  [FAIL] Shannon diversity is not monotonic!")
        total_failed += 1

    # ------------------------------------------------------------------
    # Part 6: Key Findings
    # ------------------------------------------------------------------
    print(f"\n{'=' * 72}")
    print("KEY FINDINGS:")
    print(f"{'=' * 72}")

    print(f"\n1. Sequencing Depth Thresholds:")
    print(f"   All phyla detected:       {phylum_depth:>7d} reads")
    print(f"   Genus saturation:         {saturation_depth:>7d} reads")
    print(f"   Shannon convergence (5%): {convergence_depth:>7d} reads")

    print(f"\n2. Noise Floor at High Depth (100k reads):")
    if high_depth_result:
        print(f"   Genus detection:  {high_depth_result['genera_detected']['std']:.1f} genera (stochastic)")
        print(f"   Shannon noise:    ±{high_depth_result['shannon']['std']:.4f}")

    print(f"\n3. Implications for wetSpring:")
    print(f"   - Below {phylum_depth} reads, phylum-level analysis is unreliable")
    print(f"   - Below {convergence_depth} reads, diversity comparisons are noisy")
    print(f"   - Above {saturation_depth} reads, diminishing returns for genus discovery")
    print(f"   - For pond crash detection, minimum ~{convergence_depth} reads recommended")
    print(f"   - Low-depth samples need noise-aware interpretation (groundSpring↔neuralSpring)")

    # ------------------------------------------------------------------
    # Summary
    # ------------------------------------------------------------------
    total = total_passed + total_failed
    print(f"\n{'=' * 72}")
    print(f"TOTAL: {total_passed}/{total} PASS, {total_failed}/{total} FAIL")
    print(f"{'=' * 72}")

    return 0 if total_failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
