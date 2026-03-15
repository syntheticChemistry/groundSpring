// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Hardware inventory — collect all substrates on this machine.

use crate::probe;
use crate::substrate::{AdaptiveBatch, Capability, GpuArch, Substrate, SubstrateKind};

/// Full hardware inventory discovered at runtime.
#[derive(Debug, Clone)]
pub struct Inventory {
    /// All discovered substrates.
    pub substrates: Vec<Substrate>,
}

impl Inventory {
    /// Discover all available substrates (GPU, NPU, CPU).
    #[must_use]
    pub fn discover() -> Self {
        let mut substrates = Vec::new();
        substrates.extend(probe::probe_gpus());
        substrates.extend(probe::probe_npus());
        substrates.push(probe::probe_cpu());
        Self { substrates }
    }

    /// Count substrates of a given kind.
    #[must_use]
    pub fn count(&self, kind: crate::substrate::SubstrateKind) -> usize {
        self.substrates.iter().filter(|s| s.kind == kind).count()
    }

    /// Find the first substrate of a given kind.
    #[must_use]
    pub fn first(&self, kind: crate::substrate::SubstrateKind) -> Option<&Substrate> {
        self.substrates.iter().find(|s| s.kind == kind)
    }

    /// Find a GPU by architecture family (e.g. `GpuArch::Volta` for Titan V).
    #[must_use]
    pub fn find_gpu_by_arch(&self, arch: GpuArch) -> Option<&Substrate> {
        self.substrates
            .iter()
            .find(|s| s.kind == SubstrateKind::Gpu && s.properties.gpu_arch == Some(arch))
    }

    /// Find the best GPU for f64 workloads.
    ///
    /// Prefers native f64 GPUs (Volta 1:2 ratio) over DF64-only (Ada 1:64).
    #[must_use]
    pub fn best_f64_gpu(&self) -> Option<&Substrate> {
        self.substrates
            .iter()
            .find(|s| s.kind == SubstrateKind::Gpu && s.has(&Capability::NativeF64))
            .or_else(|| {
                self.substrates
                    .iter()
                    .find(|s| s.kind == SubstrateKind::Gpu && s.has(&Capability::F64Compute))
            })
    }

    /// Compute adaptive batch parameters for a GPU workload.
    ///
    /// Selects the best f64 GPU and returns batch sizing that fits
    /// within its memory. For Volta/NAK, uses resident memory mode
    /// (buffers stay on-device between dispatches).
    #[must_use]
    pub fn adaptive_batch(&self, element_bytes: usize) -> Option<AdaptiveBatch> {
        let gpu = self.best_f64_gpu()?;
        Some(AdaptiveBatch::for_gpu(&gpu.properties, element_bytes))
    }

    /// Merge remote substrates from NUCLEUS nodes into this inventory.
    ///
    /// Uses [`crate::remote::merge_remote`] to prefix remote device names
    /// with their node ID (e.g. `TITAN V@biomegate`).
    pub fn merge_remote(&mut self, remote: &[crate::remote::RemoteSubstrate]) {
        let local = std::mem::take(&mut self.substrates);
        self.substrates = crate::remote::merge_remote(local, remote);
    }

    /// Print a summary table of all discovered substrates.
    pub fn print_summary(&self) {
        println!(
            "  {:<4} {:<30} {:<8} {:<30}",
            "Kind", "Name", "Arch", "Capabilities"
        );
        println!(
            "  {:<4} {:<30} {:<8} {:<30}",
            "----", "----", "----", "------------"
        );
        for s in &self.substrates {
            let arch_str = s
                .properties
                .gpu_arch
                .map_or_else(|| "-".to_string(), |a| format!("{a:?}"));
            println!(
                "  {:<4} {:<30} {:<8} {:<30}",
                s.kind.to_string(),
                &s.identity.name,
                arch_str,
                s.capability_summary()
            );
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::substrate::{Capability, Identity, Properties, Substrate, SubstrateKind};

    #[test]
    fn discover_finds_cpu() {
        let inv = Inventory::discover();
        assert!(inv.count(SubstrateKind::Cpu) >= 1);
        let cpu = inv.first(SubstrateKind::Cpu).unwrap();
        assert!(!cpu.identity.name.is_empty());
    }

    #[test]
    fn count_returns_zero_for_absent_kind() {
        let inv = Inventory {
            substrates: vec![Substrate {
                kind: SubstrateKind::Cpu,
                identity: Identity::named("Test CPU"),
                properties: Properties::default(),
                capabilities: vec![Capability::F64Compute],
            }],
        };
        assert_eq!(inv.count(SubstrateKind::Gpu), 0);
        assert_eq!(inv.count(SubstrateKind::Npu), 0);
        assert_eq!(inv.count(SubstrateKind::Cpu), 1);
    }

    #[test]
    fn first_returns_none_for_absent_kind() {
        let inv = Inventory {
            substrates: vec![Substrate {
                kind: SubstrateKind::Cpu,
                identity: Identity::named("Test CPU"),
                properties: Properties::default(),
                capabilities: vec![Capability::F64Compute],
            }],
        };
        assert!(inv.first(SubstrateKind::Gpu).is_none());
        assert!(inv.first(SubstrateKind::Cpu).is_some());
    }

    #[test]
    fn print_summary_does_not_panic() {
        let inv = Inventory {
            substrates: vec![
                Substrate {
                    kind: SubstrateKind::Cpu,
                    identity: Identity::named("Test CPU"),
                    properties: Properties::default(),
                    capabilities: vec![Capability::F64Compute, Capability::F32Compute],
                },
                Substrate {
                    kind: SubstrateKind::Gpu,
                    identity: Identity::named("Test GPU"),
                    properties: Properties::default(),
                    capabilities: vec![Capability::ShaderDispatch],
                },
            ],
        };
        inv.print_summary();
    }

    #[test]
    fn empty_inventory() {
        let inv = Inventory { substrates: vec![] };
        assert_eq!(inv.count(SubstrateKind::Cpu), 0);
        assert!(inv.first(SubstrateKind::Cpu).is_none());
        inv.print_summary();
    }

    fn gpu_substrate(name: &str, caps: Vec<Capability>) -> Substrate {
        let arch = GpuArch::from_name(name);
        Substrate {
            kind: SubstrateKind::Gpu,
            identity: Identity::named(name),
            properties: Properties {
                gpu_arch: Some(arch),
                ..Properties::default()
            },
            capabilities: caps,
        }
    }

    #[test]
    fn find_gpu_by_arch_volta() {
        let inv = Inventory {
            substrates: vec![
                gpu_substrate("NVIDIA TITAN V", vec![Capability::NativeF64]),
                gpu_substrate("NVIDIA RTX 4070", vec![Capability::F64Compute]),
            ],
        };
        let volta = inv.find_gpu_by_arch(GpuArch::Volta);
        assert!(volta.is_some());
        assert!(volta.unwrap().identity.name.contains("TITAN V"));
        assert!(inv.find_gpu_by_arch(GpuArch::Turing).is_none());
    }

    #[test]
    fn best_f64_gpu_prefers_native() {
        let inv = Inventory {
            substrates: vec![
                gpu_substrate("NVIDIA RTX 4070", vec![Capability::F64Compute]),
                gpu_substrate(
                    "NVIDIA TITAN V",
                    vec![Capability::NativeF64, Capability::F64Compute],
                ),
            ],
        };
        let best = inv.best_f64_gpu().unwrap();
        assert!(best.identity.name.contains("TITAN V"));
    }

    #[test]
    fn best_f64_gpu_falls_back_to_non_native() {
        let inv = Inventory {
            substrates: vec![gpu_substrate(
                "NVIDIA RTX 4070",
                vec![Capability::F64Compute],
            )],
        };
        let best = inv.best_f64_gpu().unwrap();
        assert!(best.identity.name.contains("RTX 4070"));
    }

    #[test]
    fn best_f64_gpu_none_when_no_gpu() {
        let inv = Inventory {
            substrates: vec![Substrate {
                kind: SubstrateKind::Cpu,
                identity: Identity::named("Test CPU"),
                properties: Properties::default(),
                capabilities: vec![Capability::F64Compute],
            }],
        };
        assert!(inv.best_f64_gpu().is_none());
    }

    #[test]
    fn adaptive_batch_returns_some_for_gpu() {
        let inv = Inventory {
            substrates: vec![gpu_substrate(
                "NVIDIA TITAN V",
                vec![Capability::NativeF64, Capability::F64Compute],
            )],
        };
        let batch = inv.adaptive_batch(8);
        assert!(batch.is_some());
        let b = batch.unwrap();
        assert!(b.max_batch_elements > 0);
    }

    #[test]
    fn adaptive_batch_returns_none_without_gpu() {
        let inv = Inventory {
            substrates: vec![Substrate {
                kind: SubstrateKind::Cpu,
                identity: Identity::named("Test CPU"),
                properties: Properties::default(),
                capabilities: vec![Capability::F64Compute],
            }],
        };
        assert!(inv.adaptive_batch(8).is_none());
    }

    #[test]
    fn merge_remote_adds_substrates() {
        let mut inv = Inventory {
            substrates: vec![Substrate {
                kind: SubstrateKind::Cpu,
                identity: Identity::named("local CPU"),
                properties: Properties::default(),
                capabilities: vec![Capability::F64Compute],
            }],
        };

        let remote = vec![crate::remote::RemoteSubstrate {
            substrate: Substrate {
                kind: SubstrateKind::Gpu,
                identity: Identity::named("NVIDIA TITAN V"),
                properties: Properties::default(),
                capabilities: vec![Capability::NativeF64],
            },
            origin: crate::remote::RemoteOrigin {
                node_id: "biomegate".to_string(),
                is_lan: true,
                estimated_latency_ms: 1,
            },
        }];

        inv.merge_remote(&remote);
        assert!(inv.count(SubstrateKind::Gpu) >= 1);
        assert!(inv.count(SubstrateKind::Cpu) >= 1);
    }

    #[test]
    fn multiple_gpus_counted() {
        let inv = Inventory {
            substrates: vec![
                gpu_substrate("NVIDIA TITAN V", vec![Capability::NativeF64]),
                gpu_substrate("NVIDIA RTX 4070", vec![Capability::F64Compute]),
                gpu_substrate("NVIDIA RTX 3090", vec![Capability::F64Compute]),
            ],
        };
        assert_eq!(inv.count(SubstrateKind::Gpu), 3);
        assert!(inv.first(SubstrateKind::Gpu).is_some());
    }
}
