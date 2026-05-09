// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scenario: FAO-56 Penman-Monteith reference evapotranspiration.

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "fao56-et0-penman-monteith",
        track: Track::AgriculturalScience,
        tier: Tier::Rust,
        provenance_crate: "validate_fao56",
        provenance_date: "2026-05-09",
        description: "FAO-56 Penman-Monteith ET₀ against Table 2 reference",
    },
    run: run_scenario,
};

fn run_scenario(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    let input = crate::fao56::DailyWeatherInputs {
        tmax_c: 21.5,
        tmin_c: 12.3,
        rhmax_pct: 84.0,
        rhmin_pct: 63.0,
        wind_speed_10m_km_h: 2.078 * 3.6,
        sunshine_hours: 9.25,
        latitude_deg_n: 52.0,
        altitude_m: 100.0,
        day_of_year: 135,
    };

    let et0 = crate::fao56::daily_et0(&input);

    v.check_bool(
        "fao56:et0_positive",
        et0 > 0.0,
        &format!("ET₀ = {et0:.4} mm/day"),
    );

    v.check_bool(
        "fao56:et0_plausible_range",
        et0 > 1.0 && et0 < 15.0,
        &format!("1.0 < ET₀={et0:.4} < 15.0"),
    );
}
