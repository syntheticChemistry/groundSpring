// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Bias-variance error decomposition.
//!
//! The core groundSpring operation: decompose total RMSE into a correctable
//! systematic component (bias) and an irreducible random component (noise).
//!
//! ```text
//! RMSE² = MBE² + σ²(random)
//! ```

/// Result of a bias-variance decomposition.
#[derive(Debug, Clone, Copy)]
pub struct Decomposition {
    /// Mean Bias Error (systematic, correctable).
    pub bias: f64,
    /// Absolute value of bias.
    pub bias_abs: f64,
    /// Standard deviation of random noise component.
    pub random_std: f64,
    /// Total RMSE (input).
    pub total_rmse: f64,
    /// Bias squared.
    pub bias_sq: f64,
    /// Random noise variance.
    pub variance: f64,
    /// Fraction of total error that is systematic bias.
    pub bias_fraction: f64,
    /// Fraction of total error that is random noise.
    pub noise_fraction: f64,
}

/// Decompose total RMSE into bias and random noise components.
///
/// Given Mean Bias Error (MBE) and Root Mean Square Error (RMSE):
///
/// - `random_std = sqrt(RMSE² - MBE²)`
/// - `bias_fraction = MBE² / RMSE²`
///
/// # Panics
///
/// Never panics; guards against negative variance from floating-point error.
///
/// # Examples
///
/// ```
/// let d = groundspring::decompose::decompose_error(0.5, 1.0);
/// assert!((d.bias_fraction - 0.25).abs() < 1e-12);
/// assert!((d.noise_fraction - 0.75).abs() < 1e-12);
/// assert!((d.random_std - 0.75_f64.sqrt()).abs() < 1e-12);
/// ```
#[must_use]
pub fn decompose_error(mbe: f64, rmse: f64) -> Decomposition {
    let bias_sq = mbe.powi(2);
    let total_sq = rmse.powi(2);
    let variance = (total_sq - bias_sq).max(0.0);
    let random_std = variance.sqrt();
    let bias_fraction = if total_sq > 0.0 {
        bias_sq / total_sq
    } else {
        0.0
    };

    Decomposition {
        bias: mbe,
        bias_abs: mbe.abs(),
        random_std,
        total_rmse: rmse,
        bias_sq,
        variance,
        bias_fraction,
        noise_fraction: 1.0 - bias_fraction,
    }
}

/// Result of noise floor analysis.
#[derive(Debug, Clone, Copy)]
pub struct NoiseFloor {
    /// RMSE before correction.
    pub factory_rmse: f64,
    /// RMSE after soil-specific correction.
    pub corrected_rmse: f64,
    /// Error component that was removed by correction.
    pub removed_error: f64,
    /// Irreducible noise floor (= corrected RMSE).
    pub noise_floor: f64,
    /// Percentage reduction in RMSE from correction.
    pub reduction_pct: f64,
}

/// Quantify how much error was removable vs irreducible.
///
/// After site-specific calibration, the corrected RMSE is the noise floor.
#[must_use]
pub fn noise_floor_reduction(factory_rmse: f64, corrected_rmse: f64) -> NoiseFloor {
    let factory_sq = factory_rmse.powi(2);
    let corrected_sq = corrected_rmse.powi(2);
    let diff_sq = corrected_sq.mul_add(-1.0, factory_sq);
    let removed = if diff_sq > 0.0 { diff_sq.sqrt() } else { 0.0 };
    let reduction_pct = if factory_rmse > 0.0 {
        (1.0 - corrected_rmse / factory_rmse) * 100.0
    } else {
        0.0
    };

    NoiseFloor {
        factory_rmse,
        corrected_rmse,
        removed_error: removed,
        noise_floor: corrected_rmse,
        reduction_pct,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn pure_bias() {
        let d = decompose_error(0.05, 0.05);
        // Pure bias: RMSE=MBE ⇒ variance=0 exactly; EXACT is double-precision algebraic precision.
        assert!((d.random_std).abs() < tol::EXACT);
        assert!((d.bias_fraction - 1.0).abs() < tol::EXACT);
    }

    #[test]
    fn pure_noise() {
        let d = decompose_error(0.0, 0.03);
        // Pure noise: MBE=0 ⇒ random_std=RMSE exactly; EXACT is double-precision algebraic precision.
        assert!((d.random_std - 0.03).abs() < tol::EXACT);
        assert!(d.bias_fraction.abs() < tol::EXACT);
    }

    #[test]
    fn pythagorean_identity() {
        for (mbe, rmse) in [(-0.01, 0.017), (-0.03, 0.039), (0.03, 0.038)] {
            let d = decompose_error(mbe, rmse);
            let reconstructed = (d.bias_sq + d.variance).sqrt();
            // RMSE² = MBE² + σ² is exact; ANALYTICAL absorbs floating-point in sqrt/sum reconstruction.
            assert!(
                (reconstructed - rmse).abs() < tol::ANALYTICAL,
                "RMSE² = MBE² + σ² must hold"
            );
        }
    }

    #[test]
    fn dong2020_cs616_sand() {
        let d = decompose_error(-0.01, 0.017);
        // Dong2020 literature values; LITERATURE/DECOMPOSITION allow for rounding in published digits.
        assert!((d.random_std - 0.0137).abs() < tol::LITERATURE);
        assert!((d.bias_fraction - 0.346).abs() < tol::DECOMPOSITION);
    }

    #[test]
    fn dong2020_ec5_sandy_clay_loam() {
        let d = decompose_error(-0.05, 0.057);
        // Dong2020 literature values; LITERATURE/DECOMPOSITION allow for rounding in published digits.
        assert!((d.random_std - 0.0274).abs() < tol::LITERATURE);
        assert!((d.bias_fraction - 0.7695).abs() < tol::DECOMPOSITION);
    }

    #[test]
    fn noise_floor_improvement() {
        let nf = noise_floor_reduction(0.039, 0.012);
        assert!(nf.removed_error > 0.0);
        assert!(nf.reduction_pct > 0.0);
        // Corrected RMSE is passed through; EXACT is algebraic precision.
        assert!((nf.noise_floor - 0.012).abs() < tol::EXACT);
        let reconstructed = nf.removed_error.hypot(nf.noise_floor);
        // factory_rmse² = removed² + corrected²; ANALYTICAL absorbs floating-point in hypot.
        assert!((reconstructed - 0.039).abs() < tol::ANALYTICAL);
    }
}
