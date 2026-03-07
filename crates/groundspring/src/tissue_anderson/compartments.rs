// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Preset tissue compartment constructors for Anderson lattice modeling.
//!
//! Provides canonical configurations for healthy, inflamed, and disrupted
//! skin compartments used in Paper 12 (Exp 033–034).

use super::{CellType, SkinLayer, TissueCompartment};

/// Healthy epidermis composition (low disorder, quasi-2D).
#[must_use]
pub fn healthy_epidermis() -> TissueCompartment {
    TissueCompartment {
        layer: SkinLayer::Epidermis,
        sites_per_dim: 50,
        d_eff: 2.0,
        base_disorder: 0.5,
        cell_composition: vec![
            (CellType::Keratinocyte, 0.85),
            (CellType::LangerhansCell, 0.10),
            (CellType::Neuron, 0.05),
        ],
    }
}

/// Healthy dermis composition (moderate disorder, 3D).
#[must_use]
pub fn healthy_dermis() -> TissueCompartment {
    TissueCompartment {
        layer: SkinLayer::Dermis,
        sites_per_dim: 30,
        d_eff: 3.0,
        base_disorder: 1.5,
        cell_composition: vec![
            (CellType::Fibroblast, 0.60),
            (CellType::Neuron, 0.15),
            (CellType::MastCell, 0.10),
            (CellType::Th2Cell, 0.10),
            (CellType::Eosinophil, 0.05),
        ],
    }
}

/// Inflamed dermis (AD flare): Th2 cells and eosinophils infiltrate,
/// increasing heterogeneity and disorder.
#[must_use]
pub fn inflamed_dermis() -> TissueCompartment {
    TissueCompartment {
        layer: SkinLayer::Dermis,
        sites_per_dim: 30,
        d_eff: 3.0,
        base_disorder: 3.0,
        cell_composition: vec![
            (CellType::Fibroblast, 0.30),
            (CellType::Th2Cell, 0.25),
            (CellType::Eosinophil, 0.15),
            (CellType::MastCell, 0.15),
            (CellType::Neuron, 0.10),
            (CellType::LangerhansCell, 0.05),
        ],
    }
}

/// Barrier-disrupted epidermis (scratching → dimensional promotion).
///
/// Scratching opens 3D channels through the normally 2D barrier,
/// increasing `d_eff` from 2.0 toward 3.0. The `breach_fraction`
/// parameter controls how much of the barrier is disrupted.
#[must_use]
pub fn disrupted_epidermis(breach_fraction: f64) -> TissueCompartment {
    let clamped = breach_fraction.clamp(0.0, 1.0);
    TissueCompartment {
        layer: SkinLayer::Epidermis,
        sites_per_dim: 50,
        d_eff: 2.0_f64.mul_add(1.0 - clamped, 3.0 * clamped),
        base_disorder: 0.5 + clamped * 2.0,
        cell_composition: vec![
            (CellType::Keratinocyte, 0.85 - clamped * 0.25),
            (CellType::LangerhansCell, 0.10 + clamped * 0.10),
            (CellType::Neuron, 0.05 + clamped * 0.05),
            (CellType::Th2Cell, clamped * 0.10),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healthy_epidermis_is_2d() {
        let epi = healthy_epidermis();
        assert_eq!(epi.layer, SkinLayer::Epidermis);
        assert!(
            (epi.d_eff - 2.0).abs() < f64::EPSILON,
            "healthy epi should be quasi-2D"
        );
    }

    #[test]
    fn healthy_epidermis_low_disorder() {
        let epi = healthy_epidermis();
        assert!(
            epi.base_disorder < 1.0,
            "healthy epi should have low disorder"
        );
    }

    #[test]
    fn healthy_epidermis_composition_sums_to_one() {
        let epi = healthy_epidermis();
        let total: f64 = epi.cell_composition.iter().map(|(_, f)| f).sum();
        assert!(
            (total - 1.0).abs() < 1e-10,
            "composition should sum to 1.0, got {total}"
        );
    }

    #[test]
    fn healthy_dermis_is_3d() {
        let derm = healthy_dermis();
        assert_eq!(derm.layer, SkinLayer::Dermis);
        assert!(
            (derm.d_eff - 3.0).abs() < f64::EPSILON,
            "healthy dermis should be 3D"
        );
    }

    #[test]
    fn inflamed_dermis_higher_disorder() {
        let healthy = healthy_dermis();
        let inflamed = inflamed_dermis();
        assert!(
            inflamed.base_disorder > healthy.base_disorder,
            "inflamed should have higher disorder"
        );
    }

    #[test]
    fn inflamed_dermis_composition_sums_to_one() {
        let derm = inflamed_dermis();
        let total: f64 = derm.cell_composition.iter().map(|(_, f)| f).sum();
        assert!(
            (total - 1.0).abs() < 1e-10,
            "composition should sum to 1.0, got {total}"
        );
    }

    #[test]
    fn disrupted_at_zero_matches_healthy() {
        let healthy = healthy_epidermis();
        let disrupted = disrupted_epidermis(0.0);
        assert!((disrupted.d_eff - healthy.d_eff).abs() < 1e-10);
        assert!((disrupted.base_disorder - healthy.base_disorder).abs() < 1e-10);
    }

    #[test]
    fn disrupted_at_one_is_3d() {
        let disrupted = disrupted_epidermis(1.0);
        assert!(
            (disrupted.d_eff - 3.0).abs() < 1e-10,
            "fully disrupted should be 3D"
        );
    }

    #[test]
    fn disrupted_clamped_above_one() {
        let d1 = disrupted_epidermis(1.0);
        let d2 = disrupted_epidermis(1.5);
        assert!((d1.d_eff - d2.d_eff).abs() < 1e-10, "should clamp at 1.0");
    }

    #[test]
    fn disrupted_monotonic_dimension() {
        let d0 = disrupted_epidermis(0.0);
        let d05 = disrupted_epidermis(0.5);
        let d1 = disrupted_epidermis(1.0);
        assert!(
            d05.d_eff > d0.d_eff,
            "dimension should increase with breach"
        );
        assert!(
            d1.d_eff > d05.d_eff,
            "dimension should increase with breach"
        );
    }
}
