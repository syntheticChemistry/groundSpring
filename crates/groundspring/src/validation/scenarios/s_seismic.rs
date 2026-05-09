// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scenario: Seismic travel-time computation.

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "seismic-travel-time",
        track: Track::Geophysics,
        tier: Tier::Rust,
        provenance_crate: "validate_seismic",
        provenance_date: "2026-05-09",
        description: "Seismic travel-time computation through layered Earth model",
    },
    run: run_scenario,
};

fn run_scenario(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    let distance_km = 50.0;
    let depth_km = 20.0;
    let vp_km_s = 6.5;

    let tt = crate::seismic::travel_time_1d(distance_km, depth_km, vp_km_s);

    v.check_bool(
        "seismic:travel_time_positive",
        tt > 0.0,
        &format!("tt = {tt:.6} s"),
    );

    v.check_bool(
        "seismic:travel_time_finite",
        tt.is_finite(),
        "travel time is finite",
    );
}
