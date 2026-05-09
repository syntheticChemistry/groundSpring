// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scenario: Anderson localization — Lyapunov exponent positivity.

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "anderson-localization",
        track: Track::CondensedMatter,
        tier: Tier::Rust,
        provenance_crate: "validate_anderson",
        provenance_date: "2026-05-09",
        description: "Anderson localization — Lyapunov exponent positivity for W > 0",
    },
    run: run_scenario,
};

fn run_scenario(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    let disorder = 3.5;
    let n = 500;
    let energy = 0.0;
    let n_realizations = 50;
    let lyap = crate::anderson::lyapunov_averaged(n, disorder, energy, n_realizations, 42);

    v.check_bool(
        "anderson:lyapunov_positive",
        lyap > 0.0,
        &format!("γ(W={disorder}, N={n}) = {lyap:.6}"),
    );

    v.check_bool(
        "anderson:lyapunov_finite",
        lyap.is_finite(),
        "Lyapunov exponent is finite",
    );
}
