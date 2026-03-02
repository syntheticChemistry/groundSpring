// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Cross-substrate tolerance comparison — GPU output vs CPU reference.
//!
//! Each workload has a documented tolerance tier based on its mathematical
//! properties. Deterministic workloads (Anderson, eigenvalues, grid search)
//! should match to machine epsilon; stochastic workloads (Gillespie, bootstrap,
//! Monte Carlo) match to statistical tolerance.
//!
//! # Tolerance tiers
//!
//! | Tier | Tolerance | Workloads |
//! |------|-----------|-----------|
//! | Exact | 1e-12 | Anderson, eigenvalues, grid search, decompose |
//! | Analytical | 1e-10 | Tikhonov, band edge, transport |
//! | Statistical | 0.01 | Gillespie, bootstrap/RAWR, Wright-Fisher, MC ET₀ |
//! | Quantized | 0.25 | NPU int8 classification (round-trip error) |

/// Tolerance tier for cross-substrate comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToleranceTier {
    /// Deterministic f64 — should match to machine epsilon.
    Exact,
    /// Analytical result with ≤1 transcendental — small accumulated error.
    Analytical,
    /// Stochastic algorithm — match to statistical envelope.
    Statistical,
    /// Quantized inference — round-trip quantization error.
    Quantized,
}

impl ToleranceTier {
    /// Maximum relative tolerance for this tier.
    #[must_use]
    pub const fn relative_tolerance(self) -> f64 {
        match self {
            Self::Exact => 1e-12,
            Self::Analytical => 1e-10,
            Self::Statistical => 0.01,
            Self::Quantized => 0.25,
        }
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::Exact => "deterministic f64 (machine epsilon)",
            Self::Analytical => "analytical with transcendentals",
            Self::Statistical => "stochastic (statistical envelope)",
            Self::Quantized => "int8 quantized (round-trip error)",
        }
    }
}

/// Per-workload tolerance specification.
#[derive(Debug, Clone)]
pub struct WorkloadTolerance {
    /// Workload name (matches [`crate::workloads`] names).
    pub workload: &'static str,
    /// Which tolerance tier applies.
    pub tier: ToleranceTier,
    /// Justification for the chosen tier.
    pub justification: &'static str,
}

/// Compare a GPU result against a CPU reference within the given tolerance.
///
/// Returns `Ok(relative_diff)` if within tolerance, `Err(relative_diff)` if not.
///
/// # Errors
///
/// Returns `Err(relative_diff)` when the relative difference exceeds the
/// tolerance tier's threshold.
pub fn compare(cpu: f64, gpu: f64, tier: ToleranceTier) -> Result<f64, f64> {
    let tol = tier.relative_tolerance();
    let diff = if cpu.abs() > 1e-15 {
        ((gpu - cpu) / cpu).abs()
    } else {
        (gpu - cpu).abs()
    };
    if diff <= tol {
        Ok(diff)
    } else {
        Err(diff)
    }
}

/// Compare slices of GPU and CPU results, returning per-element results.
///
/// # Panics
///
/// Panics if `cpu` and `gpu` have different lengths.
#[must_use]
pub fn compare_all(cpu: &[f64], gpu: &[f64], tier: ToleranceTier) -> Vec<Result<f64, f64>> {
    assert_eq!(
        cpu.len(),
        gpu.len(),
        "CPU and GPU result lengths must match"
    );
    cpu.iter()
        .zip(gpu.iter())
        .map(|(&c, &g)| compare(c, g, tier))
        .collect()
}

/// Summary of a cross-substrate comparison.
#[derive(Debug)]
pub struct ComparisonSummary {
    /// Number of values compared.
    pub count: usize,
    /// Number that passed tolerance.
    pub passed: usize,
    /// Number that failed tolerance.
    pub failed: usize,
    /// Maximum relative difference observed.
    pub max_diff: f64,
    /// Mean relative difference.
    pub mean_diff: f64,
    /// Which tolerance tier was used.
    pub tier: ToleranceTier,
}

impl ComparisonSummary {
    /// Whether all comparisons passed.
    #[must_use]
    pub const fn all_pass(&self) -> bool {
        self.failed == 0
    }
}

/// Compare slices and return a summary.
#[must_use]
pub fn summarize(cpu: &[f64], gpu: &[f64], tier: ToleranceTier) -> ComparisonSummary {
    let results = compare_all(cpu, gpu, tier);
    let count = results.len();
    let mut passed = 0;
    let mut failed = 0;
    let mut max_diff = 0.0_f64;
    let mut sum_diff = 0.0_f64;

    for r in &results {
        let diff = match r {
            Ok(d) | Err(d) => *d,
        };
        if r.is_ok() {
            passed += 1;
        } else {
            failed += 1;
        }
        max_diff = max_diff.max(diff);
        sum_diff += diff;
    }

    let mean_diff = if count > 0 {
        #[expect(clippy::cast_precision_loss, reason = "count ≤ 19 workloads")]
        {
            sum_diff / count as f64
        }
    } else {
        0.0
    };

    ComparisonSummary {
        count,
        passed,
        failed,
        max_diff,
        mean_diff,
        tier,
    }
}

/// All 26 workload tolerance specifications.
#[must_use]
#[expect(
    clippy::too_many_lines,
    reason = "declarative tolerance table — one entry per workload"
)]
pub fn all_tolerances() -> Vec<WorkloadTolerance> {
    vec![
        WorkloadTolerance {
            workload: "Anderson transfer matrix (MC)",
            tier: ToleranceTier::Exact,
            justification: "deterministic transfer matrix product with fixed disorder potential",
        },
        WorkloadTolerance {
            workload: "Almost-Mathieu eigenvalues",
            tier: ToleranceTier::Exact,
            justification: "Sturm tridiag eigenvalue count is integer-exact",
        },
        WorkloadTolerance {
            workload: "Green-Kubo integration (f64)",
            tier: ToleranceTier::Analytical,
            justification: "trapezoidal accumulation with O(N) floating-point additions",
        },
        WorkloadTolerance {
            workload: "Anderson regime classification",
            tier: ToleranceTier::Quantized,
            justification: "int8 quantization introduces round-trip error up to 25%",
        },
        WorkloadTolerance {
            workload: "Diversity saturation prediction",
            tier: ToleranceTier::Quantized,
            justification: "int8 quantization introduces round-trip error up to 25%",
        },
        WorkloadTolerance {
            workload: "Bias-variance decomposition",
            tier: ToleranceTier::Exact,
            justification: "two scalar operations (MBE², RMSE² − MBE²)",
        },
        WorkloadTolerance {
            workload: "Finite-size extrapolation",
            tier: ToleranceTier::Analytical,
            justification: "linear regression with transcendental coordinate transform",
        },
        WorkloadTolerance {
            workload: "Freeze-out 2D grid fit",
            tier: ToleranceTier::Exact,
            justification: "deterministic chi-squared grid search over fixed parameter space",
        },
        WorkloadTolerance {
            workload: "Seismic 3D grid search",
            tier: ToleranceTier::Exact,
            justification: "deterministic RMS residual grid search over fixed lat/lon/depth",
        },
        WorkloadTolerance {
            workload: "Band edge energy scan",
            tier: ToleranceTier::Analytical,
            justification: "transfer matrix product per energy point with L sequential multiplies",
        },
        WorkloadTolerance {
            workload: "Quasispecies Wright-Fisher",
            tier: ToleranceTier::Statistical,
            justification: "multinomial sampling with PRNG-dependent trajectories",
        },
        WorkloadTolerance {
            workload: "Rare biosphere multinomial",
            tier: ToleranceTier::Statistical,
            justification: "batched multinomial resampling across replicates",
        },
        WorkloadTolerance {
            workload: "Gillespie SSA batch",
            tier: ToleranceTier::Statistical,
            justification: "stochastic simulation algorithm with exponential waiting times",
        },
        WorkloadTolerance {
            workload: "Spectral recon (Tikhonov)",
            tier: ToleranceTier::Analytical,
            justification: "Gauss-Jordan pivot with O(n³) accumulated rounding",
        },
        WorkloadTolerance {
            workload: "Jackknife leave-one-out",
            tier: ToleranceTier::Exact,
            justification: "deterministic leave-one-out mean computation",
        },
        WorkloadTolerance {
            workload: "MC ET₀ propagation",
            tier: ToleranceTier::Statistical,
            justification: "Monte Carlo sampling with Box-Muller noise injection",
        },
        WorkloadTolerance {
            workload: "Transport eigenvalues (tridiag)",
            tier: ToleranceTier::Analytical,
            justification: "implicit QL iteration with Givens rotations",
        },
        WorkloadTolerance {
            workload: "Wright-Fisher batch",
            tier: ToleranceTier::Statistical,
            justification: "binomial sampling in drift/selection simulation",
        },
        WorkloadTolerance {
            workload: "Bootstrap/RAWR resampling",
            tier: ToleranceTier::Statistical,
            justification: "PRNG-driven resampling with Dirichlet weights",
        },
        WorkloadTolerance {
            workload: "Shannon diversity (GPU fused)",
            tier: ToleranceTier::Analytical,
            justification: "FusedMapReduceF64 single-pass sum with ln() transcendental",
        },
        WorkloadTolerance {
            workload: "Simpson diversity (GPU fused)",
            tier: ToleranceTier::Analytical,
            justification: "FusedMapReduceF64 single-pass sum of squares",
        },
        WorkloadTolerance {
            workload: "Tissue Anderson (correlated 3D)",
            tier: ToleranceTier::Statistical,
            justification: "Gaussian-convolved disorder + Lanczos eigensolver accumulates rounding",
        },
        WorkloadTolerance {
            workload: "Barrier W_c finder",
            tier: ToleranceTier::Analytical,
            justification: "linear interpolation between sweep points (deterministic)",
        },
        WorkloadTolerance {
            workload: "MAE (GPU fused)",
            tier: ToleranceTier::Analytical,
            justification: "l1_norm / N — single fused reduction on residuals",
        },
        WorkloadTolerance {
            workload: "NSE/R² (GPU fused)",
            tier: ToleranceTier::Analytical,
            justification: "dual sum_of_squares reduction — SS_res and SS_tot",
        },
        WorkloadTolerance {
            workload: "Bistable ODE batch (GPU RK4)",
            tier: ToleranceTier::Statistical,
            justification: "fixed-step GPU RK4 may differ from CPU RK4 by FMA ordering",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match_passes() {
        assert!(compare(1.0, 1.0, ToleranceTier::Exact).is_ok());
    }

    #[test]
    fn exact_tiny_diff_passes() {
        let result = compare(1.0, 1.0 + 1e-14, ToleranceTier::Exact);
        assert!(result.is_ok());
    }

    #[test]
    fn exact_large_diff_fails() {
        let result = compare(1.0, 1.001, ToleranceTier::Exact);
        assert!(result.is_err());
    }

    #[test]
    fn statistical_moderate_diff_passes() {
        let result = compare(1.0, 1.005, ToleranceTier::Statistical);
        assert!(result.is_ok());
    }

    #[test]
    fn statistical_large_diff_fails() {
        let result = compare(1.0, 1.02, ToleranceTier::Statistical);
        assert!(result.is_err());
    }

    #[test]
    fn quantized_large_diff_passes() {
        let result = compare(1.0, 1.2, ToleranceTier::Quantized);
        assert!(result.is_ok());
    }

    #[test]
    fn compare_all_mixed() {
        let cpu = [1.0, 2.0, 3.0];
        let gpu = [1.0, 2.0, 4.0];
        let results = compare_all(&cpu, &gpu, ToleranceTier::Exact);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(results[2].is_err());
    }

    #[test]
    fn summary_reports_correctly() {
        let cpu = [1.0, 2.0, 3.0, 4.0];
        let gpu = [1.0, 2.0, 3.0, 4.0];
        let s = summarize(&cpu, &gpu, ToleranceTier::Exact);
        assert!(s.all_pass());
        assert_eq!(s.count, 4);
        assert_eq!(s.passed, 4);
        assert_eq!(s.failed, 0);
    }

    #[test]
    fn summary_with_failures() {
        let cpu = [1.0, 1.0];
        let gpu = [1.0, 2.0];
        let s = summarize(&cpu, &gpu, ToleranceTier::Exact);
        assert!(!s.all_pass());
        assert_eq!(s.failed, 1);
    }

    #[test]
    fn near_zero_cpu_uses_absolute() {
        let result = compare(0.0, 1e-14, ToleranceTier::Exact);
        assert!(result.is_ok());
    }

    #[test]
    fn all_tolerances_covers_twentysix_workloads() {
        let tols = all_tolerances();
        assert_eq!(tols.len(), 26);
    }

    #[test]
    fn tolerance_tiers_have_descriptions() {
        assert!(!ToleranceTier::Exact.description().is_empty());
        assert!(!ToleranceTier::Analytical.description().is_empty());
        assert!(!ToleranceTier::Statistical.description().is_empty());
        assert!(!ToleranceTier::Quantized.description().is_empty());
    }

    #[test]
    fn tolerance_ordering() {
        assert!(
            ToleranceTier::Exact.relative_tolerance()
                < ToleranceTier::Analytical.relative_tolerance()
        );
        assert!(
            ToleranceTier::Analytical.relative_tolerance()
                < ToleranceTier::Statistical.relative_tolerance()
        );
        assert!(
            ToleranceTier::Statistical.relative_tolerance()
                < ToleranceTier::Quantized.relative_tolerance()
        );
    }

    #[test]
    fn workload_names_match_workloads_module() {
        let tols = all_tolerances();
        let workloads = crate::workloads::all();
        for tol in &tols {
            assert!(
                workloads.iter().any(|w| w.name == tol.workload),
                "tolerance for '{}' has no matching workload",
                tol.workload
            );
        }
    }
}
