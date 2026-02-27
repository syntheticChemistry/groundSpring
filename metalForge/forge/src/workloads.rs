// SPDX-License-Identifier: AGPL-3.0-or-later

//! groundSpring-specific workload definitions for dispatch routing.

use crate::dispatch::Workload;
use crate::substrate::Capability;

/// Anderson transfer matrix / Lyapunov exponent Monte Carlo.
#[must_use]
pub fn anderson_transfer_matrix() -> Workload {
    Workload::new(
        "Anderson transfer matrix (MC)",
        vec![Capability::F64Compute, Capability::ShaderDispatch],
    )
}

/// Almost-Mathieu eigenvalue computation (Sturm tridiag).
#[must_use]
pub fn almost_mathieu_eigenvalues() -> Workload {
    Workload::new(
        "Almost-Mathieu eigenvalues",
        vec![Capability::F64Compute, Capability::ShaderDispatch],
    )
}

/// Green-Kubo integration (f64 precision).
#[must_use]
pub fn green_kubo_integration() -> Workload {
    Workload::new(
        "Green-Kubo integration (f64)",
        vec![Capability::F64Compute, Capability::ScalarReduce],
    )
}

/// Anderson regime classification (quantized int8 inference).
#[must_use]
pub fn anderson_regime_classify() -> Workload {
    Workload::new(
        "Anderson regime classification",
        vec![Capability::QuantizedInference { bits: 8 }],
    )
}

/// Diversity saturation prediction (quantized int8 inference).
#[must_use]
pub fn diversity_saturation_predict() -> Workload {
    Workload::new(
        "Diversity saturation prediction",
        vec![Capability::QuantizedInference { bits: 8 }],
    )
}

/// Bias-variance decomposition (scalar f64 — CPU preferred).
#[must_use]
pub fn bias_variance_decompose() -> Workload {
    Workload::new("Bias-variance decomposition", vec![Capability::F64Compute])
}

/// Finite-size extrapolation (scalar f64 — CPU preferred).
#[must_use]
pub fn finite_size_extrapolation() -> Workload {
    Workload::new("Finite-size extrapolation", vec![Capability::F64Compute])
}

/// Freeze-out 2D chi-squared grid search (embarrassingly parallel).
#[must_use]
pub fn freeze_out_grid_fit() -> Workload {
    Workload::new(
        "Freeze-out 2D grid fit",
        vec![Capability::F64Compute, Capability::ShaderDispatch],
    )
}

/// Seismic 3D grid search inversion (embarrassingly parallel).
#[must_use]
pub fn seismic_grid_search() -> Workload {
    Workload::new(
        "Seismic 3D grid search",
        vec![Capability::F64Compute, Capability::ShaderDispatch],
    )
}

/// Band edge energy scan (per-energy parallel transfer matrix).
#[must_use]
pub fn band_edge_scan() -> Workload {
    Workload::new(
        "Band edge energy scan",
        vec![Capability::F64Compute, Capability::ShaderDispatch],
    )
}

/// Quasispecies Wright-Fisher simulation (batched replicates).
#[must_use]
pub fn quasispecies_wright_fisher() -> Workload {
    Workload::new(
        "Quasispecies Wright-Fisher",
        vec![Capability::F64Compute, Capability::ShaderDispatch],
    )
}

/// Rare biosphere batched multinomial sampling (occupancy / tier rates).
#[must_use]
pub fn rare_biosphere_multinomial() -> Workload {
    Workload::new(
        "Rare biosphere multinomial",
        vec![Capability::F64Compute, Capability::ShaderDispatch],
    )
}

/// Return all groundSpring workloads for dispatch.
#[must_use]
pub fn all() -> Vec<Workload> {
    vec![
        anderson_transfer_matrix(),
        almost_mathieu_eigenvalues(),
        green_kubo_integration(),
        anderson_regime_classify(),
        diversity_saturation_predict(),
        bias_variance_decompose(),
        finite_size_extrapolation(),
        freeze_out_grid_fit(),
        seismic_grid_search(),
        band_edge_scan(),
        quasispecies_wright_fisher(),
        rare_biosphere_multinomial(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch;
    use crate::substrate::{Identity, Properties, Substrate, SubstrateKind};

    fn gpu_with_all_caps() -> Substrate {
        Substrate {
            kind: SubstrateKind::Gpu,
            identity: Identity::named("Test GPU"),
            properties: Properties::default(),
            capabilities: vec![
                Capability::F64Compute,
                Capability::F32Compute,
                Capability::ShaderDispatch,
                Capability::ScalarReduce,
            ],
        }
    }

    fn npu_with_quant() -> Substrate {
        Substrate {
            kind: SubstrateKind::Npu,
            identity: Identity::named("Test NPU"),
            properties: Properties::default(),
            capabilities: vec![
                Capability::F32Compute,
                Capability::QuantizedInference { bits: 8 },
                Capability::BatchInference { max_batch: 8 },
            ],
        }
    }

    fn cpu_baseline() -> Substrate {
        Substrate {
            kind: SubstrateKind::Cpu,
            identity: Identity::named("Test CPU"),
            properties: Properties::default(),
            capabilities: vec![Capability::F64Compute, Capability::F32Compute],
        }
    }

    #[test]
    fn anderson_routes_to_gpu() {
        let subs = [gpu_with_all_caps(), cpu_baseline()];
        let w = anderson_transfer_matrix();
        let d = dispatch::route(&w, &subs).expect("should route");
        assert_eq!(d.substrate.kind, SubstrateKind::Gpu);
    }

    #[test]
    fn mathieu_routes_to_gpu() {
        let subs = [gpu_with_all_caps(), cpu_baseline()];
        let w = almost_mathieu_eigenvalues();
        let d = dispatch::route(&w, &subs).expect("should route");
        assert_eq!(d.substrate.kind, SubstrateKind::Gpu);
    }

    #[test]
    fn green_kubo_routes_to_gpu() {
        let subs = [gpu_with_all_caps(), cpu_baseline()];
        let w = green_kubo_integration();
        let d = dispatch::route(&w, &subs).expect("should route");
        assert_eq!(d.substrate.kind, SubstrateKind::Gpu);
    }

    #[test]
    fn regime_classify_routes_to_npu() {
        let subs = [cpu_baseline(), npu_with_quant()];
        let w = anderson_regime_classify();
        let d = dispatch::route(&w, &subs).expect("should route");
        assert_eq!(d.substrate.kind, SubstrateKind::Npu);
    }

    #[test]
    fn saturation_routes_to_npu() {
        let subs = [cpu_baseline(), npu_with_quant()];
        let w = diversity_saturation_predict();
        let d = dispatch::route(&w, &subs).expect("should route");
        assert_eq!(d.substrate.kind, SubstrateKind::Npu);
    }

    #[test]
    fn bias_variance_routes_to_cpu_when_no_gpu() {
        let subs = [cpu_baseline()];
        let w = bias_variance_decompose();
        let d = dispatch::route(&w, &subs).expect("should route");
        assert_eq!(d.substrate.kind, SubstrateKind::Cpu);
    }

    #[test]
    fn finite_size_routes_to_cpu_when_no_gpu() {
        let subs = [cpu_baseline()];
        let w = finite_size_extrapolation();
        let d = dispatch::route(&w, &subs).expect("should route");
        assert_eq!(d.substrate.kind, SubstrateKind::Cpu);
    }

    #[test]
    fn freeze_out_routes_to_gpu() {
        let subs = [gpu_with_all_caps(), cpu_baseline()];
        let w = freeze_out_grid_fit();
        let d = dispatch::route(&w, &subs).expect("should route");
        assert_eq!(d.substrate.kind, SubstrateKind::Gpu);
    }

    #[test]
    fn seismic_routes_to_gpu() {
        let subs = [gpu_with_all_caps(), cpu_baseline()];
        let w = seismic_grid_search();
        let d = dispatch::route(&w, &subs).expect("should route");
        assert_eq!(d.substrate.kind, SubstrateKind::Gpu);
    }

    #[test]
    fn band_edge_routes_to_gpu() {
        let subs = [gpu_with_all_caps(), cpu_baseline()];
        let w = band_edge_scan();
        let d = dispatch::route(&w, &subs).expect("should route");
        assert_eq!(d.substrate.kind, SubstrateKind::Gpu);
    }

    #[test]
    fn quasispecies_routes_to_gpu() {
        let subs = [gpu_with_all_caps(), cpu_baseline()];
        let w = quasispecies_wright_fisher();
        let d = dispatch::route(&w, &subs).expect("should route");
        assert_eq!(d.substrate.kind, SubstrateKind::Gpu);
    }

    #[test]
    fn rare_biosphere_routes_to_gpu() {
        let subs = [gpu_with_all_caps(), cpu_baseline()];
        let w = rare_biosphere_multinomial();
        let d = dispatch::route(&w, &subs).expect("should route");
        assert_eq!(d.substrate.kind, SubstrateKind::Gpu);
    }

    #[test]
    fn all_returns_twelve_workloads() {
        let workloads = all();
        assert_eq!(workloads.len(), 12);
    }
}
