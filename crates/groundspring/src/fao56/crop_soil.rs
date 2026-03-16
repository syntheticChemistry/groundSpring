// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Crop coefficient and soil water balance.
//!
//! Cross-spring lineage: airSpring FAO-56 → `ToadStool` S70+ → groundSpring

/// Interpolate crop coefficient between growth stages.
///
/// FAO-56 §6.3: linear interpolation of Kc within a growth stage.
/// Delegates to `barracuda::stats::hydrology::crop_coefficient` when
/// the `barracuda` feature is enabled (airSpring → barraCuda S71+++).
#[must_use]
pub fn crop_coefficient(kc_prev: f64, kc_next: f64, day_in_stage: u32, stage_length: u32) -> f64 {
    #[cfg(feature = "barracuda")]
    return barracuda::stats::hydrology::crop_coefficient(
        kc_prev,
        kc_next,
        day_in_stage,
        stage_length,
    );
    #[cfg(not(feature = "barracuda"))]
    {
        if stage_length == 0 {
            return kc_prev;
        }
        let t = f64::from(day_in_stage) / f64::from(stage_length);
        (kc_next - kc_prev).mul_add(t.clamp(0.0, 1.0), kc_prev)
    }
}

/// Simple daily soil water balance (mm).
///
/// `θ_{t+1} = min(θ_t + P + I − ET_c, FC)`
///
/// Delegates to `barracuda::stats::hydrology::soil_water_balance` when
/// the `barracuda` feature is enabled (airSpring precision agriculture
/// → barraCuda S71+++).
#[must_use]
pub fn soil_water_balance(
    theta: f64,
    precip: f64,
    irrigation: f64,
    et_c: f64,
    field_capacity: f64,
) -> f64 {
    #[cfg(feature = "barracuda")]
    return barracuda::stats::hydrology::soil_water_balance(
        theta,
        precip,
        irrigation,
        et_c,
        field_capacity,
    );
    #[cfg(not(feature = "barracuda"))]
    (theta + precip + irrigation - et_c).clamp(0.0, field_capacity)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn crop_coefficient_endpoints() {
        let kc_start = crop_coefficient(0.3, 1.2, 0, 30);
        let kc_end = crop_coefficient(0.3, 1.2, 30, 30);
        assert!(
            (kc_start - 0.3).abs() < tol::STOCHASTIC,
            "day 0 should be kc_prev, got {kc_start}"
        );
        assert!(
            (kc_end - 1.2).abs() < tol::STOCHASTIC,
            "day=stage should be kc_next, got {kc_end}"
        );
    }

    #[test]
    fn crop_coefficient_midpoint() {
        let kc = crop_coefficient(0.4, 1.0, 15, 30);
        assert!(
            (kc - 0.7).abs() < 0.05,
            "midpoint Kc should be ~0.7, got {kc}"
        );
    }

    #[test]
    fn crop_coefficient_zero_length() {
        let kc = crop_coefficient(0.5, 1.0, 0, 0);
        assert!(
            (kc - 0.5).abs() < f64::EPSILON,
            "zero stage length returns kc_prev"
        );
    }

    #[test]
    fn soil_water_balance_basic() {
        let theta = soil_water_balance(100.0, 10.0, 5.0, 8.0, 200.0);
        assert!(
            (theta - 107.0).abs() < tol::STOCHASTIC,
            "100+10+5-8=107, got {theta}"
        );
    }

    #[test]
    fn soil_water_balance_capped_at_fc() {
        let theta = soil_water_balance(190.0, 20.0, 0.0, 2.0, 200.0);
        assert!(
            (theta - 200.0).abs() < tol::STOCHASTIC,
            "should cap at FC=200, got {theta}"
        );
    }

    #[test]
    fn soil_water_balance_floor_at_zero() {
        let theta = soil_water_balance(5.0, 0.0, 0.0, 20.0, 200.0);
        assert!(theta >= 0.0, "should not go negative, got {theta}");
    }
}
