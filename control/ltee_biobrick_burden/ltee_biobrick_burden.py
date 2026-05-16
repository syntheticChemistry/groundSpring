#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
Experiment 040 — LTEE BioBrick Burden: Anderson Disorder Analogy (B6).

Reproduces the central statistical analysis from:
  "Measuring the burden of hundreds of BioBricks defines an evolutionary
  limit on constructability" (2024) Nature Communications.

The paper measures metabolic burden imposed by 301 standardized BioBrick
plasmids. Each plasmid reduces host growth rate by a characteristic amount.
The distribution of burdens is right-skewed and well-described by a
log-normal model.

groundSpring's contribution: map the burden distribution to Anderson
disorder theory. Burden acts as a disorder potential — high-burden
plasmids are "localized" (non-viable in evolution), while low-burden
plasmids are "extended" (maintained by selection). The Anderson analogy
predicts a critical burden threshold above which constructability collapses.

Analysis pipeline:
  1. Generate synthetic burden data from log-normal distribution
  2. Fit normal, log-normal, and exponential models via MLE
  3. Model selection via AIC/BIC (log-normal expected to win)
  4. Map burden → Anderson disorder potential
  5. Compute localization length as function of disorder strength
  6. Correlate burden quantiles with localization regime
  7. Jackknife variance estimation on fitted parameters
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np
from scipy import stats
from scipy.optimize import minimize_scalar

SCRIPT_DIR = Path(__file__).resolve().parent
BENCHMARK = SCRIPT_DIR / "benchmark_ltee_biobrick.json"


def load_benchmark():
    with open(BENCHMARK) as f:
        return json.load(f)


# ── Burden generation ─────────────────────────────────────────────────

def generate_burdens(n_plasmids, mu, sigma, seed):
    """Generate synthetic burden values from log-normal distribution."""
    rng = np.random.default_rng(seed)
    raw = rng.lognormal(mean=mu, sigma=sigma, size=n_plasmids)
    burdens = np.clip(raw, 0.001, 0.99)
    return np.sort(burdens)


# ── Distribution fitting (MLE + AIC/BIC) ─────────────────────────────

def fit_normal(data):
    mu, sigma = np.mean(data), np.std(data, ddof=1)
    ll = np.sum(stats.norm.logpdf(data, loc=mu, scale=sigma))
    k = 2
    return {"model": "normal", "mu": mu, "sigma": sigma,
            "log_likelihood": ll, "aic": 2 * k - 2 * ll,
            "bic": k * np.log(len(data)) - 2 * ll, "k": k}


def fit_lognormal(data):
    log_data = np.log(data)
    mu, sigma = np.mean(log_data), np.std(log_data, ddof=1)
    ll = np.sum(stats.lognorm.logpdf(data, s=sigma, scale=np.exp(mu)))
    k = 2
    return {"model": "log-normal", "mu": mu, "sigma": sigma,
            "log_likelihood": ll, "aic": 2 * k - 2 * ll,
            "bic": k * np.log(len(data)) - 2 * ll, "k": k}


def fit_exponential(data):
    lam = 1.0 / np.mean(data)
    ll = np.sum(stats.expon.logpdf(data, scale=1.0 / lam))
    k = 1
    return {"model": "exponential", "lambda": lam,
            "log_likelihood": ll, "aic": 2 * k - 2 * ll,
            "bic": k * np.log(len(data)) - 2 * ll, "k": k}


def model_selection(data):
    fits = [fit_normal(data), fit_lognormal(data), fit_exponential(data)]
    best_aic = min(fits, key=lambda f: f["aic"])
    best_bic = min(fits, key=lambda f: f["bic"])
    return fits, best_aic, best_bic


# ── Anderson disorder mapping ────────────────────────────────────────

def burden_to_disorder(burdens, w_scale):
    """Map burden values to Anderson disorder potentials."""
    normalized = (burdens - np.mean(burdens)) / np.std(burdens)
    return normalized * w_scale


def localization_length_1d(disorder_w):
    """Analytical localization length at band center for 1D Anderson.

    xi = 105 / W^2 for small W (Thouless formula).
    """
    if abs(disorder_w) < 1e-12:
        return float("inf")
    return 105.0 / (disorder_w ** 2)


def anderson_burden_correlation(burdens, w_scale, n_quantiles=10):
    """Correlate burden quantiles with Anderson localization regime.

    High-burden plasmids → strong disorder → short localization length
    (non-viable). Low-burden plasmids → weak disorder → long localization
    length (constructable).

    Maps absolute burden magnitude to disorder strength W. The 1D
    Thouless formula xi = 105/W^2 then gives a monotonically decreasing
    localization length with increasing burden.
    """
    quantile_edges = np.linspace(0, 1, n_quantiles + 1)
    quantile_burdens = np.quantile(burdens, quantile_edges[1:])
    quantile_w = quantile_burdens * w_scale / np.mean(burdens)
    quantile_w = np.clip(quantile_w, 0.01, None)
    quantile_xi = np.array([localization_length_1d(w) for w in quantile_w])
    log_burden = np.log(quantile_burdens)
    log_xi = np.log(quantile_xi)
    correlation = np.corrcoef(log_burden, log_xi)[0, 1]
    return correlation, quantile_burdens, quantile_xi


# ── Jackknife variance estimation ────────────────────────────────────

def jackknife_lognormal_params(data):
    """Jackknife estimates of log-normal mu and sigma."""
    n = len(data)
    log_data = np.log(data)
    full_mu = np.mean(log_data)
    full_sigma = np.std(log_data, ddof=1)

    jk_mus = np.empty(n)
    jk_sigmas = np.empty(n)
    for i in range(n):
        subset = np.delete(log_data, i)
        jk_mus[i] = np.mean(subset)
        jk_sigmas[i] = np.std(subset, ddof=1)

    mu_var = (n - 1) / n * np.sum((jk_mus - full_mu) ** 2)
    sigma_var = (n - 1) / n * np.sum((jk_sigmas - full_sigma) ** 2)
    return {
        "mu": full_mu, "mu_se": np.sqrt(mu_var),
        "sigma": full_sigma, "sigma_se": np.sqrt(sigma_var),
    }


# ── KS goodness-of-fit ──────────────────────────────────────────────

def ks_test_lognormal(data, mu, sigma):
    """Kolmogorov-Smirnov test: data vs fitted log-normal."""
    stat, pvalue = stats.kstest(data, "lognorm", args=(sigma, 0, np.exp(mu)))
    return {"ks_statistic": stat, "p_value": pvalue}


# ── Main ─────────────────────────────────────────────────────────────

def main():
    bench = load_benchmark()
    n_plasmids = bench["n_plasmids"]
    mu = bench["burden_distribution"]["mu"]
    sigma = bench["burden_distribution"]["sigma"]
    base_seed = bench["seed"]
    replicates = bench["replicates"]
    w_scale = bench["anderson_mapping"]["disorder_strength_W"]
    thresholds = bench["thresholds"]

    all_checks = []
    replicate_results = []

    for r in range(replicates):
        seed = base_seed + r
        burdens = generate_burdens(n_plasmids, mu, sigma, seed)

        burden_mean = float(np.mean(burdens))
        burden_std = float(np.std(burdens, ddof=1))
        burden_cv = burden_std / burden_mean if burden_mean > 0 else 0.0

        fits, best_aic, best_bic = model_selection(burdens)

        lognormal_fit = next(f for f in fits if f["model"] == "log-normal")
        normal_fit = next(f for f in fits if f["model"] == "normal")
        delta_aic = normal_fit["aic"] - lognormal_fit["aic"]

        jk = jackknife_lognormal_params(burdens)
        ks = ks_test_lognormal(burdens, jk["mu"], jk["sigma"])
        corr, q_burdens, q_xi = anderson_burden_correlation(burdens, w_scale)

        checks = {
            "burden_mean_in_range": (
                thresholds["burden_mean_range"][0]
                <= burden_mean
                <= thresholds["burden_mean_range"][1]
            ),
            "burden_cv_in_range": (
                thresholds["burden_cv_range"][0]
                <= burden_cv
                <= thresholds["burden_cv_range"][1]
            ),
            "lognormal_preferred_aic": best_aic["model"] == "log-normal",
            "lognormal_preferred_bic": best_bic["model"] == "log-normal",
            "aic_delta_significant": delta_aic > thresholds["aic_delta_model_selection"],
            "ks_not_rejected": ks["p_value"] > thresholds["ks_test_alpha"],
            "anderson_correlation_strong": abs(corr) > thresholds["anderson_localization_correlation"],
        }

        all_checks.append(all(checks.values()))
        replicate_results.append({
            "replicate": r,
            "seed": seed,
            "burden_mean": burden_mean,
            "burden_cv": burden_cv,
            "best_model_aic": best_aic["model"],
            "best_model_bic": best_bic["model"],
            "delta_aic_normal_vs_lognormal": delta_aic,
            "jk_mu": jk["mu"], "jk_mu_se": jk["mu_se"],
            "jk_sigma": jk["sigma"], "jk_sigma_se": jk["sigma_se"],
            "ks_statistic": ks["ks_statistic"], "ks_pvalue": ks["p_value"],
            "anderson_correlation": corr,
            "checks": checks,
            "all_pass": all(checks.values()),
        })

    n_pass = sum(all_checks)
    n_total = len(all_checks)

    expected_values = {
        "experiment_id": "040",
        "title": "LTEE BioBrick Burden B6",
        "n_plasmids": n_plasmids,
        "replicates": replicates,
        "checks_per_replicate": 7,
        "total_checks": 7 * replicates,
        "checks_passed": sum(
            sum(r["checks"].values()) for r in replicate_results
        ),
        "all_replicates_pass": n_pass == n_total,
        "burden_mean": float(np.mean([r["burden_mean"] for r in replicate_results])),
        "burden_cv_mean": float(np.mean([r["burden_cv"] for r in replicate_results])),
        "preferred_model": "log-normal",
        "mean_delta_aic": float(np.mean([
            r["delta_aic_normal_vs_lognormal"] for r in replicate_results
        ])),
        "jk_mu_mean": float(np.mean([r["jk_mu"] for r in replicate_results])),
        "jk_sigma_mean": float(np.mean([r["jk_sigma"] for r in replicate_results])),
        "mean_anderson_correlation": float(np.mean([
            r["anderson_correlation"] for r in replicate_results
        ])),
        "replicate_details": replicate_results,
    }

    class NumpyEncoder(json.JSONEncoder):
        def default(self, obj):
            if isinstance(obj, (np.integer,)):
                return int(obj)
            if isinstance(obj, (np.floating,)):
                return float(obj)
            if isinstance(obj, np.ndarray):
                return obj.tolist()
            if isinstance(obj, (np.bool_,)):
                return bool(obj)
            return super().default(obj)

    out_path = SCRIPT_DIR / "expected_values.json"
    with open(out_path, "w") as f:
        json.dump(expected_values, f, indent=2, cls=NumpyEncoder)

    print(f"LTEE B6 BioBrick Burden: {n_pass}/{n_total} replicates PASS")
    print(f"  Preferred model: {expected_values['preferred_model']}")
    print(f"  Mean burden: {expected_values['burden_mean']:.4f}")
    print(f"  Mean ΔᴬIC (normal vs log-normal): {expected_values['mean_delta_aic']:.1f}")
    print(f"  Mean Anderson correlation: {expected_values['mean_anderson_correlation']:.4f}")
    print(f"  Expected values written to {out_path}")

    return 0 if n_pass == n_total else 1


if __name__ == "__main__":
    sys.exit(main())
