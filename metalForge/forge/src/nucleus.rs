// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Shared utilities for NUCLEUS validation binaries.
//!
//! Provides UID discovery, biomeOS socket directory resolution, and an
//! extended validation harness with `finish() -> bool` semantics for
//! binaries that manage their own exit code.

/// Discover the current user's UID without `libc` or `unsafe`.
///
/// Checks `$UID` (set by most shells), then falls back to parsing
/// `/proc/self/status` on Linux. Logs a warning and returns `"1000"`
/// as a last resort.
#[must_use]
pub fn discover_uid() -> String {
    if let Ok(uid) = std::env::var("UID") {
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
    log::warn!(
        "UID discovery failed ($UID unset, /proc/self/status unreadable). \
         Falling back to UID 1000. Set $UID or $BIOMEOS_SOCKET_DIR to override."
    );
    String::from("1000")
}

/// Discover the biomeOS socket directory.
///
/// Priority: `BIOMEOS_SOCKET_DIR` > `XDG_RUNTIME_DIR/biomeos` >
/// `/run/user/<uid>/biomeos`. Never hardcodes a specific UID.
#[must_use]
pub fn biomeos_socket_dir() -> String {
    if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        return dir;
    }
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        return format!("{xdg}/biomeos");
    }
    format!("/run/user/{}/biomeos", discover_uid())
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
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
