// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiment 018: Band Edge Structure.
//!
//! Where do propagating waves transition to evanescent in periodic
//! structures?
//!
//! Reference: Filonov & Kachkovskiy (2018) Acta Math 221:59-80

use groundspring::band_structure::{
    count_bands, eigenvalue_band_fraction, find_band_edges, periodic_hamiltonian,
    transfer_matrix_half_trace,
};
use groundspring::transport::tridiag_eigh;
use groundspring::validate::ValidationHarness;
use groundspring_validate::{array_field, f64_field, print_provenance_header, usize_field};
use serde_json::Value;

const BENCHMARK: &str = include_str!("../../../control/band_edge/benchmark_band_edge.json");

struct BandCtx<'a> {
    t_hop: f64,
    pot_2: &'a [f64],
    pot_3: &'a [f64],
    e_lo: f64,
    e_hi: f64,
    n_scan: usize,
}

fn validate_free_and_periodic(
    h: &mut ValidationHarness,
    ctx: &BandCtx<'_>,
    pred: &Value,
    exp: &Value,
) {
    println!("\n--- Part 1: Free Lattice ---");
    let free_edges = find_band_edges(&[0.0], ctx.t_hop, ctx.e_lo, ctx.e_hi, ctx.n_scan);
    h.check_true("Free lattice: 2 band edges", free_edges.len() == 2);
    let lo = pred["free_band_edges"][0].as_f64().expect("lo");
    let hi = pred["free_band_edges"][1].as_f64().expect("hi");
    let edge_tol = f64_field(exp, "edge_tolerance");
    if free_edges.len() == 2 {
        h.check_range(
            "Lower edge ≈ -2t",
            free_edges[0],
            lo - edge_tol,
            lo + edge_tol,
        );
        h.check_range(
            "Upper edge ≈ +2t",
            free_edges[1],
            hi - edge_tol,
            hi + edge_tol,
        );
    }

    println!("\n--- Part 2: Period-2 Gap ---");
    let p2_edges = find_band_edges(ctx.pot_2, ctx.t_hop, ctx.e_lo, ctx.e_hi, ctx.n_scan);
    let p2_bands = count_bands(ctx.pot_2, ctx.t_hop, ctx.e_lo, ctx.e_hi, ctx.n_scan);
    h.check_true("Period-2: 4 edges", p2_edges.len() == 4);
    h.check_true("Period-2: 2 bands", p2_bands == 2);
    if p2_edges.len() == 4 {
        let gap_width = p2_edges[2] - p2_edges[1];
        let expected_gap = f64_field(pred, "period_2_gap_width");
        let tol = f64_field(exp, "gap_width_tolerance");
        println!("  Gap width: {gap_width:.3} (expected {expected_gap})");
        h.check_range(
            "Gap width ≈ |V1-V2|",
            gap_width,
            expected_gap - tol,
            expected_gap + tol,
        );
    }

    println!("\n--- Part 3: Period-3 ---");
    let p3_bands = count_bands(ctx.pot_3, ctx.t_hop, ctx.e_lo, ctx.e_hi, ctx.n_scan);
    let expected_n = usize_field(pred, "n_bands_period_3");
    println!("  Bands: {p3_bands} (expected {expected_n})");
    h.check_true("Period-3: correct band count", p3_bands == expected_n);
}

fn validate_proportionality_and_finite(
    h: &mut ValidationHarness,
    ctx: &BandCtx<'_>,
    exp: &Value,
    n_periods: usize,
    dvs: &[f64],
) {
    println!("\n--- Part 4: Gap Width vs Contrast ---");
    let mut gap_widths = Vec::new();
    for &dv in dvs {
        let pot = [dv / 2.0, -dv / 2.0];
        let edges = find_band_edges(&pot, ctx.t_hop, ctx.e_lo, ctx.e_hi, ctx.n_scan);
        let gw = if edges.len() >= 4 {
            edges[2] - edges[1]
        } else {
            0.0
        };
        gap_widths.push(gw);
        println!("  ΔV={dv:.1}: gap = {gw:.3}");
    }
    let gap_slack = f64_field(exp, "gap_monotonicity_slack");
    let monotone = gap_widths.windows(2).all(|w| w[0] <= w[1] + gap_slack);
    h.check_true("Gap width increases with ΔV", monotone);

    println!("\n--- Part 5: Finite System ---");
    let (diag, offdiag) = periodic_hamiltonian(ctx.pot_2, ctx.t_hop, n_periods);
    let (eigenvalues, _) = tridiag_eigh(&diag, &offdiag).expect("eigendecomposition");
    let band_margin = f64_field(exp, "eigenvalue_band_margin");
    let frac = eigenvalue_band_fraction(&eigenvalues, ctx.pot_2, ctx.t_hop, band_margin);
    let frac_min = f64_field(exp, "eigenvalue_band_fraction_min");
    println!(
        "  {} eigenvalues, {:.1}% in bands",
        eigenvalues.len(),
        frac * 100.0
    );
    h.check_true(
        &format!("≥{:.0}% eigenvalues within bands", frac_min * 100.0),
        frac >= frac_min,
    );
}

fn run() -> i32 {
    let bench: Value = serde_json::from_str(BENCHMARK).expect("valid benchmark JSON");
    let mut h = ValidationHarness::stdout("Rust Validation: Band Edge Structure");

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: Band Edge (Exp 018)");
    println!("{}", "=".repeat(72));
    print_provenance_header(&bench, "Band Edge Structure");

    let model = &bench["model"];
    let pred = &bench["analytical_predictions"];
    let exp = &bench["expected_results"];

    let t_hop = f64_field(model, "hopping");
    let pot_2: Vec<f64> = array_field(model, "period_2_potential")
        .iter()
        .map(|v| v.as_f64().expect("f64"))
        .collect();
    let pot_3: Vec<f64> = array_field(model, "period_3_potential")
        .iter()
        .map(|v| v.as_f64().expect("f64"))
        .collect();
    let n_scan = usize_field(model, "n_energy_scan");
    let e_range = array_field(model, "energy_range");
    let e_lo = e_range[0].as_f64().expect("e_lo");
    let e_hi = e_range[1].as_f64().expect("e_hi");
    let n_periods = usize_field(model, "n_periods_finite");

    let dvs: Vec<f64> = array_field(model, "period_2_gap_widths_to_test")
        .iter()
        .map(|v| v.as_f64().expect("f64"))
        .collect();

    let ctx = BandCtx {
        t_hop,
        pot_2: &pot_2,
        pot_3: &pot_3,
        e_lo,
        e_hi,
        n_scan,
    };

    validate_free_and_periodic(&mut h, &ctx, pred, exp);
    validate_proportionality_and_finite(&mut h, &ctx, exp, n_periods, &dvs);

    println!("\n--- Part 6: Determinism ---");
    let t1 = transfer_matrix_half_trace(0.5, &pot_2, t_hop);
    let t2 = transfer_matrix_half_trace(0.5, &pot_2, t_hop);
    h.check_true(
        "Transfer matrix deterministic",
        (t1 - t2).abs() < f64::EPSILON,
    );

    h.summary()
}

fn main() {
    std::process::exit(run());
}

#[cfg(test)]
mod tests {
    #[test]
    fn validation_passes() {
        assert_eq!(super::run(), 0);
    }
}
