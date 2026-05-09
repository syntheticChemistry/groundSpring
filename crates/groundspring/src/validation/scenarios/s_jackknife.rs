// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scenario: Delete-one jackknife resampling.

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "jackknife-delete-one",
        track: Track::Resampling,
        tier: Tier::Rust,
        provenance_crate: "validate_jackknife",
        provenance_date: "2026-05-09",
        description: "Delete-one jackknife bias estimation and variance",
    },
    run: run_scenario,
};

fn run_scenario(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    match crate::jackknife::jackknife_mean_variance(&data) {
        Ok(result) => {
            v.check_bool(
                "jackknife:estimate_finite",
                result.estimate.is_finite(),
                &format!("θ̂ = {:.6}", result.estimate),
            );

            v.check_bool(
                "jackknife:variance_non_negative",
                result.variance >= 0.0,
                &format!("Var = {:.6}", result.variance),
            );

            v.check_bool(
                "jackknife:estimate_near_mean",
                (result.estimate - 5.5).abs() < crate::tol::EXACT,
                &format!("mean estimate ≈ 5.5: got {:.6}", result.estimate),
            );
        }
        Err(e) => {
            v.check_bool(
                "jackknife:estimate_finite",
                false,
                &format!("jackknife error: {e}"),
            );
        }
    }
}
