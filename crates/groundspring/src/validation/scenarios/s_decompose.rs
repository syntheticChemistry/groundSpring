// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scenario: Bias-variance decomposition determinism and Pythagorean identity.

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};
use crate::decompose::decompose_error;

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "decompose-bias-variance",
        track: Track::NoiseDecomposition,
        tier: Tier::Rust,
        provenance_crate: "validate_decompose",
        provenance_date: "2026-05-09",
        description: "Bias-variance decomposition determinism and Pythagorean identity",
    },
    run: run_scenario,
};

fn run_scenario(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    let d = decompose_error(0.5, 1.0);

    v.check_bool(
        "decompose:bias_fraction_valid",
        d.bias_fraction >= 0.0 && d.bias_fraction <= 1.0,
        &format!("bias_fraction = {:.6}", d.bias_fraction),
    );

    v.check_bool(
        "decompose:pythagorean",
        (d.bias_fraction + d.noise_fraction - 1.0).abs() < 1e-15,
        "bias_frac + noise_frac = 1.0",
    );

    v.check_bool(
        "decompose:random_std_positive",
        d.random_std > 0.0,
        &format!("random_std = {:.6}", d.random_std),
    );
}
