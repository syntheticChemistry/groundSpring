// SPDX-License-Identifier: AGPL-3.0-or-later

//! IPC interface for Songbird network discovery and mesh.
//!
//! Songbird provides capability-based primal discovery. groundSpring uses
//! Songbird for:
//! - Discovering which primals provide `compute.*`, `storage.*`, etc.
//! - Resolving primal socket addresses at runtime
//! - Mesh topology queries
//!
//! # Capability surface
//!
//! - `discovery.query` — find primals by capability
//! - `discovery.find_primals` — enumerate all known primals
//! - `mesh.join` — join the discovery mesh

/// Discovery service traits via Songbird.
#[tarpc::service]
pub trait DiscoveryService {
    /// Find primals that provide a specific capability.
    async fn query(capability: String) -> Result<String, String>;

    /// Enumerate all known primals in the mesh.
    async fn find_primals() -> Result<String, String>;

    /// Resolve the socket address for a specific primal.
    async fn resolve(primal_id: String) -> Result<String, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tarpc_trait_compiles() {
        fn _assert_service<T: DiscoveryService>() {}
    }

    #[test]
    fn discovery_role_is_songbird() {
        assert_eq!(crate::primal_names::roles::DISCOVERY, "songbird");
    }
}
