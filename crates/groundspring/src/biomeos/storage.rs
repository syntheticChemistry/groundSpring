// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Capability-based storage routing for biomeOS (`storage.put` / `storage.get`).

use std::path::Path;

use super::routing::capability_call_value;
use super::{FAMILY_ID, Result};

/// Store a value via biomeOS capability-based storage routing.
///
/// Routes through `storage.put` capability — biomeOS translates to the
/// storage provider's actual method at runtime.
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable or the RPC fails.
pub fn storage_put(socket: &Path, key: &str, value: &str) -> Result<()> {
    let args = serde_json::json!({
        "key": key,
        "value": value,
        "family_id": FAMILY_ID,
    });
    capability_call_value(socket, "storage.put", &args)?;
    Ok(())
}

/// Retrieve a value via biomeOS capability-based storage routing.
///
/// Routes through `storage.get` capability — biomeOS translates to the
/// storage provider's actual method at runtime.
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable, the key does not exist,
/// or the RPC fails.
pub fn storage_get(socket: &Path, key: &str) -> Result<String> {
    let args = serde_json::json!({
        "key": key,
        "family_id": FAMILY_ID,
    });
    capability_call_value(socket, "storage.get", &args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_put_on_nonexistent_socket_fails() {
        let path = std::path::Path::new("/tmp/nonexistent_groundspring_test.sock");
        assert!(storage_put(path, "key", "value").is_err());
    }

    #[test]
    fn storage_get_on_nonexistent_socket_fails() {
        let path = std::path::Path::new("/tmp/nonexistent_groundspring_test.sock");
        assert!(storage_get(path, "key").is_err());
    }
}
