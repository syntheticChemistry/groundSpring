// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Tissue geometry types and disorder functions for the Anderson lattice.
//!
//! Defines skin layers, cell types, compartment configurations, and the
//! pure functions that compute disorder from cell composition. These are
//! the building blocks consumed by the simulation functions in `mod.rs`.

use crate::cast::usize_f64;
use crate::prng::Xorshift64;

/// Skin compartment in the tissue Anderson lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkinLayer {
    /// Stratum corneum: acellular barrier (~10-20 µm). No signal propagation.
    StratumCorneum,
    /// Viable epidermis: quasi-2D (4-8 cell layers, ~50-100 µm).
    /// Signals localize under normal conditions.
    Epidermis,
    /// Papillary dermis: full 3D matrix (~100-200 µm). Fibroblasts,
    /// Th2, mast cells, nerve endings. Signals propagate when produced.
    Dermis,
}

/// Cell type identity contributing to disorder `W`.
///
/// Each cell type has a characteristic on-site energy that contributes
/// to the effective disorder in that tissue compartment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    /// Keratinocyte (epidermis, low immune activity). On-site energy ≈ 0.
    Keratinocyte,
    /// Langerhans cell (epidermis, antigen presentation). Moderate disorder.
    LangerhansCell,
    /// Th2 lymphocyte (dermis, cytokine producer). High disorder.
    Th2Cell,
    /// Mast cell (dermis, histamine + cytokines). High disorder.
    MastCell,
    /// Sensory neuron ending (dermis, itch receptor). Moderate disorder.
    Neuron,
    /// Eosinophil (dermis, inflammation amplifier). High disorder.
    Eosinophil,
    /// Fibroblast (dermis, structural). Low disorder.
    Fibroblast,
}

impl CellType {
    /// Characteristic on-site energy for this cell type.
    ///
    /// Higher values represent greater immune activation and cytokine
    /// production capacity, contributing more to effective disorder W.
    #[must_use]
    pub const fn on_site_energy(self) -> f64 {
        match self {
            Self::Keratinocyte => 0.1,
            Self::LangerhansCell => 0.5,
            Self::Th2Cell => 1.2,
            Self::MastCell => 1.0,
            Self::Neuron => 0.4,
            Self::Eosinophil => 0.9,
            Self::Fibroblast => 0.15,
        }
    }
}

/// Configuration for a tissue compartment in the Anderson lattice.
#[derive(Debug, Clone)]
pub struct TissueCompartment {
    /// Skin layer type.
    pub layer: SkinLayer,
    /// Number of lattice sites along each spatial dimension.
    pub sites_per_dim: usize,
    /// Effective dimensionality (2.0 for epidermis, 3.0 for dermis).
    pub d_eff: f64,
    /// Base disorder from cell-type heterogeneity (Pielou evenness).
    pub base_disorder: f64,
    /// Cell type composition (fractions summing to 1.0).
    pub cell_composition: Vec<(CellType, f64)>,
}

/// Result of a tissue Anderson lattice simulation.
#[derive(Debug, Clone)]
pub struct TissueResult {
    /// Lyapunov exponent γ for each compartment.
    pub gamma_per_compartment: Vec<f64>,
    /// Localization length ξ = 1/γ for each compartment.
    pub xi_per_compartment: Vec<f64>,
    /// Effective disorder W for each compartment.
    pub w_per_compartment: Vec<f64>,
    /// Whether cytokines propagate (extended) in each compartment.
    pub signal_extended: Vec<bool>,
    /// Whether cytokines can cross the barrier between compartments.
    pub barrier_breached: bool,
    /// Effective dimensionality of the combined system.
    pub d_eff_system: f64,
}

/// Compute effective disorder W from cell-type composition.
///
/// The disorder strength is the weighted standard deviation of on-site
/// energies across cell types, scaled by a heterogeneity factor.
/// Higher cell-type diversity → higher W → more signal scattering.
#[must_use]
pub fn effective_disorder(composition: &[(CellType, f64)]) -> f64 {
    if composition.is_empty() {
        return 0.0;
    }

    let mean_energy: f64 = composition
        .iter()
        .map(|&(ct, frac)| ct.on_site_energy() * frac)
        .sum();

    let variance: f64 = composition
        .iter()
        .map(|&(ct, frac)| {
            let dev = ct.on_site_energy() - mean_energy;
            frac * dev * dev
        })
        .sum();

    variance.sqrt() * 6.0
}

/// Compute Pielou evenness J' from cell-type composition.
///
/// J' = H' / ln(S) where H' is Shannon entropy and S is species richness.
/// Perfect evenness (J'=1) means maximum disorder; low evenness means
/// one cell type dominates (lower effective W).
#[must_use]
pub fn pielou_evenness(composition: &[(CellType, f64)]) -> f64 {
    let s = composition.iter().filter(|&&(_, f)| f > 0.0).count();
    if s <= 1 {
        return 0.0;
    }
    let h: f64 = composition
        .iter()
        .filter(|&&(_, f)| f > 0.0)
        .map(|&(_, f)| -f * f.ln())
        .sum();
    h / usize_f64(s).ln()
}

/// Generate a tissue-specific Anderson potential.
///
/// On-site energies are drawn from a mixture distribution where each
/// cell type contributes its characteristic energy with random perturbation.
/// The perturbation magnitude is scaled by the compartment's base disorder.
#[must_use]
pub fn tissue_potential(n_sites: usize, compartment: &TissueCompartment, seed: u64) -> Vec<f64> {
    let mut rng = Xorshift64::new(seed);
    let w = compartment.base_disorder;
    let half_w = w / 2.0;

    (0..n_sites)
        .map(|_| {
            let u = rng.next_f64();
            let cell = select_cell_type(&compartment.cell_composition, u);
            let perturbation = rng.next_f64().mul_add(w, -half_w);
            cell.on_site_energy() + perturbation
        })
        .collect()
}

/// Select a cell type from composition fractions using a uniform random value.
pub(super) fn select_cell_type(composition: &[(CellType, f64)], u: f64) -> CellType {
    let mut cumulative = 0.0;
    for &(ct, frac) in composition {
        cumulative += frac;
        if u < cumulative {
            return ct;
        }
    }
    composition
        .last()
        .map_or(CellType::Keratinocyte, |&(ct, _)| ct)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn effective_disorder_empty() {
        assert!((effective_disorder(&[]) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn effective_disorder_single_cell_type() {
        let comp = vec![(CellType::Keratinocyte, 1.0)];
        assert!(
            effective_disorder(&comp) < f64::EPSILON,
            "single cell type should have zero disorder"
        );
    }

    #[test]
    fn effective_disorder_heterogeneous() {
        let comp = vec![(CellType::Keratinocyte, 0.5), (CellType::Th2Cell, 0.5)];
        let w = effective_disorder(&comp);
        assert!(
            w > 0.5,
            "mixed Th2+keratinocyte should have substantial disorder: {w}"
        );
    }

    #[test]
    fn pielou_evenness_single_type() {
        let comp = vec![(CellType::Fibroblast, 1.0)];
        assert!((pielou_evenness(&comp) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn pielou_evenness_perfectly_even() {
        let comp = vec![
            (CellType::Keratinocyte, 0.25),
            (CellType::Th2Cell, 0.25),
            (CellType::MastCell, 0.25),
            (CellType::Fibroblast, 0.25),
        ];
        let j = pielou_evenness(&comp);
        assert!(
            (j - 1.0).abs() < tol::ANALYTICAL,
            "perfectly even should be J'=1: {j}"
        );
    }

    #[test]
    fn pielou_evenness_uneven() {
        let comp = vec![(CellType::Keratinocyte, 0.9), (CellType::Th2Cell, 0.1)];
        let j = pielou_evenness(&comp);
        assert!(j < 0.8, "highly uneven should have low J': {j}");
        assert!(j > 0.0, "non-zero diversity: {j}");
    }

    #[test]
    fn cell_type_energies_ordered() {
        assert!(CellType::Keratinocyte.on_site_energy() < CellType::Th2Cell.on_site_energy());
        assert!(CellType::Fibroblast.on_site_energy() < CellType::MastCell.on_site_energy());
    }

    #[test]
    fn tissue_potential_correct_length() {
        let comp = crate::tissue_anderson::compartments::healthy_epidermis();
        let pot = tissue_potential(100, &comp, 42);
        assert_eq!(pot.len(), 100);
    }

    #[test]
    fn tissue_potential_deterministic() {
        let comp = crate::tissue_anderson::compartments::healthy_epidermis();
        let a = tissue_potential(100, &comp, 42);
        let b = tissue_potential(100, &comp, 42);
        assert_eq!(a, b, "same seed must produce identical potential");
    }
}
