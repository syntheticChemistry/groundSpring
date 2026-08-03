// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Primal dependencies and feature gates for niche deployment.

/// Primal dependencies for niche deployment.
///
/// Each entry: `(primal_id, required, description)`.
/// `required = true` means the niche cannot function without it.
/// `required = false` means graceful degradation is supported.
pub const DEPENDENCIES: &[(&str, bool, &str)] = &[
    (
        crate::primal_names::roles::SECURITY,
        true,
        "cryptographic identity and trust",
    ),
    (
        crate::primal_names::roles::DISCOVERY,
        true,
        "service discovery and IPC mesh",
    ),
    (
        crate::primal_names::roles::COMPUTE,
        false,
        "GPU compute dispatch (sovereign fallback to CPU)",
    ),
    (
        crate::primal_names::roles::STORAGE,
        false,
        "data storage and NCBI/NOAA/IRIS providers (sovereign fallback to synthetic)",
    ),
    (
        crate::primal_names::roles::AUDIT,
        false,
        "audit event logging via audit provider (JH-5, fallback: skip)",
    ),
];

/// Feature gates that expand niche capabilities.
pub const FEATURE_GATES: &[(&str, &str)] = &[
    (
        "barracuda",
        "CPU delegation to CPU math provider primitives",
    ),
    (
        "barracuda-gpu",
        "GPU dispatch via GPU math + compute providers",
    ),
    (
        crate::primal_names::roles::ORCHESTRATOR,
        "biomeOS Neural API integration",
    ),
    ("npu", "BrainChip AKD1000 NPU inference"),
];
