// SPDX-License-Identifier: AGPL-3.0-only
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
//!
//! # barracuda delegation
//!
//! [`find_band_edges`] is partially delegated:
//! - **Brent refinement** (barracuda-gpu): coarse edges are refined to
//!   machine precision via `barracuda::optimize::brent` (airSpring V035
//!   → barraCuda S71+++). This is the high-value delegation — refinement
//!   dominates accuracy improvement.
//! - **Coarse scan** (CPU): evaluates `|Tr(T(E))/2|` at `n_points` energies.
//!   Each point performs L sequential 2×2 matrix multiplications —
//!   data-dependent chains not expressible in current barraCuda ops.
//!   For typical periods (L=2-10) at 2000 points, this is ~20K
//!   multiplications on CPU, well below GPU dispatch threshold.
//! - [`detect_band_ranges`]: fully delegated to `barracuda::spectral::detect_bands`.
//! - [`transfer_matrix_half_trace`], [`count_bands`]: stays local (CPU).

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
///
/// When the `barracuda` feature is enabled, each coarse-grid sign change
/// is refined using `barracuda::optimize::brent` (airSpring V035 →
/// barraCuda S71+++) to locate the exact band edge to `tol = 1e-12`.
/// Without barracuda, falls back to the coarse-grid scan alone.
#[must_use]
pub fn find_band_edges(
    potential: &[f64],
    hopping: f64,
    e_lo: f64,
    e_hi: f64,
    n_points: usize,
) -> Vec<f64> {
    let coarse = find_band_edges_cpu(potential, hopping, e_lo, e_hi, n_points);
    #[cfg(feature = "barracuda-gpu")]
    {
        refine_edges_brent(potential, hopping, e_lo, e_hi, n_points, &coarse)
    }
    #[cfg(not(feature = "barracuda-gpu"))]
    coarse
}

/// Refine coarse band edges using Brent's method on the function
/// `f(E) = |Tr(T(E))/2| − 1`.  The root of this function is exactly
/// the band edge.
///
/// Cross-spring lineage: `brent` — airSpring V035 (Richards PDE
/// root-finding) → barraCuda S71+++ `barracuda::optimize::brent`
/// → groundSpring band structure refinement.
#[cfg(feature = "barracuda-gpu")]
fn refine_edges_brent(
    potential: &[f64],
    hopping: f64,
    e_lo: f64,
    e_hi: f64,
    n_points: usize,
    coarse_edges: &[f64],
) -> Vec<f64> {
    /// Brent root-finder convergence tolerance for band edge refinement.
    /// Set to machine-precision scale (1e-12) since the transfer matrix
    /// trace is an exact algebraic function of energy.
    const BRENT_TOL: f64 = 1e-12;
    const BRENT_MAX_ITER: usize = 100;

    let step = (e_hi - e_lo) / usize_f64(n_points - 1);

    coarse_edges
        .iter()
        .map(|&edge| {
            let a = (edge - step).max(e_lo);
            let b = (edge + step).min(e_hi);
            let f = |e: f64| transfer_matrix_half_trace(e, potential, hopping).abs() - 1.0;
            barracuda::optimize::brent(f, a, b, BRENT_TOL, BRENT_MAX_ITER)
                .map_or(edge, |result| result.root)
        })
        .collect()
}

fn find_band_edges_cpu(
    potential: &[f64],
    hopping: f64,
    e_lo: f64,
    e_hi: f64,
    n_points: usize,
) -> Vec<f64> {
    let step = (e_hi - e_lo) / usize_f64(n_points - 1);

    let band_flags: Vec<bool> = (0..n_points)
        .map(|i| {
            let e = usize_f64(i).mul_add(step, e_lo);
            transfer_matrix_half_trace(e, potential, hopping).abs() <= 1.0
        })
        .collect();

    band_flags
        .windows(2)
        .enumerate()
        .filter_map(|(i, w)| {
            let e = usize_f64(i + 1).mul_add(step, e_lo);
            (w[0] != w[1]).then_some(e)
        })
        .collect()
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

/// Detect band ranges from eigenvalue spectrum using gap detection.
///
/// When `barracuda-gpu` is enabled, delegates to
/// `barracuda::spectral::detect_bands` (absorbed from hotSpring v0.6
/// spectral theory). Returns `(lo, hi)` pairs defining each band.
#[must_use]
pub fn detect_band_ranges(eigenvalues: &[f64], gap_factor: f64) -> Vec<(f64, f64)> {
    #[cfg(feature = "barracuda-gpu")]
    {
        barracuda::spectral::detect_bands(eigenvalues, gap_factor)
    }
    #[cfg(not(feature = "barracuda-gpu"))]
    detect_band_ranges_cpu(eigenvalues, gap_factor)
}

#[cfg(not(feature = "barracuda-gpu"))]
fn detect_band_ranges_cpu(eigenvalues: &[f64], gap_factor: f64) -> Vec<(f64, f64)> {
    if eigenvalues.is_empty() {
        return Vec::new();
    }
    let mut sorted: Vec<f64> = eigenvalues.to_vec();
    sorted.sort_by(f64::total_cmp);
    let n = sorted.len();
    if n == 1 {
        return vec![(sorted[0], sorted[0])];
    }

    let mean_spacing = (sorted[n - 1] - sorted[0]) / usize_f64(n - 1);
    let threshold = mean_spacing * gap_factor;

    let mut bands = Vec::new();
    let mut band_start = sorted[0];
    for i in 1..n {
        if sorted[i] - sorted[i - 1] > threshold {
            bands.push((band_start, sorted[i - 1]));
            band_start = sorted[i];
        }
    }
    bands.push((band_start, sorted[n - 1]));
    bands
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

    #[test]
    fn detect_band_ranges_single_band() {
        let eigenvalues: Vec<f64> = (0..100)
            .map(|i| f64::from(i).mul_add(4.0 / 99.0, -2.0))
            .collect();
        let bands = detect_band_ranges(&eigenvalues, 3.0);
        assert_eq!(bands.len(), 1, "uniform spectrum should have 1 band");
    }

    #[test]
    fn detect_band_ranges_two_bands() {
        let mut eigenvalues = Vec::new();
        for i in 0..50 {
            eigenvalues.push(f64::from(i).mul_add(0.01, -2.0));
        }
        for i in 0..50 {
            eigenvalues.push(f64::from(i).mul_add(0.01, 1.0));
        }
        let bands = detect_band_ranges(&eigenvalues, 3.0);
        assert_eq!(bands.len(), 2, "gapped spectrum should have 2 bands");
        assert!(bands[0].1 < bands[1].0, "bands should be separated by gap");
    }

    #[test]
    fn detect_band_ranges_empty() {
        let bands = detect_band_ranges(&[], 3.0);
        assert!(bands.is_empty());
    }
}
