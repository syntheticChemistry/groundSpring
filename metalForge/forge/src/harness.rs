// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Shared validation harness for metalForge binaries.
//!
//! Follows the hotSpring pattern: hardcoded expected values, explicit
//! pass/fail, exit code 0 (all pass) / 1 (any failure).

/// Lightweight pass/fail harness for validation binaries.
///
/// Tracks individual check outcomes and terminates the process with
/// exit code 1 if any check fails.
pub struct Harness {
    pass: u32,
    fail: u32,
}

impl Harness {
    /// Create a new harness with zero counters.
    #[must_use]
    pub const fn new() -> Self {
        Self { pass: 0, fail: 0 }
    }

    /// Record a named check result.
    pub fn check(&mut self, name: &str, ok: bool) {
        if ok {
            println!("  PASS  {name}");
            self.pass += 1;
        } else {
            println!("  FAIL  {name}");
            self.fail += 1;
        }
    }

    /// Print summary and exit with code 1 if any check failed.
    pub fn finish(self) {
        let total = self.pass + self.fail;
        println!("\n=== {}/{total} checks passed ===", self.pass);
        if self.fail > 0 {
            std::process::exit(1);
        }
    }
}

impl Default for Harness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_pass_does_not_panic() {
        let mut h = Harness::new();
        h.check("trivial", true);
        assert_eq!(h.pass, 1);
        assert_eq!(h.fail, 0);
    }

    #[test]
    fn failure_tracked() {
        let mut h = Harness::new();
        h.check("ok", true);
        h.check("bad", false);
        assert_eq!(h.pass, 1);
        assert_eq!(h.fail, 1);
    }

    #[test]
    fn default_is_zero() {
        let h = Harness::default();
        assert_eq!(h.pass, 0);
        assert_eq!(h.fail, 0);
    }
}
