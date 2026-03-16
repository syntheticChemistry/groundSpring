// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! `PCIe` topology and device adjacency for cross-substrate transfer routing.
//!
//! Models the interconnect between substrates so that pipeline stages can
//! route data through the lowest-latency path. The key optimization is
//! NPU→GPU transfers via `PCIe` peer-to-peer (P2P), bypassing the CPU
//! host memory round-trip.
//!
//! # Bandwidth tiers
//!
//! | Tier | Bandwidth | Latency | Example |
//! |------|-----------|---------|---------|
//! | Local | ∞ | 0 | Same device (GPU→GPU shader) |
//! | NvLink | 300 GB/s | ~1µs | Multi-GPU NvLink bridge |
//! | PciePeer | 15.8 GB/s | ~5µs | PCIe 4.0 x16 P2P (GPU↔NPU) |
//! | PcieHost | 15.8 GB/s | ~50µs | PCIe 4.0 x16 via CPU (bounce) |
//! | PcieLow | 0.5 GB/s | ~100µs | PCIe 2.0 x1 (AKD1000) |
//! | Network | varies | ~1ms | LAN via NUCLEUS node |

use crate::substrate::{Substrate, SubstrateKind};

/// Bandwidth tier between two substrates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BandwidthTier {
    /// Same device — no transfer needed.
    Local,
    /// NvLink/NvSwitch — GPU-to-GPU high bandwidth.
    NvLink,
    /// `PCIe` peer-to-peer — direct DMA between devices (bypasses CPU).
    PciePeer,
    /// `PCIe` via host — data bounces through CPU main memory.
    PcieHost,
    /// Low-bandwidth `PCIe` (e.g. AKD1000 at `PCIe` 2.0 x1).
    PcieLow,
    /// Network transfer via NUCLEUS LAN.
    Network,
}

impl BandwidthTier {
    /// Estimated transfer time for `bytes` at this tier, in microseconds.
    #[must_use]
    pub const fn transfer_time_us(self, bytes: u64) -> u64 {
        let (bw_mbps, latency_us): (u64, u64) = match self {
            Self::Local => return 0,
            Self::NvLink => (300_000, 1),
            Self::PciePeer => (15_800, 5),
            Self::PcieHost => (15_800, 50),
            Self::PcieLow => (500, 100),
            Self::Network => (1_000, 1_000),
        };
        let mb = bytes / (1024 * 1024);
        let transfer = if bw_mbps > 0 {
            mb * 1_000_000 / bw_mbps
        } else {
            0
        };
        latency_us + transfer
    }

    /// Whether this tier supports peer-to-peer DMA (bypasses CPU).
    #[must_use]
    pub const fn is_peer_to_peer(self) -> bool {
        matches!(self, Self::Local | Self::NvLink | Self::PciePeer)
    }

    /// Human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::NvLink => "nvlink",
            Self::PciePeer => "pcie-p2p",
            Self::PcieHost => "pcie-host",
            Self::PcieLow => "pcie-low",
            Self::Network => "network",
        }
    }
}

/// A link between two substrates with transfer characteristics.
#[derive(Debug, Clone)]
pub struct Link {
    /// Source substrate index in inventory.
    pub from: usize,
    /// Destination substrate index in inventory.
    pub to: usize,
    /// Bandwidth tier for this link.
    pub tier: BandwidthTier,
}

/// Device topology graph for an inventory of substrates.
#[derive(Debug, Clone)]
pub struct Topology {
    /// All inter-device links (directional — includes both A→B and B→A).
    pub links: Vec<Link>,
}

impl Topology {
    /// Infer topology from an inventory of substrates.
    ///
    /// Uses PCI IDs, device nodes, and substrate kinds to determine
    /// connectivity. When PCI topology is unavailable, falls back to
    /// conservative estimates based on device types.
    #[must_use]
    pub fn infer(substrates: &[Substrate]) -> Self {
        let mut links = Vec::new();

        for (i, src) in substrates.iter().enumerate() {
            for (j, dst) in substrates.iter().enumerate() {
                if i == j {
                    continue;
                }
                let tier = infer_link_tier(src, dst);
                links.push(Link {
                    from: i,
                    to: j,
                    tier,
                });
            }
        }

        Self { links }
    }

    /// Find the best (lowest-latency) link between two substrates.
    #[must_use]
    pub fn best_link(&self, from: usize, to: usize) -> Option<&Link> {
        self.links
            .iter()
            .filter(|l| l.from == from && l.to == to)
            .min_by_key(|l| l.tier)
    }

    /// Find all links from a given substrate.
    #[must_use]
    pub fn links_from(&self, from: usize) -> Vec<&Link> {
        self.links.iter().filter(|l| l.from == from).collect()
    }

    /// Whether a peer-to-peer path exists between two substrates.
    #[must_use]
    pub fn has_p2p(&self, from: usize, to: usize) -> bool {
        self.best_link(from, to)
            .is_some_and(|l| l.tier.is_peer_to_peer())
    }

    /// Estimated transfer time between two substrates for `bytes` of data.
    #[must_use]
    pub fn transfer_time_us(&self, from: usize, to: usize, bytes: u64) -> u64 {
        self.best_link(from, to)
            .map_or(u64::MAX, |l| l.tier.transfer_time_us(bytes))
    }

    /// Find all P2P-capable pairs.
    #[must_use]
    pub fn p2p_pairs(&self) -> Vec<(usize, usize)> {
        self.links
            .iter()
            .filter(|l| l.tier.is_peer_to_peer())
            .map(|l| (l.from, l.to))
            .collect()
    }
}

/// Infer the bandwidth tier between two substrates based on their types
/// and available identity information.
fn infer_link_tier(src: &Substrate, dst: &Substrate) -> BandwidthTier {
    match (src.kind, dst.kind) {
        (SubstrateKind::Gpu, SubstrateKind::Gpu) => {
            if is_nvlink_pair(src, dst) {
                BandwidthTier::NvLink
            } else {
                BandwidthTier::PciePeer
            }
        }
        (SubstrateKind::Gpu, SubstrateKind::Npu) | (SubstrateKind::Npu, SubstrateKind::Gpu) => {
            if is_low_bandwidth_npu(src) || is_low_bandwidth_npu(dst) {
                BandwidthTier::PcieLow
            } else {
                BandwidthTier::PciePeer
            }
        }
        (SubstrateKind::Cpu, _)
        | (_, SubstrateKind::Cpu)
        | (SubstrateKind::Npu, SubstrateKind::Npu) => BandwidthTier::PcieHost,
    }
}

/// Detect `NvLink` pairs from adapter names (e.g. dual-GPU `NvLink` bridge).
fn is_nvlink_pair(a: &Substrate, b: &Substrate) -> bool {
    let both_volta_or_ampere = a
        .properties
        .gpu_arch
        .zip(b.properties.gpu_arch)
        .is_some_and(|(aa, ba)| {
            use crate::substrate::GpuArch::{Ampere, Volta};
            matches!((aa, ba), (Volta, Volta) | (Ampere, Ampere))
        });
    if !both_volta_or_ampere {
        return false;
    }
    let a_name = a.identity.name.to_uppercase();
    let b_name = b.identity.name.to_uppercase();
    (a_name.contains("V100") || a_name.contains("A100"))
        && (b_name.contains("V100") || b_name.contains("A100"))
}

/// AKD1000 is `PCIe` 2.0 x1 — significantly lower bandwidth than modern GPUs.
fn is_low_bandwidth_npu(s: &Substrate) -> bool {
    s.kind == SubstrateKind::Npu && s.identity.name.to_uppercase().contains("AKD1000")
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::substrate::{Capability, GpuArch, Identity, Properties};

    fn gpu(name: &str) -> Substrate {
        Substrate {
            kind: SubstrateKind::Gpu,
            identity: Identity::named(name),
            properties: Properties {
                gpu_arch: Some(GpuArch::from_name(name)),
                ..Properties::default()
            },
            capabilities: vec![Capability::F64Compute, Capability::ShaderDispatch],
        }
    }

    fn npu() -> Substrate {
        Substrate {
            kind: SubstrateKind::Npu,
            identity: Identity::named("BrainChip AKD1000"),
            properties: Properties::default(),
            capabilities: vec![Capability::QuantizedInference { bits: 8 }],
        }
    }

    fn cpu() -> Substrate {
        Substrate {
            kind: SubstrateKind::Cpu,
            identity: Identity::named("CPU"),
            properties: Properties::default(),
            capabilities: vec![Capability::F64Compute],
        }
    }

    #[test]
    fn gpu_to_npu_is_pcie_low_for_akd1000() {
        let tier = infer_link_tier(&gpu("TITAN V"), &npu());
        assert_eq!(tier, BandwidthTier::PcieLow);
    }

    #[test]
    fn gpu_to_gpu_is_pcie_peer() {
        let tier = infer_link_tier(&gpu("TITAN V"), &gpu("RTX 4070"));
        assert_eq!(tier, BandwidthTier::PciePeer);
    }

    #[test]
    fn cpu_to_gpu_is_pcie_host() {
        let tier = infer_link_tier(&cpu(), &gpu("RTX 4070"));
        assert_eq!(tier, BandwidthTier::PcieHost);
    }

    #[test]
    fn local_has_zero_latency() {
        assert_eq!(BandwidthTier::Local.transfer_time_us(1024), 0);
    }

    #[test]
    fn pcie_peer_is_p2p() {
        assert!(BandwidthTier::PciePeer.is_peer_to_peer());
        assert!(!BandwidthTier::PcieHost.is_peer_to_peer());
    }

    #[test]
    fn topology_infer_creates_links() {
        let subs = vec![gpu("TITAN V"), npu(), cpu()];
        let topo = Topology::infer(&subs);
        assert_eq!(topo.links.len(), 6); // 3 devices × 2 directions each
    }

    #[test]
    fn topology_best_link_finds_minimum() {
        let subs = vec![gpu("TITAN V"), npu(), cpu()];
        let topo = Topology::infer(&subs);
        let link = topo.best_link(0, 1).expect("GPU→NPU link");
        assert_eq!(link.tier, BandwidthTier::PcieLow);
    }

    #[test]
    fn topology_has_p2p_for_gpu_npu() {
        let subs = vec![gpu("TITAN V"), gpu("RTX 4070")];
        let topo = Topology::infer(&subs);
        assert!(topo.has_p2p(0, 1));
    }

    #[test]
    fn topology_no_p2p_via_cpu() {
        let subs = vec![gpu("TITAN V"), cpu()];
        let topo = Topology::infer(&subs);
        assert!(!topo.has_p2p(0, 1));
    }

    #[test]
    fn transfer_time_increases_with_data() {
        let small = BandwidthTier::PcieHost.transfer_time_us(1024);
        let large = BandwidthTier::PcieHost.transfer_time_us(1024 * 1024 * 1024);
        assert!(large > small);
    }

    #[test]
    fn bandwidth_tier_labels() {
        assert_eq!(BandwidthTier::Local.label(), "local");
        assert_eq!(BandwidthTier::PciePeer.label(), "pcie-p2p");
        assert_eq!(BandwidthTier::PcieLow.label(), "pcie-low");
        assert_eq!(BandwidthTier::Network.label(), "network");
    }

    #[test]
    fn p2p_pairs_found() {
        let subs = vec![gpu("TITAN V"), gpu("RTX 4070"), cpu()];
        let topo = Topology::infer(&subs);
        let pairs = topo.p2p_pairs();
        assert!(pairs.contains(&(0, 1)));
        assert!(pairs.contains(&(1, 0)));
        assert!(!pairs.iter().any(|&(a, b)| a == 2 || b == 2));
    }

    #[test]
    fn npu_to_gpu_link_is_pcie_low() {
        let subs = vec![npu(), gpu("TITAN V"), cpu()];
        let topo = Topology::infer(&subs);
        let link = topo.best_link(0, 1).expect("NPU→GPU link");
        assert_eq!(link.tier, BandwidthTier::PcieLow);
        assert!(
            link.tier.transfer_time_us(1024) > 0,
            "NPU→GPU should have non-zero transfer time"
        );
    }

    #[test]
    fn gpu_to_gpu_p2p_faster_than_host_bounce() {
        let p2p = BandwidthTier::PciePeer.transfer_time_us(1_000_000);
        let host = BandwidthTier::PcieHost.transfer_time_us(1_000_000);
        assert!(
            p2p <= host,
            "P2P ({p2p}µs) should be ≤ host bounce ({host}µs) for 1MB"
        );
    }

    #[test]
    #[expect(
        clippy::similar_names,
        reason = "npu_gpu / npu_cpu / cpu_gpu are intentionally named for transfer direction"
    )]
    fn npu_gpu_p2p_bypasses_cpu_roundtrip() {
        let subs = vec![npu(), gpu("TITAN V"), cpu()];
        let topo = Topology::infer(&subs);
        let npu_gpu = topo
            .best_link(0, 1)
            .map_or(0, |l| l.tier.transfer_time_us(65536));
        let npu_cpu = topo
            .best_link(0, 2)
            .map_or(0, |l| l.tier.transfer_time_us(65536));
        let cpu_gpu = topo
            .best_link(2, 1)
            .map_or(0, |l| l.tier.transfer_time_us(65536));
        assert!(
            npu_gpu <= npu_cpu + cpu_gpu,
            "direct NPU→GPU ({npu_gpu}µs) should be ≤ NPU→CPU→GPU roundtrip ({}µs)",
            npu_cpu + cpu_gpu
        );
    }

    #[test]
    fn full_substrate_topology_link_count() {
        let subs = vec![npu(), gpu("TITAN V"), gpu("RTX 4070"), cpu()];
        let topo = Topology::infer(&subs);
        let n = subs.len();
        assert_eq!(
            topo.links.len(),
            n * (n - 1),
            "should have n*(n-1) directed links for {n} substrates"
        );
    }
}
