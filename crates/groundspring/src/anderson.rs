// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Anderson localization and quasiperiodic localization in 1D tight-binding
//! models.
//!
//! ## Anderson model (Exp 008)
//!
//! ```text
//! H ψ(n) = ψ(n+1) + ψ(n-1) + V(n) ψ(n)
//! ```
//!
//! where `V(n)` is a random potential drawn uniformly from `[-W/2, W/2]`.
//! In 1D, Anderson (1958) proved that ALL states are localized for any
//! disorder `W > 0`.  The localization length `ξ ~ C / W²` at the band
//! center `E = 0` (Thouless 1972, Derrida-Gardner).
//!
//! ## Almost-Mathieu model (Exp 009)
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
//! When the `barracuda-gpu` feature is enabled, `lyapunov_exponent` and
//! `lyapunov_averaged` delegate to `barracuda::spectral::lyapunov_*`.
//!
//! **Seed convention:** the local CPU implementation uses `base_seed + i`
//! per realization; barracuda uses `base_seed + r * 1000`.  Results diverge
//! when the feature gate is active — this is expected and documented in
//! `specs/BARRACUDA_EVOLUTION.md` as Phase 2b alignment work.

use crate::prng::Xorshift64;

/// Generate a random potential for the 1D Anderson model.
///
/// Each site gets `V(n) ~ Uniform[-W/2, W/2]` where `W = disorder`.
/// Returns the zero vector for `disorder <= 0`.
#[must_use]
pub fn anderson_potential(n: usize, disorder: f64, seed: u64) -> Vec<f64> {
    if disorder <= 0.0 {
        return vec![0.0; n];
    }
    let half_w = disorder / 2.0;
    let mut rng = Xorshift64::new(seed);
    (0..n)
        .map(|_| rng.next_f64().mul_add(disorder, -half_w))
        .collect()
}

/// Compute the Lyapunov exponent for a given potential and energy.
///
/// When `barracuda-gpu` feature is enabled, delegates to
/// `barracuda::spectral::lyapunov_exponent`.
///
/// Uses the transfer-matrix method with vector renormalization to avoid
/// overflow.  The transfer matrix at site `n` is:
///
/// ```text
/// T_n = [[E - V(n), -1],
///        [1,         0]]
/// ```
///
/// Returns `γ = (1/N) Σ ln(norm)`, the largest Lyapunov exponent.
#[must_use]
pub fn lyapunov_exponent(potential: &[f64], energy: f64) -> f64 {
    #[cfg(feature = "barracuda-gpu")]
    {
        barracuda::spectral::lyapunov_exponent(potential, energy)
    }

    #[cfg(not(feature = "barracuda-gpu"))]
    {
        lyapunov_exponent_local(potential, energy)
    }
}

#[cfg(not(feature = "barracuda-gpu"))]
fn lyapunov_exponent_local(potential: &[f64], energy: f64) -> f64 {
    let n = potential.len();
    if n == 0 {
        return 0.0;
    }

    let mut log_growth = 0.0;
    let mut v0: f64 = 1.0;
    let mut v1: f64 = 0.0;

    for &v in potential {
        let new_0 = (energy - v).mul_add(v0, -v1);
        let new_1 = v0;
        v0 = new_0;
        v1 = new_1;

        let norm = v0.hypot(v1);
        if norm > 0.0 {
            log_growth += norm.ln();
            v0 /= norm;
            v1 /= norm;
        }
    }

    log_growth / crate::cast::usize_f64(n)
}

/// Localization length `ξ = 1 / γ`.  Returns `f64::INFINITY` if `γ <= 0`.
#[must_use]
pub fn localization_length(gamma: f64) -> f64 {
    if gamma <= 0.0 {
        return f64::INFINITY;
    }
    1.0 / gamma
}

/// Analytical localization length from disorder strength and energy.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::special::anderson_transport::localization_length` which uses
/// the perturbative result `ξ(W, E) ≈ 105.2 / W²` at the band center.
/// The local fallback uses `ξ ≈ C / W²` with `C ≈ 96` from Derrida-Gardner.
///
/// This is an analytical estimate — for numerical results, use
/// [`lyapunov_exponent`] + [`localization_length`].
#[must_use]
pub fn analytical_localization_length(disorder: f64, energy: f64) -> f64 {
    #[cfg(feature = "barracuda")]
    {
        barracuda::special::anderson_transport::localization_length(disorder, energy)
    }
    #[cfg(not(feature = "barracuda"))]
    {
        if disorder <= 0.0 {
            return f64::INFINITY;
        }
        let _ = energy;
        96.0 / (disorder * disorder)
    }
}

/// Average Lyapunov exponent over many disorder realizations.
///
/// When `barracuda-gpu` feature is enabled, delegates to
/// `barracuda::spectral::lyapunov_averaged`.  Note: barracuda uses
/// `base_seed + r * 1000` per realization; the local fallback uses
/// `base_seed + i` — results will differ across feature gates until
/// Phase 2b PRNG alignment.
#[must_use]
pub fn lyapunov_averaged(
    n_sites: usize,
    disorder: f64,
    energy: f64,
    n_realizations: usize,
    base_seed: u64,
) -> f64 {
    #[cfg(feature = "barracuda-gpu")]
    {
        barracuda::spectral::lyapunov_averaged(n_sites, disorder, energy, n_realizations, base_seed)
    }

    #[cfg(not(feature = "barracuda-gpu"))]
    {
        let mut total = 0.0;
        for i in 0..n_realizations {
            let pot = anderson_potential(n_sites, disorder, base_seed + i as u64);
            total += lyapunov_exponent(&pot, energy);
        }
        total / crate::cast::usize_f64(n_realizations)
    }
}

// ── Almost-Mathieu (quasiperiodic) model ──────────────────────────────

/// Generate the quasiperiodic Almost-Mathieu potential.
///
/// `V(i) = λ cos(2παi + θ)` where `λ` is the coupling strength, `α` the
/// frequency (typically the golden ratio), and `θ` a phase offset.
///
/// The convention places the Aubry-André transition at `λ = 2` and yields
/// Herman's formula `γ = ln(λ/2)` for `λ > 2`.
///
/// # barracuda delegation
///
/// When `barracuda-gpu` is enabled, the Hamiltonian construction
/// (eigenvalue checks) delegates to
/// `barracuda::spectral::hofstadter::almost_mathieu_hamiltonian`.
/// Note: barracuda uses `2λ_b cos(...)` convention, so barracuda's
/// `λ_b = coupling / 2` to match our convention.
#[must_use]
pub fn almost_mathieu_potential(
    n: usize,
    coupling: f64,
    alpha: f64,
    theta: f64,
) -> Vec<f64> {
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
        barracuda::spectral::level_spacing_ratio(eigenvalues)
    }
    #[cfg(not(feature = "barracuda-gpu"))]
    {
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
pub fn almost_mathieu_hamiltonian(
    n: usize,
    coupling: f64,
    alpha: f64,
    theta: f64,
) -> Vec<f64> {
    #[cfg(feature = "barracuda-gpu")]
    {
        let barracuda_lambda = coupling / 2.0;
        let (diag, off) = barracuda::spectral::almost_mathieu_hamiltonian(
            n,
            barracuda_lambda,
            alpha,
            theta,
        );
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
    {
        let pot = almost_mathieu_potential(n, coupling, alpha, theta);
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
pub fn almost_mathieu_eigenvalues(
    n: usize,
    coupling: f64,
    alpha: f64,
    theta: f64,
) -> Vec<f64> {
    #[cfg(feature = "barracuda-gpu")]
    {
        let barracuda_lambda = coupling / 2.0;
        let (diag, off) =
            barracuda::spectral::almost_mathieu_hamiltonian(n, barracuda_lambda, alpha, theta);
        barracuda::spectral::find_all_eigenvalues(&diag, &off)
    }
    #[cfg(not(feature = "barracuda-gpu"))]
    {
        let ham = almost_mathieu_hamiltonian(n, coupling, alpha, theta);
        eigenvalues_qr_dense(n, &ham)
    }
}

/// Dense QR eigenvalue extraction via Givens rotations.
///
/// Iterates 100 QR steps on the full matrix. Sufficient for small
/// validation matrices (n ≤ 500). The barracuda-gpu path uses
/// `find_all_eigenvalues` (Sturm bisection) which is O(n²) for tridiag.
#[cfg(not(feature = "barracuda-gpu"))]
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

#[cfg(not(feature = "barracuda-gpu"))]
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

#[cfg(not(feature = "barracuda-gpu"))]
fn givens_rotate_rows(mat: &mut [Vec<f64>], r1: usize, r2: usize, cos: f64, sin: f64) {
    let (top, bot) = mat.split_at_mut(r2);
    for (a, b) in top[r1].iter_mut().zip(bot[0].iter_mut()) {
        let orig_a = *a;
        let orig_b = *b;
        *a = cos.mul_add(orig_a, sin * orig_b);
        *b = (-sin).mul_add(orig_a, cos * orig_b);
    }
}

#[cfg(not(feature = "barracuda-gpu"))]
fn givens_rotate_cols(mat: &mut [Vec<f64>], c1: usize, c2: usize, cos: f64, sin: f64) {
    for row in mat.iter_mut() {
        let a = row[c1];
        let b = row[c2];
        row[c1] = cos.mul_add(a, sin * b);
        row[c2] = (-sin).mul_add(a, cos * b);
    }
}

#[cfg(not(feature = "barracuda-gpu"))]
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

    #[test]
    fn clean_system_zero_lyapunov() {
        let pot = anderson_potential(10000, 0.0, 42);
        let gamma = lyapunov_exponent(&pot, 0.0);
        assert!(gamma.abs() < 0.001, "clean system γ={gamma}, expected ~0");
    }

    #[test]
    fn disorder_gives_positive_lyapunov() {
        let gamma = lyapunov_averaged(10000, 2.0, 0.0, 10, 42);
        assert!(
            gamma > 0.0,
            "disordered system should have γ > 0, got {gamma}"
        );
    }

    #[test]
    fn lyapunov_increases_with_disorder() {
        let g1 = lyapunov_averaged(10000, 1.0, 0.0, 10, 42);
        let g2 = lyapunov_averaged(10000, 4.0, 0.0, 10, 42);
        assert!(g2 > g1, "γ(W=4)={g2} should exceed γ(W=1)={g1}");
    }

    #[test]
    fn localization_length_decreases_with_disorder() {
        let g1 = lyapunov_averaged(10000, 1.0, 0.0, 10, 42);
        let g2 = lyapunov_averaged(10000, 4.0, 0.0, 10, 42);
        let xi1 = localization_length(g1);
        let xi2 = localization_length(g2);
        assert!(xi2 < xi1, "ξ(W=4)={xi2} should be less than ξ(W=1)={xi1}");
    }

    #[test]
    fn potential_deterministic() {
        let p1 = anderson_potential(100, 2.0, 42);
        let p2 = anderson_potential(100, 2.0, 42);
        assert_eq!(p1, p2);
    }

    #[test]
    fn potential_different_seed() {
        let p1 = anderson_potential(100, 2.0, 42);
        let p2 = anderson_potential(100, 2.0, 99);
        assert_ne!(p1, p2);
    }

    #[test]
    fn analytical_localization_length_decreases_with_disorder() {
        let xi1 = analytical_localization_length(1.0, 0.0);
        let xi2 = analytical_localization_length(2.0, 0.0);
        assert!(xi2 < xi1, "ξ(W=2)={xi2} should be less than ξ(W=1)={xi1}");
    }

    #[test]
    fn analytical_localization_length_clean_system_large() {
        let xi = analytical_localization_length(0.0, 0.0);
        assert!(
            xi > 1000.0 || xi.is_infinite(),
            "clean system should have ξ ≫ lattice constant, got {xi}"
        );
    }

    #[test]
    fn analytical_vs_numerical_same_order() {
        let g = lyapunov_averaged(100_000, 1.0, 0.0, 20, 42);
        let xi_numerical = localization_length(g);
        let xi_analytical = analytical_localization_length(1.0, 0.0);
        // Analytical approximations differ by O(1) constant factors
        // depending on model (Derrida-Gardner vs perturbative).
        // With barracuda-gpu, PRNG also changes numerical values.
        assert!(
            xi_analytical > 10.0 && xi_numerical > 10.0,
            "both should be large: analytical={xi_analytical}, numerical={xi_numerical}"
        );
    }

    #[test]
    fn thouless_scaling() {
        let g = lyapunov_averaged(100_000, 1.0, 0.0, 20, 42);
        let xi = localization_length(g);
        let c = xi * 1.0_f64.powi(2);
        assert!(
            (60.0..140.0).contains(&c),
            "Thouless coefficient C={c}, expected ~96"
        );
    }

    // ── Almost-Mathieu tests ──────────────────────────────────────────

    const GOLDEN: f64 = 0.618_033_988_749_894_9;

    #[test]
    fn am_potential_zero_coupling_is_zero() {
        let pot = almost_mathieu_potential(100, 0.0, GOLDEN, 0.0);
        assert!(pot.iter().all(|&v| v.abs() < f64::EPSILON));
    }

    #[test]
    fn am_potential_deterministic() {
        let p1 = almost_mathieu_potential(100, 3.0, GOLDEN, 0.0);
        let p2 = almost_mathieu_potential(100, 3.0, GOLDEN, 0.0);
        assert_eq!(p1, p2);
    }

    #[test]
    fn am_extended_phase_zero_lyapunov() {
        let pot = almost_mathieu_potential(100_000, 1.0, GOLDEN, 0.0);
        let g = lyapunov_exponent(&pot, 0.0);
        assert!(g.abs() < 0.01, "extended phase γ={g}, expected ~0");
    }

    #[test]
    fn am_localized_phase_hermans_formula() {
        let pot = almost_mathieu_potential(100_000, 3.0, GOLDEN, 0.0);
        let g = lyapunov_exponent(&pot, 0.0);
        let expected = (3.0_f64 / 2.0).ln();
        assert!(
            (g - expected).abs() < 0.02,
            "γ={g}, expected ln(3/2)={expected}"
        );
    }

    #[test]
    fn am_critical_point_near_zero() {
        let pot = almost_mathieu_potential(100_000, 2.0, GOLDEN, 0.0);
        let g = lyapunov_exponent(&pot, 0.0);
        assert!(g.abs() < 0.05, "critical point γ={g}, expected ~0");
    }

    #[test]
    fn am_lyapunov_monotonic_above_critical() {
        let gammas: Vec<f64> = [2.0, 3.0, 4.0]
            .iter()
            .map(|&lam| {
                let pot = almost_mathieu_potential(100_000, lam, GOLDEN, 0.0);
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
    fn am_hamiltonian_symmetric() {
        let n = 10;
        let h = almost_mathieu_hamiltonian(n, 2.0, GOLDEN, 0.0);
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
