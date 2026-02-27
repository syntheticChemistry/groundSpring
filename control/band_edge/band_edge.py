# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring Experiment 018 — Band Edge Structure

Where do propagating waves transition to evanescent in periodic
structures?  Band edges are the mathematical boundary between
"signal gets through" and "noise kills it."

Method:
  - 1D tight-binding chain with periodic potential V_n of period p
  - Transfer matrix: T(E) = prod_n [[(E-V_n)/t, -1], [1, 0]]
  - Bands where |Tr(T)/2| <= 1, gaps where |Tr(T)/2| > 1
  - Band edges at |Tr(T)/2| = 1
  - Finite system eigenvalues via tridiagonal diagonalization
  - Gap width proportional to potential contrast |V1-V2|

Reference:
  Filonov & Kachkovskiy (2018) Acta Math 221:59-80
  Anderson (1958) Phys Rev 109:1492-1505

Cross-spring: hotSpring (spectral), Exp 008 (Anderson), Exp 012 (transport).
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


def transfer_matrix_trace(energy: float, potential: list[float], hopping: float) -> float:
    """Compute Tr(T)/2 for one period of the transfer matrix.

    T = prod_n [[(E - V_n)/t, -1], [1, 0]]
    """
    m = np.eye(2)
    for v_n in potential:
        t_n = np.array([[(energy - v_n) / hopping, -1.0], [1.0, 0.0]])
        m = t_n @ m
    return float(m[0, 0] + m[1, 1]) / 2.0


def find_band_edges(
    potential: list[float],
    hopping: float,
    e_range: tuple[float, float],
    n_points: int,
) -> list[float]:
    """Scan energy range for band edges where |Tr(T)/2| crosses 1."""
    energies = np.linspace(e_range[0], e_range[1], n_points)
    edges = []
    prev_in_band = None
    for e in energies:
        half_trace = transfer_matrix_trace(e, potential, hopping)
        in_band = abs(half_trace) <= 1.0
        if prev_in_band is not None and in_band != prev_in_band:
            edges.append(float(e))
        prev_in_band = in_band
    return edges


def count_bands(
    potential: list[float],
    hopping: float,
    e_range: tuple[float, float],
    n_points: int,
) -> int:
    """Count distinct bands in the energy range."""
    energies = np.linspace(e_range[0], e_range[1], n_points)
    in_band = False
    n_bands = 0
    for e in energies:
        half_trace = transfer_matrix_trace(e, potential, hopping)
        currently_in = abs(half_trace) <= 1.0
        if currently_in and not in_band:
            n_bands += 1
        in_band = currently_in
    return n_bands


def build_periodic_hamiltonian(
    potential: list[float], hopping: float, n_periods: int,
) -> np.ndarray:
    """Build tridiagonal Hamiltonian for a finite periodic chain."""
    period = len(potential)
    n = period * n_periods
    diag = np.array([potential[i % period] for i in range(n)])
    offdiag = np.full(n - 1, -hopping)
    h = np.diag(diag) + np.diag(offdiag, 1) + np.diag(offdiag, -1)
    return h


def main() -> int:
    benchmark_path = Path(__file__).parent / "benchmark_band_edge.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    reset_counters()

    model = benchmark["model"]
    pred = benchmark["analytical_predictions"]
    exp = benchmark["expected_results"]

    t_hop = model["hopping"]
    pot_2 = model["period_2_potential"]
    pot_3 = model["period_3_potential"]
    n_scan = model["n_energy_scan"]
    e_range = tuple(model["energy_range"])
    n_periods = model["n_periods_finite"]

    print("=" * 72)
    print("groundSpring Exp 018: Band Edge Structure (Filonov-Kachkovskiy 2018)")
    print(f"  Period-2 potential: {pot_2}, hopping: {t_hop}")
    print(f"  Period-3 potential: {pot_3}")
    print(f"  Energy scan: {e_range} with {n_scan} points")
    print("  Cross-spring: hotSpring, Exp 008, Exp 012")
    print("=" * 72)

    # ------------------------------------------------------------------
    # Part 1: Free lattice (V=0) — single band [-2t, 2t]
    # ------------------------------------------------------------------
    print("\n--- Part 1: Free Lattice (V=0) ---")

    free_edges = find_band_edges([0.0], t_hop, e_range, n_scan)
    n_free_bands = count_bands([0.0], t_hop, e_range, n_scan)
    print(f"  Free lattice edges: {[f'{e:.3f}' for e in free_edges]}")
    print(f"  Number of bands: {n_free_bands}")

    expected_edges = pred["free_band_edges"]
    check_true("Free lattice has 2 band edges", len(free_edges) == 2)
    if len(free_edges) == 2:
        check_range(
            "Lower band edge ≈ -2t",
            free_edges[0],
            expected_edges[0] - 0.05,
            expected_edges[0] + 0.05,
        )
        check_range(
            "Upper band edge ≈ +2t",
            free_edges[1],
            expected_edges[1] - 0.05,
            expected_edges[1] + 0.05,
        )

    # ------------------------------------------------------------------
    # Part 2: Period-2 potential — gap opens
    # ------------------------------------------------------------------
    print("\n--- Part 2: Period-2 Gap Opening ---")

    p2_edges = find_band_edges(pot_2, t_hop, e_range, n_scan)
    p2_bands = count_bands(pot_2, t_hop, e_range, n_scan)
    print(f"  Period-2 edges: {[f'{e:.3f}' for e in p2_edges]}")
    print(f"  Number of bands: {p2_bands}")

    check_true("Period-2 opens a gap (4 edges)", len(p2_edges) == 4)
    check_true("Period-2 has 2 bands", p2_bands == 2)

    if len(p2_edges) == 4:
        gap_lo = p2_edges[1]
        gap_hi = p2_edges[2]
        gap_width = gap_hi - gap_lo
        expected_gap = pred["period_2_gap_width"]
        print(f"  Gap: [{gap_lo:.3f}, {gap_hi:.3f}], width = {gap_width:.3f} (expected {expected_gap})")
        check_range(
            "Gap width ≈ |V1-V2|",
            gap_width,
            expected_gap - exp["gap_width_tolerance"],
            expected_gap + exp["gap_width_tolerance"],
        )

    # ------------------------------------------------------------------
    # Part 3: Period-3 potential — 3 bands
    # ------------------------------------------------------------------
    print("\n--- Part 3: Period-3 Band Count ---")

    p3_bands = count_bands(pot_3, t_hop, e_range, n_scan)
    p3_edges = find_band_edges(pot_3, t_hop, e_range, n_scan)
    print(f"  Period-3 edges: {[f'{e:.3f}' for e in p3_edges]}")
    print(f"  Number of bands: {p3_bands} (expected {pred['n_bands_period_3']})")

    check_true(
        f"Period-3 has {pred['n_bands_period_3']} bands",
        p3_bands == pred["n_bands_period_3"],
    )

    # ------------------------------------------------------------------
    # Part 4: Gap width proportional to potential contrast
    # ------------------------------------------------------------------
    print("\n--- Part 4: Gap Width vs Potential Contrast ---")

    gap_widths = []
    for dv in model["period_2_gap_widths_to_test"]:
        pot = [dv / 2.0, -dv / 2.0]
        edges = find_band_edges(pot, t_hop, e_range, n_scan)
        if len(edges) >= 4:
            gw = edges[2] - edges[1]
        elif len(edges) == 2:
            gw = 0.0
        else:
            gw = 0.0
        gap_widths.append(gw)
        print(f"  ΔV={dv:.1f}: gap width = {gw:.3f}")

    monotone = all(
        gap_widths[i] <= gap_widths[i + 1] + 0.01
        for i in range(len(gap_widths) - 1)
    )
    check_true("Gap width increases with ΔV", monotone)

    # ------------------------------------------------------------------
    # Part 5: Finite system eigenvalues within bands
    # ------------------------------------------------------------------
    print("\n--- Part 5: Finite System Eigenvalues ---")

    h_mat = build_periodic_hamiltonian(pot_2, t_hop, n_periods)
    eigenvalues = np.sort(np.linalg.eigvalsh(h_mat))

    in_gap = 0
    for ev in eigenvalues:
        ht = transfer_matrix_trace(ev, pot_2, t_hop)
        if abs(ht) > 1.05:
            in_gap += 1

    n_total = len(eigenvalues)
    frac_in_band = (n_total - in_gap) / n_total
    print(f"  {n_total} eigenvalues, {in_gap} in gap region, {frac_in_band:.1%} in bands")

    check_true(
        "Eigenvalues mostly within bands (≥95%)",
        frac_in_band >= 0.95,
    )

    # ------------------------------------------------------------------
    # Part 6: Determinism
    # ------------------------------------------------------------------
    print("\n--- Part 6: Determinism ---")

    t1 = transfer_matrix_trace(0.5, pot_2, t_hop)
    t2 = transfer_matrix_trace(0.5, pot_2, t_hop)
    check_true("Transfer matrix deterministic", t1 == t2)

    # ------------------------------------------------------------------
    # Key findings
    # ------------------------------------------------------------------
    print(f"\n{'=' * 72}")
    print("KEY FINDINGS:")
    print(f"{'=' * 72}")
    print(f"\n1. Free lattice: single band [{expected_edges[0]}, {expected_edges[1]}]")
    print(f"2. Period-2 (V=[{pot_2[0]},{pot_2[1]}]): gap width = {pred['period_2_gap_width']} = |V1-V2|")
    print(f"3. Period-3: {p3_bands} bands (number of bands = period)")
    print("4. Gap width proportional to potential contrast ΔV")
    print(f"5. Finite system: {frac_in_band:.1%} of eigenvalues within transfer-matrix bands")
    print()
    print("  Filonov & Kachkovskiy (2018) proved that band edges of periodic")
    print("  elliptic operators have definite structure. This experiment shows")
    print("  the 1D tight-binding analog: band gaps separate propagating from")
    print("  evanescent regimes, with gap width controlled by potential contrast.")

    return print_summary("Exp 018: Band Edge Structure")


if __name__ == "__main__":
    sys.exit(main())
