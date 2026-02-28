// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Substrate abstraction — runtime-discovered compute devices.
//!
//! A substrate is a compute device found on this machine. GPUs come from
//! wgpu adapter enumeration. NPUs come from local device node probing.
//! CPU comes from procfs. Capabilities are what matters for dispatch —
//! code asks "can you do f64?" not "are you an RTX 4070?".

use std::fmt;

/// A compute substrate discovered at runtime.
#[derive(Debug, Clone)]
pub struct Substrate {
    /// What kind of device this is.
    pub kind: SubstrateKind,
    /// How we found it and what to call it.
    pub identity: Identity,
    /// Measured hardware properties.
    pub properties: Properties,
    /// What this device can do.
    pub capabilities: Vec<Capability>,
}

/// How we found this device and what to call it.
#[derive(Debug, Clone)]
pub struct Identity {
    /// Human-readable device name.
    pub name: String,
    /// GPU driver string from wgpu.
    pub driver: Option<String>,
    /// wgpu backend (e.g. "Vulkan").
    pub backend: Option<String>,
    /// wgpu adapter index for GPU selection.
    pub adapter_index: Option<usize>,
    /// Device node (e.g. "/dev/akida0").
    pub device_node: Option<String>,
    /// PCI vendor:device if available.
    pub pci_id: Option<String>,
}

/// GPU architecture family, detected from adapter name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpuArch {
    /// NVIDIA Volta (`GV100`) — native f64 at 1:2 ratio, HBM2.
    Volta,
    /// NVIDIA Turing (`TU1xx`) — f64 at 1:32 ratio.
    Turing,
    /// NVIDIA Ampere (`GA1xx`) — f64 at 1:64 ratio (consumer).
    Ampere,
    /// NVIDIA Ada Lovelace (`AD1xx`) — f64 at 1:64 ratio, good NAK.
    Ada,
    /// AMD RDNA or other.
    Other,
}

impl GpuArch {
    /// f64:f32 throughput ratio for this architecture.
    ///
    /// Volta is special: `GV100` has 1:2 native f64 (SM 7.0 full-rate
    /// double precision). Consumer Ampere/Ada only get 1:64.
    /// `ToadStool`'s DF64 (double-float on FP32 cores) narrows this gap
    /// to ~9.9× on FP32 cores, but native f64 is always preferred.
    #[must_use]
    pub const fn f64_ratio(self) -> u32 {
        match self {
            Self::Volta => 2,
            Self::Turing => 32,
            Self::Ampere | Self::Ada | Self::Other => 64,
        }
    }

    /// Maximum recommended workgroup size for f64 workloads.
    ///
    /// Volta's SM architecture prefers 32-wide warps with 2 f64 units;
    /// Ada/Ampere shaders should use larger workgroups to hide f64 latency.
    /// NAK on Volta may have tighter limits than proprietary driver.
    #[must_use]
    pub const fn recommended_f64_workgroup(self) -> u32 {
        match self {
            Self::Turing => 128,
            Self::Ampere | Self::Ada => 256,
            Self::Volta | Self::Other => 64,
        }
    }

    /// Whether this architecture has native f64 at production throughput.
    ///
    /// Only Volta (`GV100`/`GV100GL`) has ≥ 1:4 f64:f32 ratio.
    /// All others need DF64 or accept severe throughput penalty.
    #[must_use]
    pub const fn has_native_f64(self) -> bool {
        matches!(self, Self::Volta)
    }

    /// Conservative VRAM default when wgpu reports API limits instead of
    /// actual device memory (common on NVK/NAK and some Mesa drivers).
    #[must_use]
    pub const fn default_vram_bytes(self) -> u64 {
        match self {
            Self::Volta | Self::Ada => 12 * 1024 * 1024 * 1024,
            Self::Turing => 8 * 1024 * 1024 * 1024,
            Self::Ampere => 10 * 1024 * 1024 * 1024,
            Self::Other => 4 * 1024 * 1024 * 1024,
        }
    }

    /// Detect architecture from GPU adapter name string.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        let upper = name.to_uppercase();
        if upper.contains("TITAN V") || upper.contains("GV100") || upper.contains("V100") {
            Self::Volta
        } else if upper.contains("RTX 20") || upper.contains("GTX 16") || upper.contains("TU1") {
            Self::Turing
        } else if upper.contains("RTX 30") || upper.contains("GA1") || upper.contains("A100") {
            Self::Ampere
        } else if upper.contains("RTX 40") || upper.contains("AD1") || upper.contains("L4") {
            Self::Ada
        } else {
            Self::Other
        }
    }
}

/// Adaptive batch configuration for GPU memory management.
///
/// Older architectures (Volta/NAK) may lack hardware memory batching
/// that newer GPUs handle automatically. This computes software-side
/// batch sizes that keep the working set within GPU memory, enabling
/// unidirectional streaming where data stays on-device between batches.
#[derive(Debug, Clone)]
pub struct AdaptiveBatch {
    /// Maximum elements per dispatch batch.
    pub max_batch_elements: usize,
    /// Workgroup size for this GPU.
    pub workgroup_size: u32,
    /// Whether to use resident memory (keep buffers alive between dispatches).
    pub use_resident_memory: bool,
    /// Whether native f64 is available (vs DF64 emulation).
    pub native_f64: bool,
}

impl AdaptiveBatch {
    /// Compute adaptive batch parameters for a given GPU and workload.
    ///
    /// `element_bytes` is the memory footprint per work item (input + output).
    /// The batch size is chosen to use at most 75% of GPU memory, leaving
    /// headroom for shader scratch space and driver allocations.
    ///
    /// When wgpu reports unrealistically large `max_buffer_size` (common
    /// on NVK/NAK), falls back to architecture-specific VRAM defaults.
    #[must_use]
    pub fn for_gpu(props: &Properties, element_bytes: usize) -> Self {
        let arch = props.gpu_arch.unwrap_or(GpuArch::Other);
        let reported = props.memory_bytes.unwrap_or(0);

        let vram = if reported > 0 && reported <= 64 * 1024 * 1024 * 1024 {
            reported
        } else {
            arch.default_vram_bytes()
        };

        #[expect(
            clippy::cast_possible_truncation,
            reason = "VRAM fits in usize on 64-bit"
        )]
        let usable = (vram / 4 * 3) as usize;
        let max_elements = if element_bytes > 0 {
            usable / element_bytes
        } else {
            1_000_000
        };

        Self {
            max_batch_elements: max_elements,
            workgroup_size: arch.recommended_f64_workgroup(),
            use_resident_memory: arch.has_native_f64(),
            native_f64: arch.has_native_f64(),
        }
    }
}

/// Measured properties of a substrate.
#[derive(Debug, Clone, Default)]
pub struct Properties {
    /// Total memory in bytes (RAM for CPU, VRAM for GPU).
    pub memory_bytes: Option<u64>,
    /// Physical core count (CPU).
    pub core_count: Option<u32>,
    /// Logical thread count (CPU).
    pub thread_count: Option<u32>,
    /// Cache size in KB (CPU).
    pub cache_kb: Option<u32>,
    /// Supports IEEE 754 f64 in shaders (GPU).
    pub has_f64: bool,
    /// Supports timestamp queries (GPU).
    pub has_timestamps: bool,
    /// Detected GPU architecture (from adapter name).
    pub gpu_arch: Option<GpuArch>,
}

/// The kind of compute device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubstrateKind {
    /// GPU via wgpu/Vulkan.
    Gpu,
    /// Neural Processing Unit (e.g. `BrainChip` AKD1000).
    Npu,
    /// Host CPU.
    Cpu,
}

/// A capability discovered at runtime on a substrate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    /// IEEE 754 f64 compute.
    F64Compute,
    /// f32 compute.
    F32Compute,
    /// Integer quantized inference at a given bit width.
    QuantizedInference {
        /// Quantization bit width (e.g. 4, 8).
        bits: u8,
    },
    /// Batch inference with amortized dispatch.
    BatchInference {
        /// Maximum batch size supported.
        max_batch: usize,
    },
    /// Weight mutation without full reprogramming.
    WeightMutation,
    /// Scalar reduction (GPU fused map-reduce).
    ScalarReduce,
    /// WGSL shader dispatch via wgpu.
    ShaderDispatch,
    /// AVX2/SSE SIMD on CPU.
    SimdVector,
    /// GPU timestamp query support.
    TimestampQuery,
    /// Native f64 at production throughput (Volta 1:2 ratio).
    /// Without this, f64 workloads use DF64 emulation (~9.9× on FP32 cores).
    NativeF64,
}

impl fmt::Display for SubstrateKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gpu => write!(f, "GPU"),
            Self::Npu => write!(f, "NPU"),
            Self::Cpu => write!(f, "CPU"),
        }
    }
}

impl fmt::Display for Substrate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}]", self.identity.name, self.kind)?;
        if let Some(ref driver) = self.identity.driver {
            write!(f, " {driver}")?;
        }
        if let Some(mem) = self.properties.memory_bytes {
            let mb = mem / (1024 * 1024);
            write!(f, " {mb}MB")?;
        }
        Ok(())
    }
}

impl Substrate {
    /// Check if this substrate has a specific capability.
    #[must_use]
    pub fn has(&self, cap: &Capability) -> bool {
        self.capabilities.contains(cap)
    }

    /// Return capabilities as a summary string.
    #[must_use]
    pub fn capability_summary(&self) -> String {
        let labels: Vec<&str> = self.capabilities.iter().map(Capability::label).collect();
        labels.join(", ")
    }
}

impl Capability {
    /// Human-readable label for display.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::F64Compute => "f64",
            Self::F32Compute => "f32",
            Self::QuantizedInference { .. } => "quant",
            Self::BatchInference { .. } => "batch",
            Self::WeightMutation => "weight-mut",
            Self::ScalarReduce => "reduce",
            Self::ShaderDispatch => "shader",
            Self::SimdVector => "simd",
            Self::TimestampQuery => "timestamps",
            Self::NativeF64 => "native-f64",
        }
    }
}

impl Identity {
    /// Minimal identity with just a name.
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            driver: None,
            backend: None,
            adapter_index: None,
            device_node: None,
            pci_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_gpu() -> Substrate {
        Substrate {
            kind: SubstrateKind::Gpu,
            identity: Identity {
                name: String::from("Test GPU"),
                adapter_index: Some(0),
                ..Identity::named("Test GPU")
            },
            properties: Properties {
                has_f64: true,
                ..Properties::default()
            },
            capabilities: vec![Capability::F64Compute, Capability::ShaderDispatch],
        }
    }

    #[test]
    fn has_capability() {
        let gpu = test_gpu();
        assert!(gpu.has(&Capability::F64Compute));
        assert!(gpu.has(&Capability::ShaderDispatch));
        assert!(!gpu.has(&Capability::QuantizedInference { bits: 8 }));
    }

    #[test]
    fn display_shows_kind_and_name() {
        let gpu = test_gpu();
        let s = gpu.to_string();
        assert!(s.contains("Test GPU"));
        assert!(s.contains("GPU"));
    }

    #[test]
    fn substrate_kind_display() {
        assert_eq!(SubstrateKind::Gpu.to_string(), "GPU");
        assert_eq!(SubstrateKind::Npu.to_string(), "NPU");
        assert_eq!(SubstrateKind::Cpu.to_string(), "CPU");
    }

    #[test]
    fn capability_labels() {
        assert_eq!(Capability::F64Compute.label(), "f64");
        assert_eq!(Capability::F32Compute.label(), "f32");
        assert_eq!(Capability::QuantizedInference { bits: 8 }.label(), "quant");
        assert_eq!(Capability::BatchInference { max_batch: 8 }.label(), "batch");
        assert_eq!(Capability::WeightMutation.label(), "weight-mut");
        assert_eq!(Capability::ScalarReduce.label(), "reduce");
        assert_eq!(Capability::ShaderDispatch.label(), "shader");
        assert_eq!(Capability::SimdVector.label(), "simd");
        assert_eq!(Capability::TimestampQuery.label(), "timestamps");
        assert_eq!(Capability::NativeF64.label(), "native-f64");
    }

    #[test]
    fn gpu_arch_from_titan_v() {
        assert_eq!(GpuArch::from_name("NVIDIA TITAN V"), GpuArch::Volta);
        assert_eq!(GpuArch::from_name("Tesla V100-SXM2"), GpuArch::Volta);
        assert_eq!(GpuArch::from_name("GV100GL"), GpuArch::Volta);
    }

    #[test]
    fn gpu_arch_from_rtx_4070() {
        assert_eq!(GpuArch::from_name("NVIDIA GeForce RTX 4070"), GpuArch::Ada);
        assert_eq!(GpuArch::from_name("NVIDIA L4"), GpuArch::Ada);
    }

    #[test]
    fn volta_has_native_f64() {
        assert!(GpuArch::Volta.has_native_f64());
        assert!(!GpuArch::Ada.has_native_f64());
        assert!(!GpuArch::Ampere.has_native_f64());
    }

    #[test]
    fn volta_f64_ratio() {
        assert_eq!(GpuArch::Volta.f64_ratio(), 2);
        assert_eq!(GpuArch::Ada.f64_ratio(), 64);
    }

    #[test]
    fn adaptive_batch_volta() {
        let props = Properties {
            memory_bytes: Some(12 * 1024 * 1024 * 1024),
            gpu_arch: Some(GpuArch::Volta),
            has_f64: true,
            ..Properties::default()
        };
        let batch = AdaptiveBatch::for_gpu(&props, 64);
        assert!(batch.native_f64);
        assert!(batch.use_resident_memory);
        assert_eq!(batch.workgroup_size, 64);
        assert!(batch.max_batch_elements > 100_000_000);
    }

    #[test]
    fn adaptive_batch_ada() {
        let props = Properties {
            memory_bytes: Some(12 * 1024 * 1024 * 1024),
            gpu_arch: Some(GpuArch::Ada),
            has_f64: true,
            ..Properties::default()
        };
        let batch = AdaptiveBatch::for_gpu(&props, 64);
        assert!(!batch.native_f64);
        assert!(!batch.use_resident_memory);
        assert_eq!(batch.workgroup_size, 256);
    }

    #[test]
    fn capability_summary_joins_labels() {
        let s = Substrate {
            kind: SubstrateKind::Cpu,
            identity: Identity::named("CPU"),
            properties: Properties::default(),
            capabilities: vec![Capability::F64Compute, Capability::F32Compute],
        };
        let summary = s.capability_summary();
        assert!(summary.contains("f64"));
        assert!(summary.contains("f32"));
    }

    #[test]
    fn display_with_driver_and_memory() {
        let s = Substrate {
            kind: SubstrateKind::Gpu,
            identity: Identity {
                name: String::from("RTX 4070"),
                driver: Some(String::from("NVIDIA 550.67")),
                ..Identity::named("RTX 4070")
            },
            properties: Properties {
                memory_bytes: Some(12 * 1024 * 1024 * 1024),
                ..Properties::default()
            },
            capabilities: vec![],
        };
        let display = s.to_string();
        assert!(display.contains("RTX 4070"));
        assert!(display.contains("NVIDIA 550.67"));
        assert!(display.contains("MB"));
    }

    #[test]
    fn identity_named_defaults() {
        let id = Identity::named("Test");
        assert_eq!(id.name, "Test");
        assert!(id.driver.is_none());
        assert!(id.backend.is_none());
        assert!(id.adapter_index.is_none());
        assert!(id.device_node.is_none());
        assert!(id.pci_id.is_none());
    }

    #[test]
    fn properties_default_values() {
        let p = Properties::default();
        assert!(p.memory_bytes.is_none());
        assert!(p.core_count.is_none());
        assert!(p.thread_count.is_none());
        assert!(p.cache_kb.is_none());
        assert!(!p.has_f64);
        assert!(!p.has_timestamps);
    }
}
