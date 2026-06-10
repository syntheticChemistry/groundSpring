// SPDX-License-Identifier: AGPL-3.0-or-later

//! IPC interface for `BearDog` cryptographic operations.
//!
//! `BearDog` provides the security foundation for the `NUCLEUS`. `groundSpring`
//! uses `BearDog` for:
//! - Cryptographic signing of validation artifacts
//! - Hash verification (BLAKE3 witness)
//! - Seed fingerprinting for PRNG consistency (Wave 102)
//! - BTSP session authentication (future: GAP-GS-009)
//!
//! # Wire name convention
//!
//! `BearDog` JSON-RPC uses base64-encoded `message` field (not raw `data`).
//! When constructing JSON-RPC calls, payloads must be base64-encoded and
//! sent as `"message": "<base64>"`. This was confirmed by ludoSpring's
//! Tower atomic live validation.
//!
//! # Capability surface
//!
//! - `crypto.sign` — sign a base64-encoded message
//! - `crypto.verify` — verify a signature against a message
//! - `crypto.hash_blake3` — compute BLAKE3 hash
//! - `crypto.seed_fingerprint` — fingerprint a PRNG seed for consistency (Wave 102)

/// Cryptographic service traits via `BearDog`.
#[tarpc::service]
pub trait CryptoService {
    /// Sign a payload and return the signature.
    async fn sign(payload: Vec<u8>, key_id: String) -> Result<Vec<u8>, String>;

    /// Verify a signature against a payload.
    async fn verify(payload: Vec<u8>, signature: Vec<u8>, key_id: String) -> Result<bool, String>;

    /// Compute a BLAKE3 hash of the payload.
    async fn hash_blake3(payload: Vec<u8>) -> Result<String, String>;

    /// Fingerprint a PRNG seed for cross-primal consistency.
    ///
    /// Returns a deterministic fingerprint that can be compared across
    /// primals to verify seed alignment. Upstream: `crypto.seed_fingerprint`
    /// (Wave 102).
    async fn seed_fingerprint(seed: Vec<u8>) -> Result<String, String>;
}

/// Sign a message via `BearDog` JSON-RPC.
///
/// `message_b64` must be a base64-encoded payload (BearDog convention:
/// `message` field, not `data`).
///
/// # Errors
///
/// Returns `BiomeOsError` if `BearDog` is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn crypto_sign(
    socket: &std::path::Path,
    message_b64: &str,
    key_id: &str,
) -> crate::biomeos::Result<serde_json::Value> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "crypto.sign",
        "params": {
            "message": message_b64,
            "key_id": key_id,
        },
        "id": 1
    })
    .to_string();
    let response = crate::biomeos::raw_rpc_call(socket, &request)?;
    crate::biomeos::protocol::extract_rpc_result(&response)
}

/// Compute a BLAKE3 hash via `BearDog` JSON-RPC.
///
/// # Errors
///
/// Returns `BiomeOsError` if `BearDog` is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn crypto_hash_blake3(
    socket: &std::path::Path,
    message_b64: &str,
) -> crate::biomeos::Result<serde_json::Value> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "crypto.hash_blake3",
        "params": {
            "message": message_b64,
        },
        "id": 1
    })
    .to_string();
    let response = crate::biomeos::raw_rpc_call(socket, &request)?;
    crate::biomeos::protocol::extract_rpc_result(&response)
}

/// Fingerprint a PRNG seed via `BearDog` JSON-RPC.
///
/// # Errors
///
/// Returns `BiomeOsError` if `BearDog` is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn crypto_seed_fingerprint(
    socket: &std::path::Path,
    seed_b64: &str,
) -> crate::biomeos::Result<serde_json::Value> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "crypto.seed_fingerprint",
        "params": {
            "seed": seed_b64,
        },
        "id": 1
    })
    .to_string();
    let response = crate::biomeos::raw_rpc_call(socket, &request)?;
    crate::biomeos::protocol::extract_rpc_result(&response)
}

/// Attempt to discover `BearDog` and sign a message.
///
/// Returns `Ok(None)` if `BearDog` is not available (graceful degradation).
#[cfg(feature = "biomeos")]
pub fn try_crypto_sign(
    message_b64: &str,
    key_id: &str,
) -> crate::biomeos::Result<Option<serde_json::Value>> {
    crate::primal_names::discover_socket(crate::primal_names::roles::SECURITY).map_or_else(
        || {
            tracing::debug!("BearDog not discovered — crypto sign skipped");
            Ok(None)
        },
        |socket| crypto_sign(&socket, message_b64, key_id).map(Some),
    )
}

/// Attempt to discover `BearDog` and compute a BLAKE3 hash.
///
/// Returns `Ok(None)` if `BearDog` is not available (graceful degradation).
#[cfg(feature = "biomeos")]
pub fn try_crypto_hash_blake3(
    message_b64: &str,
) -> crate::biomeos::Result<Option<serde_json::Value>> {
    crate::primal_names::discover_socket(crate::primal_names::roles::SECURITY).map_or_else(
        || {
            tracing::debug!("BearDog not discovered — BLAKE3 hash skipped");
            Ok(None)
        },
        |socket| crypto_hash_blake3(&socket, message_b64).map(Some),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tarpc_trait_compiles() {
        fn _assert_service<T: CryptoService>() {}
    }

    #[test]
    fn security_role_is_beardog() {
        assert_eq!(crate::primal_names::roles::SECURITY, "beardog");
    }
}
