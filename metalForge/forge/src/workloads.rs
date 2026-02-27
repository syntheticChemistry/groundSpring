// SPDX-License-Identifier: AGPL-3.0-or-later

//! groundSpring-specific workload definitions for dispatch routing.

use crate::dispatch::Workload;
use crate::substrate::Capability;

/// Anderson transfer matrix / Lyapunov exponent Monte Carlo.
pub fn anderson_transfer_matrix() -> Workload {
    Workload::new(
        "Anderson transfer matrix (MC)",
        vec![Capability::F64Compute, Capability::ShaderDispatch],
    )
}

/// Almost-Mathieu eigenvalue computation (Sturm tridiag).
pub fn almost_mathieu_eigenvalues() -> Workload {
    Workload::new(
        "Almost-Mathieu eigenvalues",
        vec![Capability::F64Compute, Capability::ShaderDispatch],
    )
}

/// Green-Kubo integration (f64 precision).
pub fn green_kubo_integration() -> Workload {
    Workload::new(
        "Green-Kubo integration (f64)",
        vec![Capability::F64Compute, Capability::ScalarReduce],
    )
}

/// Anderson regime classification (quantized int8 inference).
pub fn anderson_regime_classify() -> Workload {
    Workload::new(
        "Anderson regime classification",
        vec![Capability::QuantizedInference { bits: 8 }],
    )
}

/// Diversity saturation prediction (quantized int8 inference).
pub fn diversity_saturation_predict() -> Workload {
    Workload::new(
        "Diversity saturation prediction",
        vec![Capability::QuantizedInference { bits: 8 }],
    )
}

/// Bias-variance decomposition (scalar f64 — CPU preferred).
pub fn bias_variance_decompose() -> Workload {
    Workload::new("Bias-variance decomposition", vec![Capability::F64Compute])
}

/// Finite-size extrapolation (scalar f64 — CPU preferred).
pub fn finite_size_extrapolation() -> Workload {
    Workload::new("Finite-size extrapolation", vec![Capability::F64Compute])
}

/// Return all groundSpring workloads for dispatch.
pub fn all() -> Vec<Workload> {
    vec![
        anderson_transfer_matrix(),
        almost_mathieu_eigenvalues(),
        green_kubo_integration(),
        anderson_regime_classify(),
        diversity_saturation_predict(),
        bias_variance_decompose(),
        finite_size_extrapolation(),
    ]
}
