// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Band structure of 1D periodic tight-binding chains.
//!
//! Computes band structure using the transfer matrix method:
//! for a periodic potential with period *p*, the transfer matrix
//! product `T(E) = ∏ₙ Tₙ(E)` has `|Tr(T)/2| ≤ 1` inside bands
//! and `|Tr(T)/2| > 1` inside gaps. Band edges occur at
//! `|Tr(T)/2| = 1`.
//!
//! This is the periodic-potential generalization of Anderson
//! localization (Exp 008): a periodic system has bands (propagating)
//! and gaps (evanescent), while a *disordered* system localizes
//! everything.
//!
//! # References
//!
//! - Filonov & Kachkovskiy (2018) Acta Math 221:59-80
//! - Anderson (1958) Phys Rev 109:1492-1505
//! - Kachkovskiy (2016) CMP 345:659-673

use crate::cast::usize_f64;

/// Compute `Tr(T)/2` for one period of the transfer matrix at energy `e`.
///
/// The transfer matrix for site *n* is
/// `Tₙ = [[(E − Vₙ)/t, −1], [1, 0]]`.
///
/// The half-trace of the product `T = ∏ₙ Tₙ` determines band
/// structure: `|Tr(T)/2| ≤ 1` ⟹ band, `> 1` ⟹ gap.
#[must_use]
pub fn transfer_matrix_half_trace(energy: f64, potential: &[f64], hopping: f64) -> f64 {
    let mut mat = [1.0_f64, 0.0, 0.0, 1.0]; // [a, b, c, d] = identity

    for &v_n in potential {
        let x = (energy - v_n) / hopping;
        let new_a = x.mul_add(mat[0], -mat[2]);
        let new_b = x.mul_add(mat[1], -mat[3]);
        mat = [new_a, new_b, mat[0], mat[1]];
    }

    f64::midpoint(mat[0], mat[3])
}

/// Scan the energy range for band edges (sign changes of `|Tr/2| − 1`).
///
/// Returns energies where the system transitions between band and gap.
#[must_use]
pub fn find_band_edges(
    potential: &[f64],
    hopping: f64,
    e_lo: f64,
    e_hi: f64,
    n_points: usize,
) -> Vec<f64> {
    let mut edges = Vec::new();
    let step = (e_hi - e_lo) / usize_f64(n_points - 1);
    let mut prev_in_band: Option<bool> = None;

    for i in 0..n_points {
        let e = usize_f64(i).mul_add(step, e_lo);
        let ht = transfer_matrix_half_trace(e, potential, hopping);
        let in_band = ht.abs() <= 1.0;

        if let Some(prev) = prev_in_band {
            if in_band != prev {
                edges.push(e);
            }
        }
        prev_in_band = Some(in_band);
    }

    edges
}

/// Count distinct bands in the energy range.
#[must_use]
pub fn count_bands(
    potential: &[f64],
    hopping: f64,
    e_lo: f64,
    e_hi: f64,
    n_points: usize,
) -> usize {
    let step = (e_hi - e_lo) / usize_f64(n_points - 1);
    let mut in_band = false;
    let mut n_bands = 0;

    for i in 0..n_points {
        let e = usize_f64(i).mul_add(step, e_lo);
        let ht = transfer_matrix_half_trace(e, potential, hopping);
        let currently_in = ht.abs() <= 1.0;
        if currently_in && !in_band {
            n_bands += 1;
        }
        in_band = currently_in;
    }

    n_bands
}

/// Build the diagonal and off-diagonal of a finite periodic
/// tridiagonal Hamiltonian.
///
/// Returns `(diag, offdiag)` suitable for
/// [`transport::tridiag_eigh`](crate::transport::tridiag_eigh).
#[must_use]
pub fn periodic_hamiltonian(
    potential: &[f64],
    hopping: f64,
    n_periods: usize,
) -> (Vec<f64>, Vec<f64>) {
    let period = potential.len();
    let n = period * n_periods;
    let diag: Vec<f64> = (0..n).map(|i| potential[i % period]).collect();
    let offdiag = vec![-hopping; n - 1];
    (diag, offdiag)
}

/// Fraction of eigenvalues that lie within bands (|Tr/2| ≤ threshold).
///
/// Useful for verifying that finite-system eigenvalues match
/// infinite-system band structure predictions.
#[must_use]
pub fn eigenvalue_band_fraction(
    eigenvalues: &[f64],
    potential: &[f64],
    hopping: f64,
    tolerance: f64,
) -> f64 {
    if eigenvalues.is_empty() {
        return 1.0;
    }

    let in_band = eigenvalues
        .iter()
        .filter(|&&ev| transfer_matrix_half_trace(ev, potential, hopping).abs() <= 1.0 + tolerance)
        .count();

    usize_f64(in_band) / usize_f64(eigenvalues.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_lattice_single_band() {
        let edges = find_band_edges(&[0.0], 1.0, -4.0, 4.0, 2000);
        assert_eq!(edges.len(), 2, "free lattice should have 2 edges");
        assert!((edges[0] - (-2.0)).abs() < 0.05);
        assert!((edges[1] - 2.0).abs() < 0.05);
    }

    #[test]
    fn free_lattice_one_band_count() {
        assert_eq!(count_bands(&[0.0], 1.0, -4.0, 4.0, 2000), 1);
    }

    #[test]
    fn period_2_opens_gap() {
        let pot = [1.0, -1.0];
        let n = count_bands(&pot, 1.0, -4.0, 4.0, 2000);
        assert_eq!(n, 2, "period-2 should have 2 bands");
    }

    #[test]
    fn period_2_gap_width() {
        let pot = [1.0, -1.0];
        let edges = find_band_edges(&pot, 1.0, -4.0, 4.0, 2000);
        assert_eq!(edges.len(), 4);
        let gap = edges[2] - edges[1];
        assert!(
            (gap - 2.0).abs() < 0.15,
            "gap width should be ≈ |V1-V2| = 2, got {gap}"
        );
    }

    #[test]
    fn period_3_has_three_bands() {
        let pot = [1.5, 0.0, -0.5];
        let n = count_bands(&pot, 1.0, -4.0, 4.0, 2000);
        assert_eq!(n, 3, "period-3 should have 3 bands");
    }

    #[test]
    fn gap_width_increases_with_contrast() {
        let gw = |dv: f64| {
            let pot = [dv / 2.0, -dv / 2.0];
            let edges = find_band_edges(&pot, 1.0, -4.0, 4.0, 2000);
            if edges.len() >= 4 {
                edges[2] - edges[1]
            } else {
                0.0
            }
        };
        assert!(gw(2.0) > gw(1.0));
        assert!(gw(3.0) > gw(2.0));
    }

    #[test]
    fn transfer_matrix_deterministic() {
        let a = transfer_matrix_half_trace(0.5, &[1.0, -1.0], 1.0);
        let b = transfer_matrix_half_trace(0.5, &[1.0, -1.0], 1.0);
        assert!((a - b).abs() < f64::EPSILON);
    }

    #[test]
    fn periodic_hamiltonian_sizes() {
        let (diag, offdiag) = periodic_hamiltonian(&[1.0, -1.0], 1.0, 50);
        assert_eq!(diag.len(), 100);
        assert_eq!(offdiag.len(), 99);
    }

    #[test]
    fn eigenvalue_band_fraction_all_in_band() {
        let eigenvalues = vec![0.0, 0.5, -0.5, 1.0, -1.0];
        let frac = eigenvalue_band_fraction(&eigenvalues, &[0.0], 1.0, 0.05);
        assert!((frac - 1.0).abs() < f64::EPSILON);
    }
}
