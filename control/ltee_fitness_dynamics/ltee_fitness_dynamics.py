#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
Experiment 036 — LTEE Fitness Dynamics (Wiser et al. 2013).

Reproduces the central model-selection analysis from:
  Wiser MJ, Ribeck N, Lenski RE (2013) Long-Term Dynamics of Adaptation
  in Asexual Populations. Science 342(6164):1364-1367.

Three fitness trajectory models are compared:
  1. Power-law:    w(t) = 1 + A·t^b         (unbounded increase)
  2. Hyperbolic:   w(t) = 1 + a·t/(1 + b·t) (asymptotic plateau)
  3. Logarithmic:  w(t) = 1 + c·ln(t)       (slow unbounded)

Model selection uses AIC and BIC on jackknifed population data.

This is the ecosystem critical path: B2 reproduction → lithoSpore module 1.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np
from scipy.optimize import curve_fit

SCRIPT_DIR = Path(__file__).resolve().parent
BENCHMARK = SCRIPT_DIR / "benchmark_ltee_fitness.json"


def load_benchmark():
    with open(BENCHMARK) as f:
        return json.load(f)


# ── Fitness models ────────────────────────────────────────────────────

def power_law(t, a, b):
    """w(t) = 1 + A·t^b for t > 0."""
    return 1.0 + a * np.power(np.maximum(t, 1e-12), b)


def hyperbolic(t, a, b):
    """w(t) = 1 + a·t/(1 + b·t)."""
    return 1.0 + a * t / (1.0 + b * t)


def logarithmic(t, c, d):
    """w(t) = 1 + c·ln(t) + d for t > 0."""
    return 1.0 + c * np.log(np.maximum(t, 1e-12)) + d


MODELS = {
    "power_law": (power_law, [0.01, 0.5], 2),
    "hyperbolic": (hyperbolic, [1e-3, 1e-4], 2),
    "logarithmic": (logarithmic, [0.1, 0.0], 2),
}


# ── Synthetic data generation ─────────────────────────────────────────

def generate_ltee_data(cfg):
    """Generate synthetic LTEE fitness trajectories from published params."""
    rng = np.random.default_rng(cfg["seed"])
    gens = np.array(cfg["generations"], dtype=np.float64)
    params = cfg["power_law_params"]
    alpha, beta = params["alpha"], params["beta"]
    sigma = cfg["noise_sigma"]
    n_pop = cfg["n_populations"]

    populations = []
    for i in range(n_pop):
        pop_alpha = alpha * (1.0 + 0.15 * rng.standard_normal())
        pop_beta = beta * (1.0 + 0.08 * rng.standard_normal())
        pop_beta = np.clip(pop_beta, 0.1, 0.9)

        fitness = np.empty_like(gens)
        for j, t in enumerate(gens):
            if t == 0:
                fitness[j] = 1.0
            else:
                true_w = (1.0 + pop_alpha * t) ** pop_beta
                fitness[j] = true_w + sigma * rng.standard_normal()
                fitness[j] = max(fitness[j], 1.0)
        populations.append(fitness)

    return gens, np.array(populations)


# ── Model fitting ─────────────────────────────────────────────────────

def fit_model(gens, fitness_mean, model_name):
    """Fit a single model; return (params, rss, r_squared, aic, bic)."""
    func, p0, k = MODELS[model_name]
    mask = gens > 0
    t = gens[mask]
    y = fitness_mean[mask]
    n = len(t)

    try:
        popt, _ = curve_fit(func, t, y, p0=p0, maxfev=10000)
    except RuntimeError:
        return None

    predicted = func(t, *popt)
    ss_res = float(np.sum((y - predicted) ** 2))
    ss_tot = float(np.sum((y - np.mean(y)) ** 2))
    r_squared = 1.0 - ss_res / ss_tot if ss_tot > 0 else 1.0

    if ss_res <= 0:
        ss_res = 1e-30

    aic_val = n * np.log(ss_res / n) + 2 * k
    bic_val = n * np.log(ss_res / n) + k * np.log(n)

    return {
        "model": model_name,
        "params": [float(p) for p in popt],
        "k": k,
        "rss": float(ss_res),
        "r_squared": r_squared,
        "aic": float(aic_val),
        "bic": float(bic_val),
    }


# ── Jackknife analysis ───────────────────────────────────────────────

def jackknife_model_params(gens, populations, model_name):
    """Delete-one jackknife across populations for model parameter variance."""
    n_pop = len(populations)
    param_estimates = []

    for i in range(n_pop):
        subset = np.delete(populations, i, axis=0)
        mean_fitness = np.mean(subset, axis=0)
        result = fit_model(gens, mean_fitness, model_name)
        if result is not None:
            param_estimates.append(result["params"])

    if not param_estimates:
        return None

    param_arr = np.array(param_estimates)
    full_mean = np.mean(populations, axis=0)
    full_result = fit_model(gens, full_mean, model_name)
    if full_result is None:
        return None

    n = len(param_estimates)
    jk_var = ((n - 1) / n) * np.sum(
        (param_arr - np.mean(param_arr, axis=0)) ** 2, axis=0
    )
    jk_se = np.sqrt(jk_var)

    return {
        "full_params": full_result["params"],
        "jackknife_se": [float(s) for s in jk_se],
        "jackknife_variance": [float(v) for v in jk_var],
        "n_jackknife": n,
    }


# ── Main ──────────────────────────────────────────────────────────────

def main():
    bench = load_benchmark()
    cfg = bench["model"]
    expected = bench["expected_results"]

    print("=" * 72)
    print("  Experiment 036: LTEE Fitness Dynamics — Wiser et al. (2013)")
    print("  B2 Reproduction | lithoSpore Module 1 | Ecosystem Critical Path")
    print("=" * 72)

    gens, populations = generate_ltee_data(cfg)
    mean_fitness = np.mean(populations, axis=0)

    print(f"\n  Populations: {len(populations)}")
    print(f"  Generations: {gens.tolist()}")
    print(f"  Mean fitness at 50k: {mean_fitness[-1]:.4f}")

    checks_passed = 0
    checks_total = 0

    # Fitness increasing
    checks_total += 1
    all_increasing = all(
        np.all(np.diff(pop[1:]) >= -0.1) for pop in populations
    )
    status = "PASS" if all_increasing else "FAIL"
    print(f"\n  [{'PASS' if all_increasing else 'FAIL'}] All populations fitness increasing")
    if all_increasing:
        checks_passed += 1

    # Fit all three models
    print("\n  Model Comparison (mean fitness across 12 populations):")
    print(f"  {'Model':<15} {'R²':>8} {'AIC':>10} {'BIC':>10} {'Params'}")
    print(f"  {'-'*60}")

    results = {}
    for name in ["power_law", "hyperbolic", "logarithmic"]:
        result = fit_model(gens, mean_fitness, name)
        if result is not None:
            results[name] = result
            params_str = ", ".join(f"{p:.6g}" for p in result["params"])
            print(
                f"  {name:<15} {result['r_squared']:>8.5f} "
                f"{result['aic']:>10.3f} {result['bic']:>10.3f} "
                f"[{params_str}]"
            )

    # AIC selection
    checks_total += 1
    if results:
        best_aic = min(results.values(), key=lambda r: r["aic"])
        aic_pass = best_aic["model"] == expected["best_model_aic"]
        status = "PASS" if aic_pass else "FAIL"
        print(f"\n  [{status}] Best model by AIC: {best_aic['model']} "
              f"(expected: {expected['best_model_aic']})")
        if aic_pass:
            checks_passed += 1

    # BIC selection
    checks_total += 1
    if results:
        best_bic = min(results.values(), key=lambda r: r["bic"])
        bic_pass = best_bic["model"] == expected["best_model_bic"]
        status = "PASS" if bic_pass else "FAIL"
        print(f"  [{status}] Best model by BIC: {best_bic['model']} "
              f"(expected: {expected['best_model_bic']})")
        if bic_pass:
            checks_passed += 1

    # Power-law R²
    checks_total += 1
    if "power_law" in results:
        pl_r2 = results["power_law"]["r_squared"]
        r2_pass = pl_r2 >= expected["power_law_r_squared_min"]
        status = "PASS" if r2_pass else "FAIL"
        print(f"  [{status}] Power-law R² = {pl_r2:.5f} "
              f"(min: {expected['power_law_r_squared_min']})")
        if r2_pass:
            checks_passed += 1

    # AIC power_law < hyperbolic
    checks_total += 1
    if "power_law" in results and "hyperbolic" in results:
        aic_lt = results["power_law"]["aic"] < results["hyperbolic"]["aic"]
        status = "PASS" if aic_lt else "FAIL"
        print(f"  [{status}] AIC(power_law) < AIC(hyperbolic): "
              f"{results['power_law']['aic']:.3f} vs {results['hyperbolic']['aic']:.3f}")
        if aic_lt:
            checks_passed += 1

    # BIC power_law < hyperbolic
    checks_total += 1
    if "power_law" in results and "hyperbolic" in results:
        bic_lt = results["power_law"]["bic"] < results["hyperbolic"]["bic"]
        status = "PASS" if bic_lt else "FAIL"
        print(f"  [{status}] BIC(power_law) < BIC(hyperbolic): "
              f"{results['power_law']['bic']:.3f} vs {results['hyperbolic']['bic']:.3f}")
        if bic_lt:
            checks_passed += 1

    # Power-law exponent in expected range
    checks_total += 1
    if "power_law" in results:
        b_exp = results["power_law"]["params"][1]
        exp_range = expected["power_law_exponent_range"]
        in_range = exp_range[0] <= b_exp <= exp_range[1]
        status = "PASS" if in_range else "FAIL"
        print(f"  [{status}] Power-law exponent b = {b_exp:.4f} "
              f"(expected: [{exp_range[0]}, {exp_range[1]}])")
        if in_range:
            checks_passed += 1

    # Jackknife on power-law exponent
    print("\n  Jackknife Analysis (delete-one across 12 populations):")
    jk = jackknife_model_params(gens, populations, "power_law")
    if jk is not None:
        print(f"  Full estimate: A={jk['full_params'][0]:.6g}, b={jk['full_params'][1]:.4f}")
        print(f"  Jackknife SE:  A_se={jk['jackknife_se'][0]:.6g}, b_se={jk['jackknife_se'][1]:.4f}")

        checks_total += 1
        b_se = jk["jackknife_se"][1]
        se_pass = b_se <= expected["jackknife_exponent_se_max"]
        status = "PASS" if se_pass else "FAIL"
        print(f"  [{status}] Jackknife SE(b) = {b_se:.4f} "
              f"(max: {expected['jackknife_exponent_se_max']})")
        if se_pass:
            checks_passed += 1

    # Determinism check
    checks_total += 1
    gens2, pop2 = generate_ltee_data(cfg)
    mean2 = np.mean(pop2, axis=0)
    det_pass = np.allclose(mean_fitness, mean2, atol=1e-12)
    status = "PASS" if det_pass else "FAIL"
    print(f"\n  [{status}] Deterministic (same seed → same data)")
    if det_pass:
        checks_passed += 1

    # Summary
    print(f"\n{'=' * 72}")
    print(f"  RESULT: {checks_passed}/{checks_total} checks PASS")
    print(f"{'=' * 72}")

    # Write expected values JSON for lithoSpore absorption
    expected_values = {
        "experiment": "036_ltee_fitness_dynamics",
        "paper": "Wiser2013",
        "paper_id": "B2",
        "litho_module": 1,
        "generations": gens.tolist(),
        "mean_fitness": mean_fitness.tolist(),
        "model_fits": results,
        "jackknife": jk,
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
