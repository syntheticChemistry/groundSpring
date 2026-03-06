// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Anderson localization in tissue geometry (Paper 12, Exp 033–034).
//!
//! Extends the 1D Anderson framework from microbial quorum sensing
//! (Papers 01, 05, 06) to immunological cytokine signaling in skin tissue.
//!
//! # Tissue as Anderson lattice
//!
//! | Anderson QS (Paper 01)     | Immunological Extension           |
//! |---------------------------|-----------------------------------|
//! | Lattice site              | Cell position in tissue           |
//! | On-site energy `ε_i`     | Cell type identity (keratinocyte, Th2, neuron) |
//! | Hopping parameter `t`    | Cytokine diffusion coefficient    |
//! | Disorder `W`             | Cell-type heterogeneity (Pielou evenness) |
//! | Dimension `d`            | Tissue geometry (`d=2` epidermis, `d=3` dermis) |
//! | Level spacing ratio `r`  | Diagnostic: signal extended vs localized |
//!
//! # Dimensional promotion–collapse duality
//!
//! Paper 06 (no-till): tillage → dimensional COLLAPSE (3D → 2D) → bad.
//! Paper 12 (AD): scratching → dimensional PROMOTION (2D → 3D) → bad.
//! Same physics, opposite direction, context-dependent outcome.
//!
//! # Exp 033: Cytokine Anderson lattice
//!
//! Multi-layer skin model: quasi-2D epidermis (thin slab) coupled to
//! 3D dermis (full lattice). Barrier disruption opens channels between
//! compartments, changing `d_eff` and enabling signal delocalization.
//!
//! # Exp 034: Geometry-aware drug scoring
//!
//! Anderson-augmented drug repurposing score: `Score(drug, tissue) =
//! pathway_score × geometry_factor`. The geometry factor quantifies
//! whether the drug can physically reach its target through tissue
//! Anderson geometry.

mod drug_scoring;

pub mod compartments;
pub mod sweeps;

pub use compartments::*;
pub use drug_scoring::{
    DeliveryRoute, DrugCandidate, DrugScore, TissueState, ad_drug_panel, geometry_drug_score,
    score_drug_panel,
};
pub use sweeps::*;

use crate::anderson::lyapunov_exponent;
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
fn select_cell_type(composition: &[(CellType, f64)], u: f64) -> CellType {
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

/// Simulate cytokine propagation through a multi-compartment tissue.
///
/// For each compartment, generates a 1D chain of the appropriate length
/// and computes the Lyapunov exponent to determine whether cytokine
/// signals are localized (confined) or extended (propagating).
///
/// The barrier is considered breached when the epidermis `d_eff > 2.5`
/// (dimensional promotion allows 3D diffusion channels).
#[must_use]
pub fn simulate_tissue(
    compartments: &[TissueCompartment],
    n_realizations: usize,
    base_seed: u64,
) -> TissueResult {
    let mut gamma_per_compartment = Vec::with_capacity(compartments.len());
    let mut xi_per_compartment = Vec::with_capacity(compartments.len());
    let mut w_per_compartment = Vec::with_capacity(compartments.len());
    let mut signal_extended = Vec::with_capacity(compartments.len());

    for (ci, comp) in compartments.iter().enumerate() {
        let n_sites = comp.sites_per_dim * comp.sites_per_dim;
        let w_eff = effective_disorder(&comp.cell_composition) + comp.base_disorder;
        w_per_compartment.push(w_eff);

        let mut gamma_sum = 0.0;
        for r in 0..n_realizations {
            let seed = base_seed + (ci as u64) * 10_000 + (r as u64);
            let potential = tissue_potential(n_sites, comp, seed);
            gamma_sum += lyapunov_exponent(&potential, 0.0);
        }
        let gamma_mean = gamma_sum / usize_f64(n_realizations);
        gamma_per_compartment.push(gamma_mean);

        let xi = xi_from_gamma(gamma_mean);
        xi_per_compartment.push(xi);

        let extended = is_signal_extended(gamma_mean, comp.d_eff, w_eff);
        signal_extended.push(extended);
    }

    let barrier_breached = compartments
        .iter()
        .any(|c| c.layer == SkinLayer::Epidermis && c.d_eff > 2.5);

    let d_eff_system = if barrier_breached {
        compartments
            .iter()
            .map(|c| c.d_eff)
            .fold(f64::NEG_INFINITY, f64::max)
    } else {
        compartments
            .iter()
            .map(|c| c.d_eff)
            .fold(f64::INFINITY, f64::min)
    };

    TissueResult {
        gamma_per_compartment,
        xi_per_compartment,
        w_per_compartment,
        signal_extended,
        barrier_breached,
        d_eff_system,
    }
}

/// Determine whether a signal is extended (propagating) or localized.
///
/// In `d=3`, the Anderson metal-insulator transition occurs at `W_c` ≈ 16.5.
/// Below `W_c` signals propagate; above they localize. In `d=2`, ALL states
/// are localized for any `W > 0` (no mobility edge), but the localization
/// length may be larger than the system size.
///
/// For tissue: we use the localization length relative to the compartment
/// size as the diagnostic. If `ξ > L_compartment`, signal effectively
/// propagates within that compartment.
fn is_signal_extended(gamma: f64, d_eff: f64, w_eff: f64) -> bool {
    if d_eff >= 2.5 {
        w_eff < W_C_3D
    } else {
        xi_from_gamma(gamma) > 100.0
    }
}

/// Critical disorder for 3D Anderson metal-insulator transition.
const W_C_3D: f64 = 16.5;

/// Localization length from Lyapunov exponent: ξ = 1/γ.
fn xi_from_gamma(gamma: f64) -> f64 {
    if gamma > 0.0 {
        1.0 / gamma
    } else {
        f64::INFINITY
    }
}

/// Simulate tissue with spatially correlated disorder.
///
/// When `barracuda-gpu` is enabled, uses `anderson_3d_correlated` to generate
/// disorder potentials with spatial correlation length `xi_corr`. This models
/// cell-type clustering (e.g., Langerhans cell networks in epidermis) where
/// neighboring sites share similar disorder. Falls back to uncorrelated
/// `simulate_tissue` when GPU is unavailable.
///
/// `xi_corr` is the spatial correlation length in lattice units:
/// - `xi_corr < 0.01`: uncorrelated (same as `simulate_tissue`)
/// - `xi_corr ~ 1.0`: short-range clustering (typical tissue)
/// - `xi_corr > 5.0`: long-range order (structured tissue layers)
#[cfg(feature = "barracuda-gpu")]
#[must_use]
pub fn correlated_tissue_simulation(
    compartments: &[TissueCompartment],
    xi_corr: f64,
    n_eigenvalues: usize,
    base_seed: u64,
) -> CorrelatedTissueResult {
    let mut compartment_results = Vec::with_capacity(compartments.len());

    for (ci, comp) in compartments.iter().enumerate() {
        let l = comp.sites_per_dim;
        let w_eff = effective_disorder(&comp.cell_composition) + comp.base_disorder;
        let seed = base_seed + (ci as u64) * 10_000;

        let csr = barracuda::spectral::anderson_3d_correlated(l, w_eff, xi_corr, seed);
        let mut eigenvalues =
            crate::lanczos::eigenvalues_from_csr(&csr, n_eigenvalues, seed.wrapping_add(1));

        let r_ratio = if eigenvalues.len() >= 3 {
            crate::almost_mathieu::level_spacing_ratio(&mut eigenvalues)
        } else {
            0.0
        };

        compartment_results.push(CorrelatedCompartmentResult {
            layer: comp.layer,
            d_eff: comp.d_eff,
            w_eff,
            xi_corr,
            eigenvalues,
            level_spacing_ratio: r_ratio,
        });
    }

    CorrelatedTissueResult {
        compartments: compartment_results,
    }
}

/// Result from correlated tissue simulation.
#[cfg(feature = "barracuda-gpu")]
#[derive(Debug, Clone)]
pub struct CorrelatedTissueResult {
    /// Per-compartment results.
    pub compartments: Vec<CorrelatedCompartmentResult>,
}

/// Per-compartment result from correlated tissue simulation.
#[cfg(feature = "barracuda-gpu")]
#[derive(Debug, Clone)]
pub struct CorrelatedCompartmentResult {
    /// Skin layer.
    pub layer: SkinLayer,
    /// Effective dimensionality.
    pub d_eff: f64,
    /// Effective disorder W.
    pub w_eff: f64,
    /// Spatial correlation length used.
    pub xi_corr: f64,
    /// Lowest eigenvalues from Lanczos.
    pub eigenvalues: Vec<f64>,
    /// Level spacing ratio (diagnostic: ~0.39 Poisson/localized, ~0.53 GOE/extended).
    pub level_spacing_ratio: f64,
}

/// 4D Anderson tissue simulation for spatio-temporal disorder modeling.
///
/// Constructs a 4D lattice where the first three dimensions represent tissue
/// space (x, y, z) and the fourth represents an immune response gradient
/// (e.g., cytokine concentration over time). Uses `barracuda::spectral::anderson_4d`
/// (absorbed barraCuda S84) to build the Hamiltonian, then Lanczos for eigenvalues.
///
/// Cross-spring lineage: hotSpring precision shaders (DF64 Lanczos) →
/// barraCuda S84 `anderson_4d` → groundSpring 4D tissue disorder.
#[cfg(feature = "barracuda-gpu")]
#[must_use]
pub fn tissue_4d_simulation(
    l: usize,
    disorder: f64,
    n_eigenvalues: usize,
    seed: u64,
) -> Tissue4dResult {
    let csr = barracuda::spectral::anderson::anderson_4d(l, disorder, seed);
    let mut eigenvalues =
        crate::lanczos::eigenvalues_from_csr(&csr, n_eigenvalues, seed.wrapping_add(1));

    let r_ratio = if eigenvalues.len() >= 3 {
        crate::almost_mathieu::level_spacing_ratio(&mut eigenvalues)
    } else {
        0.0
    };

    Tissue4dResult {
        l,
        disorder,
        dimension: 4,
        n_sites: l.pow(4),
        eigenvalues,
        level_spacing_ratio: r_ratio,
    }
}

/// 4D Wegner block renormalization group coarsening for tissue modeling.
///
/// Applies Wegner's real-space RG to the 4D Anderson Hamiltonian, coarsening
/// the lattice by a factor of 2 in each dimension. This reveals how disorder
/// flows under coarse-graining — critical for tissue models where the relevant
/// length scale spans cell clusters rather than individual cells.
///
/// Cross-spring lineage: hotSpring precision + condensed matter →
/// barraCuda S84 `wegner_block_4d` → groundSpring tissue RG.
#[cfg(feature = "barracuda-gpu")]
#[must_use]
pub fn tissue_4d_rg_coarsen(
    l: usize,
    disorder: f64,
    n_eigenvalues: usize,
    seed: u64,
) -> (Tissue4dResult, Tissue4dResult) {
    let csr_fine = barracuda::spectral::anderson::anderson_4d(l, disorder, seed);
    let csr_coarse = barracuda::spectral::anderson::wegner_block_4d(&csr_fine, l);
    let l_coarse = l / 2;

    let mut eig_fine =
        crate::lanczos::eigenvalues_from_csr(&csr_fine, n_eigenvalues, seed.wrapping_add(1));
    let mut eig_coarse =
        crate::lanczos::eigenvalues_from_csr(&csr_coarse, n_eigenvalues, seed.wrapping_add(2));

    let r_fine = if eig_fine.len() >= 3 {
        crate::almost_mathieu::level_spacing_ratio(&mut eig_fine)
    } else {
        0.0
    };
    let r_coarse = if eig_coarse.len() >= 3 {
        crate::almost_mathieu::level_spacing_ratio(&mut eig_coarse)
    } else {
        0.0
    };

    let fine = Tissue4dResult {
        l,
        disorder,
        dimension: 4,
        n_sites: l.pow(4),
        eigenvalues: eig_fine,
        level_spacing_ratio: r_fine,
    };
    let coarse = Tissue4dResult {
        l: l_coarse,
        disorder,
        dimension: 4,
        n_sites: l_coarse.pow(4),
        eigenvalues: eig_coarse,
        level_spacing_ratio: r_coarse,
    };

    (fine, coarse)
}

/// Result from 4D Anderson tissue simulation.
#[cfg(feature = "barracuda-gpu")]
#[derive(Debug, Clone)]
pub struct Tissue4dResult {
    /// Linear lattice size in each dimension.
    pub l: usize,
    /// Disorder strength W.
    pub disorder: f64,
    /// Spatial dimension (always 4).
    pub dimension: u8,
    /// Total number of lattice sites (l^4).
    pub n_sites: usize,
    /// Lowest eigenvalues from Lanczos.
    pub eigenvalues: Vec<f64>,
    /// Level spacing ratio (diagnostic: ~0.39 Poisson/localized, ~0.53 GOE/extended).
    pub level_spacing_ratio: f64,
}

/// Find the critical disorder `W_c` for a tissue barrier transition.
///
/// Performs a disorder sweep over the specified range and uses barracuda's
/// `find_w_c` interpolation to locate the transition point where the level
/// spacing ratio crosses the midpoint between Poisson (localized, r ≈ 0.386)
/// and GOE (extended, r ≈ 0.530).
///
/// Returns `None` if no transition is found in the sweep range.
#[cfg(feature = "barracuda-gpu")]
#[must_use]
pub fn find_barrier_transition_w_c(
    n_sites: usize,
    w_min: f64,
    w_max: f64,
    n_points: usize,
    n_realizations: usize,
    base_seed: u64,
) -> Option<f64> {
    let sweep = barracuda::spectral::anderson_sweep_averaged(
        n_sites,
        w_min,
        w_max,
        n_points,
        n_realizations,
        base_seed,
    );

    let midpoint = 0.458;
    barracuda::spectral::find_w_c(&sweep, midpoint)
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
    fn healthy_skin_localizes() {
        let epi = healthy_epidermis();
        let derm = healthy_dermis();
        let result = simulate_tissue(&[epi, derm], 10, 42);
        assert_eq!(result.gamma_per_compartment.len(), 2);
        assert!(
            !result.barrier_breached,
            "healthy skin barrier should be intact"
        );
    }

    #[test]
    fn inflamed_dermis_signal_propagates() {
        let derm = inflamed_dermis();
        let result = simulate_tissue(&[derm], 10, 42);
        assert!(
            result.w_per_compartment[0] < W_C_3D,
            "inflamed dermis W={} should be below W_c={}",
            result.w_per_compartment[0],
            W_C_3D,
        );
    }

    #[test]
    fn barrier_disruption_opens_propagation() {
        let epi_healthy = disrupted_epidermis(0.0);
        let epi_breached = disrupted_epidermis(1.0);
        assert!(epi_healthy.d_eff < 2.5, "healthy d_eff should be 2D-ish");
        assert!(
            epi_breached.d_eff > 2.5,
            "breached d_eff should approach 3D"
        );
    }

    #[test]
    fn barrier_disruption_sweep_transitions() {
        let sweep = barrier_disruption_sweep(11, 5, 42);
        assert_eq!(sweep.len(), 11);
        assert!(!sweep[0].barrier_breached, "frac=0 should not breach");
        assert!(sweep[10].barrier_breached, "frac=1 should breach");
    }

    #[test]
    fn dimensional_duality_sweep_covers_range() {
        let sweep = dimensional_duality_sweep(11, 5, 42);
        assert_eq!(sweep.len(), 11);
        assert!(sweep[0].parameter < -0.9, "first point near -1");
        assert!(sweep[10].parameter > 0.9, "last point near +1");
        assert!(sweep[0].context.contains("collapse"));
        assert!(sweep[10].context.contains("promotion"));
    }

    #[test]
    fn cell_type_energies_ordered() {
        assert!(CellType::Keratinocyte.on_site_energy() < CellType::Th2Cell.on_site_energy());
        assert!(CellType::Fibroblast.on_site_energy() < CellType::MastCell.on_site_energy());
    }

    #[test]
    fn tissue_potential_correct_length() {
        let comp = healthy_epidermis();
        let pot = tissue_potential(100, &comp, 42);
        assert_eq!(pot.len(), 100);
    }

    #[test]
    fn tissue_potential_deterministic() {
        let comp = healthy_epidermis();
        let a = tissue_potential(100, &comp, 42);
        let b = tissue_potential(100, &comp, 42);
        assert_eq!(a, b, "same seed must produce identical potential");
    }
}
