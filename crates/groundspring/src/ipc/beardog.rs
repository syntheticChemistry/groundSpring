// SPDX-License-Identifier: AGPL-3.0-or-later

//! IPC interface for BearDog cryptographic operations.
//!
//! BearDog provides the security foundation for the NUCLEUS. groundSpring
//! uses BearDog for:
//! - Cryptographic signing of validation artifacts
//! - Hash verification (BLAKE3 witness)
//! - BTSP session authentication (future: GAP-GS-009)
//!
//! # Capability surface
//!
//! - `crypto.sign` — sign a payload
//! - `crypto.verify` — verify a signature

/// Cryptographic service traits via BearDog.
#[tarpc::service]
pub trait CryptoService {
    /// Sign a payload and return the signature.
    async fn sign(payload: Vec<u8>, key_id: String) -> Result<Vec<u8>, String>;

    /// Verify a signature against a payload.
    async fn verify(payload: Vec<u8>, signature: Vec<u8>, key_id: String) -> Result<bool, String>;

    /// Compute a BLAKE3 hash of the payload.
    async fn hash_blake3(payload: Vec<u8>) -> Result<String, String>;
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
