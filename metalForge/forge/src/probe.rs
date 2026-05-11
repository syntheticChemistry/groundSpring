// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Hardware probing — GPU via wgpu, NPU via device nodes, CPU via procfs.

use crate::substrate::{Capability, GpuArch, Identity, Properties, Substrate, SubstrateKind};
use std::fs;
use std::sync::OnceLock;

/// Environment variable to override the NPU device node.
///
/// Capability-based: primals discover hardware at runtime, never hardcode
/// paths. The default device node is platform-specific (Linux: `/dev/akida0`).
const NPU_DEVICE_ENV: &str = "GROUNDSPRING_NPU_DEVICE";

/// Platform-default device node for `BrainChip` AKD1000 NPU.
///
/// Only meaningful on Linux; other platforms must set [`NPU_DEVICE_ENV`].
#[cfg(target_os = "linux")]
const DEFAULT_NPU_DEVICE: &str = "/dev/akida0";

#[cfg(not(target_os = "linux"))]
const DEFAULT_NPU_DEVICE: &str = "";

/// Platform-specific CPU information source.
///
/// Linux: `/proc/cpuinfo`. Other platforms return minimal defaults.
#[cfg(target_os = "linux")]
const PROCFS_CPUINFO: &str = "/proc/cpuinfo";

/// Platform-specific memory information source.
///
/// Linux: `/proc/meminfo`. Other platforms return `None` for memory.
#[cfg(target_os = "linux")]
const PROCFS_MEMINFO: &str = "/proc/meminfo";

/// Cached GPU probe result.
///
/// Creating `wgpu::Instance` concurrently from multiple threads (e.g. parallel
/// tests) can trigger SIGSEGV on some drivers (toadStool S158 finding).
/// `OnceLock` ensures the probe runs exactly once; subsequent calls return
/// the cached result.
static GPU_PROBE_CACHE: OnceLock<Vec<Substrate>> = OnceLock::new();

/// Probe all GPU adapters via wgpu (cached after first call).
///
/// Each adapter becomes a substrate with capabilities derived from its
/// feature flags (`SHADER_F64` -> `F64Compute`, etc.).
///
/// Results are cached in a process-wide `OnceLock` to prevent SIGSEGV
/// from concurrent `wgpu::Instance` creation in parallel test environments.
#[must_use]
pub fn probe_gpus() -> Vec<Substrate> {
    GPU_PROBE_CACHE.get_or_init(probe_gpus_inner).clone()
}

fn probe_gpus_inner() -> Vec<Substrate> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
    let mut gpus = Vec::new();

    for (idx, adapter) in adapters.into_iter().enumerate() {
        let info = adapter.get_info();
        let features = adapter.features();
        let limits = adapter.limits();

        if info.device_type == wgpu::DeviceType::Cpu {
            continue;
        }

        let has_f64 = features.contains(wgpu::Features::SHADER_F64);
        let has_timestamps = features.contains(wgpu::Features::TIMESTAMP_QUERY);
        let arch = GpuArch::from_name(&info.name);

        let memory_bytes = if limits.max_buffer_size > 0 {
            Some(limits.max_buffer_size)
        } else {
            None
        };

        let mut capabilities = vec![Capability::F32Compute, Capability::ShaderDispatch];
        if has_f64 {
            capabilities.push(Capability::F64Compute);
            capabilities.push(Capability::ScalarReduce);
        }
        if arch.has_native_f64() && has_f64 {
            capabilities.push(Capability::NativeF64);
        }
        if has_timestamps {
            capabilities.push(Capability::TimestampQuery);
        }

        gpus.push(Substrate {
            kind: SubstrateKind::Gpu,
            identity: Identity {
                name: info.name.clone(),
                driver: Some(format!("{} ({})", info.driver, info.driver_info)),
                backend: Some(format!("{:?}", info.backend)),
                adapter_index: Some(idx),
                device_node: None,
                pci_id: None,
            },
            properties: Properties {
                memory_bytes,
                has_f64,
                has_timestamps,
                gpu_arch: Some(arch),
                ..Properties::default()
            },
            capabilities,
        });
    }

    gpus
}

/// Probe CPU via platform-specific discovery.
///
/// On Linux, reads `/proc/cpuinfo` and `/proc/meminfo`.
/// On other platforms, returns a minimal CPU substrate with f64/f32
/// capabilities (runtime discovery only, no hardcoded assumptions).
#[must_use]
pub fn probe_cpu() -> Substrate {
    #[cfg(target_os = "linux")]
    let (cpuinfo_content, meminfo_content) = (
        fs::read_to_string(PROCFS_CPUINFO).unwrap_or_else(|e| {
            tracing::warn!("failed to read {PROCFS_CPUINFO}: {e}");
            String::new()
        }),
        fs::read_to_string(PROCFS_MEMINFO).unwrap_or_else(|e| {
            tracing::warn!("failed to read {PROCFS_MEMINFO}: {e}");
            String::new()
        }),
    );
    #[cfg(not(target_os = "linux"))]
    let (cpuinfo_content, meminfo_content) = (String::new(), String::new());

    let (model, cores, threads, cache_kb, has_avx2) = parse_cpuinfo(&cpuinfo_content);
    let mem_bytes = parse_meminfo(&meminfo_content);

    let name = model.unwrap_or_else(|| String::from("Unknown CPU"));

    let mut capabilities = vec![Capability::F64Compute, Capability::F32Compute];
    if has_avx2 {
        capabilities.push(Capability::SimdVector);
    }

    Substrate {
        kind: SubstrateKind::Cpu,
        identity: Identity::named(name),
        properties: Properties {
            memory_bytes: mem_bytes,
            core_count: cores,
            thread_count: threads,
            cache_kb,
            ..Properties::default()
        },
        capabilities,
    }
}

/// Probe for NPU devices.
///
/// Discovers `BrainChip` AKD1000 via device node (default: `/dev/akida0`,
/// override with `GROUNDSPRING_NPU_DEVICE`).
#[must_use]
pub fn probe_npus() -> Vec<Substrate> {
    let mut npus = Vec::new();

    let npu_device =
        std::env::var(NPU_DEVICE_ENV).unwrap_or_else(|_| DEFAULT_NPU_DEVICE.to_owned());
    let akida_path = std::path::Path::new(&npu_device);
    if akida_path.exists() {
        npus.push(Substrate {
            kind: SubstrateKind::Npu,
            identity: Identity {
                name: String::from("BrainChip AKD1000"),
                device_node: Some(npu_device),
                ..Identity::named("BrainChip AKD1000")
            },
            properties: Properties::default(),
            capabilities: vec![
                Capability::F32Compute,
                Capability::QuantizedInference { bits: 8 },
                Capability::QuantizedInference { bits: 4 },
                Capability::BatchInference { max_batch: 8 },
                Capability::WeightMutation,
            ],
        });
    }

    npus
}

fn parse_cpuinfo(content: &str) -> (Option<String>, Option<u32>, Option<u32>, Option<u32>, bool) {
    let mut model = None;
    let mut cores = None;
    let mut siblings = None;
    let mut cache_kb = None;
    let mut has_avx2 = false;

    for line in content.lines() {
        if let Some((key, val)) = line.split_once(':') {
            let key = key.trim();
            let val = val.trim();
            match key {
                "model name" if model.is_none() => model = Some(val.to_string()),
                "cpu cores" if cores.is_none() => cores = val.parse().ok(),
                "siblings" if siblings.is_none() => siblings = val.parse().ok(),
                "cache size" if cache_kb.is_none() => {
                    cache_kb = val.trim_end_matches(" KB").parse().ok();
                }
                "flags" if !has_avx2 => {
                    has_avx2 = val.split_whitespace().any(|f| f == "avx2");
                }
                _ => {}
            }
        }
    }

    (model, cores, siblings, cache_kb, has_avx2)
}

fn parse_meminfo(content: &str) -> Option<u64> {
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            let kb_str = rest.trim().trim_end_matches(" kB").trim();
            let kb: u64 = kb_str.parse().ok()?;
            return Some(kb * 1024);
        }
    }
    None
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn cpu_always_discovered() {
        let cpu = probe_cpu();
        assert_eq!(cpu.kind, SubstrateKind::Cpu);
        assert!(cpu.has(&Capability::F64Compute));
        assert!(!cpu.identity.name.is_empty());
    }

    #[test]
    fn parse_cpuinfo_extracts_model() {
        let content = "model name\t: Intel(R) Core(TM) i9-12900K\ncpu cores\t: 8\nsiblings\t: 24\ncache size\t: 30720 KB\nflags\t\t: fpu vme de sse sse2 avx avx2\n";
        let (model, cores, threads, cache, avx2) = parse_cpuinfo(content);
        assert_eq!(model.unwrap(), "Intel(R) Core(TM) i9-12900K");
        assert_eq!(cores.unwrap(), 8);
        assert_eq!(threads.unwrap(), 24);
        assert_eq!(cache.unwrap(), 30720);
        assert!(avx2);
    }

    #[test]
    fn parse_meminfo_extracts_total() {
        let content = "MemTotal:       32749772 kB\nMemFree:        15000000 kB\n";
        let bytes = parse_meminfo(content).unwrap();
        assert_eq!(bytes, 32_749_772 * 1024);
    }
}
