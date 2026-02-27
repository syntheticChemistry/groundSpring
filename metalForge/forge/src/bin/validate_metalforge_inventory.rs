// SPDX-License-Identifier: AGPL-3.0-or-later

//! Validation binary: discover all substrates and assert minimum hardware.
//!
//! Exit 0 if all checks pass, exit 1 on any failure.

use groundspring_forge::inventory::Inventory;
use groundspring_forge::substrate::{Capability, SubstrateKind};

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

fn run_hardware_checks(inv: &Inventory, h: &mut Harness) {
    let gpu_count = inv.count(SubstrateKind::Gpu);
    h.check(
        &format!("GPU count >= 1 (found {gpu_count})"),
        gpu_count >= 1,
    );

    let f64_gpus = inv
        .substrates
        .iter()
        .filter(|s| s.kind == SubstrateKind::Gpu && s.has(&Capability::F64Compute))
        .count();
    h.check(
        &format!("f64-capable GPU exists (found {f64_gpus})"),
        f64_gpus >= 1,
    );

    let shader_gpus = inv
        .substrates
        .iter()
        .filter(|s| s.kind == SubstrateKind::Gpu && s.has(&Capability::ShaderDispatch))
        .count();
    h.check(
        &format!("shader-capable GPU exists (found {shader_gpus})"),
        shader_gpus >= 1,
    );

    let npu_count = inv.count(SubstrateKind::Npu);
    h.check(&format!("NPU detected (found {npu_count})"), npu_count >= 1);

    let npu_int8 = inv
        .substrates
        .iter()
        .filter(|s| {
            s.kind == SubstrateKind::Npu && s.has(&Capability::QuantizedInference { bits: 8 })
        })
        .count();
    h.check(
        &format!("NPU supports int8 inference (found {npu_int8})"),
        npu_int8 >= 1,
    );

    let cpu_count = inv.count(SubstrateKind::Cpu);
    h.check(
        &format!("CPU discovered (found {cpu_count})"),
        cpu_count >= 1,
    );

    let cpu_f64 = inv
        .substrates
        .iter()
        .any(|s| s.kind == SubstrateKind::Cpu && s.has(&Capability::F64Compute));
    h.check("CPU has f64 compute", cpu_f64);

    let cpu_simd = inv
        .substrates
        .iter()
        .any(|s| s.kind == SubstrateKind::Cpu && s.has(&Capability::SimdVector));
    h.check("CPU has SIMD vector (AVX2)", cpu_simd);

    let total = inv.substrates.len();
    h.check(
        &format!("Total substrates >= 3 (found {total})"),
        total >= 3,
    );
}

fn run_routing_checks(inv: &Inventory, h: &mut Harness) {
    println!("\n--- Workload Routing ---\n");
    let workloads = groundspring_forge::workloads::all();
    let mut routed = 0usize;
    for w in &workloads {
        if let Some(d) = groundspring_forge::dispatch::route(w, &inv.substrates) {
            println!(
                "  {:<40} -> {} [{}] ({:?})",
                w.name, d.substrate.identity.name, d.substrate.kind, d.reason
            );
            routed += 1;
        } else {
            println!("  {:<40} -> NO ROUTE", w.name);
        }
    }
    h.check(
        &format!("All workloads routable ({routed}/{})", workloads.len()),
        routed == workloads.len(),
    );
}

fn main() {
    println!("=== validate-metalforge-inventory ===\n");
    println!("Discovering hardware substrates...\n");

    let inv = Inventory::discover();

    println!("Hardware inventory:");
    inv.print_summary();
    println!("\nTotal substrates discovered: {}", inv.substrates.len());

    let mut h = Harness::new();

    println!("\n--- Checks ---\n");
    run_hardware_checks(&inv, &mut h);
    run_routing_checks(&inv, &mut h);

    h.finish();
}
