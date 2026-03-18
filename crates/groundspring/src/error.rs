// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Typed errors for input validation in groundSpring library APIs.
//!
//! Functions that receive runtime data (experiment results, sample arrays,
//! user parameters) return `Result<T, InputError>` so callers can propagate
//! or handle failures without panicking. Functions that take programmer-
//! controlled inputs (paired slices of known equal length) continue to
//! use `assert!` per Rust convention.

use thiserror::Error;

/// Error returned by the JSON-RPC dispatch layer.
///
/// Wraps library-level [`InputError`] and adds dispatch-specific variants
/// so the JSON-RPC boundary is the only place that converts to strings.
#[derive(Debug, Error)]
pub enum DispatchError {
    /// The requested JSON-RPC method is not implemented.
    #[error("method not found: {0}")]
    MethodNotFound(String),
    /// A required parameter was missing or had an invalid type.
    #[error("missing parameter: {0}")]
    MissingParam(String),
    /// A parameter value was out of its valid domain.
    #[error("invalid parameter: {0}")]
    InvalidParam(String),
    /// The underlying library function returned an input validation error.
    #[error(transparent)]
    Input(#[from] InputError),
}

/// Error returned when a function receives invalid input.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum InputError {
    /// Two parallel slices had different lengths.
    #[error("{first} (len {first_len}) and {second} (len {second_len}) must have equal length")]
    LengthMismatch {
        /// Name of the first parameter.
        first: &'static str,
        /// Length of the first parameter.
        first_len: usize,
        /// Name of the second parameter.
        second: &'static str,
        /// Length of the second parameter.
        second_len: usize,
    },
    /// A slice did not have enough elements.
    #[error("{name} requires at least {min} elements, got {got}")]
    InsufficientData {
        /// Name of the parameter.
        name: &'static str,
        /// Minimum required length.
        min: usize,
        /// Actual length.
        got: usize,
    },
    /// A scalar parameter was outside its valid range.
    #[error("{name} must be in [{lo}, {hi}], got {got}")]
    OutOfRange {
        /// Name of the parameter.
        name: &'static str,
        /// Lower bound (inclusive).
        lo: f64,
        /// Upper bound (inclusive).
        hi: f64,
        /// Actual value.
        got: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_mismatch_display() {
        let e = InputError::LengthMismatch {
            first: "sizes",
            first_len: 5,
            second: "values",
            second_len: 3,
        };
        let s = e.to_string();
        assert!(s.contains("sizes"));
        assert!(s.contains("values"));
        assert!(s.contains('5'));
        assert!(s.contains('3'));
    }

    #[test]
    fn insufficient_data_display() {
        let e = InputError::InsufficientData {
            name: "data",
            min: 2,
            got: 1,
        };
        let s = e.to_string();
        assert!(s.contains("data"));
        assert!(s.contains('2'));
    }

    #[test]
    fn out_of_range_display() {
        let e = InputError::OutOfRange {
            name: "percentile",
            lo: 0.0,
            hi: 100.0,
            got: 150.0,
        };
        let s = e.to_string();
        assert!(s.contains("percentile"));
        assert!(s.contains("150"));
    }

    #[test]
    fn input_error_derives() {
        let e1 = InputError::InsufficientData {
            name: "x",
            min: 2,
            got: 0,
        };
        let e2 = e1.clone();
        assert_eq!(e1, e2);
        assert_eq!(format!("{e1:?}"), format!("{e2:?}"));
    }

    #[test]
    fn dispatch_error_method_not_found_display() {
        let e = DispatchError::MethodNotFound("foo.bar".into());
        assert!(e.to_string().contains("foo.bar"));
    }

    #[test]
    fn dispatch_error_from_input_error() {
        let input = InputError::InsufficientData {
            name: "data",
            min: 2,
            got: 0,
        };
        let dispatch: DispatchError = input.into();
        assert!(dispatch.to_string().contains("data"));
    }

    #[test]
    fn dispatch_error_missing_param_display() {
        let e = DispatchError::MissingParam("temperature_max".into());
        assert!(e.to_string().contains("temperature_max"));
    }
}
