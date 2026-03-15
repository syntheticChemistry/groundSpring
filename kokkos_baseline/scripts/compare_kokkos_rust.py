#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
Compare Kokkos Tier 1 baseline against Rust (groundSpring) results.

Usage:
    # Run Kokkos baseline, capture JSON tail:
    ./build/kokkos_baseline 2>&1 | python3 scripts/compare_kokkos_rust.py

    # Or from saved JSON:
    python3 scripts/compare_kokkos_rust.py < kokkos_results.json

    # With Rust comparison (run Rust benchmarks first):
    python3 scripts/compare_kokkos_rust.py --rust-json rust_results.json < kokkos_results.json
"""

import argparse
import json
import sys


def load_json_from_stdin():
    """Extract JSON object from stdin (may have non-JSON preamble)."""
    lines = sys.stdin.read().splitlines()
    json_start = None
    for i, line in enumerate(lines):
        if line.strip() == "{":
            json_start = i
            break
    if json_start is None:
        print("ERROR: No JSON found in input", file=sys.stderr)
        sys.exit(1)
    return json.loads("\n".join(lines[json_start:]))


def print_comparison(kokkos_data, rust_data=None):
    """Print formatted comparison table."""
    backend = kokkos_data.get("_provenance", {}).get("backend", "unknown")
    kokkos_ver = kokkos_data.get("_provenance", {}).get("kokkos_version", "?")

    print(f"\n{'=' * 72}")
    print(f"groundSpring: Kokkos vs Rust Performance Comparison")
    print(f"  Kokkos backend: {backend}  (v{kokkos_ver})")
    print(f"{'=' * 72}\n")

    header = f"{'Kernel':<30} {'Kokkos (us)':>12} {'Rust (us)':>12} {'Speedup':>10}"
    print(header)
    print("-" * len(header))

    for kr in kokkos_data.get("results", []):
        name = kr["name"]
        k_us = kr["elapsed_us"]

        if rust_data:
            rr = next((r for r in rust_data.get("results", [])
                        if r["name"] == name), None)
            if rr:
                r_us = rr["elapsed_us"]
                if k_us > 0:
                    speedup = r_us / k_us
                    label = f"{speedup:.2f}x"
                    if speedup > 1.0:
                        label += " (Kokkos faster)"
                    else:
                        label += " (Rust faster)"
                else:
                    label = "N/A"
                print(f"{name:<30} {k_us:>12.0f} {r_us:>12.0f} {label:>10}")
            else:
                print(f"{name:<30} {k_us:>12.0f} {'—':>12} {'—':>10}")
        else:
            print(f"{name:<30} {k_us:>12.0f} {'(run Rust)':>12} {'—':>10}")

    # Value comparison (correctness)
    if rust_data:
        print(f"\n{'Kernel':<30} {'Kokkos value':>20} {'Rust value':>20} {'Diff':>12}")
        print("-" * 82)
        for kr in kokkos_data.get("results", []):
            name = kr["name"]
            k_val = kr["value"]
            rr = next((r for r in rust_data.get("results", [])
                        if r["name"] == name), None)
            if rr:
                r_val = rr["value"]
                diff = abs(k_val - r_val)
                print(f"{name:<30} {k_val:>20.12e} {r_val:>20.12e} {diff:>12.2e}")

    print()


def main():
    parser = argparse.ArgumentParser(
        description="Compare Kokkos vs Rust groundSpring benchmarks")
    parser.add_argument("--rust-json", help="Path to Rust benchmark JSON")
    args = parser.parse_args()

    kokkos_data = load_json_from_stdin()

    rust_data = None
    if args.rust_json:
        with open(args.rust_json) as f:
            rust_data = json.load(f)

    print_comparison(kokkos_data, rust_data)


if __name__ == "__main__":
    main()
