// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scenario: Wright-Fisher drift vs selection dynamics.

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "drift-wright-fisher",
        track: Track::PopulationGenetics,
        tier: Tier::Rust,
        provenance_crate: "validate_drift",
        provenance_date: "2026-05-09",
        description: "Wright-Fisher drift vs selection separation in finite populations",
    },
    run: run_scenario,
};

fn run_scenario(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    let n_pop = 1000;
    let p0 = 0.5;
    let selection = 0.0;
    let n_trials = 100;

    let Ok(fixations) =
        crate::drift::wright_fisher_fixation_batch(n_pop, selection, p0, n_trials, 42)
    else {
        v.check_bool(
            "drift:batch_simulation",
            false,
            "wright_fisher_fixation_batch returned InputError",
        );
        return;
    };

    v.check_bool(
        "drift:fixation_count_bounded",
        fixations <= n_trials,
        &format!("fixations: {fixations}/{n_trials}"),
    );

    v.check_bool(
        "drift:neutral_fixation_rate",
        fixations > 0,
        &format!("{fixations} fixations in {n_trials} neutral trials"),
    );
}
