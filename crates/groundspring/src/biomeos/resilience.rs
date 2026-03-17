// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! IPC resilience primitives: retry with backoff and circuit breaker.
//!
//! Absorbed from petalTongue v1.6.6 / rhizoCrypt v0.13 patterns.
//! Used for provenance trio calls (rhizoCrypt, loamSpine, sweetGrass)
//! and other IPC paths where transient failures are expected.

use std::time::{Duration, Instant};

/// Exponential backoff retry policy.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (0 = no retries).
    pub max_retries: u32,
    /// Initial delay before first retry.
    pub initial_delay: Duration,
    /// Maximum delay cap.
    pub max_delay: Duration,
    /// Backoff multiplier (typically 2.0).
    pub multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    /// Compute the delay for a given attempt (0-indexed).
    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_possible_wrap
    )]
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base_ms = self.initial_delay.as_millis() as f64;
        let delay_ms = base_ms * self.multiplier.powi(attempt as i32);
        let capped = Duration::from_millis(delay_ms.min(self.max_delay.as_millis() as f64) as u64);
        capped.min(self.max_delay)
    }

    /// Execute `f` with retries. Returns `Ok` on first success or the last error.
    ///
    /// # Errors
    ///
    /// Returns the last error from `f` if all attempts fail.
    ///
    /// # Panics
    ///
    /// Panics if `max_retries` is set such that no attempts are made (internal invariant).
    pub fn execute<T, E>(&self, mut f: impl FnMut() -> Result<T, E>) -> Result<T, E> {
        let mut last_err = None;
        for attempt in 0..=self.max_retries {
            match f() {
                Ok(val) => return Ok(val),
                Err(e) => {
                    last_err = Some(e);
                    if attempt < self.max_retries {
                        std::thread::sleep(self.delay_for_attempt(attempt));
                    }
                }
            }
        }
        Err(last_err.unwrap_or_else(|| unreachable!("at least one attempt")))
    }
}

/// Circuit breaker states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation — requests flow through.
    Closed,
    /// Failures exceeded threshold — requests are rejected immediately.
    Open,
    /// Cooldown elapsed — next request is a probe.
    HalfOpen,
}

/// Circuit breaker for IPC endpoints.
///
/// Tracks consecutive failures and opens the circuit when a threshold
/// is exceeded, preventing cascading failures during outages.
#[derive(Debug)]
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    cooldown: Duration,
    last_failure: Option<Instant>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    #[must_use]
    pub const fn new(failure_threshold: u32, cooldown: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            cooldown,
            last_failure: None,
        }
    }

    /// Current circuit state.
    #[must_use]
    pub fn state(&self) -> CircuitState {
        match self.state {
            CircuitState::Open => {
                if let Some(last) = self.last_failure
                    && last.elapsed() >= self.cooldown
                {
                    return CircuitState::HalfOpen;
                }
                CircuitState::Open
            }
            other => other,
        }
    }

    /// Check if requests should be allowed through.
    #[must_use]
    pub fn is_allowed(&self) -> bool {
        matches!(self.state(), CircuitState::Closed | CircuitState::HalfOpen)
    }

    /// Record a successful call — resets failure count and closes circuit.
    pub const fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
    }

    /// Record a failed call — increments count and may open circuit.
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Instant::now());
        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_policy_delay_exponential() {
        let policy = RetryPolicy {
            initial_delay: Duration::from_millis(100),
            multiplier: 2.0,
            max_delay: Duration::from_secs(10),
            ..Default::default()
        };
        assert_eq!(policy.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(policy.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(policy.delay_for_attempt(2), Duration::from_millis(400));
    }

    #[test]
    fn retry_policy_caps_at_max() {
        let policy = RetryPolicy {
            initial_delay: Duration::from_millis(100),
            multiplier: 10.0,
            max_delay: Duration::from_millis(500),
            ..Default::default()
        };
        assert_eq!(policy.delay_for_attempt(3), Duration::from_millis(500));
    }

    #[test]
    fn retry_succeeds_on_second_attempt() {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay: Duration::from_millis(1),
            ..Default::default()
        };
        let mut attempts = 0u32;
        let result: Result<&str, &str> = policy.execute(|| {
            attempts += 1;
            if attempts < 2 {
                Err("transient")
            } else {
                Ok("success")
            }
        });
        assert_eq!(result, Ok("success"));
        assert_eq!(attempts, 2);
    }

    #[test]
    fn retry_exhausts_retries() {
        let policy = RetryPolicy {
            max_retries: 2,
            initial_delay: Duration::from_millis(1),
            ..Default::default()
        };
        let mut attempts = 0u32;
        let result: Result<(), &str> = policy.execute(|| {
            attempts += 1;
            Err("always fails")
        });
        assert_eq!(result, Err("always fails"));
        assert_eq!(attempts, 3); // initial + 2 retries
    }

    #[test]
    fn circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(5));
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.is_allowed());
    }

    #[test]
    fn circuit_breaker_opens_after_threshold() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(5));
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.is_allowed());
    }

    #[test]
    fn circuit_breaker_resets_on_success() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(5));
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn circuit_breaker_half_open_after_cooldown() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(1));
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        std::thread::sleep(Duration::from_millis(5));
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        assert!(cb.is_allowed());
    }
}
