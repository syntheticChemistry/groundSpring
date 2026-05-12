// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! Exp 028 — NPU Anderson regime classification.
//!
//! Validates int8-quantized Anderson regime classification on the
//! `BrainChip` AKD1000 NPU. CPU analytical classification provides
//! ground truth; NPU DMA inference proves hardware portability.

#![expect(
    clippy::cast_precision_loss,
    reason = "integer counts → f64 for tolerance comparison"
)]

use groundspring::npu;
use groundspring::validate::ValidationHarness;
use groundspring_validate::{
    EPS_SAFE_DIV, f64_field, parse_benchmark, print_provenance_header, usize_field,
};
use serde_json::Value;

const BENCHMARK: &str = include_str!("../../../control/npu_anderson/benchmark_npu_anderson.json");

#[expect(
    clippy::expect_used,
    reason = "validation harness: malformed benchmark config is a fatal infrastructure error"
)]
fn json_f64_vec(v: &Value) -> Vec<f64> {
    v.as_array()
        .expect("expected JSON array")
        .iter()
        .map(|x| x.as_f64().expect("array element must be f64"))
        .collect()
}

#[expect(
    clippy::expect_used,
    reason = "validation harness: malformed benchmark config is a fatal infrastructure error"
)]
fn json_str_vec(v: &Value) -> Vec<String> {
    v.as_array()
        .expect("expected JSON array")
        .iter()
        .map(|x| x.as_str().expect("array element must be string").to_owned())
        .collect()
}

fn make_train_disorders(n: usize, w_min: f64, w_max: f64) -> Vec<f64> {
    let mut rng = groundspring::prng::Xorshift64::new(42);
    (0..n)
        .map(|_| rng.next_f64().mul_add(w_max - w_min, w_min))
        .collect()
}

const fn to_f64(n: usize) -> f64 {
    n as f64
}

fn run_cpu_checks(bench: &Value, h: &mut ValidationHarness) {
    let model = &bench["model"];
    let expected = &bench["expected_results"];
    let n_sites = usize_field(model, "n_sites");
    let energy = f64_field(model, "energy");
    let disorders = json_f64_vec(&model["disorders"]);
    let expected_regimes = json_str_vec(&expected["cpu_regimes"]);
    let l_f64 = to_f64(n_sites);

    let cpu_regimes: Vec<String> = disorders
        .iter()
        .map(|&w| npu::classify_regime_cpu(w, energy, n_sites).to_string())
        .collect();
    h.check_true(
        &format!("CPU regimes match expected: {cpu_regimes:?}"),
        cpu_regimes == expected_regimes,
    );

    let max_tol = f64_field(expected, "quantization_roundtrip_max_error");
    let mut max_err = 0.0_f64;
    for &w in &disorders {
        let features = npu::quantize_features(w, energy, l_f64);
        let w_deq = npu::dequantize_i8(features[0], 0.0, 10.0);
        let err = (w - w_deq).abs() / w.abs().max(EPS_SAFE_DIV);
        max_err = max_err.max(err);
    }
    h.check_max("Quantization roundtrip max error", max_err, max_tol);

    let n_train = usize_field(model, "n_training_disorders");
    let w_min = f64_field(model, "training_W_min");
    let w_max = f64_field(model, "training_W_max");
    let train_disorders = make_train_disorders(n_train, w_min, w_max);
    let weights = npu::train_classifier_weights(&train_disorders, n_sites);
    h.check_true("Classifier weights: 9 i8 values", weights.len() == 9);

    let accuracy_min = f64_field(expected, "cpu_accuracy_min");
    let mut correct = 0usize;
    for &w in &train_disorders {
        let features = npu::quantize_features(w, energy, l_f64);
        let true_class = npu::classify_regime_cpu(w, energy, n_sites);
        let pred = cpu_classify_with_weights(features, &weights);
        if pred == true_class {
            correct += 1;
        }
    }
    let accuracy = to_f64(correct) / to_f64(train_disorders.len());
    h.check_min(
        &format!("CPU classifier accuracy ({:.1}%)", accuracy * 100.0),
        accuracy,
        accuracy_min,
    );

    let unique: std::collections::HashSet<_> = cpu_regimes.iter().collect();
    let coverage_min = usize_field(expected, "regime_coverage_min");
    h.check_true(
        &format!("Regime coverage >= {coverage_min} ({} found)", unique.len()),
        unique.len() >= coverage_min,
    );

    let weak = npu::classify_regime_cpu(0.1, energy, n_sites);
    h.check_true(
        &format!("Extended for W=0.1 (got {weak})"),
        weak == npu::RegimeClass::Extended,
    );

    let strong = npu::classify_regime_cpu(10.0, energy, n_sites);
    h.check_true(
        &format!("Localized for W=10 (got {strong})"),
        strong == npu::RegimeClass::Localized,
    );
}

fn cpu_classify_with_weights(features: [i8; 3], weights: &[i8; 9]) -> npu::RegimeClass {
    let mut best_score = i64::MIN;
    let mut best_class = 0usize;
    for c in 0..3 {
        let mut score = 0i64;
        for j in 0..3 {
            score += i64::from(weights[c * 3 + j]) * i64::from(features[j]);
        }
        if score > best_score {
            best_score = score;
            best_class = c;
        }
    }
    npu::RegimeClass::from_index(best_class)
}

fn run_npu_checks(bench: &Value, h: &mut ValidationHarness) {
    let model = &bench["model"];
    let expected = &bench["expected_results"];
    let n_sites = usize_field(model, "n_sites");
    let energy = f64_field(model, "energy");
    let disorders = json_f64_vec(&model["disorders"]);
    let l_f64 = to_f64(n_sites);

    println!("\n--- NPU Live Hardware ---\n");

    if !npu::npu_available() {
        println!("  SKIP  No Akida NPU detected — skipping live hardware checks");
        return;
    }

    let mut handle = match npu::discover_npu() {
        Ok(h) => {
            println!(
                "  NPU: {:?}, {} NPs, {} MB SRAM",
                h.chip_version(),
                h.npu_count(),
                h.memory_mb()
            );
            h
        }
        Err(e) => {
            println!("  SKIP  NPU open failed: {e}");
            return;
        }
    };

    let n_train = usize_field(model, "n_training_disorders");
    let w_min = f64_field(model, "training_W_min");
    let w_max = f64_field(model, "training_W_max");
    let train_disorders = make_train_disorders(n_train, w_min, w_max);
    let weights = npu::train_classifier_weights(&train_disorders, n_sites);

    match npu::load_classifier_weights(&mut handle, &weights) {
        Ok(bytes) => println!("  Loaded {bytes} bytes of classifier weights to NPU SRAM"),
        Err(e) => {
            println!("  SKIP  Weight load failed: {e}");
            return;
        }
    }

    let accuracy_min = f64_field(expected, "npu_accuracy_min");
    let latency_max_us = f64_field(expected, "npu_latency_max_us");
    let mut correct = 0usize;
    let mut total_us = 0.0_f64;

    for &w in &disorders {
        let features = npu::quantize_features(w, energy, l_f64);
        let true_class = npu::classify_regime_cpu(w, energy, n_sites);

        match npu::npu_classify_regime(&mut handle, features) {
            Ok((pred_class, metrics)) => {
                let us = metrics.total_us();
                total_us += us;
                if pred_class == true_class {
                    correct += 1;
                }
                println!("    W={w:5.1} -> CPU:{true_class} NPU:{pred_class} {us:.1}µs");
            }
            Err(e) => {
                println!("    W={w:5.1} -> DMA error: {e}");
            }
        }
    }

    let npu_accuracy = to_f64(correct) / to_f64(disorders.len());
    h.check_min(
        &format!("NPU accuracy ({:.0}%)", npu_accuracy * 100.0),
        npu_accuracy,
        accuracy_min,
    );

    let mean_us = total_us / to_f64(disorders.len());
    h.check_max(
        &format!("NPU mean latency ({mean_us:.1} µs)"),
        mean_us,
        latency_max_us,
    );
}

fn main() {
    let bench = parse_benchmark(BENCHMARK);
    let mut h = ValidationHarness::from_args("Exp 028 — NPU Anderson Regime Classification");

    print_provenance_header(&bench, "NPU Anderson Regime Classification");
    println!("--- CPU Classification ---\n");
    run_cpu_checks(&bench, &mut h);
    run_npu_checks(&bench, &mut h);

    std::process::exit(h.summary());
}
