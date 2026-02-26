// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Wavepacket transport in 1D tight-binding models.
//!
//! Computes the mean square displacement (MSD) of a wavepacket evolving under
//! Schrödinger dynamics in a tridiagonal Hamiltonian. The transport exponent β
//! extracted from σ(t) ~ t^β distinguishes ballistic (β=1), diffusive (β=0.5),
//! and localized (β=0) regimes.
//!
//! # References
//!
//! - Kachkovskiy (2016) Comm Math Phys 345:659-673
//! - Jitomirskaya & Kachkovskiy (2018) JEMS 21:777-795
//!
//! # Future GPU path
//!
//! Future: `tridiag_eigh` could delegate to barracuda's eigenvector primitives
//! when available.

use crate::cast::usize_f64;

/// Maximum QL iterations before convergence failure.
const QL_MAX_ITERATIONS: usize = 30;
/// Minimum MSD threshold for log-log regression (avoids log(0)).
const MSD_MIN_THRESHOLD: f64 = 1e-20;
/// Denominator epsilon for regression singularity check.
const REGRESSION_EPSILON: f64 = 1e-30;

/// Eigendecomposition of a symmetric tridiagonal matrix via implicit QL
/// algorithm with Wilkinson shifts.
///
/// Returns `(eigenvalues, eigenvectors)` where `eigenvectors[k]` is the k-th
/// eigenvector (column of the orthogonal matrix U such that H = U Λ U^T).
///
/// # Panics
///
/// Panics if `diag` is empty or `offdiag.len() != diag.len() - 1`.
#[must_use]
pub fn tridiag_eigh(diag: &[f64], offdiag: &[f64]) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = diag.len();
    assert!(n > 0, "matrix must be non-empty");
    assert_eq!(offdiag.len(), n - 1, "offdiag must have n-1 elements");

    if n == 1 {
        return (vec![diag[0]], vec![vec![1.0]]);
    }

    let mut d = diag.to_vec();
    let mut e = vec![0.0; n];
    for (i, &val) in offdiag.iter().enumerate() {
        e[i] = val;
    }

    let mut z = vec![vec![0.0; n]; n];
    for (i, row) in z.iter_mut().enumerate() {
        row[i] = 1.0;
    }

    implicit_ql(&mut d, &mut e, &mut z, n);

    sort_eigenpairs(&mut d, &mut z, n);

    (d, z)
}

/// Implicit QL algorithm for symmetric tridiagonal eigenvalue/eigenvector
/// computation. Modifies `d` (diagonal → eigenvalues), `e` (off-diagonal →
/// zeros), and `z` (identity → eigenvector columns).
#[allow(clippy::many_single_char_names)]
fn implicit_ql(d: &mut [f64], e: &mut [f64], z: &mut [Vec<f64>], n: usize) {
    let eps = f64::EPSILON;

    for l in 0..n {
        let mut iter_count = 0;
        loop {
            let mut m = l;
            while m < n - 1 {
                let threshold = eps * (d[m].abs() + d[m + 1].abs());
                if e[m].abs() <= threshold {
                    break;
                }
                m += 1;
            }

            if m == l {
                break;
            }

            assert!(
                iter_count < QL_MAX_ITERATIONS,
                "QL algorithm failed to converge after {QL_MAX_ITERATIONS} iterations"
            );
            iter_count += 1;

            let g0 = (d[l + 1] - d[l]) / (2.0 * e[l]);
            let r0 = g0.hypot(1.0);
            let mut g = d[m] - d[l] + e[l] / (g0 + r0.copysign(g0));

            let mut c = 1.0;
            let mut s = 1.0;
            let mut p = 0.0;

            for i in (l..m).rev() {
                let f = s * e[i];
                let b = c * e[i];

                let r_rot;
                if f.abs() >= g.abs() {
                    c = g / f;
                    r_rot = c.hypot(1.0);
                    e[i + 1] = f * r_rot;
                    s = 1.0 / r_rot;
                    c *= s;
                } else {
                    s = f / g;
                    r_rot = s.hypot(1.0);
                    e[i + 1] = g * r_rot;
                    c = 1.0 / r_rot;
                    s *= c;
                }

                let gi = d[i + 1] - p;
                let r_val = (d[i] - gi).mul_add(s, 2.0 * c * b);
                p = s * r_val;
                d[i + 1] = gi + p;
                g = c.mul_add(r_val, -b);

                for row in z.iter_mut() {
                    let f_z = row[i + 1];
                    row[i + 1] = s.mul_add(row[i], c * f_z);
                    row[i] = c.mul_add(row[i], -(s * f_z));
                }
            }

            d[l] -= p;
            e[l] = g;
            e[m] = 0.0;
        }
    }
}

/// Sort eigenvalues in ascending order and permute eigenvectors accordingly.
fn sort_eigenpairs(d: &mut [f64], z: &mut [Vec<f64>], n: usize) {
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_unstable_by(|&a, &b| d[a].total_cmp(&d[b]));

    let d_sorted: Vec<f64> = indices.iter().map(|&i| d[i]).collect();
    d.copy_from_slice(&d_sorted);

    let z_copy: Vec<Vec<f64>> = z.to_vec();
    for row_idx in 0..n {
        for (new_col, &old_col) in indices.iter().enumerate() {
            z[row_idx][new_col] = z_copy[row_idx][old_col];
        }
    }
}

/// Compute the mean square displacement of a wavepacket at a given time.
///
/// Given the eigendecomposition (eigenvalues, eigenvectors) of the Hamiltonian,
/// evolves an initial delta-function wavepacket at `init_site` to time `t` and
/// returns `(msd, normalization)`.
///
/// # Theory
///
/// `ψ_j(t) = Σ_k U_{j,k} U_{n₀,k} exp(-i E_k t)`
/// `P_j(t) = |ψ_j(t)|²`
/// `σ²(t) = Σ_j (j - n₀)² P_j(t)`
///
/// # Panics
///
/// Panics if `init_site >= eigenvalues.len()`.
#[must_use]
pub fn wavepacket_msd(
    eigenvalues: &[f64],
    eigenvectors: &[Vec<f64>],
    init_site: usize,
    time: f64,
) -> (f64, f64) {
    let n = eigenvalues.len();
    assert!(init_site < n, "init_site must be < n");

    let coeffs: Vec<f64> = (0..n).map(|k| eigenvectors[init_site][k]).collect();

    let mut msd = 0.0;
    let mut norm = 0.0;
    let init_f = usize_f64(init_site);

    for (j, evec_row) in eigenvectors.iter().enumerate() {
        let mut re = 0.0;
        let mut im = 0.0;
        for k in 0..n {
            let u_jk = evec_row[k];
            let c_k = coeffs[k];
            let phase = eigenvalues[k] * time;
            re += u_jk * c_k * phase.cos();
            im -= u_jk * c_k * phase.sin();
        }
        let prob = re.mul_add(re, im * im);
        let displacement = usize_f64(j) - init_f;
        msd += displacement * displacement * prob;
        norm += prob;
    }

    (msd, norm)
}

/// Extract the transport exponent β from MSD data via log-log linear regression.
///
/// Fits log(σ(t)) = β log(t) + const, where σ = √MSD.
/// Returns the slope β.
///
/// # Panics
///
/// Panics if `times` and `msds` have different lengths.
#[must_use]
pub fn transport_exponent(times: &[f64], msds: &[f64]) -> f64 {
    assert_eq!(times.len(), msds.len(), "times and msds must match");

    let valid: Vec<(f64, f64)> = times
        .iter()
        .zip(msds.iter())
        .filter(|(&t, &m)| t > 0.0 && m > MSD_MIN_THRESHOLD)
        .map(|(&t, &m)| (t.ln(), 0.5 * m.ln()))
        .collect();

    if valid.len() < 2 {
        return 0.0;
    }

    let n = usize_f64(valid.len());
    let sx: f64 = valid.iter().map(|(x, _)| x).sum();
    let sy: f64 = valid.iter().map(|(_, y)| y).sum();
    let sxx: f64 = valid.iter().map(|(x, _)| x * x).sum();
    let sxy: f64 = valid.iter().map(|(x, y)| x * y).sum();

    let denom = n.mul_add(sxx, -(sx * sx));
    if denom.abs() < REGRESSION_EPSILON {
        return 0.0;
    }

    n.mul_add(sxy, -(sx * sy)) / denom
}

#[cfg(test)]
mod tests {
    use super::*;

    fn almost_mathieu_diag_offdiag(
        n: usize,
        coupling: f64,
        alpha: f64,
        theta: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        let diag: Vec<f64> = (0..n)
            .map(|i| coupling * (2.0 * std::f64::consts::PI * alpha * usize_f64(i) + theta).cos())
            .collect();
        let offdiag = vec![1.0; n - 1];
        (diag, offdiag)
    }

    #[test]
    fn trivial_eigh() {
        let (vals, vecs) = tridiag_eigh(&[3.0], &[]);
        assert!((vals[0] - 3.0).abs() < 1e-14);
        assert!((vecs[0][0] - 1.0).abs() < 1e-14);
    }

    #[test]
    fn two_by_two_eigh() {
        let (vals, vecs) = tridiag_eigh(&[0.0, 0.0], &[1.0]);
        assert!((vals[0] - (-1.0)).abs() < 1e-12);
        assert!((vals[1] - 1.0).abs() < 1e-12);

        let norm0: f64 = vecs.iter().map(|row| row[0] * row[0]).sum();
        assert!((norm0 - 1.0).abs() < 1e-12, "eigenvector 0 not normalized");
    }

    #[test]
    fn orthogonality() {
        let n = 20;
        let (diag, offdiag) = almost_mathieu_diag_offdiag(n, 1.0, 0.618_033_988_749_894_9, 0.0);
        let (_, vecs) = tridiag_eigh(&diag, &offdiag);

        for i in 0..n {
            for j in 0..n {
                let dot: f64 = (0..n).map(|k| vecs[k][i] * vecs[k][j]).sum();
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (dot - expected).abs() < 1e-8,
                    "dot({i},{j}) = {dot}, expected {expected}"
                );
            }
        }
    }

    #[test]
    fn ballistic_transport() {
        let n = 101;
        let (diag, offdiag) = almost_mathieu_diag_offdiag(n, 0.5, 0.618_033_988_749_894_9, 0.0);
        let (evals, evecs) = tridiag_eigh(&diag, &offdiag);

        let times = [1.0, 2.0, 5.0, 10.0, 20.0];
        let msds: Vec<f64> = times
            .iter()
            .map(|&t| wavepacket_msd(&evals, &evecs, 50, t).0)
            .collect();

        let beta = transport_exponent(&times, &msds);
        assert!(beta > 0.5, "ballistic β should be > 0.5, got {beta}");
    }

    #[test]
    fn localized_transport() {
        let n = 101;
        let (diag, offdiag) = almost_mathieu_diag_offdiag(n, 4.0, 0.618_033_988_749_894_9, 0.0);
        let (evals, evecs) = tridiag_eigh(&diag, &offdiag);

        let times = [1.0, 5.0, 10.0, 20.0, 40.0];
        let msds: Vec<f64> = times
            .iter()
            .map(|&t| wavepacket_msd(&evals, &evecs, 50, t).0)
            .collect();

        let beta = transport_exponent(&times, &msds);
        assert!(beta < 0.3, "localized β should be < 0.3, got {beta}");
    }

    #[test]
    fn normalization_preserved() {
        let n = 51;
        let (diag, offdiag) = almost_mathieu_diag_offdiag(n, 1.0, 0.618_033_988_749_894_9, 0.0);
        let (evals, evecs) = tridiag_eigh(&diag, &offdiag);

        for &t in &[0.0, 1.0, 5.0, 20.0] {
            let (_, norm) = wavepacket_msd(&evals, &evecs, 25, t);
            assert!((norm - 1.0).abs() < 1e-8, "normalization at t={t}: {norm}");
        }
    }

    #[test]
    fn transport_exponent_linear() {
        let times = [1.0, 2.0, 4.0, 8.0, 16.0];
        let msds: Vec<f64> = times.iter().map(|&t| t * t).collect();
        let beta = transport_exponent(&times, &msds);
        assert!(
            (beta - 1.0).abs() < 0.01,
            "β for σ²~t² should be 1.0, got {beta}"
        );
    }

    #[test]
    fn transport_exponent_constant() {
        let times = [1.0, 2.0, 4.0, 8.0, 16.0];
        let msds = [5.0; 5];
        let beta = transport_exponent(&times, &msds);
        assert!(
            beta.abs() < 0.01,
            "β for constant MSD should be 0.0, got {beta}"
        );
    }
}
