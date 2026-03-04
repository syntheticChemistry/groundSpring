// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Almost-Mathieu (quasiperiodic) localization in 1D (Exp 009).
//!
//! ```text
//! H ψ(n) = ψ(n+1) + ψ(n-1) + λ cos(2παn + θ) ψ(n)
//! ```
//!
//! where `α` is typically the golden ratio (maximally irrational). The
//! Aubry-André duality gives a sharp metal-insulator transition at `λ = 2`.
//! Herman's formula: `γ = max(0, ln(λ/2))` for `λ > 2`.
//!
//! # barracuda delegation
//!
//! When the `barracuda-gpu` feature is enabled, the Hamiltonian construction
//! and eigenvalue computation delegate to `barracuda::spectral` (the
//! `almost_mathieu_hamiltonian` and `find_all_eigenvalues` functions —
//! Sturm bisection, O(n²) for tridiag vs O(n³) dense QR).
//!
//! Note: barracuda uses `2λ_b cos(...)` convention, so we pass
//! `coupling / 2` as barracuda's `λ_b` to match our convention where the
//! Aubry-André transition sits at `λ = 2`.

/// Maximum QR iterations for eigenvalue extraction.
///
/// 100 iterations is sufficient for tridiagonal and near-tridiagonal
/// matrices up to n = 500. The Givens QR on a symmetric matrix converges
/// at a cubic rate per sub-diagonal element.
/// Reference: Golub & Van Loan (2013) Matrix Computations, §8.3.
#[cfg(not(feature = "barracuda-gpu"))]
const QR_MAX_ITERATIONS: usize = 100;

/// Threshold below which a matrix element is treated as zero in Givens
/// rotations. Chosen to be well below f64 precision (~1e-16) so that
/// only truly negligible entries are skipped, avoiding unnecessary
/// rotation of near-zero sub-diagonals without losing significant digits.
#[cfg(not(feature = "barracuda-gpu"))]
const GIVENS_ZERO_THRESHOLD: f64 = 1e-30;

/// Generate the quasiperiodic Almost-Mathieu potential.
///
/// `V(i) = λ cos(2παi + θ)` where `λ` is the coupling strength, `α` the
/// frequency (typically the golden ratio), and `θ` a phase offset.
///
/// The convention places the Aubry-André transition at `λ = 2` and yields
/// Herman's formula `γ = ln(λ/2)` for `λ > 2`.
#[must_use]
pub fn potential(n: usize, coupling: f64, alpha: f64, theta: f64) -> Vec<f64> {
    let two_pi_alpha = 2.0 * std::f64::consts::PI * alpha;
    (0..n)
        .map(|i| coupling * (two_pi_alpha.mul_add(crate::cast::usize_f64(i), theta)).cos())
        .collect()
}

/// Mean level spacing ratio for a sorted eigenvalue sequence.
///
/// When the `barracuda-gpu` feature is enabled, delegates to
/// `barracuda::spectral::level_spacing_ratio` (after sorting).
///
/// `r_n = min(δ_n, δ_{n+1}) / max(δ_n, δ_{n+1})`
///
/// Expected values: ~0.53 (GOE / extended), ~0.39 (Poisson / localized).
/// For quasiperiodic models in the extended phase, `<r>` is typically
/// higher (~0.9) due to quasi-integrable dynamics.
#[must_use]
pub fn level_spacing_ratio(eigenvalues: &mut [f64]) -> f64 {
    eigenvalues.sort_unstable_by(f64::total_cmp);
    #[cfg(feature = "barracuda-gpu")]
    {
        barracuda::spectral::level_spacing_ratio(eigenvalues)
    }
    #[cfg(not(feature = "barracuda-gpu"))]
    level_spacing_ratio_cpu(eigenvalues)
}

#[cfg(not(feature = "barracuda-gpu"))]
fn level_spacing_ratio_cpu(eigenvalues: &[f64]) -> f64 {
    if eigenvalues.len() < 3 {
        return 0.0;
    }
    let (sum, count) = eigenvalues.windows(3).fold((0.0, 0usize), |(s, c), w| {
        let (d1, d2) = (w[1] - w[0], w[2] - w[1]);
        let (small, large) = (d1.min(d2), d1.max(d2));
        if large > 0.0 {
            (s + small / large, c + 1)
        } else {
            (s, c)
        }
    });
    if count == 0 {
        return 0.0;
    }
    sum / crate::cast::usize_f64(count)
}

/// Build an `n × n` Almost-Mathieu Hamiltonian as a flat row-major vector.
///
/// Diagonal: `V(i) = λ cos(2παi + θ)`, off-diagonal: nearest-neighbor
/// hopping = 1.  Returns the matrix elements as a flat row-major array.
///
/// When `barracuda-gpu` is enabled, delegates to
/// `barracuda::spectral::almost_mathieu_hamiltonian` which
/// returns a tridiagonal form and then expands it. The barracuda
/// convention uses `2λ_b cos(...)`, so we pass `coupling / 2` as their λ.
#[must_use]
pub fn hamiltonian(n: usize, coupling: f64, alpha: f64, theta: f64) -> Vec<f64> {
    #[cfg(feature = "barracuda-gpu")]
    {
        let barracuda_lambda = coupling / 2.0;
        let (diag, off) =
            barracuda::spectral::almost_mathieu_hamiltonian(n, barracuda_lambda, alpha, theta);
        let mut h = vec![0.0; n * n];
        for (i, &d) in diag.iter().enumerate() {
            h[i * n + i] = d;
        }
        for (i, &o) in off.iter().enumerate() {
            h[i * n + (i + 1)] = o;
            h[(i + 1) * n + i] = o;
        }
        h
    }
    #[cfg(not(feature = "barracuda-gpu"))]
    hamiltonian_cpu(n, coupling, alpha, theta)
}

#[cfg(not(feature = "barracuda-gpu"))]
fn hamiltonian_cpu(n: usize, coupling: f64, alpha: f64, theta: f64) -> Vec<f64> {
    let pot = potential(n, coupling, alpha, theta);
    let mut h = vec![0.0; n * n];
    for i in 0..n {
        h[i * n + i] = pot[i];
        if i + 1 < n {
            h[i * n + (i + 1)] = 1.0;
            h[(i + 1) * n + i] = 1.0;
        }
    }
    h
}

/// Compute eigenvalues of an `n × n` Almost-Mathieu Hamiltonian.
///
/// When `barracuda-gpu` is enabled, uses `barracuda::spectral::find_all_eigenvalues`
/// — a Sturm bisection solver that exploits the tridiagonal structure, running
/// in O(n²) rather than O(n³) dense QR. This closes the LAPACK performance
/// gap in Exp 009.
///
/// Without barracuda, falls back to a Givens QR algorithm on the dense matrix.
#[must_use]
pub fn eigenvalues(n: usize, coupling: f64, alpha: f64, theta: f64) -> Vec<f64> {
    #[cfg(feature = "barracuda-gpu")]
    {
        let barracuda_lambda = coupling / 2.0;
        let (diag, off) =
            barracuda::spectral::almost_mathieu_hamiltonian(n, barracuda_lambda, alpha, theta);
        barracuda::spectral::find_all_eigenvalues(&diag, &off)
    }
    #[cfg(not(feature = "barracuda-gpu"))]
    eigenvalues_cpu(n, coupling, alpha, theta)
}

#[cfg(not(feature = "barracuda-gpu"))]
fn eigenvalues_cpu(n: usize, coupling: f64, alpha: f64, theta: f64) -> Vec<f64> {
    let ham = hamiltonian(n, coupling, alpha, theta);
    eigenvalues_qr_dense(n, &ham)
}

/// Dense QR eigenvalue extraction via Givens rotations on a flat row-major
/// buffer. Iterates `QR_MAX_ITERATIONS` QR steps. Sufficient for small
/// validation matrices (n ≤ 500). The barracuda-gpu path uses
/// `find_all_eigenvalues` (Sturm bisection) which is O(n²) for tridiag.
#[cfg(not(feature = "barracuda-gpu"))]
fn eigenvalues_qr_dense(n: usize, matrix: &[f64]) -> Vec<f64> {
    let nn = n * n;
    let mut mat = matrix.to_vec();
    let mut r = vec![0.0; nn];
    let mut q = vec![0.0; nn];

    for _ in 0..QR_MAX_ITERATIONS {
        init_identity(&mut q, n);
        r.copy_from_slice(&mat);
        givens_qr_flat(&mut q, &mut r, n);
        dense_mul_flat(&r, &q, &mut mat, n);
    }
    (0..n).map(|i| mat[i * n + i]).collect()
}

/// In-place Givens QR decomposition on flat row-major buffers.
#[cfg(not(feature = "barracuda-gpu"))]
fn givens_qr_flat(q: &mut [f64], r: &mut [f64], n: usize) {
    for col in 0..n.saturating_sub(1) {
        for row in (col + 1)..n {
            if r[row * n + col].abs() < GIVENS_ZERO_THRESHOLD {
                continue;
            }
            let diag = r[col * n + col];
            let below = r[row * n + col];
            let hyp = diag.hypot(below);
            if hyp < GIVENS_ZERO_THRESHOLD {
                continue;
            }
            let cos = diag / hyp;
            let sin = below / hyp;

            givens_rotate_rows_flat(r, col, row, cos, sin, n);
            givens_rotate_cols_flat(q, col, row, cos, sin, n);
        }
    }
}

#[cfg(not(feature = "barracuda-gpu"))]
fn givens_rotate_rows_flat(m: &mut [f64], r1: usize, r2: usize, cos: f64, sin: f64, n: usize) {
    for j in 0..n {
        let a = m[r1 * n + j];
        let b = m[r2 * n + j];
        m[r1 * n + j] = cos.mul_add(a, sin * b);
        m[r2 * n + j] = (-sin).mul_add(a, cos * b);
    }
}

#[cfg(not(feature = "barracuda-gpu"))]
fn givens_rotate_cols_flat(m: &mut [f64], c1: usize, c2: usize, cos: f64, sin: f64, n: usize) {
    for i in 0..n {
        let a = m[i * n + c1];
        let b = m[i * n + c2];
        m[i * n + c1] = cos.mul_add(a, sin * b);
        m[i * n + c2] = (-sin).mul_add(a, cos * b);
    }
}

/// Flat row-major matrix multiplication: `out = a × b` (both `n × n`).
#[cfg(not(feature = "barracuda-gpu"))]
fn dense_mul_flat(a: &[f64], b: &[f64], out: &mut [f64], n: usize) {
    for i in 0..n {
        for j in 0..n {
            out[i * n + j] = (0..n).map(|k| a[i * n + k] * b[k * n + j]).sum();
        }
    }
}

/// Write the `n × n` identity matrix into an existing buffer.
#[cfg(not(feature = "barracuda-gpu"))]
fn init_identity(buf: &mut [f64], n: usize) {
    buf.fill(0.0);
    for i in 0..n {
        buf[i * n + i] = 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anderson::lyapunov_exponent;
    use crate::tol;

    const GOLDEN: f64 = 0.618_033_988_749_894_9;

    #[test]
    fn potential_zero_coupling_is_zero() {
        let pot = potential(100, 0.0, GOLDEN, 0.0);
        assert!(pot.iter().all(|&v| v.abs() < f64::EPSILON));
    }

    #[test]
    fn potential_deterministic() {
        let p1 = potential(100, 3.0, GOLDEN, 0.0);
        let p2 = potential(100, 3.0, GOLDEN, 0.0);
        assert_eq!(p1, p2);
    }

    #[test]
    fn extended_phase_zero_lyapunov() {
        let pot = potential(100_000, 1.0, GOLDEN, 0.0);
        let g = lyapunov_exponent(&pot, 0.0);
        // For 100k sites in extended phase (λ=1), transfer-matrix averaging converges to ~0 with O(1/√N) fluctuations; 0.01 is ~3× the expected O(10⁻²·⁵) statistical error.
        assert!(
            g.abs() < tol::STOCHASTIC,
            "extended phase γ={g}, expected ~0"
        );
    }

    #[test]
    fn localized_phase_hermans_formula() {
        let pot = potential(100_000, 3.0, GOLDEN, 0.0);
        let g = lyapunov_exponent(&pot, 0.0);
        let expected = (3.0_f64 / 2.0).ln();
        // For 100k sites, Herman's formula convergence has O(1/N) systematic correction; 0.02 absorbs the finite-size effect.
        assert!(
            (g - expected).abs() < tol::NORM_2PCT,
            "γ={g}, expected ln(3/2)={expected}"
        );
    }

    #[test]
    fn critical_point_near_zero() {
        let pot = potential(100_000, 2.0, GOLDEN, 0.0);
        let g = lyapunov_exponent(&pot, 0.0);
        // Critical point has logarithmic corrections and slowest convergence; 0.05 is the minimal bound for N=100k.
        assert!(g.abs() < 0.05, "critical point γ={g}, expected ~0");
    }

    #[test]
    fn lyapunov_monotonic_above_critical() {
        let gammas: Vec<f64> = [2.0, 3.0, 4.0]
            .iter()
            .map(|&lam| {
                let pot = potential(100_000, lam, GOLDEN, 0.0);
                lyapunov_exponent(&pot, 0.0)
            })
            .collect();
        for w in gammas.windows(2) {
            assert!(
                w[1] >= w[0],
                "monotonicity failed: γ[i+1]={} < γ[i]={}",
                w[1],
                w[0]
            );
        }
    }

    #[test]
    fn hamiltonian_symmetric() {
        let n = 10;
        let h = hamiltonian(n, 2.0, GOLDEN, 0.0);
        for i in 0..n {
            for j in 0..n {
                assert!(
                    (h[i * n + j] - h[j * n + i]).abs() < f64::EPSILON,
                    "H[{i},{j}] != H[{j},{i}]"
                );
            }
        }
    }

    #[test]
    fn level_spacing_ratio_trivial() {
        let mut eigs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r = level_spacing_ratio(&mut eigs);
        assert!(
            (r - 1.0).abs() < f64::EPSILON,
            "uniform spacing should give r=1, got {r}"
        );
    }

    #[test]
    fn level_spacing_ratio_too_few() {
        let mut eigs = vec![1.0, 2.0];
        let r = level_spacing_ratio(&mut eigs);
        assert!(r.abs() < f64::EPSILON, "too few eigenvalues should give 0");
    }

    #[test]
    fn eigenvalues_count_matches_n() {
        let n = 20;
        let eigs = eigenvalues(n, 2.0, GOLDEN, 0.0);
        assert_eq!(eigs.len(), n);
    }

    #[test]
    fn eigenvalues_bounded_by_spectrum() {
        let n = 50;
        let coupling = 3.0;
        let eigs = eigenvalues(n, coupling, GOLDEN, 0.0);
        let spectral_bound = 2.0 + coupling;
        for &e in &eigs {
            assert!(
                e.abs() <= spectral_bound + 0.01,
                "eigenvalue {e} exceeds spectral bound {spectral_bound}"
            );
        }
    }

    #[test]
    fn eigenvalues_deterministic() {
        let e1 = eigenvalues(30, 2.5, GOLDEN, 0.0);
        let e2 = eigenvalues(30, 2.5, GOLDEN, 0.0);
        assert_eq!(e1, e2);
    }

    #[test]
    fn hamiltonian_correct_size() {
        let n = 15;
        let h = hamiltonian(n, 1.5, GOLDEN, 0.0);
        assert_eq!(h.len(), n * n);
    }

    #[test]
    fn hamiltonian_tridiagonal_structure() {
        let n = 10;
        let h = hamiltonian(n, 2.0, GOLDEN, 0.0);
        for i in 0..n {
            for j in 0..n {
                let diff = i.abs_diff(j);
                if diff > 1 {
                    assert!(
                        h[i * n + j].abs() < f64::EPSILON,
                        "H[{i},{j}] = {} should be 0 for tridiagonal",
                        h[i * n + j]
                    );
                }
            }
        }
    }

    #[test]
    fn level_spacing_ratio_localized_vs_extended() {
        let n = 200;
        let mut eigs_ext = eigenvalues(n, 0.5, GOLDEN, 0.0);
        let r_ext = level_spacing_ratio(&mut eigs_ext);

        let mut eigs_loc = eigenvalues(n, 4.0, GOLDEN, 0.0);
        let r_loc = level_spacing_ratio(&mut eigs_loc);

        assert!(
            r_ext > r_loc,
            "extended r={r_ext} should exceed localized r={r_loc}"
        );
    }

    #[test]
    fn potential_length_matches_n() {
        assert_eq!(potential(50, 2.0, GOLDEN, 0.0).len(), 50);
        assert_eq!(potential(1, 2.0, GOLDEN, 0.0).len(), 1);
    }

    #[test]
    fn level_spacing_ratio_unsorted_input() {
        let mut eigs = vec![5.0, 1.0, 3.0, 2.0, 4.0];
        let r = level_spacing_ratio(&mut eigs);
        assert!(
            (r - 1.0).abs() < f64::EPSILON,
            "should sort internally and return r=1 for uniform spacing"
        );
    }

    #[test]
    fn level_spacing_ratio_empty() {
        let mut eigs: Vec<f64> = vec![];
        assert!(level_spacing_ratio(&mut eigs).abs() < f64::EPSILON);
    }
}
