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
    pub fn discover() -> Self {
        let mut substrates = Vec::new();
        substrates.extend(probe::probe_gpus());
        substrates.extend(probe::probe_npus());
        substrates.push(probe::probe_cpu());
        Self { substrates }
    }

    /// Count substrates of a given kind.
    pub fn count(&self, kind: crate::substrate::SubstrateKind) -> usize {
        self.substrates.iter().filter(|s| s.kind == kind).count()
    }

    /// Find the first substrate of a given kind.
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
                format!("{}", s.kind),
                &s.identity.name,
                s.capability_summary()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::substrate::SubstrateKind;

    #[test]
    fn discover_finds_cpu() {
        let inv = Inventory::discover();
        assert!(inv.count(SubstrateKind::Cpu) >= 1);
        let cpu = inv.first(SubstrateKind::Cpu).unwrap();
        assert!(!cpu.identity.name.is_empty());
    }
}
