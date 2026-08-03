// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

use super::types::{ResolvedPipeline, StageResolution, TransferStrategy};

impl ResolvedPipeline<'_> {
    /// Check if all stages have assigned substrates (no skips, no failures).
    #[must_use]
    pub fn all_assigned(&self) -> bool {
        self.stages.iter().all(|s| s.substrate.is_some())
    }

    /// Count how many stages use P2P transfers.
    #[must_use]
    pub fn p2p_transfer_count(&self) -> usize {
        self.stages
            .iter()
            .filter(|s| s.transfer == TransferStrategy::PeerToPeer)
            .count()
    }

    /// Count how many stages had to degrade to a fallback substrate.
    #[must_use]
    pub fn degraded_count(&self) -> usize {
        self.stages
            .iter()
            .filter(|s| s.reason == StageResolution::Degraded)
            .count()
    }

    /// Print a human-readable pipeline summary.
    pub fn print_summary(&self) {
        println!("Pipeline: {}", self.name);
        println!(
            "  Stages: {} | Transfer overhead: {}µs | Optimal: {}",
            self.stages.len(),
            self.total_transfer_us,
            self.fully_optimal,
        );
        for (i, rs) in self.stages.iter().enumerate() {
            let sub_name = rs
                .substrate
                .map_or("(skipped)", |s| s.identity.name.as_str());
            let transfer_str = match rs.transfer {
                TransferStrategy::PeerToPeer => "←P2P←",
                TransferStrategy::HostBounce => "←HOST←",
                TransferStrategy::None => "",
            };
            println!(
                "  [{i}] {:<30} → {:<20} {transfer_str} ({:?})",
                rs.stage.name, sub_name, rs.reason,
            );
        }
    }
}
