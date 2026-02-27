// SPDX-License-Identifier: AGPL-3.0-or-later

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
        self.substrates.iter().find(|s| {
            s.kind == SubstrateKind::Gpu && s.properties.gpu_arch == Some(arch)
        })
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
        let inv = Inventory {
            substrates: vec![],
        };
        assert_eq!(inv.count(SubstrateKind::Cpu), 0);
        assert!(inv.first(SubstrateKind::Cpu).is_none());
        inv.print_summary();
    }
}
