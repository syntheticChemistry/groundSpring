// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Integration tests for FAO-56 alternate reference ET₀ methods.
//!
//! Exercises Hargreaves, Makkink, Turc, Hamon, and Thornthwaite through
//! the public `groundspring::fao56` API. Validates physical bounds,
//! edge-case behavior, and relative ordering under contrasting climates.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use groundspring::fao56::{
    daylight_hours, extraterrestrial_radiation, hamon_et0, hargreaves_et0, makkink_et0,
    solar_radiation_from_sunshine, thornthwaite_et0, thornthwaite_heat_index, turc_et0,
};

/// Typical upper bound for daily reference ET₀ (mm day⁻¹).
const ET0_DAILY_MAX: f64 = 15.0;

/// FAO-56 Example 18 — Uccle, Belgium (mid-summer).
const UCCLE: Site = Site {
    tmax_c: 21.5,
    tmin_c: 12.3,
    rh_mean_pct: 73.5,
    sunshine_hours: 9.25,
    latitude_deg_n: 50.8,
    day_of_year: 187,
};

/// Hot, dry mid-latitude summer day.
const HOT_DRY: Site = Site {
    tmax_c: 38.0,
    tmin_c: 22.0,
    rh_mean_pct: 25.0,
    sunshine_hours: 12.0,
    latitude_deg_n: 40.0,
    day_of_year: 200,
};

/// Cool, humid overcast day.
const COOL_WET: Site = Site {
    tmax_c: 12.0,
    tmin_c: 8.0,
    rh_mean_pct: 90.0,
    sunshine_hours: 2.0,
    latitude_deg_n: 50.0,
    day_of_year: 30,
};

#[derive(Clone, Copy)]
struct Site {
    tmax_c: f64,
    tmin_c: f64,
    rh_mean_pct: f64,
    sunshine_hours: f64,
    latitude_deg_n: f64,
    day_of_year: u16,
}

impl Site {
    fn tmean(&self) -> f64 {
        f64::midpoint(self.tmax_c, self.tmin_c)
    }

    fn rs_mj(&self) -> f64 {
        let ra = extraterrestrial_radiation(self.latitude_deg_n, self.day_of_year);
        let big_n = daylight_hours(self.latitude_deg_n, self.day_of_year);
        let n = self.sunshine_hours.min(big_n).max(0.0);
        solar_radiation_from_sunshine(n, big_n, ra)
    }

    fn daylight(&self) -> f64 {
        daylight_hours(self.latitude_deg_n, self.day_of_year)
    }
}

fn assert_daily_et0_bounds(label: &str, et0: f64) {
    assert!(et0.is_finite(), "{label}: ET₀ must be finite, got {et0}");
    assert!(
        (0.0..=ET0_DAILY_MAX).contains(&et0),
        "{label}: ET₀={et0:.3} mm/day outside [0, {ET0_DAILY_MAX}]"
    );
}

fn temperate_monthly_temps() -> [f64; 12] {
    [
        -2.0, 0.5, 5.0, 10.0, 15.0, 20.0, 25.0, 24.0, 18.0, 12.0, 5.0, -1.0,
    ]
}

// ── Known-input method tests ─────────────────────────────────────────

#[test]
fn fao56_hargreaves_uccle_example_18() {
    let et0 = hargreaves_et0(
        UCCLE.tmax_c,
        UCCLE.tmin_c,
        UCCLE.latitude_deg_n,
        UCCLE.day_of_year,
    );
    assert_daily_et0_bounds("Hargreaves/Uccle", et0);
    assert!(
        et0 > 2.0,
        "Hargreaves mid-summer ET₀ should exceed 2 mm/day, got {et0:.3}"
    );
}

#[test]
fn fao56_makkink_uccle_example_18() {
    let et0 = makkink_et0(UCCLE.tmean(), UCCLE.rs_mj());
    assert_daily_et0_bounds("Makkink/Uccle", et0);
    assert!(
        et0 > 1.0,
        "Makkink mid-summer ET₀ should exceed 1 mm/day, got {et0:.3}"
    );
}

#[test]
fn fao56_turc_uccle_example_18() {
    let et0 = turc_et0(UCCLE.tmean(), UCCLE.rs_mj(), UCCLE.rh_mean_pct);
    assert_daily_et0_bounds("Turc/Uccle", et0);
    assert!(
        et0 > 1.0,
        "Turc mid-summer ET₀ should exceed 1 mm/day, got {et0:.3}"
    );
}

#[test]
fn fao56_hamon_uccle_example_18() {
    let et0 = hamon_et0(UCCLE.tmean(), UCCLE.daylight());
    assert_daily_et0_bounds("Hamon/Uccle", et0);
    assert!(
        et0 > 0.0,
        "Hamon mid-summer ET₀ should be positive, got {et0:.3}"
    );
}

#[test]
fn fao56_thornthwaite_uccle_climate() {
    let monthly = temperate_monthly_temps();
    let hi = thornthwaite_heat_index(&monthly);
    assert!(hi > 0.0, "heat index should be positive, got {hi}");

    let et0_monthly = thornthwaite_et0(UCCLE.tmean(), hi, UCCLE.daylight(), 30.0);
    assert!(
        et0_monthly.is_finite() && et0_monthly > 0.0,
        "Thornthwaite monthly ET₀ should be positive, got {et0_monthly:.3}"
    );

    let et0_daily = et0_monthly / 30.0;
    assert_daily_et0_bounds("Thornthwaite/Uccle (daily equiv.)", et0_daily);
}

// ── Edge cases ───────────────────────────────────────────────────────

#[test]
fn fao56_makkink_zero_radiation() {
    let et0 = makkink_et0(20.0, 0.0);
    assert!(
        et0 <= 0.05,
        "Makkink with zero radiation should be near zero, got {et0:.4}"
    );
}

#[test]
fn fao56_turc_zero_radiation_still_positive() {
    // Turc formula includes a +50 cal/cm²/day offset independent of Rs.
    let et0 = turc_et0(20.0, 0.0, 65.0);
    let et0_full = turc_et0(20.0, 25.0, 65.0);
    assert!(
        et0 > 0.0 && et0 < et0_full,
        "Turc with zero radiation should be positive but below full-sun: \
         zero={et0:.3}, full={et0_full:.3}"
    );
}

#[test]
fn fao56_turc_negative_radiation_clamped() {
    let et0 = turc_et0(20.0, -5.0, 65.0);
    assert!(
        et0.abs() < f64::EPSILON,
        "Turc with negative radiation should clamp to 0, got {et0:.4}"
    );
}

#[test]
fn fao56_hamon_negative_daylight_clamped() {
    let et0 = hamon_et0(20.0, -1.0);
    assert!(
        et0.abs() < f64::EPSILON,
        "Hamon with negative daylight should be 0, got {et0:.4}"
    );
}

#[test]
fn fao56_hargreaves_freezing_diurnal_range() {
    let et0 = hargreaves_et0(-2.0, -8.0, 45.0, 15);
    assert!(
        et0 >= 0.0 && et0.is_finite(),
        "Hargreaves at freezing should be non-negative, got {et0:.4}"
    );
}

#[test]
fn fao56_thornthwaite_freezing_temperature() {
    let hi = thornthwaite_heat_index(&temperate_monthly_temps());
    let et0 = thornthwaite_et0(-5.0, hi, 8.0, 31.0);
    assert!(
        et0.abs() < f64::EPSILON,
        "Thornthwaite should be 0 for negative temps, got {et0:.4}"
    );
}

#[test]
fn fao56_thornthwaite_zero_heat_index() {
    let frozen = [-10.0; 12];
    let hi = thornthwaite_heat_index(&frozen);
    assert!(hi.abs() < f64::EPSILON, "frozen climate heat index = {hi}");

    let et0 = thornthwaite_et0(20.0, hi, 14.0, 30.0);
    assert!(
        et0.abs() < f64::EPSILON,
        "Thornthwaite with zero heat index should be 0, got {et0:.4}"
    );
}

#[test]
fn fao56_turc_extreme_humidity_dry_correction() {
    let base_rh = 60.0;
    let et0_humid = turc_et0(25.0, 22.0, base_rh);
    let et0_dry = turc_et0(25.0, 22.0, 10.0);
    let et0_saturated = turc_et0(25.0, 22.0, 100.0);

    assert_daily_et0_bounds("Turc/humid", et0_humid);
    assert_daily_et0_bounds("Turc/dry", et0_dry);
    assert_daily_et0_bounds("Turc/saturated", et0_saturated);

    assert!(
        et0_dry > et0_humid,
        "dry air should increase Turc ET₀: humid={et0_humid:.3}, dry={et0_dry:.3}"
    );
    assert!(
        (et0_saturated - et0_humid).abs() < 0.01,
        "RH ≥ 50% should not apply dry correction: humid={et0_humid:.3}, sat={et0_saturated:.3}"
    );
}

// ── Relative ordering (hot dry vs cool wet) ───────────────────────────

#[test]
fn fao56_hargreaves_hot_dry_exceeds_cool_wet() {
    let hot = hargreaves_et0(
        HOT_DRY.tmax_c,
        HOT_DRY.tmin_c,
        HOT_DRY.latitude_deg_n,
        HOT_DRY.day_of_year,
    );
    let cool = hargreaves_et0(
        COOL_WET.tmax_c,
        COOL_WET.tmin_c,
        COOL_WET.latitude_deg_n,
        COOL_WET.day_of_year,
    );
    assert!(
        hot > cool,
        "hot dry Hargreaves ({hot:.3}) should exceed cool wet ({cool:.3})"
    );
}

#[test]
fn fao56_makkink_hot_dry_exceeds_cool_wet() {
    let hot = makkink_et0(HOT_DRY.tmean(), HOT_DRY.rs_mj());
    let cool = makkink_et0(COOL_WET.tmean(), COOL_WET.rs_mj());
    assert!(
        hot > cool,
        "hot dry Makkink ({hot:.3}) should exceed cool wet ({cool:.3})"
    );
}

#[test]
fn fao56_turc_hot_dry_exceeds_cool_wet() {
    let hot = turc_et0(HOT_DRY.tmean(), HOT_DRY.rs_mj(), HOT_DRY.rh_mean_pct);
    let cool = turc_et0(COOL_WET.tmean(), COOL_WET.rs_mj(), COOL_WET.rh_mean_pct);
    assert!(
        hot > cool,
        "hot dry Turc ({hot:.3}) should exceed cool wet ({cool:.3})"
    );
}

#[test]
fn fao56_hamon_hot_dry_exceeds_cool_wet() {
    let hot = hamon_et0(HOT_DRY.tmean(), HOT_DRY.daylight());
    let cool = hamon_et0(COOL_WET.tmean(), COOL_WET.daylight());
    assert!(
        hot > cool,
        "hot dry Hamon ({hot:.3}) should exceed cool wet ({cool:.3})"
    );
}

#[test]
fn fao56_thornthwaite_summer_exceeds_winter_month() {
    let monthly = temperate_monthly_temps();
    let hi = thornthwaite_heat_index(&monthly);

    let summer = thornthwaite_et0(22.0, hi, 15.0, 31.0);
    let winter = thornthwaite_et0(2.0, hi, 9.0, 31.0);
    assert!(
        summer > winter,
        "Thornthwaite summer ({summer:.3}) should exceed winter ({winter:.3}) mm/month"
    );
}

// ── Cross-method consistency at Uccle ───────────────────────────────

#[test]
fn fao56_all_daily_methods_physically_plausible_at_uccle() {
    let hg = hargreaves_et0(
        UCCLE.tmax_c,
        UCCLE.tmin_c,
        UCCLE.latitude_deg_n,
        UCCLE.day_of_year,
    );
    let mk = makkink_et0(UCCLE.tmean(), UCCLE.rs_mj());
    let tu = turc_et0(UCCLE.tmean(), UCCLE.rs_mj(), UCCLE.rh_mean_pct);
    let ha = hamon_et0(UCCLE.tmean(), UCCLE.daylight());

    for (name, val) in [
        ("Hargreaves", hg),
        ("Makkink", mk),
        ("Turc", tu),
        ("Hamon", ha),
    ] {
        assert_daily_et0_bounds(name, val);
    }

    // Hamon is temperature-only and often lower; compare radiation-based trio.
    let min = hg.min(mk).min(tu);
    let max = hg.max(mk).max(tu);
    assert!(
        max / min < 5.0,
        "radiation-based methods should agree within 5× at Uccle: min={min:.3}, max={max:.3}"
    );
}

#[test]
fn fao56_makkink_increases_with_radiation_at_fixed_temperature() {
    let low = makkink_et0(20.0, 8.0);
    let high = makkink_et0(20.0, 28.0);
    assert!(
        high > low,
        "more radiation → higher Makkink ET₀: low={low:.3}, high={high:.3}"
    );
}

#[test]
fn fao56_hamon_increases_with_daylight_at_fixed_temperature() {
    let short = hamon_et0(20.0, 8.0);
    let long = hamon_et0(20.0, 16.0);
    assert!(
        long > short,
        "longer days → higher Hamon ET₀: short={short:.3}, long={long:.3}"
    );
}

#[test]
fn fao56_all_methods_deterministic() {
    let hg_a = hargreaves_et0(25.0, 15.0, 45.0, 180);
    let hg_b = hargreaves_et0(25.0, 15.0, 45.0, 180);
    assert!((hg_a - hg_b).abs() < f64::EPSILON);

    let mk_a = makkink_et0(20.0, 25.0);
    let mk_b = makkink_et0(20.0, 25.0);
    assert!((mk_a - mk_b).abs() < f64::EPSILON);

    let tu_a = turc_et0(20.0, 25.0, 65.0);
    let tu_b = turc_et0(20.0, 25.0, 65.0);
    assert!((tu_a - tu_b).abs() < f64::EPSILON);

    let ha_a = hamon_et0(20.0, 14.0);
    let ha_b = hamon_et0(20.0, 14.0);
    assert!((ha_a - ha_b).abs() < f64::EPSILON);

    let hi = thornthwaite_heat_index(&temperate_monthly_temps());
    let th_a = thornthwaite_et0(20.0, hi, 14.0, 30.0);
    let th_b = thornthwaite_et0(20.0, hi, 14.0, 30.0);
    assert!((th_a - th_b).abs() < f64::EPSILON);
}
