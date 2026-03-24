// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! IPC client error type shared with [`crate::ipc`] when `tarpc-ipc` is enabled.

/// Error type for typed IPC client operations.
///
/// Structured variants for the IPC lifecycle: connect, transport, and
/// remote (application-level) errors. Pattern source: rhizoCrypt v0.13.0
/// `IpcErrorPhase` / healthSpring V30.
#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    /// Failed to connect to the IPC socket.
    #[error("ipc connect: {0}")]
    Connect(String),
    /// Transport-level error during an RPC call.
    #[error("ipc transport: {0}")]
    Transport(String),
    /// Remote endpoint returned an application error.
    #[error("ipc remote: {0}")]
    Remote(String),
    /// No IPC socket discovered via environment.
    #[error("ipc discovery: {0}")]
    Discovery(String),
}

impl IpcError {
    /// Whether this error is transient and the operation may succeed on retry.
    ///
    /// Connect and transport errors are typically transient (socket busy,
    /// timeout, temporary network issue). Remote and discovery errors are
    /// permanent (method not found, no socket configured).
    ///
    /// Absorbed from wetSpring V132 / healthSpring V41 `is_recoverable()` pattern.
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(self, Self::Connect(_) | Self::Transport(_))
    }
}

/// Result alias for IPC operations.
#[cfg_attr(
    not(feature = "tarpc-ipc"),
    expect(dead_code, reason = "used only with tarpc-ipc feature")
)]
pub type IpcResult<T> = Result<T, IpcError>;

#[cfg(test)]
mod tests {
    use super::IpcError;

    #[test]
    fn connect_and_transport_are_recoverable() {
        assert!(IpcError::Connect("x".into()).is_recoverable());
        assert!(IpcError::Transport("y".into()).is_recoverable());
    }

    #[test]
    fn remote_and_discovery_are_not_recoverable() {
        assert!(!IpcError::Remote("z".into()).is_recoverable());
        assert!(!IpcError::Discovery("w".into()).is_recoverable());
    }
}
