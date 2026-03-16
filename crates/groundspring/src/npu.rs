// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! NPU integration for groundSpring via `ToadStool` `akida-driver`.
//!
//! Provides Anderson localization regime classification on `BrainChip`
//! AKD1000 neuromorphic hardware. Features extracted from disorder
//! parameters (W, E, L) are quantized to int8 and dispatched to the
//! NPU for classification into Localized / Critical / Extended regimes.
//!
//! # Architecture
//!
//! - **Zero Mocks**: Real hardware only; functions error when no device
//! - **Capability-Based**: Devices discovered at runtime
//! - **Primal Self-Knowledge**: groundSpring discovers NPU, never hardcodes

use akida_driver::{AkidaDevice, AkidaError, Capabilities, ChipVersion, DeviceManager, Result};

/// Int8 maximum for linear quantization (AKD1000 uses unsigned 7-bit range).
const INT8_QUANT_MAX: f64 = 127.0;

/// Disorder strength quantization range: W ∈ \[0, 10\].
///
/// Upper bound chosen to cover the full localized regime; W > 10 is
/// deep-localized with ξ < 1 site (uninteresting for regime classification).
const W_RANGE: (f64, f64) = (0.0, 10.0);

/// Band-centre energy quantization range: E ∈ \[-3, 3\].
///
/// Covers the full single-particle bandwidth (±2t for nearest-neighbour
/// hopping t = 1) with headroom for disorder broadening.
const E_RANGE: (f64, f64) = (-3.0, 3.0);

/// System length quantization range: L ∈ \[10, 10 000\].
///
/// Lower bound ensures meaningful localization-length ratios; upper bound
/// covers the largest systems used in validation (N = 200–1000).
const L_RANGE: (f64, f64) = (10.0, 10_000.0);

/// ξ/L below this threshold → Localized regime.
///
/// When the localization length is less than half the system length,
/// the wavefunction decays significantly within the sample.
/// Reference: Kramer & `MacKinnon` (1993) Rep Prog Phys 56:1469, §3.1.
const LOCALIZED_RATIO_THRESHOLD: f64 = 0.5;

/// ξ/L above this threshold → Extended regime.
///
/// When ξ exceeds twice the system size, finite-size effects dominate
/// and the system behaves as if extended.
/// Reference: Kramer & `MacKinnon` (1993) Rep Prog Phys 56:1469, §3.1.
const EXTENDED_RATIO_THRESHOLD: f64 = 2.0;

/// Nanoseconds per microsecond.
const NS_PER_US: f64 = 1000.0;

/// Anderson localization regime classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegimeClass {
    /// Strong disorder (W > 4): exponentially localized, ξ << L.
    Localized = 0,
    /// Critical regime (1 < W < 4): ξ ~ L.
    Critical = 1,
    /// Weak disorder (W < 1): ξ >> L, effectively extended.
    Extended = 2,
}

impl RegimeClass {
    /// Human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Localized => "Localized",
            Self::Critical => "Critical",
            Self::Extended => "Extended",
        }
    }

    /// From a class index (0, 1, 2).
    #[must_use]
    pub const fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Localized,
            1 => Self::Critical,
            _ => Self::Extended,
        }
    }
}

impl std::fmt::Display for RegimeClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// NPU handle for groundSpring — wraps an opened Akida device.
pub struct NpuHandle {
    device: AkidaDevice,
    caps: Capabilities,
}

impl NpuHandle {
    /// Device capabilities (discovered at runtime).
    #[must_use]
    pub const fn capabilities(&self) -> &Capabilities {
        &self.caps
    }

    /// Chip version.
    #[must_use]
    pub const fn chip_version(&self) -> ChipVersion {
        self.caps.chip_version
    }

    /// Number of neural processors.
    #[must_use]
    pub const fn npu_count(&self) -> u32 {
        self.caps.npu_count
    }

    /// SRAM in megabytes.
    #[must_use]
    pub const fn memory_mb(&self) -> u32 {
        self.caps.memory_mb
    }

    /// Write raw bytes to device SRAM.
    ///
    /// # Errors
    ///
    /// Returns error on DMA failure.
    pub fn write_raw(&mut self, data: &[u8]) -> Result<usize> {
        self.device.write(data)
    }

    /// Read raw bytes from device SRAM.
    ///
    /// # Errors
    ///
    /// Returns error on DMA failure.
    pub fn read_raw(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.device.read(buf)
    }
}

/// Discover and open the first available Akida NPU.
///
/// # Errors
///
/// Returns error if no Akida hardware is detected or open fails.
pub fn discover_npu() -> Result<NpuHandle> {
    let manager = DeviceManager::discover()?;
    let info = manager
        .devices()
        .first()
        .ok_or(AkidaError::NoDevicesFound)?;
    let caps = info.capabilities().clone();
    let device = AkidaDevice::open(info)?;
    Ok(NpuHandle { device, caps })
}

/// Check if an Akida NPU is available without opening it.
#[must_use]
pub fn npu_available() -> bool {
    DeviceManager::discover()
        .map(|m| m.device_count() > 0)
        .unwrap_or(false)
}

/// Quantize Anderson features `(W, E, L)` to int8 for NPU inference.
///
/// Ranges: W ∈ \[0, 10\], E ∈ \[-3, 3\], L ∈ \[10, 10000\].
/// Each value maps linearly to \[0, 127\] with clamping.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "quantized features clamped to i8 range"
)]
pub fn quantize_features(w: f64, e: f64, l: f64) -> [i8; 3] {
    let q = |val: f64, lo: f64, hi: f64| -> i8 {
        let n = ((val - lo) / (hi - lo)).clamp(0.0, 1.0);
        (n * INT8_QUANT_MAX) as i8
    };
    [
        q(w, W_RANGE.0, W_RANGE.1),
        q(e, E_RANGE.0, E_RANGE.1),
        q(l, L_RANGE.0, L_RANGE.1),
    ]
}

/// Dequantize an int8 value back to f64 given the original range.
#[must_use]
pub fn dequantize_i8(val: i8, lo: f64, hi: f64) -> f64 {
    let n = f64::from(val) / INT8_QUANT_MAX;
    n.mul_add(hi - lo, lo)
}

/// Classify Anderson regime analytically (CPU reference).
///
/// Uses ξ/L ratio to assign regime — this is the ground truth against
/// which NPU classification accuracy is measured.
#[must_use]
pub fn classify_regime_cpu(disorder: f64, energy: f64, n_sites: usize) -> RegimeClass {
    let xi = crate::anderson::analytical_localization_length(disorder, energy);
    let l = crate::cast::usize_f64(n_sites);
    let ratio = xi / l;
    if ratio < LOCALIZED_RATIO_THRESHOLD {
        RegimeClass::Localized
    } else if ratio > EXTENDED_RATIO_THRESHOLD {
        RegimeClass::Extended
    } else {
        RegimeClass::Critical
    }
}

/// Compute a simple int8 readout weight matrix for Anderson regime
/// classification using the analytical ξ/L ratio as training signal.
///
/// This produces a deterministic 3×3 weight matrix (3 input features,
/// 3 output classes) that can be loaded onto the AKD1000.
#[must_use]
pub fn train_classifier_weights(disorders: &[f64], n_sites: usize) -> [i8; 9] {
    let mut counts = [0i32; 3];
    let mut sums = [[0i64; 3]; 3];

    for &w in disorders {
        let features = quantize_features(w, 0.0, crate::cast::usize_f64(n_sites));
        #[expect(clippy::as_conversions, reason = "RegimeClass repr is 0..=2")]
        let class = classify_regime_cpu(w, 0.0, n_sites) as usize;
        counts[class] += 1;
        for (j, &f) in features.iter().enumerate() {
            sums[class][j] += i64::from(f);
        }
    }

    let mut weights = [0i8; 9];
    for c in 0..3 {
        let n = counts[c].max(1);
        for j in 0..3 {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "mean clamped to [-128, 127] before cast"
            )]
            let w = (sums[c][j] / i64::from(n)).clamp(-128, 127) as i8;
            weights[c * 3 + j] = w;
        }
    }
    weights
}

/// Run int8 classification on NPU for a single set of features.
///
/// Writes 3 input bytes, reads 3 output bytes, returns argmax class.
///
/// # Errors
///
/// Returns error on DMA failure.
#[expect(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "i8→u8 DMA encoding and u8→RegimeClass with checked bounds"
)]
pub fn npu_classify_regime(
    handle: &mut NpuHandle,
    features: [i8; 3],
) -> Result<(RegimeClass, NpuInferMetrics)> {
    let input: Vec<u8> = features.iter().map(|&x| x as u8).collect();

    let t = std::time::Instant::now();
    handle.write_raw(&input)?;
    let write_ns = t.elapsed().as_nanos() as u64;

    let mut output = [0u8; 3];
    let t = std::time::Instant::now();
    handle.read_raw(&mut output)?;
    let read_ns = t.elapsed().as_nanos() as u64;

    let signed: Vec<i8> = output.iter().map(|&b| b as i8).collect();
    let class_idx = signed
        .iter()
        .enumerate()
        .max_by_key(|&(_, v)| *v)
        .map_or(0, |(i, _)| i);

    Ok((
        RegimeClass::from_index(class_idx),
        NpuInferMetrics { write_ns, read_ns },
    ))
}

/// Metrics from a single NPU inference.
#[derive(Debug, Clone)]
pub struct NpuInferMetrics {
    /// DMA write latency (nanoseconds).
    pub write_ns: u64,
    /// DMA read latency (nanoseconds).
    pub read_ns: u64,
}

impl NpuInferMetrics {
    /// Total round-trip in microseconds.
    #[must_use]
    pub fn total_us(&self) -> f64 {
        crate::cast::u64_f64(self.write_ns + self.read_ns) / NS_PER_US
    }
}

/// Load classifier weights to NPU SRAM via DMA.
///
/// # Errors
///
/// Returns error on DMA failure.
#[expect(
    clippy::cast_sign_loss,
    reason = "i8 weights reinterpreted as u8 for DMA byte buffer"
)]
pub fn load_classifier_weights(handle: &mut NpuHandle, weights: &[i8; 9]) -> Result<usize> {
    let bytes: Vec<u8> = weights.iter().map(|&x| x as u8).collect();
    handle.write_raw(&bytes)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn quantize_midpoint() {
        let f = quantize_features(5.0, 0.0, 5005.0);
        assert!((f[0] - 63).abs() <= 1, "W=5/10 -> ~63, got {}", f[0]);
        assert!((f[1] - 63).abs() <= 1, "E=0 mid[-3,3] -> ~63, got {}", f[1]);
        assert!((f[2] - 63).abs() <= 1, "L=5005 mid -> ~63, got {}", f[2]);
    }

    #[test]
    fn quantize_clamps() {
        let lo = quantize_features(0.0, -3.0, 10.0);
        let hi = quantize_features(10.0, 3.0, 10000.0);
        assert_eq!(lo, [0, 0, 0]);
        assert_eq!(hi, [127, 127, 127]);
    }

    #[test]
    fn classify_regime_boundaries() {
        assert_eq!(classify_regime_cpu(8.0, 0.0, 100), RegimeClass::Localized);
        assert_eq!(classify_regime_cpu(0.1, 0.0, 100), RegimeClass::Extended);
    }

    #[test]
    fn train_weights_deterministic() {
        let w1 = train_classifier_weights(&[0.1, 2.0, 8.0], 100);
        let w2 = train_classifier_weights(&[0.1, 2.0, 8.0], 100);
        assert_eq!(w1, w2);
    }

    #[test]
    fn regime_class_display() {
        assert_eq!(RegimeClass::Localized.to_string(), "Localized");
        assert_eq!(RegimeClass::Critical.to_string(), "Critical");
        assert_eq!(RegimeClass::Extended.to_string(), "Extended");
    }

    #[test]
    fn npu_availability_check() {
        let avail = npu_available();
        println!("NPU available: {avail}");
    }

    #[test]
    fn dequantize_midpoint() {
        let val = dequantize_i8(0, 0.0, 10.0);
        assert!(
            (val - 0.0).abs() < tol::EQUILIBRIUM,
            "i8(0) → lo, got {val}"
        );
    }

    #[test]
    fn dequantize_max() {
        let val = dequantize_i8(127, 0.0, 10.0);
        assert!(
            (val - 10.0).abs() < tol::EQUILIBRIUM,
            "i8(127) → hi, got {val}"
        );
    }

    #[test]
    fn quantize_dequantize_roundtrip() {
        let features = quantize_features(5.0, 0.0, 5005.0);
        let w_back = dequantize_i8(features[0], W_RANGE.0, W_RANGE.1);
        assert!(
            (w_back - 5.0).abs() < 0.5,
            "W roundtrip: 5.0 → {} → {w_back}",
            features[0]
        );
    }
}
