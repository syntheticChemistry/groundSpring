// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Hargreaves reference ET₀ (temperature-only).
//!
//! Cross-spring lineage: airSpring V035 → `ToadStool` S70+ → groundSpring

use super::constants::{HARGREAVES_COEFF, HARGREAVES_TEMP_OFFSET};
use super::equations::extraterrestrial_radiation;

/// Hargreaves reference ET₀ from temperature only (mm day⁻¹).
///
/// `ET₀ = 0.0023 · (T_mean + 17.8) · (T_max − T_min)^0.5 · Ra`
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::hydrology::hargreaves_et0` (absorbed from
/// airSpring V035 via barraCuda S71+++).
#[must_use]
pub fn hargreaves_et0(tmax_c: f64, tmin_c: f64, latitude_deg_n: f64, day_of_year: u16) -> f64 {
    let ra = extraterrestrial_radiation(latitude_deg_n, day_of_year);
    #[cfg(feature = "barracuda")]
    {
        if let Some(et0) = barracuda::stats::hydrology::hargreaves_et0(ra, tmax_c, tmin_c) {
            return et0;
        }
    }
    hargreaves_et0_cpu(ra, tmax_c, tmin_c)
}

fn hargreaves_et0_cpu(ra: f64, tmax_c: f64, tmin_c: f64) -> f64 {
    let tmean = f64::midpoint(tmax_c, tmin_c);
    let td = (tmax_c - tmin_c).max(0.0);
    HARGREAVES_COEFF * (tmean + HARGREAVES_TEMP_OFFSET) * td.sqrt() * ra
}

/// Compute Hargreaves ET₀ for a batch of days.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::hydrology::hargreaves_et0_batch`.
/// When `barracuda-gpu` is enabled and a GPU is available, dispatches
/// via `BatchedElementwiseF64::execute` with `Op::HargreavesEt0`
/// (airSpring V035 → barraCuda S71+++).
#[must_use]
pub fn hargreaves_et0_batch(
    tmax_c: &[f64],
    tmin_c: &[f64],
    latitude_deg_n: f64,
    day_of_year: u16,
) -> Vec<f64> {
    let n = tmax_c.len().min(tmin_c.len());
    let ra = extraterrestrial_radiation(latitude_deg_n, day_of_year);
    #[cfg(any(feature = "barracuda", feature = "barracuda-gpu"))]
    {
        let ra_vec: Vec<f64> = vec![ra; n];
        #[cfg(feature = "barracuda-gpu")]
        {
            if let Some(results) = hargreaves_et0_batch_gpu(&ra_vec, tmax_c, tmin_c) {
                return results;
            }
        }
        if let Some(results) =
            barracuda::stats::hydrology::hargreaves_et0_batch(&ra_vec, &tmax_c[..n], &tmin_c[..n])
        {
            return results;
        }
    }
    (0..n)
        .map(|i| hargreaves_et0_cpu(ra, tmax_c[i], tmin_c[i]))
        .collect()
}

#[cfg(feature = "barracuda-gpu")]
fn hargreaves_et0_batch_gpu(ra: &[f64], tmax: &[f64], tmin: &[f64]) -> Option<Vec<f64>> {
    use barracuda::ops::batched_elementwise_f64::{BatchedElementwiseF64, Op};

    let device = crate::gpu::get_device()?;
    if let Ok(gpu) = barracuda::stats::hydrology::HargreavesBatchGpu::new(device.clone())
        && let Ok(result) = gpu.dispatch(ra, tmax, tmin)
    {
        return Some(result);
    }
    let gpu = BatchedElementwiseF64::new(device).ok()?;
    let n = ra.len();
    let mut data = Vec::with_capacity(n * 3);
    for i in 0..n {
        data.push(ra[i]);
        data.push(tmax[i]);
        data.push(tmin[i]);
    }
    gpu.execute(&data, n, Op::HargreavesEt0).ok()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn hargreaves_positive() {
        let et0 = hargreaves_et0(21.5, 12.3, 50.8, 187);
        assert!(et0 > 0.0, "Hargreaves ET₀ should be positive, got {et0}");
    }

    #[test]
    fn hargreaves_summer_gt_winter() {
        let summer = hargreaves_et0(30.0, 18.0, 45.0, 180);
        let winter = hargreaves_et0(5.0, -2.0, 45.0, 15);
        assert!(
            summer > winter,
            "summer ET₀ ({summer:.2}) should exceed winter ({winter:.2})"
        );
    }

    #[test]
    fn hargreaves_batch_matches_scalar() {
        let tmax = [25.0, 30.0, 20.0];
        let tmin = [15.0, 18.0, 10.0];
        let batch = hargreaves_et0_batch(&tmax, &tmin, 45.0, 180);
        for (i, &val) in batch.iter().enumerate() {
            let scalar = hargreaves_et0(tmax[i], tmin[i], 45.0, 180);
            assert!(
                (val - scalar).abs() < tol::ANALYTICAL,
                "batch[{i}]={val} != scalar={scalar}"
            );
        }
    }

    #[test]
    fn hargreaves_deterministic() {
        let a = hargreaves_et0(25.0, 15.0, 45.0, 180);
        let b = hargreaves_et0(25.0, 15.0, 45.0, 180);
        assert!((a - b).abs() < f64::EPSILON);
    }
}
