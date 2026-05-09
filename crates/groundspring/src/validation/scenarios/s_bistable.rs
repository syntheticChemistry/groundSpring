// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scenario: Bistable phenotypic switching (c-di-GMP circuit).

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};
use crate::bistable;

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "bistable-phenotypic-switch",
        track: Track::DynamicalSystems,
        tier: Tier::Rust,
        provenance_crate: "validate_bistable",
        provenance_date: "2026-05-09",
        description: "Bistable c-di-GMP phenotypic switching dynamics",
    },
    run: run_scenario,
};

fn run_scenario(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    let params = bistable::BistableParams::default();
    let state0 = [0.1_f64, 0.1, 0.0, 0.0, 0.0];
    let dt = 0.01;
    let n_steps = 10_000;
    let final_state = bistable::integrate(&state0, &params, dt, n_steps);

    v.check_bool(
        "bistable:final_state_finite",
        final_state.iter().all(|x| x.is_finite()),
        &format!("final: [{:.4}, {:.4}, ...]", final_state[0], final_state[1]),
    );

    v.check_bool(
        "bistable:final_state_non_negative",
        final_state.iter().all(|&x| x >= 0.0),
        "all concentrations ≥ 0",
    );
}
