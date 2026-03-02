#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
Exp 028 — NPU Anderson Regime Classification (Python baseline).

Classifies Anderson localization regimes (Localized / Critical / Extended)
using int8-quantized features (W, E, L) and a simple centroid classifier.
This baseline establishes ground truth for the Rust + AKD1000 NPU path.

Validation checks:
  1. CPU regime classification matches expected labels
  2. Quantization round-trip error within tolerance
  3. Training produces a 3×3 weight matrix
  4. CPU classifier accuracy is 100% on training set
  5. All three regime classes are covered
  6. Extended regime detected for weak disorder
  7. Localized regime detected for strong disorder
"""

import json
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from common import (
    check_approx,
    check_true,
    print_summary,
    reset_counters,
)


def load_benchmark():
    p = Path(__file__).resolve().parent / "benchmark_npu_anderson.json"
    with open(p) as f:
        return json.load(f)


def analytical_xi(disorder, energy, derrida_c=96.0):
    """Analytical localization length xi = C / W^2 at band center."""
    if disorder <= 0:
        return float("inf")
    return derrida_c / (disorder ** 2)


def classify_regime(disorder, energy, n_sites, derrida_c=96.0):
    """Classify Anderson regime from xi/L ratio."""
    xi = analytical_xi(disorder, energy, derrida_c)
    ratio = xi / n_sites
    if ratio < 0.5:
        return "Localized"
    elif ratio > 2.0:
        return "Extended"
    else:
        return "Critical"


def quantize_i8(val, lo, hi):
    """Map [lo, hi] -> [0, 127] with clamping."""
    n = np.clip((val - lo) / (hi - lo), 0.0, 1.0)
    return int(n * 127)


def dequantize_i8(q, lo, hi):
    """Map [0, 127] -> [lo, hi]."""
    return lo + (q / 127.0) * (hi - lo)


def quantize_features(disorder_w, energy_e, length_l, model):
    """Quantize (W, E, L) to int8 features."""
    q_w = quantize_i8(disorder_w, *model["quantization"]["W_range"])
    q_e = quantize_i8(energy_e, *model["quantization"]["E_range"])
    q_l = quantize_i8(length_l, *model["quantization"]["L_range"])
    return [q_w, q_e, q_l]


def train_centroid_classifier(disorders, n_sites, model, derrida_c=96.0):
    """Train a simple centroid classifier: mean quantized features per class."""
    class_names = ["Localized", "Critical", "Extended"]
    sums = {c: np.zeros(3) for c in class_names}
    counts = {c: 0 for c in class_names}

    for w in disorders:
        features = quantize_features(w, 0.0, n_sites, model)
        regime = classify_regime(w, 0.0, n_sites, derrida_c)
        sums[regime] += features
        counts[regime] += 1

    weights = np.zeros((3, 3), dtype=np.int8)
    for i, c in enumerate(class_names):
        if counts[c] > 0:
            mean = sums[c] / counts[c]
            weights[i] = np.clip(mean, -128, 127).astype(np.int8)

    return weights


def classify_with_weights(features, weights):
    """Classify by finding the class whose centroid is closest (dot product)."""
    features = np.array(features, dtype=np.float64)
    scores = weights.astype(np.float64) @ features
    return int(np.argmax(scores))


def main():
    bench = load_benchmark()
    model = bench["model"]
    expected = bench["expected_results"]

    n_sites = model["n_sites"]
    energy = model["energy"]
    derrida_c = model["derrida_gardner_C"]
    disorders = model["disorders"]
    expected_regimes = expected["cpu_regimes"]

    reset_counters()

    print("=" * 72)
    print("groundSpring Exp 028: NPU Anderson Regime Classification")
    print("=" * 72)

    # Check 1: CPU regime classification
    cpu_regimes = [classify_regime(w, energy, n_sites, derrida_c) for w in disorders]
    check_true(
        f"CPU regimes match expected: {cpu_regimes}",
        cpu_regimes == expected_regimes,
    )

    # Check 2: Quantization round-trip error
    max_err = 0.0
    for w in disorders:
        features = quantize_features(w, energy, n_sites, model)
        w_deq = dequantize_i8(features[0], *model["quantization"]["W_range"])
        err = abs(w - w_deq) / max(abs(w), 1e-10)
        max_err = max(max_err, err)
    tol = expected["quantization_roundtrip_max_error"]
    check_approx("Quantization roundtrip max error", max_err, 0.0, tol)

    # Check 3: Training produces 3x3 weight matrix
    rng = np.random.default_rng(model.get("seed", 42))
    n_train = model["n_training_disorders"]
    w_min, w_max = model["training_W_min"], model["training_W_max"]
    train_disorders = rng.uniform(w_min, w_max, n_train)
    weights = train_centroid_classifier(train_disorders, n_sites, model, derrida_c)
    check_true(
        f"Classifier weight matrix shape: {weights.shape}",
        weights.shape == (3, 3),
    )

    # Check 4: CPU classifier accuracy on training data
    correct = 0
    for w in train_disorders:
        features = quantize_features(w, energy, n_sites, model)
        pred_idx = classify_with_weights(features, weights)
        true_label = classify_regime(w, energy, n_sites, derrida_c)
        pred_label = ["Localized", "Critical", "Extended"][pred_idx]
        if pred_label == true_label:
            correct += 1
    accuracy = correct / len(train_disorders)
    check_true(
        f"CPU accuracy >= {expected['cpu_accuracy_min']:.0%}: {accuracy:.2%}",
        accuracy >= expected["cpu_accuracy_min"],
    )

    # Check 5: All three regime classes covered
    unique_regimes = set(cpu_regimes)
    check_true(
        f"Regime coverage >= {expected['regime_coverage_min']} classes",
        len(unique_regimes) >= expected["regime_coverage_min"],
    )

    # Check 6: Extended detected for weak disorder
    check_true(
        "Extended for W=0.1",
        classify_regime(0.1, energy, n_sites, derrida_c) == "Extended",
    )

    # Check 7: Localized detected for strong disorder
    check_true(
        "Localized for W=10",
        classify_regime(10.0, energy, n_sites, derrida_c) == "Localized",
    )

    return print_summary("Exp 028: NPU Anderson Regime Classification")


if __name__ == "__main__":
    sys.exit(main())
