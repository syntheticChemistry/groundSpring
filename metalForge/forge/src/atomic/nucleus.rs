// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

use super::types::{AtomicCapability, FullNucleus, PrimalHealth};

impl FullNucleus {
    /// Check if all capabilities are healthy.
    #[must_use]
    pub fn is_fully_healthy(&self) -> bool {
        self.node.tower.is_healthy()
            && self.node.can_compute()
            && matches!(self.storage, PrimalHealth::Healthy)
            && matches!(self.inference, PrimalHealth::Healthy)
    }

    /// List all capabilities of the full NUCLEUS.
    #[must_use]
    pub fn capabilities(&self) -> Vec<AtomicCapability> {
        let mut caps = self.node.capabilities();
        if matches!(self.storage, PrimalHealth::Healthy) {
            caps.push(AtomicCapability::DataStorage);
            caps.push(AtomicCapability::LiveData);
        }
        if matches!(self.inference, PrimalHealth::Healthy) {
            caps.push(AtomicCapability::AiInference);
        }
        caps
    }

    /// The sovereign degradation level — what's available when parts fail.
    #[must_use]
    pub fn degradation_level(&self) -> &'static str {
        if self.is_fully_healthy() {
            "Full NUCLEUS"
        } else if self.node.can_compute() && matches!(self.storage, PrimalHealth::Healthy) {
            "Node + Nest (no AI)"
        } else if self.node.can_compute() {
            "Node only (no storage)"
        } else if self.node.tower.is_healthy() {
            "Tower only (no compute)"
        } else {
            "Sovereign (local only)"
        }
    }
}
