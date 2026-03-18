// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! IPC resilience primitives: retry with backoff and circuit breaker.
//!
//! Absorbed from petalTongue v1.6.6 / rhizoCrypt v0.13 patterns.
//! Used for provenance trio calls (rhizoCrypt, loamSpine, sweetGrass)
//! and other IPC paths where transient failures are expected.

use std::time::{Duration, Instant};

/// Typed error from [`resilient_call`].
///
/// Distinguishes between a circuit-open fast-fail and retry exhaustion
/// that preserves the last underlying error.
#[derive(Debug, thiserror::Error)]
pub enum ResilienceError<E: std::fmt::Debug> {
    /// The circuit breaker is open — IPC endpoint is considered unavailable.
    #[error("circuit open — IPC endpoint unavailable")]
    CircuitOpen,
    /// All retry attempts failed; carries the last error from the closure.
    #[error("retries exhausted after {attempts} attempts: {last_error}")]
    RetriesExhausted {
        /// Total attempts made (initial + retries).
        attempts: u32,
        /// The error from the final attempt.
        last_error: E,
    },
}

/// Default exponential backoff multiplier (doubles each attempt).
const DEFAULT_BACKOFF_MULTIPLIER: f64 = 2.0;

/// Default maximum retry attempts before giving up.
const DEFAULT_MAX_RETRIES: u32 = 3;

/// Default initial delay before the first retry.
const DEFAULT_INITIAL_DELAY: Duration = Duration::from_millis(100);

/// Default maximum delay cap to prevent unbounded waits.
const DEFAULT_MAX_DELAY: Duration = Duration::from_secs(5);

/// Scale factor for converting the floating-point multiplier to an
/// integer ratio — avoids f64→Duration casts in the delay computation.
const RATIO_SCALE: u32 = 1000;

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
            max_retries: DEFAULT_MAX_RETRIES,
            initial_delay: DEFAULT_INITIAL_DELAY,
            max_delay: DEFAULT_MAX_DELAY,
            multiplier: DEFAULT_BACKOFF_MULTIPLIER,
        }
    }
}

impl RetryPolicy {
    /// Compute the delay for a given attempt (0-indexed).
    ///
    /// Uses saturating integer arithmetic to avoid floating-point casts.
    #[must_use]
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base = self.initial_delay;
        let factor = self.multiplier_ratio();
        let mut delay = base;
        for _ in 0..attempt {
            delay = delay
                .saturating_mul(factor.0)
                .checked_div(factor.1)
                .unwrap_or(self.max_delay);
        }
        delay.min(self.max_delay)
    }

    /// Execute `f` with retries. Returns `Ok` on first success or the last error.
    ///
    /// # Errors
    ///
    /// Returns the last error from `f` if all attempts fail.
    pub fn execute<T, E>(&self, mut f: impl FnMut() -> Result<T, E>) -> Result<T, E> {
        let mut last_err = f();
        if last_err.is_ok() || self.max_retries == 0 {
            return last_err;
        }
        for attempt in 0..self.max_retries {
            std::thread::sleep(self.delay_for_attempt(attempt));
            last_err = f();
            if last_err.is_ok() {
                return last_err;
            }
        }
        last_err
    }

    /// Approximate the multiplier as an integer ratio (numerator, denominator)
    /// so delay computation is pure integer arithmetic — no float casts.
    fn multiplier_ratio(&self) -> (u32, u32) {
        #[expect(
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation,
            reason = "multiplier is a positive backoff factor, typically 2.0; \
                      truncation to integer ratio is intentional approximation"
        )]
        let numer = (self.multiplier * f64::from(RATIO_SCALE)) as u32;
        (numer, RATIO_SCALE)
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

/// Execute an IPC call with combined circuit-breaker + retry protection.
///
/// If the circuit is open, returns [`ResilienceError::CircuitOpen`] immediately
/// (fail-fast). Otherwise, retries transient failures according to the retry
/// policy, recording successes/failures on the circuit breaker.
///
/// Absorbed from neuralSpring V113 `resilient_call()` / airSpring V0.8.8
/// `resilient_send()` pattern. Synchronous — suitable for the blocking
/// Unix socket transport used in groundSpring's JSON-RPC client.
///
/// # Errors
///
/// Returns [`ResilienceError::RetriesExhausted`] with the last error from `f`
/// if all retry attempts fail, or [`ResilienceError::CircuitOpen`] if the
/// breaker is open.
pub fn resilient_call<T, E: std::fmt::Display + std::fmt::Debug>(
    breaker: &mut CircuitBreaker,
    policy: &RetryPolicy,
    mut f: impl FnMut() -> Result<T, E>,
) -> Result<T, ResilienceError<E>> {
    if !breaker.is_allowed() {
        return Err(ResilienceError::CircuitOpen);
    }

    let total_attempts = policy.max_retries + 1;

    let mut last_error = match f() {
        Ok(val) => {
            breaker.record_success();
            return Ok(val);
        }
        Err(e) => {
            log::debug!("resilient_call attempt 0 failed: {e}");
            e
        }
    };

    for attempt in 1..total_attempts {
        std::thread::sleep(policy.delay_for_attempt(attempt - 1));
        match f() {
            Ok(val) => {
                breaker.record_success();
                return Ok(val);
            }
            Err(e) => {
                log::debug!("resilient_call attempt {attempt} failed: {e}");
                last_error = e;
            }
        }
    }

    breaker.record_failure();
    Err(ResilienceError::RetriesExhausted {
        attempts: total_attempts,
        last_error,
    })
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

    #[test]
    fn resilient_call_circuit_open_returns_typed_error() {
        let mut cb = CircuitBreaker::new(1, Duration::from_secs(60));
        cb.record_failure();
        let policy = RetryPolicy {
            max_retries: 0,
            initial_delay: Duration::from_millis(1),
            ..Default::default()
        };
        let result: Result<(), ResilienceError<String>> =
            resilient_call(&mut cb, &policy, || Err("never called".to_owned()));
        assert!(matches!(result, Err(ResilienceError::CircuitOpen)));
    }

    #[test]
    fn resilient_call_retries_exhausted_preserves_last_error() {
        let mut cb = CircuitBreaker::new(5, Duration::from_secs(60));
        let policy = RetryPolicy {
            max_retries: 2,
            initial_delay: Duration::from_millis(1),
            ..Default::default()
        };
        let mut attempt = 0u32;
        let result: Result<(), ResilienceError<String>> = resilient_call(&mut cb, &policy, || {
            attempt += 1;
            Err(format!("fail-{attempt}"))
        });
        match result {
            Err(ResilienceError::RetriesExhausted {
                attempts,
                last_error,
            }) => {
                assert_eq!(attempts, 3);
                assert_eq!(last_error, "fail-3");
            }
            other => panic!("expected RetriesExhausted, got {other:?}"),
        }
    }

    #[test]
    fn resilient_call_succeeds_after_transient() {
        let mut cb = CircuitBreaker::new(5, Duration::from_secs(60));
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay: Duration::from_millis(1),
            ..Default::default()
        };
        let mut attempt = 0u32;
        let result: Result<&str, ResilienceError<String>> =
            resilient_call(&mut cb, &policy, || {
                attempt += 1;
                if attempt < 3 {
                    Err("transient".to_owned())
                } else {
                    Ok("ok")
                }
            });
        assert_eq!(result.ok(), Some("ok"));
    }
}
