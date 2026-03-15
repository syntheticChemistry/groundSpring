// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Anderson-augmented drug repurposing scoring (Exp 034).
//!
//! Extends the Fajgenbaum MATRIX pathway-based scoring with a spatial
//! geometry factor. A drug must both target the right pathway AND
//! physically reach its target through tissue Anderson geometry.

use super::{SkinLayer, W_C_3D};

// ─── Penetration Model Constants ─────────────────────────────────────────────
//
// Molecular weight thresholds and penetration penalties from Lipinski's
// Rule of Five and topical drug delivery literature.  Size penalties model
// the probability of crossing the stratum corneum as an intact barrier.

/// MW above which topical penetration is essentially blocked (e.g. mAbs).
const MW_BLOCKED_DA: f64 = 100_000.0;
/// Residual penetration probability for molecules above `MW_BLOCKED_DA`.
const PENETRATION_BLOCKED: f64 = 0.05;

/// MW above which topical penetration is heavily attenuated (large peptides).
const MW_LARGE_DA: f64 = 10_000.0;
/// Penetration probability for molecules in the `MW_LARGE_DA`..`MW_BLOCKED_DA` range.
const PENETRATION_LARGE: f64 = 0.2;

/// MW above which topical penetration is moderate (Lipinski boundary ~500 Da).
const MW_MODERATE_DA: f64 = 500.0;
/// Penetration probability for molecules in the `MW_MODERATE_DA`..`MW_LARGE_DA` range.
const PENETRATION_MODERATE: f64 = 0.5;

/// Penetration probability for small molecules (MW ≤ 500 Da).
const PENETRATION_SMALL: f64 = 0.9;

/// Baseline bioavailability at the stratum corneum (topical surface layer).
const BIOAVAIL_STRATUM_CORNEUM: f64 = 0.95;

/// Barrier integrity factor — fraction of intact barrier that blocks deeper
/// dermis penetration. Represents the stratum corneum's effectiveness as a
/// diffusion barrier for topically-applied drugs.
const BARRIER_BLOCK_FACTOR: f64 = 0.8;

/// Minimum penetration factor for a drug to be considered "reachable".
///
/// Below this threshold the drug cannot meaningfully interact with
/// its target compartment regardless of pathway score.
const MIN_REACHABLE_PENETRATION: f64 = 0.3;

// ─── Systemic Penetration Constants ──────────────────────────────────────────

/// Systemic bioavailability at the dermis (blood-borne delivery).
const SYSTEMIC_DERMIS: f64 = 0.95;
/// Systemic bioavailability at the epidermis.
const SYSTEMIC_EPIDERMIS: f64 = 0.7;
/// Systemic bioavailability at the stratum corneum.
const SYSTEMIC_STRATUM: f64 = 0.3;

/// Drug candidate for geometry-aware scoring.
#[derive(Debug, Clone)]
pub struct DrugCandidate {
    /// Drug name.
    pub name: String,
    /// Pathway overlap score from MATRIX or similar (0.0 to 1.0).
    pub pathway_score: f64,
    /// Molecular weight in Daltons.
    pub molecular_weight_da: f64,
    /// Delivery route.
    pub delivery: DeliveryRoute,
    /// Target skin compartment.
    pub target_compartment: SkinLayer,
}

/// Drug delivery route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryRoute {
    /// Oral or injectable — enters via bloodstream, reaches dermis first.
    Systemic,
    /// Applied to skin surface — must cross epidermal barrier.
    Topical,
}

/// Current tissue state for drug scoring.
#[derive(Debug, Clone)]
pub struct TissueState {
    /// Fraction of epidermal barrier disrupted.
    pub barrier_disruption: f64,
    /// Effective dimensionality of the epidermis.
    pub d_eff_epidermis: f64,
    /// Effective disorder in the dermis.
    pub w_dermis: f64,
}

/// Drug repurposing score with Anderson geometry.
#[derive(Debug, Clone)]
pub struct DrugScore {
    /// Raw pathway overlap score.
    pub pathway_score: f64,
    /// Spatial penetration factor (0.0 to 1.0).
    pub penetration_factor: f64,
    /// Anderson geometry factor — signal propagation at target.
    pub anderson_factor: f64,
    /// Composite score = pathway × penetration × Anderson.
    pub composite_score: f64,
    /// Whether the drug physically reaches its target.
    pub reaches_target: bool,
}

/// Compute the geometry-aware drug repurposing score.
///
/// `composite = pathway_score × penetration_factor × anderson_factor`
#[must_use]
pub fn geometry_drug_score(drug: &DrugCandidate, tissue: &TissueState) -> DrugScore {
    let penetration = compute_penetration_factor(drug, tissue);
    let anderson_factor = compute_anderson_factor(drug, tissue);
    let composite = drug.pathway_score * penetration * anderson_factor;

    DrugScore {
        pathway_score: drug.pathway_score,
        penetration_factor: penetration,
        anderson_factor,
        composite_score: composite,
        reaches_target: penetration > MIN_REACHABLE_PENETRATION,
    }
}

/// Compute the spatial penetration factor for a drug in tissue.
///
/// Systemic drugs reach the 3D dermis easily. Topical drugs must cross
/// the 2D epidermal barrier — large molecules (mAbs) are blocked unless
/// the barrier is disrupted. Small molecules can penetrate intact skin.
fn compute_penetration_factor(drug: &DrugCandidate, tissue: &TissueState) -> f64 {
    match drug.delivery {
        DeliveryRoute::Systemic => match drug.target_compartment {
            SkinLayer::Dermis => SYSTEMIC_DERMIS,
            SkinLayer::Epidermis => SYSTEMIC_EPIDERMIS,
            SkinLayer::StratumCorneum => SYSTEMIC_STRATUM,
        },
        DeliveryRoute::Topical => {
            let barrier_intact = 1.0 - tissue.barrier_disruption;
            let size_penalty = if drug.molecular_weight_da > MW_BLOCKED_DA {
                PENETRATION_BLOCKED
            } else if drug.molecular_weight_da > MW_LARGE_DA {
                PENETRATION_LARGE
            } else if drug.molecular_weight_da > MW_MODERATE_DA {
                PENETRATION_MODERATE
            } else {
                PENETRATION_SMALL
            };
            let base = match drug.target_compartment {
                SkinLayer::StratumCorneum => BIOAVAIL_STRATUM_CORNEUM,
                SkinLayer::Epidermis => size_penalty,
                SkinLayer::Dermis => size_penalty * (1.0 - barrier_intact * BARRIER_BLOCK_FACTOR),
            };
            base.clamp(0.0, 1.0)
        }
    }
}

/// Compute the Anderson geometry factor for drug-target interaction.
///
/// In tissue with high effective disorder (near `W_c`), signals localize
/// and drug effects are confined. In tissue with low disorder relative
/// to dimensionality, drug effects propagate to neighboring cells.
fn compute_anderson_factor(drug: &DrugCandidate, tissue: &TissueState) -> f64 {
    let d_eff = match drug.target_compartment {
        SkinLayer::Epidermis | SkinLayer::StratumCorneum => tissue.d_eff_epidermis,
        SkinLayer::Dermis => 3.0,
    };

    if d_eff >= 2.5 {
        let w_ratio = tissue.w_dermis / W_C_3D;
        if w_ratio < 1.0 {
            (-0.5_f64).mul_add(w_ratio, 1.0)
        } else {
            0.3 / w_ratio
        }
    } else {
        0.4
    }
}

/// Score a batch of drug candidates against a tissue state.
#[must_use]
pub fn score_drug_panel(drugs: &[DrugCandidate], tissue: &TissueState) -> Vec<DrugScore> {
    drugs
        .iter()
        .map(|d| geometry_drug_score(d, tissue))
        .collect()
}

/// The AD drug panel from Paper 12 §3.3.
#[must_use]
pub fn ad_drug_panel() -> Vec<DrugCandidate> {
    vec![
        DrugCandidate {
            name: "Oclacitinib (Apoquel)".into(),
            pathway_score: 0.92,
            molecular_weight_da: 337.0,
            delivery: DeliveryRoute::Systemic,
            target_compartment: SkinLayer::Dermis,
        },
        DrugCandidate {
            name: "Lokivetmab (Cytopoint)".into(),
            pathway_score: 0.95,
            molecular_weight_da: 150_000.0,
            delivery: DeliveryRoute::Systemic,
            target_compartment: SkinLayer::Dermis,
        },
        DrugCandidate {
            name: "Dupilumab (Dupixent)".into(),
            pathway_score: 0.90,
            molecular_weight_da: 147_000.0,
            delivery: DeliveryRoute::Systemic,
            target_compartment: SkinLayer::Dermis,
        },
        DrugCandidate {
            name: "Rapamycin (Sirolimus)".into(),
            pathway_score: 0.65,
            molecular_weight_da: 914.0,
            delivery: DeliveryRoute::Systemic,
            target_compartment: SkinLayer::Dermis,
        },
        DrugCandidate {
            name: "Crisaborole (Eucrisa)".into(),
            pathway_score: 0.55,
            molecular_weight_da: 251.0,
            delivery: DeliveryRoute::Topical,
            target_compartment: SkinLayer::Epidermis,
        },
        DrugCandidate {
            name: "Nemolizumab".into(),
            pathway_score: 0.88,
            molecular_weight_da: 145_000.0,
            delivery: DeliveryRoute::Systemic,
            target_compartment: SkinLayer::Dermis,
        },
    ]
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn systemic_reaches_dermis() {
        let tissue = TissueState {
            barrier_disruption: 0.0,
            d_eff_epidermis: 2.0,
            w_dermis: 3.0,
        };
        let drug = DrugCandidate {
            name: "Oclacitinib".into(),
            pathway_score: 0.92,
            molecular_weight_da: 337.0,
            delivery: DeliveryRoute::Systemic,
            target_compartment: SkinLayer::Dermis,
        };
        let score = geometry_drug_score(&drug, &tissue);
        assert!(score.reaches_target);
        assert!(score.composite_score > 0.5);
    }

    #[test]
    fn topical_mab_blocked() {
        let tissue = TissueState {
            barrier_disruption: 0.0,
            d_eff_epidermis: 2.0,
            w_dermis: 3.0,
        };
        let drug = DrugCandidate {
            name: "Topical mAb".into(),
            pathway_score: 0.95,
            molecular_weight_da: 150_000.0,
            delivery: DeliveryRoute::Topical,
            target_compartment: SkinLayer::Dermis,
        };
        let score = geometry_drug_score(&drug, &tissue);
        assert!(score.penetration_factor < 0.15);
    }

    #[test]
    fn topical_mab_on_disrupted_barrier() {
        let tissue = TissueState {
            barrier_disruption: 0.8,
            d_eff_epidermis: 2.8,
            w_dermis: 3.0,
        };
        let drug = DrugCandidate {
            name: "Topical mAb disrupted".into(),
            pathway_score: 0.95,
            molecular_weight_da: 150_000.0,
            delivery: DeliveryRoute::Topical,
            target_compartment: SkinLayer::Dermis,
        };
        let score = geometry_drug_score(&drug, &tissue);
        assert!(score.penetration_factor > 0.01);
    }

    #[test]
    fn ad_panel_scores_valid() {
        let tissue = TissueState {
            barrier_disruption: 0.2,
            d_eff_epidermis: 2.2,
            w_dermis: 4.0,
        };
        let panel = ad_drug_panel();
        let scores = score_drug_panel(&panel, &tissue);
        assert_eq!(scores.len(), 6);
        for score in &scores {
            assert!(score.composite_score >= 0.0);
            assert!(score.composite_score <= 1.0);
        }
    }
}
