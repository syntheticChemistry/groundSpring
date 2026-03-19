// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! RAWR — Resampling with Analytical Weights for Reproducibility.
//!
//! Bayesian bootstrap confidence intervals using Dirichlet(1,...,1) weights
//! (Wang et al. 2021, Bioinformatics/ISMB).
//!
//! Extracted from `bootstrap.rs` (V116) for cohesion: RAWR is a distinct
//! algorithm from the percentile bootstrap despite sharing the same result
//! type ([`BootstrapResult`]).
//!
//! # barracuda delegation
//!
//! When the `barracuda` feature is enabled, `rawr_mean` delegates to
//! `barracuda::stats::rawr_mean()` (absorbed in barraCuda S66).

use crate::bootstrap::{BootstrapResult, percentile_ci, validate_bootstrap_inputs};
use crate::prng::DefaultRng;

/// Cap for -ln(0) fallback when Exp(1) variate would be infinite.
const EXP_VARIATE_CAP: f64 = 30.0;

/// RAWR (Bayesian bootstrap) confidence interval for the mean.
///
/// Generates Dirichlet(1,...,1) weights (via normalized Exp(1) variates)
/// and computes the weighted mean for each replicate.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::rawr_mean` (absorbed in barraCuda S66).
///
/// # Errors
///
/// Returns [`InputError`](crate::error::InputError) if `data` is empty
/// or `confidence` is outside (0, 1).
pub fn rawr_mean(
    data: &[f64],
    n_replicates: usize,
    confidence: f64,
    seed: u64,
) -> Result<BootstrapResult, crate::error::InputError> {
    validate_bootstrap_inputs(data, 1, confidence)?;

    #[cfg(feature = "barracuda")]
    {
        use crate::bootstrap::from_barracuda_ci;
        if let Ok(ci) = barracuda::stats::rawr_mean(data, n_replicates, confidence, seed) {
            return Ok(from_barracuda_ci(&ci));
        }
    }

    Ok(rawr_mean_cpu(data, n_replicates, confidence, seed))
}

fn rawr_mean_cpu(data: &[f64], n_replicates: usize, confidence: f64, seed: u64) -> BootstrapResult {
    let n = data.len();
    let mut rng = DefaultRng::new(seed);
    let mut means = Vec::with_capacity(n_replicates);

    for _ in 0..n_replicates {
        let mut weights = Vec::with_capacity(n);
        let mut wsum = 0.0;
        for _ in 0..n {
            let u = rng.next_f64();
            let w = if u > 0.0 { -u.ln() } else { EXP_VARIATE_CAP };
            weights.push(w);
            wsum += w;
        }

        let mut weighted_mean = 0.0;
        for (j, &d) in data.iter().enumerate() {
            weighted_mean = (weights[j] / wsum).mul_add(d, weighted_mean);
        }
        means.push(weighted_mean);
    }

    percentile_ci(&means, n_replicates, confidence)
}

#[cfg(test)]
#[expect(clippy::float_cmp, reason = "bitwise determinism test")]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;
    use crate::prng::Xorshift64;

    #[test]
    fn rawr_deterministic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r1 = rawr_mean(&data, 500, 0.95, 42).unwrap();
        let r2 = rawr_mean(&data, 500, 0.95, 42).unwrap();
        assert_eq!(r1.estimate, r2.estimate);
    }

    #[test]
    fn rawr_ci_contains_true_mean() {
        let mut rng = Xorshift64::new(42);
        let data: Vec<f64> = (0..200)
            .map(|_| (rng.next_f64() - 0.5).mul_add(4.0, 5.0))
            .collect();
        let r = rawr_mean(&data, 1000, 0.95, 123).unwrap();
        assert!(
            r.ci_lower <= 5.0 && 5.0 <= r.ci_upper,
            "RAWR CI [{}, {}] should contain 5.0",
            r.ci_lower,
            r.ci_upper
        );
    }

    #[test]
    fn rawr_ci_width_positive() {
        let data: Vec<f64> = (0..50).map(f64::from).collect();
        let r = rawr_mean(&data, 500, 0.95, 42).unwrap();
        assert!(r.ci_upper > r.ci_lower, "RAWR CI width must be positive");
        assert!(r.std_error > 0.0);
    }

    #[test]
    fn rawr_different_seeds_differ() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let r1 = rawr_mean(&data, 500, 0.95, 42).unwrap();
        let r2 = rawr_mean(&data, 500, 0.95, 99).unwrap();
        assert!(
            r1.ci_lower.to_bits() != r2.ci_lower.to_bits()
                || r1.ci_upper.to_bits() != r2.ci_upper.to_bits()
                || r1.estimate.to_bits() != r2.estimate.to_bits(),
            "at least one field should differ between seeds"
        );
    }

    #[test]
    fn rawr_mean_cpu_direct() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r = rawr_mean_cpu(&data, 200, 0.95, 42);
        assert!(r.ci_lower < r.ci_upper);
        assert!((r.estimate - 3.0).abs() < 1.0);
    }

    #[test]
    fn rawr_single_value() {
        let data = vec![42.0];
        let r = rawr_mean(&data, 200, 0.95, 1).unwrap();
        assert!((r.estimate - 42.0).abs() < crate::tol::EXACT);
    }

    #[test]
    fn rawr_ci_narrows_with_n() {
        let mut rng = crate::prng::Xorshift64::new(77);
        let small: Vec<f64> = (0..20).map(|_| rng.next_f64() * 10.0).collect();
        let large: Vec<f64> = (0..200).map(|_| rng.next_f64() * 10.0).collect();
        let r_small = rawr_mean(&small, 500, 0.95, 42).unwrap();
        let r_large = rawr_mean(&large, 500, 0.95, 42).unwrap();
        assert!(
            r_large.ci_upper - r_large.ci_lower < r_small.ci_upper - r_small.ci_lower,
            "larger sample should have narrower RAWR CI"
        );
    }

    #[test]
    fn rawr_empty_data_returns_error() {
        let empty: Vec<f64> = vec![];
        assert!(rawr_mean(&empty, 100, 0.95, 42).is_err());
    }

    #[test]
    fn rawr_different_from_bootstrap() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = crate::bootstrap::bootstrap_mean(&data, 500, 0.95, 42).unwrap();
        let r = rawr_mean(&data, 500, 0.95, 42).unwrap();
        let b_width = b.ci_upper - b.ci_lower;
        let r_width = r.ci_upper - r.ci_lower;
        assert!(
            (b.estimate - r.estimate).abs() < 0.5 || b_width != r_width,
            "methods should produce comparable estimates but may differ in CI width"
        );
        assert!(
            b_width > 0.0 && r_width > 0.0,
            "both methods must produce non-degenerate CIs"
        );
    }
}
