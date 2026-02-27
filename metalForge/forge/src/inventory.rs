// SPDX-License-Identifier: AGPL-3.0-or-later

//! Hardware inventory — collect all substrates on this machine.

use crate::probe;
use crate::substrate::Substrate;

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

    /// Print a summary table of all discovered substrates.
    pub fn print_summary(&self) {
        println!("  {:<4} {:<30} {:<30}", "Kind", "Name", "Capabilities");
        println!("  {:<4} {:<30} {:<30}", "----", "----", "------------");
        for s in &self.substrates {
            println!(
                "  {:<4} {:<30} {:<30}",
                s.kind.to_string(),
                &s.identity.name,
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
