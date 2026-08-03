// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

use super::types::{AtomicCapability, NestAtomic, PrimalHealth, TowerAtomic};

impl NestAtomic {
    /// Create a Nest atomic.
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            tower: TowerAtomic::new(node_id),
            storage: PrimalHealth::Unavailable,
            data_capabilities: Vec::new(),
        }
    }

    /// Check if data storage is available.
    #[must_use]
    pub const fn can_store(&self) -> bool {
        matches!(self.storage, PrimalHealth::Healthy)
    }

    /// List capabilities provided by this atomic.
    #[must_use]
    pub fn capabilities(&self) -> Vec<AtomicCapability> {
        let mut caps = self.tower.capabilities();
        if self.can_store() {
            caps.push(AtomicCapability::DataStorage);
            for dc in &self.data_capabilities {
                if !caps.contains(dc) {
                    caps.push(*dc);
                }
            }
        }
        caps
    }
}
