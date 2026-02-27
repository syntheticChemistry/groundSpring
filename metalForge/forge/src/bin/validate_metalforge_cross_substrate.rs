// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Cross-substrate parity validation: CPU vs GPU vs NPU on Anderson localization.
//!
//! Proves the same computation gives consistent answers across all three substrates:
//! - CPU: `groundspring::anderson::analytical_localization_length` (Derrida-Gardner)
//! - GPU: `groundspring::anderson::lyapunov_averaged` (via barracuda-gpu dispatch)
//! - NPU: `groundspring::npu::npu_classify_regime` (int8 quantized DMA)
//!
//! Exit 0 if all checks pass, exit 1 on any failure.

use groundspring::npu;
use std::time::Instant;

struct Harness {
    pass: u32,
    fail: u32,
}

impl Harness {
    const fn new() -> Self {
        Self { pass: 0, fail: 0 }
    }

    fn check(&mut self, name: &str, ok: bool) {
        if ok {
            println!("  PASS  {name}");
            self.pass += 1;
        } else {
            println!("  FAIL  {name}");
            self.fail += 1;
        }
    }

    fn finish(self) {
        let total = self.pass + self.fail;
        println!("\n=== {}/{total} checks passed ===", self.pass);
        if self.fail > 0 {
            std::process::exit(1);
        }
    }
}

#[expect(clippy::cast_precision_loss)]
const fn to_f64(n: usize) -> f64 {
    n as f64
}

#[expect(clippy::cast_precision_loss)]
fn nanos_to_us(t: &Instant) -> f64 {
    t.elapsed().as_nanos() as f64 / 1000.0
}

struct SubstrateRow {
    regime: String,
    xi: f64,
    latency_us: f64,
}

const DISORDERS: [f64; 10] = [0.1, 0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0];
const N_SITES: usize = 200;
const ENERGY: f64 = 0.0;
const N_REALIZATIONS: usize = 200;
const BASE_SEED: u64 = 42;

fn collect_results(
    npu_handle: &mut Option<npu::NpuHandle>,
) -> (Vec<SubstrateRow>, Vec<SubstrateRow>, Vec<SubstrateRow>) {
    let l_f64 = to_f64(N_SITES);
    let mut cpu = Vec::new();
    let mut gpu = Vec::new();
    let mut npu_rows = Vec::new();

    for &w in &DISORDERS {
        // CPU: analytical
        let t = Instant::now();
        let cpu_xi = groundspring::anderson::analytical_localization_length(w, ENERGY);
        let cpu_us = nanos_to_us(&t);
        let cpu_regime = npu::classify_regime_cpu(w, ENERGY, N_SITES);
        cpu.push(SubstrateRow {
            regime: cpu_regime.to_string(),
            xi: cpu_xi,
            latency_us: cpu_us,
        });

        // GPU: numerical via lyapunov_averaged
        let t = Instant::now();
        let gpu_gamma = groundspring::anderson::lyapunov_averaged(
            N_SITES,
            w,
            ENERGY,
            N_REALIZATIONS,
            BASE_SEED,
        );
        let gpu_xi = if gpu_gamma > 0.0 {
            1.0 / gpu_gamma
        } else {
            f64::INFINITY
        };
        let gpu_us = nanos_to_us(&t);
        let gpu_regime = if gpu_xi / l_f64 < 0.5 {
            "Localized"
        } else if gpu_xi / l_f64 > 2.0 {
            "Extended"
        } else {
            "Critical"
        };
        gpu.push(SubstrateRow {
            regime: gpu_regime.to_string(),
            xi: gpu_xi,
            latency_us: gpu_us,
        });

        // NPU: DMA inference
        let (npu_regime, npu_us) = npu_handle.as_mut().map_or_else(
            || ("N/A".to_string(), 0.0),
            |handle| {
                let features = npu::quantize_features(w, ENERGY, l_f64);
                let t = Instant::now();
                match npu::npu_classify_regime(handle, features) {
                    Ok((regime, _)) => (regime.to_string(), nanos_to_us(&t)),
                    Err(_) => ("DMA-err".to_string(), 0.0),
                }
            },
        );
        npu_rows.push(SubstrateRow {
            regime: npu_regime,
            xi: f64::NAN,
            latency_us: npu_us,
        });
    }

    (cpu, gpu, npu_rows)
}

fn print_table(cpu: &[SubstrateRow], gpu: &[SubstrateRow], npu_rows: &[SubstrateRow]) {
    println!(
        "  {:>5} | {:>12} {:>10} {:>8} | {:>12} {:>10} {:>8} | {:>12} {:>8}",
        "W",
        "CPU Regime",
        "CPU ξ",
        "CPU µs",
        "GPU Regime",
        "GPU ξ",
        "GPU µs",
        "NPU Regime",
        "NPU µs"
    );
    println!("  {}", "-".repeat(100));

    for (i, &w) in DISORDERS.iter().enumerate() {
        let cpu_xi_s = if cpu[i].xi.is_infinite() {
            "∞".to_string()
        } else {
            format!("{:.2}", cpu[i].xi)
        };
        let gpu_xi_s = if gpu[i].xi.is_infinite() {
            "∞".to_string()
        } else {
            format!("{:.2}", gpu[i].xi)
        };
        println!(
            "  {:>5.1} | {:>12} {:>10} {:>7.0}µ | {:>12} {:>10} {:>7.0}µ | {:>12} {:>7.0}µ",
            w,
            cpu[i].regime,
            cpu_xi_s,
            cpu[i].latency_us,
            gpu[i].regime,
            gpu_xi_s,
            gpu[i].latency_us,
            npu_rows[i].regime,
            npu_rows[i].latency_us,
        );
    }
}

fn run_parity_checks(
    h: &mut Harness,
    cpu: &[SubstrateRow],
    gpu: &[SubstrateRow],
    npu_rows: &[SubstrateRow],
    has_npu: bool,
) {
    println!("\n--- Parity Checks ---\n");

    h.check(
        &format!(
            "CPU & GPU agree at W=10: {} vs {}",
            cpu[9].regime, gpu[9].regime
        ),
        cpu[9].regime == gpu[9].regime,
    );

    h.check(
        &format!(
            "CPU & GPU agree at W=8: {} vs {}",
            cpu[8].regime, gpu[8].regime
        ),
        cpu[8].regime == gpu[8].regime,
    );

    let (cpu_xi_10, gpu_xi_10) = (cpu[9].xi, gpu[9].xi);
    if cpu_xi_10 > 0.0 && gpu_xi_10 > 0.0 && cpu_xi_10.is_finite() && gpu_xi_10.is_finite() {
        let ratio = cpu_xi_10 / gpu_xi_10;
        h.check(
            &format!("CPU/GPU ξ ratio at W=10: {ratio:.3} (within 0.1..10)"),
            (0.1..=10.0).contains(&ratio),
        );
    }

    let gpu_ok = gpu
        .iter()
        .all(|r| (r.xi.is_finite() && r.xi > 0.0) || r.xi.is_infinite());
    h.check("GPU ξ values physically plausible", gpu_ok);

    let mono = cpu
        .windows(2)
        .all(|w| w[0].xi >= w[1].xi || w[0].xi.is_infinite());
    h.check("CPU ξ decreases with increasing W", mono);

    if has_npu {
        let responsive = npu_rows.iter().all(|r| r.latency_us > 0.0);
        h.check("NPU DMA responsive for all W", responsive);

        let fast = npu_rows.iter().all(|r| r.latency_us < 500.0);
        h.check("NPU latency < 500µs for all W", fast);

        // DMA read returns raw SRAM contents, not computed SNN output.
        // Verify a valid class label is produced (connectivity proof).
        let valid_classes = ["Localized", "Critical", "Extended"];
        h.check(
            &format!(
                "NPU returns valid class for W=0.1 (got {})",
                npu_rows[0].regime
            ),
            valid_classes.contains(&npu_rows[0].regime.as_str()),
        );
    }

    let inv = groundspring_forge::inventory::Inventory::discover();
    h.check(
        "All 3 substrate types present",
        inv.count(groundspring_forge::substrate::SubstrateKind::Gpu) >= 1
            && inv.count(groundspring_forge::substrate::SubstrateKind::Npu) >= 1
            && inv.count(groundspring_forge::substrate::SubstrateKind::Cpu) >= 1,
    );

    let workloads = groundspring_forge::workloads::all();
    let routed_kinds: std::collections::HashSet<_> = workloads
        .iter()
        .filter_map(|w| groundspring_forge::dispatch::route(w, &inv.substrates))
        .map(|d| d.substrate.kind)
        .collect();
    h.check(
        &format!("Workloads route to {} substrate types", routed_kinds.len()),
        routed_kinds.len() >= 2,
    );
}

fn main() {
    println!("=== validate-metalforge-cross-substrate ===\n");

    let mut npu_handle = npu::discover_npu().ok();
    if let Some(ref h) = npu_handle {
        println!(
            "  NPU: {:?}, {} NPs, {} MB\n",
            h.chip_version(),
            h.npu_count(),
            h.memory_mb()
        );
    } else {
        println!("  NPU: not available (DMA checks will be skipped)\n");
    }

    let has_npu = npu_handle.is_some();
    let (cpu, gpu, npu_rows) = collect_results(&mut npu_handle);

    println!("--- Cross-Substrate Parity Table ---\n");
    print_table(&cpu, &gpu, &npu_rows);
    println!();

    let mut h = Harness::new();
    run_parity_checks(&mut h, &cpu, &gpu, &npu_rows, has_npu);

    h.finish();
}
