// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Shared validation harness for metalForge binaries.
//!
//! Follows the hotSpring pattern: hardcoded expected values, explicit
//! pass/fail, exit code 0 (all pass) / 1 (any failure).

/// Maximum GPU device-lost retries before giving up.
const GPU_RETRY_LIMIT: u32 = 2;

/// Lightweight pass/fail harness for validation binaries.
///
/// Tracks individual check outcomes and terminates the process with
/// exit code 1 if any check fails.
///
/// ```
/// use groundspring_forge::harness::Harness;
///
/// let mut h = Harness::new();
/// h.check("two plus two", 2 + 2 == 4);
/// // pass count is tracked internally; `finish()` exits 0 when all pass
/// ```
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

    /// Record a GPU check with device-lost retry (barraCuda S87).
    ///
    /// Calls `gpu_fn` up to `GPU_RETRY_LIMIT` + 1 times; if the closure
    /// returns `Err(e)` where `e.is_device_lost()`, retries silently.
    /// Other errors and the final retry result become a FAIL check.
    pub fn check_gpu_resilient<F>(&mut self, name: &str, mut gpu_fn: F)
    where
        F: FnMut() -> Result<bool, barracuda::error::BarracudaError>,
    {
        for attempt in 0..=GPU_RETRY_LIMIT {
            match gpu_fn() {
                Ok(ok) => {
                    if attempt > 0 {
                        println!("  (retry {attempt} succeeded)");
                    }
                    self.check(name, ok);
                    return;
                }
                Err(e) if e.is_device_lost() && attempt < GPU_RETRY_LIMIT => {
                    println!("  RETRY {name} (device lost, attempt {})", attempt + 1);
                }
                Err(e) => {
                    println!("  FAIL  {name}: {e}");
                    self.fail += 1;
                    return;
                }
            }
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
