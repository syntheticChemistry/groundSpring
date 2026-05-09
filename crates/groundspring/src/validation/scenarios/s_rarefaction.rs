// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scenario: Rarefaction curve monotonicity and boundary conditions.

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "rarefaction-curves",
        track: Track::Ecology,
        tier: Tier::Rust,
        provenance_crate: "validate_rarefaction",
        provenance_date: "2026-05-09",
        description: "Rarefaction curve monotonicity and boundary conditions",
    },
    run: run_scenario,
};

fn run_scenario(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    let counts = vec![100u64, 50, 30, 20, 10, 5, 3, 2, 1];
    let n_total: u64 = counts.iter().sum();
    let depths: Vec<u64> = vec![10, 50, 100, n_total];

    let curve = crate::rarefaction::analytical_rarefaction(&counts, &depths);

    v.check_bool(
        "rarefaction:curve_length",
        curve.len() == depths.len(),
        &format!("expected {}, got {}", depths.len(), curve.len()),
    );

    let monotonic = curve.windows(2).all(|w| w[1] >= w[0]);
    v.check_bool(
        "rarefaction:monotonic",
        monotonic,
        "E[S(n)] is non-decreasing",
    );

    if let Some(&last) = curve.last() {
        let s_obs = counts.len() as f64;
        v.check_bool(
            "rarefaction:e_s_at_n_near_s_obs",
            (last - s_obs).abs() < crate::tol::EXACT,
            &format!("E[S(N)] = {last:.4}, S_obs = {s_obs}"),
        );
    }
}
