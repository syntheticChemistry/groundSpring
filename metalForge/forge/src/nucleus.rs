// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Shared utilities for NUCLEUS validation binaries.
//!
//! Provides UID discovery, biomeOS socket directory resolution, and an
//! extended validation harness with `finish() -> bool` semantics for
//! binaries that manage their own exit code.

/// Discover the current user's UID without `libc` or `unsafe`.
///
/// Priority chain (first success wins):
/// 1. `$UID` (set by most shells)
/// 2. `/proc/self/status` `Uid:` field (Linux only)
/// 3. `id -u` command (portable POSIX fallback)
/// 4. Enumerate `/run/user/` directory entries
/// 5. Falls back to `"0"` with a warning (never panics)
#[must_use]
pub fn discover_uid() -> String {
    if let Ok(uid) = std::env::var("UID")
        && !uid.is_empty()
    {
        return uid;
    }

    #[cfg(target_os = "linux")]
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if let Some(rest) = line.strip_prefix("Uid:")
                && let Some(uid_str) = rest.split_whitespace().next()
            {
                return uid_str.to_string();
            }
        }
    }

    if let Ok(output) = std::process::Command::new("id").arg("-u").output()
        && output.status.success()
    {
        let uid = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !uid.is_empty() {
            return uid;
        }
    }

    tracing::warn!(
        "UID discovery failed ($UID unset, /proc/self/status unreadable, \
         `id -u` unavailable). Set $UID or $BIOMEOS_SOCKET_DIR to override."
    );
    #[cfg(target_os = "linux")]
    {
        tracing::warn!("Attempting /run/user/ enumeration as last resort.");
        if let Ok(entries) = std::fs::read_dir("/run/user") {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str()
                    && name.chars().all(|c| c.is_ascii_digit())
                {
                    tracing::warn!("Using discovered UID {name} from /run/user/");
                    return name.to_string();
                }
            }
        }
    }
    tracing::error!(
        "Cannot discover UID. Set $UID or $BIOMEOS_SOCKET_DIR environment variable. \
         Falling back to UID 0."
    );
    String::from("0")
}

/// Socket directory name for `biomeOS` IPC mesh.
///
/// Mirrors `groundspring::primal_names::BIOMEOS_SOCKET_DIR` — duplicated here
/// because `metalForge/forge` is a separate crate with minimal dependencies.
const BIOMEOS_DIR: &str = "biomeos";

/// Discover the `biomeOS` socket directory.
///
/// Priority: `BIOMEOS_SOCKET_DIR` > `XDG_RUNTIME_DIR/{BIOMEOS_DIR}` >
/// `/run/user/<uid>/{BIOMEOS_DIR}`. Never hardcodes a specific UID.
#[must_use]
pub fn biomeos_socket_dir() -> String {
    if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        return dir;
    }
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        return format!("{xdg}/{BIOMEOS_DIR}");
    }
    format!("/run/user/{}/{BIOMEOS_DIR}", discover_uid())
}

/// Extended pass/fail harness for NUCLEUS validation binaries.
///
/// Unlike [`crate::harness::Harness`] (which calls `process::exit`),
/// this variant returns the pass/fail result via [`Self::finish`] so the
/// caller can set the exit code.
pub struct NucleusHarness {
    passed: u32,
    failed: u32,
    total: u32,
}

impl NucleusHarness {
    /// Create a new harness with zero counters.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            passed: 0,
            failed: 0,
            total: 0,
        }
    }

    /// Record a named check result.
    pub fn check(&mut self, name: &str, ok: bool) {
        self.total += 1;
        if ok {
            self.passed += 1;
            println!("  PASS  {name}");
        } else {
            self.failed += 1;
            println!("  FAIL  {name}");
        }
    }

    /// Print summary and return whether all checks passed.
    #[must_use]
    pub fn finish(self) -> bool {
        println!();
        println!("=== {}/{} checks passed ===", self.passed, self.total);
        self.failed == 0
    }
}

impl Default for NucleusHarness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_uid_returns_string() {
        let uid = discover_uid();
        assert!(!uid.is_empty());
    }

    #[test]
    fn biomeos_socket_dir_returns_string() {
        let dir = biomeos_socket_dir();
        assert!(!dir.is_empty());
    }

    #[test]
    fn nucleus_harness_pass() {
        let mut h = NucleusHarness::new();
        h.check("trivial", true);
        assert!(h.finish());
    }

    #[test]
    fn nucleus_harness_fail() {
        let mut h = NucleusHarness::new();
        h.check("ok", true);
        h.check("bad", false);
        assert!(!h.finish());
    }
}
