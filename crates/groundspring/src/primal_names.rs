// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Primal identity and runtime discovery helpers.
//!
//! **Self-knowledge**: groundSpring knows its own canonical identifier and
//! the socket directory convention. Everything else is discovered at runtime.
//!
//! **Well-known role names**: The ecosystem defines standard role names
//! (orchestrator, discovery, storage, compute, compiler) so primals can
//! construct env-var keys for socket lookup. These are *convention identifiers*,
//! not assumptions about what is running. A primal uses them to *ask* the
//! environment "where is the compute orchestrator?" — not to assert one exists.

// ─── Self Identity ─────

/// This niche's canonical identifier.
pub const SELF_ID: &str = "groundspring";

/// Socket directory name for biomeOS IPC mesh.
pub const BIOMEOS_SOCKET_DIR: &str = "biomeos";

/// Legacy flat-file socket name (pre-directory convention).
///
/// Earlier NUCLEUS versions placed this directly in `$TMPDIR`
/// rather than the `biomeos/` subdirectory. Retained for backward
/// compatibility during migration.
pub const LEGACY_NEURAL_API_SOCK: &str = "biomeos-neural-api.sock";

/// Well-known Neural API socket names in the biomeOS socket directory.
///
/// biomeOS sockets are named by capability, not by primal. Discovery
/// prefers the canonical `neural-api.sock` and falls back to the
/// `-default` variant for earlier NUCLEUS versions.
pub const NEURAL_API_SOCKET_NAMES: &[&str] = &["neural-api.sock", "neural-api-default.sock"];

// ─── Well-Known Ecosystem Roles ─────
//
// These are convention identifiers for env-var / socket discovery.
// They encode the *role name* in the ecosystem, not a compile-time
// dependency on another primal. At runtime, groundSpring queries the
// environment or Songbird registry — if the role is unfilled, the
// feature degrades gracefully.

/// Well-known ecosystem role identifiers for runtime discovery.
///
/// Used exclusively with [`socket_env_var`] and [`address_env_var`]
/// to probe whether a given role is available in the current deployment.
pub mod roles {
    /// biomeOS orchestrator role.
    pub const ORCHESTRATOR: &str = "biomeos";

    /// Songbird discovery mesh role.
    pub const DISCOVERY: &str = "songbird";

    /// `NestGate` content-addressed storage role.
    pub const STORAGE: &str = "nestgate";

    /// `BearDog` security foundation role.
    pub const SECURITY: &str = "beardog";

    /// `ToadStool` compute orchestrator role.
    pub const COMPUTE: &str = "toadstool";

    /// coralReef sovereign shader compiler role.
    pub const COMPILER: &str = "coralreef";

    /// petalTongue visualization role.
    pub const VISUALIZATION: &str = "petaltongue";

    /// Squirrel AI assistant role.
    pub const ASSISTANT: &str = "squirrel";

    // ─── Provenance Trio ─────

    /// rhizoCrypt — ephemeral DAG and working memory.
    pub const PROVENANCE_DAG: &str = "rhizocrypt";

    /// loamSpine — provenance and attestation.
    pub const PROVENANCE_ATTEST: &str = "loamspine";

    /// sweetGrass — semantic provenance and attribution.
    pub const PROVENANCE_SEMANTIC: &str = "sweetgrass";
}

// ─── Generic Discovery Helpers ─────

/// Generate the environment variable name for a primal's socket path.
///
/// Follows the ecosystem convention: `{UPPER_NAME}_SOCKET`.
/// E.g., `socket_env_var("groundspring")` → `"GROUNDSPRING_SOCKET"`.
///
/// Pattern source: sweetGrass v0.7.18 generic primal discovery.
#[must_use]
pub fn socket_env_var(primal_name: &str) -> String {
    format!("{}_SOCKET", primal_name.to_uppercase())
}

/// Generate the environment variable name for a primal's address (host:port).
///
/// Follows the ecosystem convention: `{UPPER_NAME}_ADDRESS`.
#[must_use]
pub fn address_env_var(primal_name: &str) -> String {
    format!("{}_ADDRESS", primal_name.to_uppercase())
}

/// Resolve the active `FAMILY_ID` for multi-family socket paths.
///
/// Checks `BIOMEOS_FAMILY_ID` env var, falls back to `"default"`.
/// Absorbed from toadStool S156 / songBird 0.2.2 family ID convention.
#[must_use]
pub fn family_id() -> String {
    std::env::var("BIOMEOS_FAMILY_ID").unwrap_or_else(|_| "default".to_string())
}

/// Probe the environment for a primal's socket path.
///
/// Discovery chain:
/// 1. `{UPPER_ROLE}_SOCKET` env var (explicit override)
/// 2. `$XDG_RUNTIME_DIR/biomeos/{role}-{family_id}.sock` (FAMILY_ID-aware)
/// 3. `$XDG_RUNTIME_DIR/biomeos/{role}.sock` (legacy flat name)
///
/// Returns `Some(path)` only if the env var is set *and* the socket
/// exists on disk — no assumptions about what is running.
///
/// Absorbed from neuralSpring V113 `discover_primal()` pattern.
#[must_use]
pub fn discover_socket(role: &str) -> Option<std::path::PathBuf> {
    let key = socket_env_var(role);
    if let Ok(val) = std::env::var(&key) {
        let path = std::path::PathBuf::from(val);
        if path.exists() {
            return Some(path);
        }
    }

    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        let dir = std::path::PathBuf::from(xdg).join(BIOMEOS_SOCKET_DIR);

        let family = family_id();
        let family_path = dir.join(format!("{role}-{family}.sock"));
        if family_path.exists() {
            return Some(family_path);
        }

        let flat_path = dir.join(format!("{role}.sock"));
        if flat_path.exists() {
            return Some(flat_path);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_env_var_uppercases() {
        assert_eq!(socket_env_var("groundspring"), "GROUNDSPRING_SOCKET");
        assert_eq!(socket_env_var("biomeos"), "BIOMEOS_SOCKET");
    }

    #[test]
    fn address_env_var_uppercases() {
        assert_eq!(address_env_var("nestgate"), "NESTGATE_ADDRESS");
    }

    #[test]
    fn discover_socket_returns_none_for_missing() {
        assert!(discover_socket("nonexistent_test_primal").is_none());
    }

    #[test]
    fn roles_are_lowercase() {
        assert_eq!(roles::ORCHESTRATOR, "biomeos");
        assert_eq!(roles::COMPUTE, "toadstool");
        assert_eq!(roles::PROVENANCE_DAG, "rhizocrypt");
    }
}
