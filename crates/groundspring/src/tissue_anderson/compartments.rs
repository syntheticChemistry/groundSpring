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
