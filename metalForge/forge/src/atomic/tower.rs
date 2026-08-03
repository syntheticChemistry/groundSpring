// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

use super::types::{AtomicCapability, PrimalHealth, ProviderHealthMap, TowerAtomic};

impl TowerAtomic {
    /// Create a Tower atomic for a given node.
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            providers: ProviderHealthMap::new(),
            socket_path: None,
        }
    }

    /// Set health for a capability provider discovered at runtime.
    pub fn set_provider_health(&mut self, capability: &str, health: PrimalHealth) {
        self.providers.insert(capability.to_string(), health);
    }

    /// Check if the Tower has healthy secure IPC (all required providers up).
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        !self.providers.is_empty()
            && self
                .providers
                .values()
                .all(|h| matches!(h, PrimalHealth::Healthy))
    }

    /// List capabilities provided by this atomic.
    #[must_use]
    pub fn capabilities(&self) -> Vec<AtomicCapability> {
        if self.is_healthy() {
            vec![AtomicCapability::SecureIpc]
        } else {
            Vec::new()
        }
    }
}
