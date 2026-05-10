// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Optional `biomeOS` Neural API client for ecosystem integration.
//!
//! When `GROUNDSPRING_COMPUTE_PROVIDER=biomeos` is set, groundSpring routes
//! compute-intensive operations through biomeOS's Neural API instead of running
//! them locally. Falls back to sovereign local computation if the socket is
//! unavailable.
//!
//! # Protocol
//!
//! JSON-RPC 2.0, newline-delimited, over platform-agnostic transport.
//!
//! Transport selection:
//! - **Unix**: Unix domain socket (preferred, zero-copy-friendly)
//! - **Non-Unix**: TCP via `GROUNDSPRING_BIOMEOS_TCP` env var
//!
//! Socket discovery (capability-based, no hardcoded paths):
//! 1. `GROUNDSPRING_BIOMEOS_SOCKET` env var (explicit override)
//! 2. `$XDG_RUNTIME_DIR/biomeos/neural-api-default.sock`
//! 3. `<temp_dir>/biomeos-neural-api.sock` (platform-agnostic fallback)
//!
//! # Sovereign fallback
//!
//! All operations work without `biomeOS`. When the socket is unavailable,
//! `capability_call` and `rpc_call` return `Err`, and callers fall back to
//! local computation. This follows the same pattern as wetSpring's `NestGate`
//! client.
//!
//! # Evolution path
//!
//! | Phase | Strategy | Status |
//! |-------|----------|--------|
//! | Phase 0 | Live NUCLEUS local, sovereign fallback | **active** |
//! | Phase 1 | Data pipeline via `NestGate` live providers | active |
//! | Phase 2 | `ToadStool` GPU dispatch via `compute.execute` | planned |
//! | Phase 3 | `metalForge` cross-substrate via Neural API | planned |

mod compute;
mod discovery;
mod health;
mod interaction;
mod protocol;
mod registration;
pub mod resilience;
mod routing;
pub mod server;
mod storage;
mod transport;

pub use compute::{compute_capabilities, compute_execute, compute_submit};
pub use discovery::{auto_connect, discover_socket, is_nucleus_available};
pub use health::{CompositionStatus, composition_status, health};
pub use interaction::{
    DiscoveredPrimal, direct_primal_rpc, discover_by_capability, discover_primals,
    dispatch_capabilities, dispatch_result, dispatch_submit, primal_health, proprioception,
    topology,
};
pub use registration::{deregister_capabilities, register_capabilities, register_methods};
pub use routing::{capability_call, capability_call_typed, direct_rpc_call};
pub use storage::{storage_get, storage_put};

use std::path::Path;
use std::time::Duration;

use transport::rpc_call;

// ─── Configuration ───────────────────────────────────────────────────────────

/// Default connect timeout in seconds when env var is unset.
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 5;

/// Default read timeout in seconds when env var is unset.
const DEFAULT_READ_TIMEOUT_SECS: u64 = 30;

/// Connect timeout, overridable via `GROUNDSPRING_BIOMEOS_CONNECT_TIMEOUT_SECS`.
fn connect_timeout() -> Duration {
    connect_timeout_with_env(|k| std::env::var(k).ok())
}

fn connect_timeout_with_env(env: impl Fn(&str) -> Option<String>) -> Duration {
    Duration::from_secs(
        env("GROUNDSPRING_BIOMEOS_CONNECT_TIMEOUT_SECS")
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_CONNECT_TIMEOUT_SECS),
    )
}

/// Read timeout, overridable via `GROUNDSPRING_BIOMEOS_READ_TIMEOUT_SECS`.
fn read_timeout() -> Duration {
    read_timeout_with_env(|k| std::env::var(k).ok())
}

fn read_timeout_with_env(env: impl Fn(&str) -> Option<String>) -> Duration {
    Duration::from_secs(
        env("GROUNDSPRING_BIOMEOS_READ_TIMEOUT_SECS")
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_READ_TIMEOUT_SECS),
    )
}

/// Family identifier for all biomeOS interactions.
///
/// Delegates to [`crate::niche::NICHE_ID`] — the single source of truth
/// for this spring's identity within the ecosystem.
pub const FAMILY_ID: &str = crate::niche::NICHE_ID;

// ─── Error Type ──────────────────────────────────────────────────────────────

/// Error type for `biomeOS` client operations.
///
/// Typed variants replace the former `BiomeOsError(String)` for better
/// error handling and pattern matching. The `Other` variant handles
/// messages that don't fit a specific category.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BiomeOsError {
    /// Transport-level failure (connect, read, write, flush, timeout).
    #[error("biomeOS transport: {0}")]
    Transport(String),
    /// JSON-RPC protocol error (invalid response, missing fields, RPC error).
    #[error("biomeOS protocol: {0}")]
    Protocol(String),
    /// Serialization error (invalid params JSON).
    #[error("biomeOS serialization: {0}")]
    Serialization(String),
    /// Capability registration failure.
    #[error("biomeOS registration: {0}")]
    Registration(String),
    /// Primal discovery or health check failure.
    #[error("biomeOS discovery: {0}")]
    Discovery(String),
    /// Data pipeline error (no results, empty response).
    #[error("biomeOS data: {0}")]
    Data(String),
    /// Uncategorized error (migration path from `BiomeOsError(String)`).
    #[error("biomeOS: {0}")]
    Other(String),
}

impl BiomeOsError {
    /// Construct from a plain string (backwards-compatible migration path).
    #[must_use]
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }

    /// Whether this error is transient and the operation may succeed on retry.
    ///
    /// Transport errors and some discovery failures are recoverable.
    /// Protocol, serialization, and data errors are permanent.
    ///
    /// Absorbed from airSpring V0.8.8 / wetSpring V121 `is_recoverable()` pattern.
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(self, Self::Transport(_) | Self::Discovery(_))
    }

    /// Whether a retry with backoff is appropriate.
    ///
    /// Same as [`is_recoverable`](Self::is_recoverable) — transient errors
    /// should be retried; permanent errors should be surfaced immediately.
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        self.is_recoverable()
    }

    /// Whether this is a JSON-RPC "method not found" error.
    ///
    /// Useful for fallback chains where the first method may not be
    /// supported by the target primal version.
    #[must_use]
    pub fn is_method_not_found(&self) -> bool {
        match self {
            Self::Protocol(msg) => msg.contains("-32601") || msg.contains("method not found"),
            _ => false,
        }
    }
}

/// Result alias for `biomeOS` operations.
pub type Result<T> = std::result::Result<T, BiomeOsError>;

// ─── Feature Detection ───────────────────────────────────────────────────────

/// Whether `biomeOS` routing is enabled via environment.
///
/// Accepts `GROUNDSPRING_COMPUTE_PROVIDER` set to:
/// - The orchestrator role name (`"biomeos"`) — original convention
/// - `"true"` or `"1"` — generic boolean enable
///
/// This avoids hardcoding which specific provider name enables routing,
/// while maintaining backward compatibility with existing deployments.
#[must_use]
pub fn is_enabled() -> bool {
    std::env::var("GROUNDSPRING_COMPUTE_PROVIDER").is_ok_and(|v| {
        let v = v.trim();
        v.eq_ignore_ascii_case(crate::primal_names::roles::ORCHESTRATOR)
            || v.eq_ignore_ascii_case("true")
            || v == "1"
    })
}

// ─── Measurement Domain ──────────────────────────────────────────────────────

/// Capability domain name for groundSpring's measurement validation.
///
/// Delegates to [`crate::niche::DOMAIN`] — the single source of truth.
pub const MEASUREMENT_DOMAIN: &str = crate::niche::DOMAIN;

/// Measurement capabilities that groundSpring registers with the NUCLEUS.
///
/// Delegates to [`crate::niche::CAPABILITIES`] — the single source of truth.
pub const MEASUREMENT_CAPABILITIES: &[&str] = crate::niche::CAPABILITIES;

/// Legacy alias — callers that referenced `SCIENCE_CAPABILITIES` will
/// continue to compile. New code should use [`MEASUREMENT_CAPABILITIES`].
#[deprecated(note = "use MEASUREMENT_CAPABILITIES (measurement.* domain)")]
pub const SCIENCE_CAPABILITIES: &[&str] = MEASUREMENT_CAPABILITIES;

/// Semantic mappings from measurement domain operations to JSON-RPC methods.
///
/// Delegates to [`crate::niche::SEMANTIC_MAPPINGS`] — the single source of truth.
pub const MEASUREMENT_MAPPINGS: &[(&str, &str)] = crate::niche::SEMANTIC_MAPPINGS;

/// Send an arbitrary JSON-RPC request over the transport and read the response.
///
/// For use by integration tests and advanced consumers that need to send
/// raw JSON-RPC to the Neural API.
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable or the RPC fails.
pub fn raw_rpc_call(socket: &Path, request: &str) -> Result<String> {
    rpc_call(socket, request)
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod client_tests;
