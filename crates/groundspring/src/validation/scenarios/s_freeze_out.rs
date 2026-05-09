// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scenario: Freeze-out curve chi-squared fitting.

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "freeze-out-chi2",
        track: Track::StatisticalFitting,
        tier: Tier::Rust,
        provenance_crate: "validate_freeze_out",
        provenance_date: "2026-05-09",
        description: "Freeze-out curve chi-squared grid fitting for QCD hadronization",
    },
    run: run_scenario,
};

fn run_scenario(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    let observed = [1.0, 2.0, 3.0];
    let predicted = [1.1, 1.9, 3.2];
    let sigma = 0.1;

    match crate::freeze_out::chi_squared(&observed, &predicted, sigma) {
        Ok(chi2_val) => {
            v.check_bool(
                "freeze_out:chi2_non_negative",
                chi2_val >= 0.0,
                &format!("χ² = {chi2_val:.6}"),
            );
        }
        Err(e) => {
            v.check_bool(
                "freeze_out:chi2_non_negative",
                false,
                &format!("chi_squared error: {e}"),
            );
        }
    }
}
