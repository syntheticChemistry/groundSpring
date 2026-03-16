// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! `N_e` × `s` drift monitoring for evolutionary populations.
//!
//! Tracks the product of effective population size and selection coefficient
//! across generations to detect when stochastic drift overwhelms selection.

use crate::cast::usize_f64;
use crate::eps;

/// `N_e` × `s` drift monitor for evolutionary populations.
///
/// Tracks the product of effective population size and selection coefficient
/// across generations. When `N_e`·`s` drops below the drift threshold for several
/// consecutive generations, the population is dominated by genetic drift
/// rather than deterministic selection — board populations stagnate, allele
/// trajectories become random walks.
///
/// # Cross-spring lineage
///
/// Concept from `bingoCube/nautilus/constraints.rs` (`DriftMonitor`).
/// The Nautilus Shell uses this to decide when to increase population
/// size or selection pressure during evolutionary reservoir computing.
/// groundSpring applies it to Wright-Fisher batch quality monitoring.
///
/// # References
///
/// - Kimura (1983) *The Neutral Theory of Molecular Evolution*
/// - hotSpring `specs/BIOMEGATE_BRAIN_ARCHITECTURE.md` §Gen 2.5
#[derive(Debug, Clone)]
pub struct DriftMonitor {
    history: Vec<(usize, f64)>,
    drift_threshold: f64,
    consecutive_drift: usize,
}

impl Default for DriftMonitor {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            drift_threshold: 1.0,
            consecutive_drift: 0,
        }
    }
}

/// Recommended action when drift is detected.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DriftAction {
    /// Selection is working normally.
    Continue,
    /// Increase selection pressure (e.g., more elite survivors).
    IncreaseSelection,
    /// Increase population size by the given factor.
    IncreasePop(f64),
}

impl DriftMonitor {
    /// Create a monitor with a custom drift threshold.
    ///
    /// The threshold is the minimum `N_e`·`s` value below which drift dominates.
    /// Default: 1.0 (Kimura's canonical boundary).
    #[must_use]
    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            drift_threshold: threshold,
            ..Self::default()
        }
    }

    /// Record a generation's fitness statistics and compute `N_e` · `s`.
    ///
    /// `s ≈ (best_fitness − mean_fitness) / mean_fitness`
    pub fn record(
        &mut self,
        generation: usize,
        pop_size: usize,
        mean_fitness: f64,
        best_fitness: f64,
    ) {
        // Guard: SAFE_DIV prevents division-by-zero when the population has near-zero
        // fitness (e.g. all-deleterious fixation). Below this threshold, the
        // selection coefficient s is numerically meaningless.
        let s = if mean_fitness > eps::SAFE_DIV {
            (best_fitness - mean_fitness) / mean_fitness
        } else {
            0.0
        };
        let ne_s = usize_f64(pop_size) * s;
        self.history.push((generation, ne_s));

        if ne_s < self.drift_threshold {
            self.consecutive_drift += 1;
        } else {
            self.consecutive_drift = 0;
        }
    }

    /// Whether the population is currently dominated by drift.
    ///
    /// Returns `true` if `N_e`·`s` has been below threshold for 3+ consecutive
    /// generations.
    #[must_use]
    pub const fn is_drifting(&self) -> bool {
        self.consecutive_drift >= 3
    }

    /// Recommended action based on drift state.
    #[must_use]
    pub const fn recommendation(&self) -> DriftAction {
        if !self.is_drifting() {
            return DriftAction::Continue;
        }
        if self.consecutive_drift >= 10 {
            DriftAction::IncreasePop(2.0)
        } else {
            DriftAction::IncreaseSelection
        }
    }

    /// Latest `N_e` · `s` ratio, or 0.0 if no history.
    #[must_use]
    pub fn latest_ne_s(&self) -> f64 {
        self.history.last().map_or(0.0, |h| h.1)
    }

    /// Full history of `(generation, N_e·s)` measurements.
    #[must_use]
    pub fn history(&self) -> &[(usize, f64)] {
        &self.history
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn drift_monitor_strong_selection() {
        let mut mon = DriftMonitor::default();
        for generation in 0..5 {
            mon.record(generation, 24, 0.5, 0.8);
        }
        assert!(!mon.is_drifting());
        assert!(mon.latest_ne_s() > 1.0);
        assert_eq!(mon.recommendation(), DriftAction::Continue);
    }

    #[test]
    fn drift_monitor_detects_drift() {
        let mut mon = DriftMonitor::default();
        for generation in 0..5 {
            mon.record(generation, 24, 0.5, 0.502);
        }
        assert!(mon.is_drifting());
        assert_eq!(mon.recommendation(), DriftAction::IncreaseSelection);
    }

    #[test]
    fn drift_monitor_prolonged_drift_recommends_pop_increase() {
        let mut mon = DriftMonitor::default();
        for generation in 0..15 {
            mon.record(generation, 24, 0.5, 0.501);
        }
        assert!(mon.is_drifting());
        assert_eq!(mon.recommendation(), DriftAction::IncreasePop(2.0));
    }

    #[test]
    fn drift_monitor_recovery() {
        let mut mon = DriftMonitor::default();
        for generation in 0..5 {
            mon.record(generation, 24, 0.5, 0.501);
        }
        assert!(mon.is_drifting());
        for generation in 5..8 {
            mon.record(generation, 24, 0.5, 0.8);
        }
        assert!(!mon.is_drifting());
        assert_eq!(mon.recommendation(), DriftAction::Continue);
    }

    #[test]
    fn drift_monitor_custom_threshold() {
        let mut mon = DriftMonitor::with_threshold(5.0);
        for generation in 0..5 {
            mon.record(generation, 24, 0.5, 0.6);
        }
        assert!(mon.is_drifting());
    }
}
