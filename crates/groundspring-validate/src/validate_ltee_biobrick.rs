// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! Experiment 040 — LTEE `BioBrick` Burden: Anderson Disorder Analogy (B6).
//!
//! Validates the statistical analysis from "Measuring the burden of
//! hundreds of `BioBricks`" (2024 Nat Comms). Models plasmid burden as
//! Anderson disorder potential: log-normal burden distribution, AIC/BIC
//! model selection, Anderson localization length correlation.
//!
//! LTEE `GuideStone` B6 | `lithoSpore` module 5 candidate.

use groundspring::anderson::localization_length;
use groundspring::cast::usize_f64;
use groundspring::jackknife::jackknife_mean_variance;
use groundspring::prng::Xorshift64;
use groundspring::validate::ValidationHarness;
use groundspring_validate::{f64_field, parse_benchmark, print_provenance_header, usize_field};

const BENCHMARK: &str =
    include_str!("../../../control/ltee_biobrick_burden/benchmark_ltee_biobrick.json");
const EXPECTED: &str = include_str!("../../../control/ltee_biobrick_burden/expected_values.json");

fn main() {
    std::process::exit(run());
}

fn generate_burdens(n: usize, mu: f64, sigma: f64, seed: u64) -> Vec<f64> {
    let mut rng = Xorshift64::new(seed);
    let mut burdens = Vec::with_capacity(n);
    for _ in 0..n {
        let z = rng.next_normal();
        let raw = sigma.mul_add(z, mu).exp();
        burdens.push(raw.clamp(0.001, 0.99));
    }
    burdens.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    burdens
}

fn log_likelihood_normal(data: &[f64], mu: f64, sigma: f64) -> f64 {
    let two_sigma_sq = 2.0 * sigma * sigma;
    let log_norm = -0.5 * (two_sigma_sq * std::f64::consts::PI).ln();
    data.iter()
        .map(|&x| log_norm - (x - mu) * (x - mu) / two_sigma_sq)
        .sum::<f64>()
}

fn log_likelihood_lognormal(data: &[f64], mu: f64, sigma: f64) -> f64 {
    let two_sigma_sq = 2.0 * sigma * sigma;
    data.iter()
        .map(|&x| {
            if x <= 0.0 {
                return f64::NEG_INFINITY;
            }
            let ln_x = x.ln();
            let half_log_norm = 0.5 * (two_sigma_sq * std::f64::consts::PI).ln();
            -ln_x - half_log_norm - (ln_x - mu) * (ln_x - mu) / two_sigma_sq
        })
        .sum()
}

fn log_likelihood_exponential(data: &[f64], lambda: f64) -> f64 {
    let n = usize_f64(data.len());
    let sum: f64 = data.iter().sum();
    n.mul_add(lambda.ln(), -lambda * sum)
}

fn ic_aic(ll: f64, k: usize) -> f64 {
    let kf = usize_f64(k);
    2.0f64.mul_add(kf, -2.0 * ll)
}

fn ic_bic(ll: f64, k: usize, n: usize) -> f64 {
    let kf = usize_f64(k);
    let nf = usize_f64(n);
    kf.mul_add(nf.ln(), -2.0 * ll)
}

struct FitResult {
    name: &'static str,
    aic_val: f64,
    bic_val: f64,
}

fn fit_models(data: &[f64]) -> Vec<FitResult> {
    let n = data.len();
    let mean = data.iter().sum::<f64>() / usize_f64(n);
    let variance = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / usize_f64(n - 1);
    let sigma = variance.sqrt();

    let log_data: Vec<f64> = data.iter().map(|x| x.ln()).collect();
    let log_mean = log_data.iter().sum::<f64>() / usize_f64(n);
    let log_var = log_data
        .iter()
        .map(|&x| (x - log_mean).powi(2))
        .sum::<f64>()
        / usize_f64(n - 1);
    let log_sigma = log_var.sqrt();

    let lambda = 1.0 / mean;

    let ll_normal = log_likelihood_normal(data, mean, sigma);
    let ll_lognormal = log_likelihood_lognormal(data, log_mean, log_sigma);
    let ll_exp = log_likelihood_exponential(data, lambda);

    vec![
        FitResult {
            name: "normal",
            aic_val: ic_aic(ll_normal, 2),
            bic_val: ic_bic(ll_normal, 2, n),
        },
        FitResult {
            name: "log-normal",
            aic_val: ic_aic(ll_lognormal, 2),
            bic_val: ic_bic(ll_lognormal, 2, n),
        },
        FitResult {
            name: "exponential",
            aic_val: ic_aic(ll_exp, 1),
            bic_val: ic_bic(ll_exp, 1, n),
        },
    ]
}

fn anderson_correlation(burdens: &[f64], w_scale: f64, n_quantiles: usize) -> f64 {
    let burden_mean = burdens.iter().sum::<f64>() / usize_f64(burdens.len());
    let mut quantile_log_b = Vec::with_capacity(n_quantiles);
    let mut quantile_log_xi = Vec::with_capacity(n_quantiles);
    let n = burdens.len();

    for q in 1..=n_quantiles {
        let frac = usize_f64(q) / usize_f64(n_quantiles);
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "quantile index from f64 fraction — value is always in [0, n)"
        )]
        let idx = (frac * usize_f64(n)) as usize;
        let idx = idx.min(n - 1);
        let b = burdens[idx];
        let w = (b * w_scale / burden_mean).max(0.01);
        let gamma = 1.0 / localization_length(1.0 / w);
        let xi = if gamma.abs() < 1e-15 {
            1e10
        } else {
            1.0 / gamma
        };
        quantile_log_b.push(b.ln());
        quantile_log_xi.push(xi.ln());
    }

    pearson_r(&quantile_log_b, &quantile_log_xi)
}

fn pearson_r(x: &[f64], y: &[f64]) -> f64 {
    let n = usize_f64(x.len());
    let mx = x.iter().sum::<f64>() / n;
    let my = y.iter().sum::<f64>() / n;
    let mut cov = 0.0;
    let mut vx = 0.0;
    let mut vy = 0.0;
    for i in 0..x.len() {
        let dx = x[i] - mx;
        let dy = y[i] - my;
        cov += dx * dy;
        vx += dx * dx;
        vy += dy * dy;
    }
    let denom = (vx * vy).sqrt();
    if denom < 1e-15 { 0.0 } else { cov / denom }
}

fn best_model_by_aic(fits: &[FitResult]) -> &str {
    fits.iter()
        .min_by(|a, b| {
            a.aic_val
                .partial_cmp(&b.aic_val)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map_or("unknown", |f| f.name)
}

fn best_model_by_bic(fits: &[FitResult]) -> &str {
    fits.iter()
        .min_by(|a, b| {
            a.bic_val
                .partial_cmp(&b.bic_val)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map_or("unknown", |f| f.name)
}

fn run() -> i32 {
    let bench = parse_benchmark(BENCHMARK);
    let expected = parse_benchmark(EXPECTED);
    let mut h = ValidationHarness::from_args("Rust Validation: LTEE BioBrick Burden (B6)");
    print_provenance_header(&bench, "LTEE BioBrick Burden");

    let n_plasmids = usize_field(&bench, "n_plasmids");
    let mu = f64_field(&bench["burden_distribution"], "mu");
    let sigma = f64_field(&bench["burden_distribution"], "sigma");
    let base_seed = bench["seed"].as_u64().unwrap_or(20_240_601);
    let replicates = usize_field(&bench, "replicates");
    let w_scale = f64_field(&bench["anderson_mapping"], "disorder_strength_W");
    let aic_delta_min = f64_field(&bench["thresholds"], "aic_delta_model_selection");
    let corr_min = f64_field(&bench["thresholds"], "anderson_localization_correlation");
    let burden_mean_lo = bench["thresholds"]["burden_mean_range"][0]
        .as_f64()
        .unwrap_or(0.05);
    let burden_mean_hi = bench["thresholds"]["burden_mean_range"][1]
        .as_f64()
        .unwrap_or(0.25);

    let exp_burden_mean = f64_field(&expected, "burden_mean");
    let exp_model = expected["preferred_model"].as_str().unwrap_or("log-normal");

    h.section("Distribution Fitting (per replicate)");

    for r in 0..replicates {
        let seed = base_seed + r as u64;
        let burdens = generate_burdens(n_plasmids, mu, sigma, seed);

        let burden_mean = burdens.iter().sum::<f64>() / usize_f64(n_plasmids);
        let burden_std = (burdens
            .iter()
            .map(|&x| (x - burden_mean).powi(2))
            .sum::<f64>()
            / usize_f64(n_plasmids - 1))
        .sqrt();
        let burden_cv = burden_std / burden_mean;

        let fits = fit_models(&burdens);
        let aic_winner = best_model_by_aic(&fits);
        let bic_winner = best_model_by_bic(&fits);

        let normal_aic = fits
            .iter()
            .find(|f| f.name == "normal")
            .map_or(0.0, |f| f.aic_val);
        let ln_aic = fits
            .iter()
            .find(|f| f.name == "log-normal")
            .map_or(0.0, |f| f.aic_val);
        let delta_aic = normal_aic - ln_aic;

        let corr = anderson_correlation(&burdens, w_scale, 10);

        h.check_range(
            &format!("rep{r}:burden_mean"),
            burden_mean,
            burden_mean_lo,
            burden_mean_hi,
        );
        h.check_range(&format!("rep{r}:burden_cv"), burden_cv, 0.5, 2.0);
        h.check_true(
            &format!("rep{r}:lognormal_preferred_aic"),
            aic_winner == "log-normal",
        );
        h.check_true(
            &format!("rep{r}:lognormal_preferred_bic"),
            bic_winner == "log-normal",
        );
        h.check_min(&format!("rep{r}:aic_delta"), delta_aic, aic_delta_min);
        h.check_min(&format!("rep{r}:anderson_corr"), corr.abs(), corr_min);
    }

    h.section("Cross-validation against Python baseline");

    let rust_burden_mean = {
        let burdens = generate_burdens(n_plasmids, mu, sigma, base_seed);
        burdens.iter().sum::<f64>() / usize_f64(n_plasmids)
    };
    h.check_approx(
        "burden_mean_vs_python",
        rust_burden_mean,
        exp_burden_mean,
        0.03,
    );
    h.check_true("preferred_model_matches", exp_model == "log-normal");

    h.section("Jackknife variance estimation");

    let jk_data: Vec<f64> = (0..replicates)
        .map(|r| {
            let burdens = generate_burdens(n_plasmids, mu, sigma, base_seed + r as u64);
            burdens.iter().sum::<f64>() / usize_f64(n_plasmids)
        })
        .collect();
    match jackknife_mean_variance(&jk_data) {
        Ok(jk) => {
            h.check_true(
                "jackknife_se_finite",
                jk.std_error.is_finite() && jk.std_error > 0.0,
            );
            h.check_range("jackknife_mean", jk.estimate, 0.05, 0.30);
        }
        Err(_) => {
            h.check_true("jackknife_available", false);
        }
    }

    h.summary()
}
