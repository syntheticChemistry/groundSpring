# Exp 033-034: Anderson Localization in Immunological Signaling

## Domain
Immunological (tissue geometry + drug repurposing) — Paper 12

## Question
Can Anderson localization theory predict cytokine signal propagation in
skin tissue, and can tissue geometry inform drug repurposing by quantifying
spatial penetration alongside pathway targeting?

## Connection to groundSpring

| Anderson QS (Paper 01)     | Immunological Extension (Paper 12)    |
|---------------------------|---------------------------------------|
| Lattice site              | Cell position in tissue               |
| On-site energy `ε_i`     | Cell type identity                    |
| Hopping `t`              | Cytokine diffusion coefficient        |
| Disorder `W`             | Cell-type heterogeneity (Pielou J')   |
| Dimension `d`            | Tissue geometry (2D epidermal, 3D dermal) |

## Dimensional Promotion–Collapse Duality

Paper 06 (no-till): tillage → dimensional COLLAPSE (3D → 2D) → bad.
Paper 12 (AD): scratching → dimensional PROMOTION (2D → 3D) → bad.
Same physics, opposite direction, context-dependent outcome.

## Method — Exp 033: Cytokine Anderson Lattice

- Multi-layer skin model: quasi-2D epidermis (keratinocytes, 85%) + 3D dermis (fibroblasts, 60%)
- Cell-type composition determines effective disorder W via weighted energy variance
- Lyapunov exponents computed per compartment (20 realizations, seed=42)
- Barrier disruption sweep: 0% → 100% in 11 steps
- d_eff transitions from 2.0 (localized) through 2.5 (critical) to 3.0 (extended)
- Dimensional duality sweep: collapse (−1) → neutral (0) → promotion (+1)

## Method — Exp 034: Geometry-Aware Drug Scoring

- Score = pathway_score × penetration_factor × anderson_factor
- Penetration factor depends on: delivery route, molecular weight, target compartment, barrier integrity
- Anderson factor: disorder ratio to W_c ≈ 16.5 determines signal propagation at target
- AD drug panel: Oclacitinib, Lokivetmab, Dupilumab, Rapamycin, Crisaborole, Nemolizumab

## Results (Rust)

- Healthy epidermis W = 0.79 (low heterogeneity), dermis W = 2.32
- Inflamed dermis W = 2.57, all below W_c = 16.5 (3D propagation maintained)
- Barrier transition at 60% disruption (d_eff crosses 2.5)
- Pielou J'(epidermis) = 0.47 < J'(inflamed) = 0.92
- Systemic drugs reach dermis (penetration > 0.8)
- Topical mAbs blocked by intact barrier (penetration = 0.01)
- Disrupted barrier increases topical penetration 42× (0.18 → 0.76)
- All 6 AD drugs score in [0, 1] with geometry correction

## Validation

- Module: `tissue_anderson` (18 unit tests)
- Binary: `validate-tissue-anderson` (29/29 PASS across 10 validation scenarios)
- GPU tier: 7 tissue Anderson parity checks in `validate-gpu-tier` (73/73 total)

## barracuda Delegation

- Uses `anderson::lyapunov_exponent` (delegates to `barracuda::spectral` when `barracuda-gpu` enabled)
- Tissue potential generated locally, Lyapunov computation inherits GPU delegation
- Drug scoring is CPU-only (combinatorial, not compute-bound)

## Key Finding

Anderson localization provides a quantitative framework for immunological
signal propagation: tissue geometry (d_eff), cell-type heterogeneity (W from
Pielou evenness), and barrier integrity jointly determine whether cytokine
signals localize (confined inflammation) or propagate (systemic response).
Drug repurposing must account for spatial geometry — a drug with 95% pathway
overlap scores 0.01 composite if it cannot penetrate the tissue barrier.
