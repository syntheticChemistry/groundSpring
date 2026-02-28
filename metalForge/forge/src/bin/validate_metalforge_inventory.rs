// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary: discover all substrates and assert minimum hardware.
//!
//! # Provenance
//!
//! Expected values are hardware capability assertions — not derived from
//! Python baselines. Checks confirm runtime discovery matches the physical
//! hardware manifest: GPU (wgpu), NPU (AKD1000 device node), CPU (procfs).
//!
//! Exit 0 if all checks pass, exit 1 on any failure.

use groundspring_forge::harness::Harness;
use groundspring_forge::inventory::Inventory;
use groundspring_forge::substrate::{AdaptiveBatch, Capability, GpuArch, SubstrateKind};

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

fn run_gpu_arch_checks(inv: &Inventory, h: &mut Harness) {
    println!("\n--- GPU Architecture ---\n");

    for s in &inv.substrates {
        if s.kind != SubstrateKind::Gpu {
            continue;
        }
        let arch = s
            .properties
            .gpu_arch
            .map_or_else(|| "Unknown".to_string(), |a| format!("{a:?}"));
        let f64_ratio = s.properties.gpu_arch.map_or(0, GpuArch::f64_ratio);
        let native = s.has(&Capability::NativeF64);
        println!(
            "  {} — arch={}, f64_ratio=1:{}, native_f64={}",
            s.identity.name, arch, f64_ratio, native
        );

        let batch = AdaptiveBatch::for_gpu(&s.properties, 64);
        println!(
            "    adaptive: max_batch={}, workgroup={}, resident={}, native={}",
            batch.max_batch_elements,
            batch.workgroup_size,
            batch.use_resident_memory,
            batch.native_f64
        );
    }

    let volta = inv.find_gpu_by_arch(GpuArch::Volta);
    h.check("Volta GPU discovered (Titan V / V100)", volta.is_some());
    if let Some(v) = volta {
        h.check(
            "Volta has NativeF64 capability",
            v.has(&Capability::NativeF64),
        );
        h.check(
            "Volta has ShaderDispatch",
            v.has(&Capability::ShaderDispatch),
        );
    }

    let best_f64 = inv.best_f64_gpu();
    if let Some(gpu) = best_f64 {
        println!(
            "\n  Best f64 GPU: {} ({:?})",
            gpu.identity.name, gpu.properties.gpu_arch
        );
        let is_volta = gpu.properties.gpu_arch == Some(GpuArch::Volta);
        h.check("Best f64 GPU prefers Volta (native 1:2 ratio)", is_volta);
    }
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
    run_gpu_arch_checks(&inv, &mut h);
    run_routing_checks(&inv, &mut h);

    h.finish();
}
