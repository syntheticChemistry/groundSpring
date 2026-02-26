// SPDX-License-Identifier: AGPL-3.0-or-later
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
//! and eigenvalue computation delegate to `barracuda::spectral::hofstadter`
//! and `barracuda::spectral::find_all_eigenvalues` (Sturm bisection, O(n²)
//! for tridiag vs O(n³) dense QR).
//!
//! Note: barracuda uses `2λ_b cos(...)` convention, so we pass
//! `coupling / 2` as barracuda's `λ_b` to match our convention where the
//! Aubry-André transition sits at `λ = 2`.

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
    eigenvalues.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    #[cfg(feature = "barracuda-gpu")]
    {
        return barracuda::spectral::level_spacing_ratio(eigenvalues);
    }

    #[allow(unreachable_code)]
    level_spacing_ratio_cpu(eigenvalues)
}

fn level_spacing_ratio_cpu(eigenvalues: &[f64]) -> f64 {
    let n = eigenvalues.len();
    if n < 3 {
        return 0.0;
    }
    let mut sum = 0.0;
    let mut count = 0usize;
    for i in 0..n - 2 {
        let d1 = eigenvalues[i + 1] - eigenvalues[i];
        let d2 = eigenvalues[i + 2] - eigenvalues[i + 1];
        let small = d1.min(d2);
        let large = d1.max(d2);
        if large > 0.0 {
            sum += small / large;
            count += 1;
        }
    }
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
/// `barracuda::spectral::hofstadter::almost_mathieu_hamiltonian` which
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
        return h;
    }

    #[allow(unreachable_code)]
    hamiltonian_cpu(n, coupling, alpha, theta)
}

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
        return barracuda::spectral::find_all_eigenvalues(&diag, &off);
    }

    #[allow(unreachable_code)]
    eigenvalues_cpu(n, coupling, alpha, theta)
}

fn eigenvalues_cpu(n: usize, coupling: f64, alpha: f64, theta: f64) -> Vec<f64> {
    let ham = hamiltonian(n, coupling, alpha, theta);
    eigenvalues_qr_dense(n, &ham)
}

/// Dense QR eigenvalue extraction via Givens rotations.
///
/// Iterates 100 QR steps on the full matrix. Sufficient for small
/// validation matrices (n ≤ 500). The barracuda-gpu path uses
/// `find_all_eigenvalues` (Sturm bisection) which is O(n²) for tridiag.
fn eigenvalues_qr_dense(n: usize, matrix: &[f64]) -> Vec<f64> {
    let mut mat: Vec<Vec<f64>> = (0..n)
        .map(|row| (0..n).map(|col| matrix[row * n + col]).collect())
        .collect();

    for _ in 0..100 {
        let (q_mat, r_mat) = givens_qr(&mat);
        mat = dense_mul(&r_mat, &q_mat);
    }
    (0..n).map(|i| mat[i][i]).collect()
}

fn givens_qr(mat: &[Vec<f64>]) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let n = mat.len();
    let mut q_mat: Vec<Vec<f64>> = (0..n)
        .map(|i| {
            let mut row = vec![0.0; n];
            row[i] = 1.0;
            row
        })
        .collect();
    let mut r_mat: Vec<Vec<f64>> = mat.to_vec();

    for col in 0..n.saturating_sub(1) {
        for row in (col + 1)..n {
            if r_mat[row][col].abs() < 1e-30 {
                continue;
            }
            let diag = r_mat[col][col];
            let below = r_mat[row][col];
            let hyp = diag.hypot(below);
            if hyp < 1e-30 {
                continue;
            }
            let cos = diag / hyp;
            let sin = below / hyp;

            givens_rotate_rows(&mut r_mat, col, row, cos, sin);
            givens_rotate_cols(&mut q_mat, col, row, cos, sin);
        }
    }
    (q_mat, r_mat)
}

fn givens_rotate_rows(mat: &mut [Vec<f64>], r1: usize, r2: usize, cos: f64, sin: f64) {
    let (top, bot) = mat.split_at_mut(r2);
    for (a, b) in top[r1].iter_mut().zip(bot[0].iter_mut()) {
        let orig_a = *a;
        let orig_b = *b;
        *a = cos.mul_add(orig_a, sin * orig_b);
        *b = (-sin).mul_add(orig_a, cos * orig_b);
    }
}

fn givens_rotate_cols(mat: &mut [Vec<f64>], c1: usize, c2: usize, cos: f64, sin: f64) {
    for row in mat.iter_mut() {
        let a = row[c1];
        let b = row[c2];
        row[c1] = cos.mul_add(a, sin * b);
        row[c2] = (-sin).mul_add(a, cos * b);
    }
}

fn dense_mul(a_mat: &[Vec<f64>], b_mat: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = a_mat.len();
    let mut result = vec![vec![0.0; n]; n];
    for (i, row) in result.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = (0..n).map(|k| a_mat[i][k] * b_mat[k][j]).sum();
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anderson::lyapunov_exponent;

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
        assert!(g.abs() < 0.01, "extended phase γ={g}, expected ~0");
    }

    #[test]
    fn localized_phase_hermans_formula() {
        let pot = potential(100_000, 3.0, GOLDEN, 0.0);
        let g = lyapunov_exponent(&pot, 0.0);
        let expected = (3.0_f64 / 2.0).ln();
        assert!(
            (g - expected).abs() < 0.02,
            "γ={g}, expected ln(3/2)={expected}"
        );
    }

    #[test]
    fn critical_point_near_zero() {
        let pot = potential(100_000, 2.0, GOLDEN, 0.0);
        let g = lyapunov_exponent(&pot, 0.0);
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
}
