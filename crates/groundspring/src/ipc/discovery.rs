// SPDX-License-Identifier: AGPL-3.0-or-later

//! IPC socket discovery for groundSpring.
//!
//! Uses the primal's own `socket_env_var` for the override key,
//! then standard XDG and temp fallback — no hardcoded primal names
//! beyond self-identity.

use std::path::PathBuf;

const TARPC_SOCK_SUFFIX: &str = "ipc.sock";

/// Build the tarpc socket filename from primal self-identity.
pub fn tarpc_sock_name() -> String {
    format!("{}-{TARPC_SOCK_SUFFIX}", crate::primal_names::SELF_ID)
}

/// Discover the groundSpring IPC socket path via environment.
///
/// Fallback chain:
/// 1. `GROUNDSPRING_SOCKET` env var
/// 2. `$XDG_RUNTIME_DIR/biomeos/groundspring-ipc.sock`
/// 3. `<temp_dir>/groundspring-ipc.sock`
pub fn discover_ipc_socket() -> Option<PathBuf> {
    let env_key = crate::primal_names::socket_env_var(crate::primal_names::SELF_ID);
    if let Ok(explicit) = std::env::var(&env_key) {
        let path = PathBuf::from(explicit);
        if path.exists() {
            return Some(path);
        }
    }

    let sock_name = tarpc_sock_name();

    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        let path = PathBuf::from(xdg)
            .join(crate::primal_names::BIOMEOS_SOCKET_DIR)
            .join(&sock_name);
        if path.exists() {
            return Some(path);
        }
    }

    let path = std::env::temp_dir().join(&sock_name);
    if path.exists() {
        return Some(path);
    }

    None
}
