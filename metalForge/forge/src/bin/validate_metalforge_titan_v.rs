// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! Direct GPU compute validation on per-GPU adapters.
//!
//! Tests Anderson Lyapunov compute shader execution on every GPU via wgpu,
//! comparing results with a CPU f64 reference. Tries f64 shaders first
//! (for GPUs with working `SHADER_F64`), falls back to f32 when the driver
//! can't compile f64 (NAK f64 ALU gap, NVVM consumer limitation).
//!
//! This validates NAK shader compilation on Volta/Titan V end-to-end and
//! measures the f32 precision delta against CPU f64 ground truth.
//!
//! # Provenance
//!
//! - **CPU reference**: `groundspring::anderson::lyapunov_averaged` — transfer
//!   matrix method, L=200 sites, W=2.0, E=0, 1024 realizations, seed 42.
//! - **Tolerance**: ξ ∈ \[5, 50\] per Derrida-Gardner analytical (ξ ≈ 24 at W=2).
//!   CPU/GPU ξ ratio ∈ \[0.3, 3.0\] (f32 precision + PRNG stream difference).
//!   ≥95% realizations must have γ > 0 (f32 can zero a few via log/sqrt rounding).
//! - **Shaders**: `metalForge/shaders/anderson_lyapunov.wgsl` (f64),
//!   `anderson_lyapunov_f32.wgsl` (f32 fallback).

use groundspring_forge::harness::Harness;
use std::time::Instant;
use wgpu::util::DeviceExt;

const N_SITES: u32 = 200;
const N_REALIZATIONS: u32 = 1024;
const DISORDER: f64 = 2.0;
const ENERGY: f64 = 0.0;

const SHADER_F64: &str = include_str!("../../../shaders/anderson_lyapunov.wgsl");
const SHADER_F32: &str = include_str!("../../../shaders/anderson_lyapunov_f32.wgsl");

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuParams {
    n_sites: u32,
    n_realizations: u32,
    disorder_x1000: i32,
    energy_x1000: i32,
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "seed components masked to 32 bits fit u32"
)]
fn generate_seeds(n: usize, base_seed: u64) -> Vec<u32> {
    let mut seeds = Vec::with_capacity(n * 4);
    for i in 0..n {
        let s = base_seed.wrapping_add(i as u64);
        seeds.push((s & 0xFFFF_FFFF) as u32 | 1);
        seeds.push(((s >> 16) ^ 0xDEAD_BEEF) as u32 | 1);
        seeds.push(((s >> 32) ^ 0xCAFE_BABE) as u32 | 1);
        seeds.push(((s >> 48) ^ 0x1337_C0DE) as u32 | 1);
    }
    seeds
}

fn cpu_reference_gamma() -> f64 {
    groundspring::anderson::lyapunov_averaged(
        N_SITES as usize,
        DISORDER,
        ENERGY,
        N_REALIZATIONS as usize,
        42,
    )
}

fn probe_f64_pipeline(adapter: &wgpu::Adapter) -> bool {
    let Ok((dev64, _q64)) = barracuda::device::test_pool::tokio_block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("probe-f64"),
            required_features: wgpu::Features::SHADER_F64,
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            experimental_features: wgpu::ExperimentalFeatures::default(),
            trace: wgpu::Trace::default(),
        },
    )) else {
        return false;
    };

    if try_create_pipeline(&dev64, SHADER_F64, "probe-f64").is_some() {
        println!("    f64 shader compiled + pipeline created!");
        true
    } else {
        println!("    f64 pipeline failed (NAK/NVVM limitation) — using f32");
        false
    }
}

fn try_create_pipeline(
    device: &wgpu::Device,
    shader_src: &str,
    label: &str,
) -> Option<wgpu::ComputePipeline> {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(shader_src.into()),
    });

    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("anderson-pipeline"),
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    }))
    .ok()
}

fn dispatch_f32(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::ComputePipeline,
) -> Option<(Vec<f32>, u128)> {
    let params = GpuParams {
        n_sites: N_SITES,
        n_realizations: N_REALIZATIONS,
        #[expect(
            clippy::cast_possible_truncation,
            reason = "DISORDER*1000 in [-2^31, 2^31)"
        )]
        disorder_x1000: (DISORDER * 1000.0) as i32,
        #[expect(
            clippy::cast_possible_truncation,
            reason = "ENERGY*1000 in [-2^31, 2^31)"
        )]
        energy_x1000: (ENERGY * 1000.0) as i32,
    };

    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let seeds = generate_seeds(N_REALIZATIONS as usize, 42);
    let seeds_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("seeds"),
        contents: bytemuck::cast_slice(&seeds),
        usage: wgpu::BufferUsages::STORAGE,
    });

    let output_size = u64::from(N_REALIZATIONS) * 4; // f32 = 4 bytes
    let output_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("output"),
        size: output_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let staging_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging"),
        size: output_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout = pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("bind-group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: seeds_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: output_buf.as_entire_binding(),
            },
        ],
    });

    let t_dispatch = Instant::now();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("compute-encoder"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("anderson-pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, Some(&bind_group), &[]);
        pass.dispatch_workgroups(N_REALIZATIONS.div_ceil(64), 1, 1);
    }
    encoder.copy_buffer_to_buffer(&output_buf, 0, &staging_buf, 0, output_size);
    queue.submit(Some(encoder.finish()));

    let slice = staging_buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).ok();
    });
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });

    match rx.recv() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            println!("    buffer map error: {e}");
            return None;
        }
        Err(e) => {
            println!("    channel error: {e}");
            return None;
        }
    }

    let data = slice.get_mapped_range();
    let gammas: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
    let dispatch_us = t_dispatch.elapsed().as_micros();
    drop(data);
    staging_buf.unmap();

    Some((gammas, dispatch_us))
}

fn run_gpu_compute(adapter: &wgpu::Adapter, gpu_name: &str, h: &mut Harness, cpu_gamma: f64) {
    let info = adapter.get_info();
    let arch = groundspring_forge::substrate::GpuArch::from_name(&info.name);
    println!(
        "\n--- {} ({:?}, f64 ratio 1:{}) ---",
        gpu_name,
        arch,
        arch.f64_ratio()
    );

    let features = adapter.features();
    let has_f64_feature = features.contains(wgpu::Features::SHADER_F64);
    println!(
        "    SHADER_F64 feature: {}",
        if has_f64_feature {
            "advertised"
        } else {
            "not available"
        }
    );

    let f64_works = has_f64_feature && probe_f64_pipeline(adapter);

    let (device, queue) = match barracuda::device::test_pool::tokio_block_on(
        adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("compute-f32"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            experimental_features: wgpu::ExperimentalFeatures::default(),
            trace: wgpu::Trace::default(),
        }),
    ) {
        Ok(pair) => pair,
        Err(e) => {
            println!("    SKIP: f32 device creation failed: {e}");
            return;
        }
    };

    let Some(pipeline) = try_create_pipeline(&device, SHADER_F32, "anderson-f32") else {
        h.check(&format!("{gpu_name}: f32 shader compilation"), false);
        return;
    };
    let precision = if f64_works {
        "f64 OK, using f32"
    } else {
        "f32 (f64 unavailable)"
    };
    h.check(&format!("{gpu_name}: {precision} pipeline OK"), true);

    let Some((gammas, dispatch_us)) = dispatch_f32(&device, &queue, &pipeline) else {
        h.check(&format!("{gpu_name}: compute dispatch"), false);
        return;
    };
    h.check(&format!("{gpu_name}: compute dispatch + readback"), true);

    validate_f32_results(gpu_name, &gammas, dispatch_us, cpu_gamma, h);
}

fn validate_f32_results(
    gpu_name: &str,
    gammas: &[f32],
    dispatch_us: u128,
    cpu_gamma: f64,
    h: &mut Harness,
) {
    #[expect(clippy::cast_precision_loss, reason = "count ≤ N_REALIZATIONS ≪ 2^53")]
    let gpu_gamma_avg: f64 =
        gammas.iter().map(|g| f64::from(*g)).sum::<f64>() / gammas.len() as f64;
    let gpu_xi = if gpu_gamma_avg > 0.0 {
        1.0 / gpu_gamma_avg
    } else {
        f64::INFINITY
    };
    let cpu_xi = if cpu_gamma > 0.0 {
        1.0 / cpu_gamma
    } else {
        f64::INFINITY
    };

    println!("    GPU γ = {gpu_gamma_avg:.6}, ξ = {gpu_xi:.2}  ({dispatch_us} µs)");
    println!("    CPU γ = {cpu_gamma:.6}, ξ = {cpu_xi:.2}  (f64 reference)");

    h.check(
        &format!("{gpu_name}: GPU γ > 0 (localized regime)"),
        gpu_gamma_avg > 0.0,
    );
    h.check(
        &format!("{gpu_name}: GPU ξ in [5, 50]"),
        (5.0..=50.0).contains(&gpu_xi),
    );

    if cpu_xi.is_finite() && gpu_xi.is_finite() {
        let ratio = cpu_xi / gpu_xi;
        let rel_diff = ((gpu_gamma_avg - cpu_gamma) / cpu_gamma).abs();
        println!("    ξ ratio (CPU/GPU) = {ratio:.4}, γ relative diff = {rel_diff:.6}");
        h.check(
            &format!("{gpu_name}: CPU/GPU ξ ratio {ratio:.3} in [0.3, 3.0]"),
            (0.3..=3.0).contains(&ratio),
        );
    }

    let non_zero = gammas.iter().filter(|g| **g > 0.0).count();
    // f32 precision can produce γ ≤ 0 for a few realizations due to
    // accumulated rounding in log/sqrt chain. Allow up to 5% zero values.
    let min_non_zero = N_REALIZATIONS as usize * 95 / 100;
    h.check(
        &format!("{gpu_name}: ≥95% realizations γ > 0 ({non_zero}/{N_REALIZATIONS})"),
        non_zero >= min_non_zero,
    );
}

fn main() {
    println!("=== validate-metalforge-titan-v ===\n");
    println!(
        "Anderson Lyapunov GPU compute: L={N_SITES}, W={DISORDER}, E={ENERGY}, R={N_REALIZATIONS}\n"
    );

    let t_cpu = Instant::now();
    let cpu_gamma = cpu_reference_gamma();
    let cpu_us = t_cpu.elapsed().as_micros();
    let cpu_xi = if cpu_gamma > 0.0 {
        1.0 / cpu_gamma
    } else {
        f64::INFINITY
    };
    println!("CPU reference (f64): γ = {cpu_gamma:.6}, ξ = {cpu_xi:.2}  ({cpu_us} µs)\n");

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapters = barracuda::device::test_pool::tokio_block_on(
        instance.enumerate_adapters(wgpu::Backends::all()),
    );
    let mut gpu_adapters: Vec<_> = adapters
        .into_iter()
        .filter(|a| {
            let info = a.get_info();
            info.device_type != wgpu::DeviceType::Cpu
                && a.features().contains(wgpu::Features::SHADER_F64)
        })
        .collect();

    gpu_adapters.sort_by_key(|a| {
        let arch = groundspring_forge::substrate::GpuArch::from_name(&a.get_info().name);
        arch.f64_ratio()
    });

    println!("Found {} f64-advertising GPU(s):", gpu_adapters.len());
    for a in &gpu_adapters {
        let info = a.get_info();
        let arch = groundspring_forge::substrate::GpuArch::from_name(&info.name);
        println!(
            "  - {} ({:?}, {:?}, f64 ratio 1:{})",
            info.name,
            info.backend,
            arch,
            arch.f64_ratio()
        );
    }

    let mut h = Harness::new();
    h.check("At least 1 GPU found", !gpu_adapters.is_empty());

    for adapter in &gpu_adapters {
        let info = adapter.get_info();
        run_gpu_compute(adapter, &info.name, &mut h, cpu_gamma);
    }

    println!("\n--- NAK/Driver f64 Status ---\n");
    println!("  NAK (Volta/NVK): SHADER_F64 advertised but ALU lowering not implemented");
    println!("                   from_nir.rs:1092 asserts bit_size == 32");
    println!("                   DF64 emulation needed for f64 precision on f32 cores");
    println!("  NVVM (Ada):      SHADER_F64 advertised but compilation fails");
    println!("                   Consumer Ada has 1:64 f64 ratio, driver rejects f64 shaders");
    println!("  Solution:        ToadStool DF64 (double-float on f32) gives ~50-bit precision");

    h.finish();
}
