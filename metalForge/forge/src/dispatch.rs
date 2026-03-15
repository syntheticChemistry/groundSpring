// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Dispatch routing — route groundSpring workloads to capable substrates.
//!
//! Selection priority:
//! 1. Preferred substrate (if specified and capable)
//! 2. GPU (for compute-heavy work)
//! 3. NPU (for inference)
//! 4. CPU (fallback, always available)

use crate::substrate::{Capability, Substrate, SubstrateKind};

/// A workload that needs to be dispatched to a substrate.
#[derive(Debug)]
pub struct Workload {
    /// Human-readable workload name.
    pub name: String,
    /// Capabilities required for this workload.
    pub required: Vec<Capability>,
    /// Preferred substrate kind (if any).
    pub preferred_substrate: Option<SubstrateKind>,
}

/// Dispatch decision — which substrate was chosen and why.
#[derive(Debug)]
pub struct Decision<'a> {
    /// The chosen substrate.
    pub substrate: &'a Substrate,
    /// Why this substrate was chosen.
    pub reason: Reason,
}

/// Why a particular substrate was chosen.
#[derive(Debug, PartialEq, Eq)]
pub enum Reason {
    /// The workload's preferred substrate had all capabilities.
    Preferred,
    /// Best capable substrate by priority (GPU > NPU > CPU).
    BestAvailable,
}

impl Workload {
    /// Create a workload with name and required capabilities.
    pub fn new(name: impl Into<String>, required: Vec<Capability>) -> Self {
        Self {
            name: name.into(),
            required,
            preferred_substrate: None,
        }
    }

    /// Set the preferred substrate kind.
    #[must_use]
    pub const fn prefer(mut self, kind: SubstrateKind) -> Self {
        self.preferred_substrate = Some(kind);
        self
    }
}

/// Route a workload to the best matching substrate.
#[must_use]
pub fn route<'a>(workload: &Workload, substrates: &'a [Substrate]) -> Option<Decision<'a>> {
    let capable: Vec<&Substrate> = substrates
        .iter()
        .filter(|s| workload.required.iter().all(|req| s.has(req)))
        .collect();

    if capable.is_empty() {
        return None;
    }

    if let Some(pref) = workload.preferred_substrate
        && let Some(s) = capable.iter().find(|s| s.kind == pref)
    {
        return Some(Decision {
            substrate: s,
            reason: Reason::Preferred,
        });
    }

    let needs_f64 = workload.required.contains(&Capability::F64Compute);

    let best = if needs_f64 {
        capable
            .iter()
            .find(|s| s.kind == SubstrateKind::Gpu && s.has(&Capability::NativeF64))
            .or_else(|| capable.iter().find(|s| s.kind == SubstrateKind::Gpu))
            .or_else(|| capable.iter().find(|s| s.kind == SubstrateKind::Npu))
            .or_else(|| capable.iter().find(|s| s.kind == SubstrateKind::Cpu))?
    } else {
        capable
            .iter()
            .find(|s| s.kind == SubstrateKind::Gpu)
            .or_else(|| capable.iter().find(|s| s.kind == SubstrateKind::Npu))
            .or_else(|| capable.iter().find(|s| s.kind == SubstrateKind::Cpu))?
    };

    Some(Decision {
        substrate: best,
        reason: Reason::BestAvailable,
    })
}

/// Ordered fallback chain — try substrates in priority order until one succeeds.
///
/// Unlike [`route`] which picks the single best substrate, this returns an
/// ordered list of all capable substrates for graceful degradation at runtime.
/// The caller executes on the first substrate; if it fails, it tries the next.
///
/// Priority: preferred → GPU (`NativeF64` first) → NPU → CPU.
#[must_use]
pub fn fallback_chain<'a>(workload: &Workload, substrates: &'a [Substrate]) -> Vec<Decision<'a>> {
    let capable: Vec<&Substrate> = substrates
        .iter()
        .filter(|s| workload.required.iter().all(|req| s.has(req)))
        .collect();

    if capable.is_empty() {
        return Vec::new();
    }

    let mut chain = Vec::new();

    if let Some(pref) = workload.preferred_substrate
        && let Some(&s) = capable.iter().find(|s| s.kind == pref)
    {
        chain.push(Decision {
            substrate: s,
            reason: Reason::Preferred,
        });
    }

    let needs_f64 = workload.required.contains(&Capability::F64Compute);

    if needs_f64 {
        for &s in &capable {
            if s.kind == SubstrateKind::Gpu
                && s.has(&Capability::NativeF64)
                && !chain.iter().any(|d| std::ptr::eq(d.substrate, s))
            {
                chain.push(Decision {
                    substrate: s,
                    reason: Reason::BestAvailable,
                });
            }
        }
    }

    for kind in [SubstrateKind::Gpu, SubstrateKind::Npu, SubstrateKind::Cpu] {
        for &s in &capable {
            if s.kind == kind && !chain.iter().any(|d| std::ptr::eq(d.substrate, s)) {
                chain.push(Decision {
                    substrate: s,
                    reason: Reason::BestAvailable,
                });
            }
        }
    }

    chain
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::substrate::{Identity, Properties};

    fn make_gpu(name: &str, caps: Vec<Capability>) -> Substrate {
        Substrate {
            kind: SubstrateKind::Gpu,
            identity: Identity::named(name),
            properties: Properties::default(),
            capabilities: caps,
        }
    }

    fn make_cpu() -> Substrate {
        Substrate {
            kind: SubstrateKind::Cpu,
            identity: Identity::named("CPU"),
            properties: Properties::default(),
            capabilities: vec![Capability::F64Compute, Capability::F32Compute],
        }
    }

    fn make_npu() -> Substrate {
        Substrate {
            kind: SubstrateKind::Npu,
            identity: Identity::named("AKD1000"),
            properties: Properties::default(),
            capabilities: vec![
                Capability::F32Compute,
                Capability::QuantizedInference { bits: 8 },
                Capability::BatchInference { max_batch: 8 },
            ],
        }
    }

    #[test]
    fn routes_anderson_to_gpu() {
        let gpu = make_gpu(
            "RTX 4070",
            vec![
                Capability::F64Compute,
                Capability::ScalarReduce,
                Capability::ShaderDispatch,
            ],
        );
        let cpu = make_cpu();
        let subs = [gpu, cpu];
        let work = Workload::new(
            "Anderson transfer matrix",
            vec![Capability::F64Compute, Capability::ShaderDispatch],
        );
        let d = route(&work, &subs).expect("should route");
        assert_eq!(d.substrate.kind, SubstrateKind::Gpu);
        assert_eq!(d.reason, Reason::BestAvailable);
    }

    #[test]
    fn routes_classification_to_npu() {
        let npu = make_npu();
        let cpu = make_cpu();
        let subs = [cpu, npu];
        let work = Workload::new(
            "Anderson regime classify",
            vec![Capability::QuantizedInference { bits: 8 }],
        );
        let d = route(&work, &subs).expect("should route");
        assert_eq!(d.substrate.kind, SubstrateKind::Npu);
    }

    #[test]
    fn falls_back_to_cpu() {
        let subs = [make_cpu()];
        let work = Workload::new("Decompose error", vec![Capability::F64Compute]);
        let d = route(&work, &subs).expect("should route to CPU");
        assert_eq!(d.substrate.kind, SubstrateKind::Cpu);
    }

    #[test]
    fn no_route_if_incapable() {
        let subs = [make_cpu()];
        let work = Workload::new(
            "NPU inference",
            vec![Capability::QuantizedInference { bits: 4 }],
        );
        assert!(route(&work, &subs).is_none());
    }

    #[test]
    fn respects_cpu_preference() {
        let gpu = make_gpu("GPU", vec![Capability::F64Compute]);
        let cpu = make_cpu();
        let subs = [gpu, cpu];
        let work =
            Workload::new("validation", vec![Capability::F64Compute]).prefer(SubstrateKind::Cpu);
        let d = route(&work, &subs).expect("should route");
        assert_eq!(d.substrate.kind, SubstrateKind::Cpu);
        assert_eq!(d.reason, Reason::Preferred);
    }

    #[test]
    fn prefers_native_f64_gpu_for_f64_workloads() {
        let ada_gpu = Substrate {
            kind: SubstrateKind::Gpu,
            identity: Identity::named("RTX 4070"),
            properties: Properties::default(),
            capabilities: vec![
                Capability::F64Compute,
                Capability::ShaderDispatch,
                Capability::ScalarReduce,
            ],
        };
        let volta_gpu = Substrate {
            kind: SubstrateKind::Gpu,
            identity: Identity::named("TITAN V"),
            properties: Properties::default(),
            capabilities: vec![
                Capability::F64Compute,
                Capability::ShaderDispatch,
                Capability::ScalarReduce,
                Capability::NativeF64,
            ],
        };
        let cpu = make_cpu();
        let subs = [ada_gpu, volta_gpu, cpu];

        let work = Workload::new(
            "Anderson transfer matrix",
            vec![Capability::F64Compute, Capability::ShaderDispatch],
        );
        let d = route(&work, &subs).expect("should route");
        assert!(d.substrate.identity.name.contains("TITAN V"));
        assert_eq!(d.reason, Reason::BestAvailable);
    }

    #[test]
    fn fallback_chain_orders_by_priority() {
        let gpu = make_gpu(
            "RTX 4070",
            vec![Capability::F64Compute, Capability::ShaderDispatch],
        );
        let cpu = make_cpu();
        let npu = make_npu();
        let subs = [gpu, npu, cpu];
        let work = Workload::new("compute", vec![Capability::F64Compute]);
        let chain = fallback_chain(&work, &subs);
        assert!(chain.len() >= 2);
        assert_eq!(chain[0].substrate.kind, SubstrateKind::Gpu);
        assert_eq!(chain.last().unwrap().substrate.kind, SubstrateKind::Cpu);
    }

    #[test]
    fn fallback_chain_empty_when_incapable() {
        let subs = [make_cpu()];
        let work = Workload::new("npu", vec![Capability::QuantizedInference { bits: 4 }]);
        let chain = fallback_chain(&work, &subs);
        assert!(chain.is_empty());
    }

    #[test]
    fn fallback_chain_preferred_first() {
        let gpu = make_gpu("GPU", vec![Capability::F64Compute]);
        let cpu = make_cpu();
        let subs = [gpu, cpu];
        let work = Workload::new("pref", vec![Capability::F64Compute]).prefer(SubstrateKind::Cpu);
        let chain = fallback_chain(&work, &subs);
        assert_eq!(chain[0].substrate.kind, SubstrateKind::Cpu);
        assert_eq!(chain[0].reason, Reason::Preferred);
        assert!(chain.len() >= 2);
    }

    #[test]
    fn fallback_chain_native_f64_before_regular_gpu() {
        let ada = Substrate {
            kind: SubstrateKind::Gpu,
            identity: Identity::named("RTX 4070"),
            properties: Properties::default(),
            capabilities: vec![Capability::F64Compute, Capability::ShaderDispatch],
        };
        let volta = Substrate {
            kind: SubstrateKind::Gpu,
            identity: Identity::named("TITAN V"),
            properties: Properties::default(),
            capabilities: vec![
                Capability::F64Compute,
                Capability::ShaderDispatch,
                Capability::NativeF64,
            ],
        };
        let cpu = make_cpu();
        let subs = [ada, volta, cpu];
        let work = Workload::new(
            "f64 compute",
            vec![Capability::F64Compute, Capability::ShaderDispatch],
        );
        let chain = fallback_chain(&work, &subs);
        assert!(chain[0].substrate.identity.name.contains("TITAN V"));
    }

    #[test]
    fn f32_workloads_still_pick_first_gpu() {
        let ada_gpu = Substrate {
            kind: SubstrateKind::Gpu,
            identity: Identity::named("RTX 4070"),
            properties: Properties::default(),
            capabilities: vec![Capability::F32Compute, Capability::ShaderDispatch],
        };
        let volta_gpu = Substrate {
            kind: SubstrateKind::Gpu,
            identity: Identity::named("TITAN V"),
            properties: Properties::default(),
            capabilities: vec![
                Capability::F32Compute,
                Capability::ShaderDispatch,
                Capability::NativeF64,
            ],
        };
        let subs = [ada_gpu, volta_gpu];

        let work = Workload::new(
            "f32 shader",
            vec![Capability::F32Compute, Capability::ShaderDispatch],
        );
        let d = route(&work, &subs).expect("should route");
        assert!(d.substrate.identity.name.contains("RTX 4070"));
    }
}
