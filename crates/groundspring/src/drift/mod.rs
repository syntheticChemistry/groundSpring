// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Drift vs selection in finite populations (Wright-Fisher model).
//!
//! Implements the Wright-Fisher model for population genetics to study
//! when stochastic drift overwhelms deterministic selection. The key
//! parameter is N×s — the product of effective population size and
//! selection coefficient.
//!
//! ## Module structure
//!
//! - [`monitor`] — `DriftMonitor` advisory system for `N_e`·`s` tracking
//! - This module — Wright-Fisher simulation, Kimura fixation, neutral drift
//!
//! # References
//!
//! - Anderson (2022) mBio 13:e00354-22
//! - Kimura (1968) Nature 217:624-626
//! - Wright (1931) Genetics 16:97-159
//!
//! # barracuda delegation
//!
//! [`kimura_fixation_prob`] delegates to
//! `barracuda::stats::evolution::kimura_fixation_prob` on CPU (S70+).
//! When `barracuda-gpu` is enabled, batch computation dispatches via
//! `KimuraGpu` (S71 — GPU-parallel via `kimura_fixation_f64.wgsl`).
//! [`wright_fisher_fixation`] is a single serial trial (Stays Local).
//! [`wright_fisher_fixation_batch`] dispatches many independent trials to
//! `barracuda::ops::bio::WrightFisherGpu` when `barracuda-gpu` is enabled,
//! running all populations through all generations on GPU in parallel, then
//! classifying fixation/loss from the final allele frequencies. Falls back
//! to a sequential CPU loop otherwise.

mod monitor;

pub use monitor::{DriftAction, DriftMonitor};

use crate::cast::usize_f64;
use crate::prng::Xorshift64;

/// Run one Wright-Fisher trial until the advantaged allele fixes or is lost.
///
/// Models `pop_size` diploid individuals (2N alleles). Allele A has fitness
/// `1 + selection` relative to allele a (fitness 1). Starting frequency
/// is `initial_freq`.
///
/// Returns `true` if allele A fixes, `false` if lost.
///
/// # Panics
///
/// Panics if `pop_size` is zero or `initial_freq` is outside [0, 1].
#[must_use]
pub fn wright_fisher_fixation(
    pop_size: usize,
    selection: f64,
    initial_freq: f64,
    seed: u64,
) -> bool {
    assert!(pop_size > 0, "pop_size must be positive");
    assert!(
        (0.0..=1.0).contains(&initial_freq),
        "initial_freq must be in [0, 1]"
    );

    let n_alleles = 2 * pop_size;
    let n_alleles_f = usize_f64(n_alleles);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "freq * 2N is non-negative and ≤ 2N which fits u64"
    )]
    let mut n_a = (initial_freq * n_alleles_f).round() as u64;
    // Factor 10: Wright-Fisher fixation typically takes O(N) generations;
    // 10× gives headroom for slow selection near neutrality.
    let max_gens = 10 * n_alleles;
    let mut rng = Xorshift64::new(seed);
    let n_alleles_u64 = n_alleles as u64;

    for _ in 0..max_gens {
        if n_a == 0 {
            return false;
        }
        if n_a == n_alleles_u64 {
            return true;
        }

        let freq_a = crate::cast::u64_f64(n_a) / n_alleles_f;
        let fitness_a = freq_a * (1.0 + selection);
        let fitness_total = fitness_a + (1.0 - freq_a);
        let prob_a = fitness_a / fitness_total;

        n_a = rng.binomial(n_alleles, prob_a);
    }

    n_a > n_alleles_u64 / 2
}

/// Kimura (1968) analytical fixation probability.
///
/// `P_fix = (1 - exp(-4Ns p₀)) / (1 - exp(-4Ns))`
///
/// For neutral evolution (s=0), returns `initial_freq`.
///
/// Delegates to `barracuda::stats::evolution::kimura_fixation_prob` when the
/// `barracuda` feature is enabled (absorbed in barraCuda S70+).
#[must_use]
pub fn kimura_fixation_prob(pop_size: usize, selection: f64, initial_freq: f64) -> f64 {
    #[cfg(feature = "barracuda")]
    {
        barracuda::stats::evolution::kimura_fixation_prob(pop_size, selection, initial_freq)
    }
    #[cfg(not(feature = "barracuda"))]
    {
        kimura_fixation_prob_cpu(pop_size, selection, initial_freq)
    }
}

/// Below this 4Ns threshold, selection is effectively neutral and we return
/// `initial_freq` directly to avoid numerical instability in the Kimura formula.
#[cfg(not(feature = "barracuda"))]
const NEUTRAL_SELECTION_THRESHOLD: f64 = 1e-10;

/// Denominator zero-guard for the Kimura exponential ratio.
#[cfg(not(feature = "barracuda"))]
const KIMURA_DENOM_EPSILON: f64 = 1e-15;

#[cfg(not(feature = "barracuda"))]
fn kimura_fixation_prob_cpu(pop_size: usize, selection: f64, initial_freq: f64) -> f64 {
    let four_ns = 4.0 * usize_f64(pop_size) * selection;
    if four_ns.abs() < NEUTRAL_SELECTION_THRESHOLD {
        return initial_freq;
    }

    let numerator = 1.0 - (-four_ns * initial_freq).exp();
    let denominator = 1.0 - (-four_ns).exp();
    if denominator.abs() < KIMURA_DENOM_EPSILON {
        return initial_freq;
    }

    numerator / denominator
}

/// Run many independent Wright-Fisher trials and return fixation count.
///
/// When the `barracuda-gpu` feature is enabled and a GPU is available,
/// dispatches all trials to `WrightFisherGpu`, running every generation
/// on the GPU. Falls back to sequential CPU execution otherwise.
///
/// Returns the number of trials in which the advantaged allele fixed.
#[must_use]
pub fn wright_fisher_fixation_batch(
    pop_size: usize,
    selection: f64,
    initial_freq: f64,
    n_trials: usize,
    base_seed: u64,
) -> usize {
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(count) = wf_batch_gpu(pop_size, selection, initial_freq, n_trials, base_seed) {
            return count;
        }
    }
    wf_batch_cpu(pop_size, selection, initial_freq, n_trials, base_seed)
}

fn wf_batch_cpu(
    pop_size: usize,
    selection: f64,
    initial_freq: f64,
    n_trials: usize,
    base_seed: u64,
) -> usize {
    (0..n_trials)
        .filter(|&i| {
            wright_fisher_fixation(
                pop_size,
                selection,
                initial_freq,
                base_seed.wrapping_add(i as u64),
            )
        })
        .count()
}

/// Generate xoshiro128**-compatible PRNG state for GPU dispatch.
///
/// Each trial needs 4 × u32 state words, seeded deterministically from
/// a single `Xorshift64` stream.
#[cfg(feature = "barracuda-gpu")]
fn wf_generate_prng_state(n_trials: usize, base_seed: u64) -> Vec<u32> {
    let mut state = Vec::with_capacity(n_trials * 4);
    let mut rng = crate::prng::Xorshift64::new(base_seed);
    for _ in 0..n_trials {
        for _ in 0..4 {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "RNG u64 → u32 seed; high bits discarded intentionally"
            )]
            state.push(rng.next_u64() as u32);
        }
    }
    state
}

/// Map a staging buffer back to the host and count fixation events.
///
/// An allele is "fixed" when its final frequency reaches 1.0.
#[cfg(feature = "barracuda-gpu")]
fn wf_readback_fixations(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    source_buf: &wgpu::Buffer,
    n_trials: usize,
) -> Option<usize> {
    let byte_len = (n_trials * 8) as u64;

    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("wf_staging"),
        size: byte_len,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("wf_readback"),
    });
    encoder.copy_buffer_to_buffer(source_buf, 0, &staging, 0, byte_len);
    queue.submit(std::iter::once(encoder.finish()));

    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        tx.send(r).ok();
    });
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    rx.recv().ok()?.ok()?;

    let data = slice.get_mapped_range();
    let freqs: &[f64] = bytemuck::cast_slice(&data);
    let count = freqs.iter().filter(|&&f| f >= 1.0).count();
    drop(data);
    staging.unmap();

    Some(count)
}

#[cfg(feature = "barracuda-gpu")]
fn wf_batch_gpu(
    pop_size: usize,
    selection: f64,
    initial_freq: f64,
    n_trials: usize,
    base_seed: u64,
) -> Option<usize> {
    use barracuda::ops::bio::WrightFisherGpu;
    use wgpu::util::DeviceExt;

    let wgpu_dev = crate::gpu::get_device()?;
    let d = wgpu_dev.device();
    let q = wgpu_dev.queue();

    #[expect(
        clippy::cast_possible_truncation,
        reason = "n_trials ≤ 10000, fits u32"
    )]
    let n_pops = n_trials as u32;
    let n_loci: u32 = 1;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "2 * pop_size ≤ 2000, fits u32"
    )]
    let two_n = (2 * pop_size) as u32;
    let max_gens = 10 * (2 * pop_size);

    let freq_init: Vec<f64> = vec![initial_freq; n_trials];
    let prng_state = wf_generate_prng_state(n_trials, base_seed);

    let freq_in_buf = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("wf_freq_in"),
        contents: bytemuck::cast_slice(&freq_init),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });
    let freq_out_buf = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("wf_freq_out"),
        contents: bytemuck::cast_slice(&freq_init),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
    });
    let sel_buf = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("wf_selection"),
        contents: bytemuck::cast_slice(&[selection]),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let prng_buf = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("wf_prng"),
        contents: bytemuck::cast_slice(&prng_state),
        usage: wgpu::BufferUsages::STORAGE,
    });

    let gpu = WrightFisherGpu::new(wgpu_dev.clone());

    for generation in 0..max_gens {
        if generation % 2 == 0 {
            gpu.dispatch(
                &freq_in_buf,
                &sel_buf,
                &freq_out_buf,
                &prng_buf,
                n_pops,
                n_loci,
                two_n,
            );
        } else {
            gpu.dispatch(
                &freq_out_buf,
                &sel_buf,
                &freq_in_buf,
                &prng_buf,
                n_pops,
                n_loci,
                two_n,
            );
        }
    }

    let final_buf = if max_gens.is_multiple_of(2) {
        &freq_in_buf
    } else {
        &freq_out_buf
    };
    wf_readback_fixations(d, q, final_buf, n_trials)
}

/// Track Shannon diversity under pure neutral drift (multi-species Wright-Fisher).
///
/// Returns a vector of Shannon diversities, one per generation.
///
/// # Panics
///
/// Panics if `n_species` or `pop_size` is zero.
#[must_use]
pub fn neutral_diversity_trajectory(
    n_species: usize,
    pop_size: usize,
    n_generations: usize,
    seed: u64,
) -> Vec<f64> {
    assert!(n_species > 0 && pop_size > 0);

    let mut rng = Xorshift64::new(seed);
    let base_count = pop_size / n_species;
    let mut abundances: Vec<u64> = vec![base_count as u64; n_species];
    let remainder = pop_size - base_count * n_species;
    abundances[0] += remainder as u64;

    let mut diversities = Vec::with_capacity(n_generations);
    let pop_f = usize_f64(pop_size);

    for _ in 0..n_generations {
        let mut shannon = 0.0;
        for &a in &abundances {
            if a > 0 {
                let p = crate::cast::u64_f64(a) / pop_f;
                shannon -= p * p.ln();
            }
        }
        diversities.push(shannon);

        let mut remaining = pop_size as u64;
        let total: u64 = abundances.iter().sum();
        let mut remaining_prob_mass = crate::cast::u64_f64(total);
        let mut new_abundances = vec![0u64; n_species];

        for sp in 0..n_species - 1 {
            if remaining == 0 {
                break;
            }
            let prob = crate::cast::u64_f64(abundances[sp]) / remaining_prob_mass;
            #[expect(
                clippy::cast_possible_truncation,
                reason = "remaining individuals ≤ community_size which fits usize"
            )]
            let n_remaining = remaining as usize;
            new_abundances[sp] = rng.binomial(n_remaining, prob);
            remaining = remaining.saturating_sub(new_abundances[sp]);
            remaining_prob_mass -= crate::cast::u64_f64(abundances[sp]);
        }
        new_abundances[n_species - 1] = remaining;
        abundances = new_abundances;
    }

    diversities
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn kimura_neutral() {
        let p = kimura_fixation_prob(100, 0.0, 0.5);
        assert!(
            (p - 0.5).abs() < tol::ANALYTICAL,
            "neutral fixation should be p₀"
        );
    }

    #[test]
    fn kimura_strong_selection() {
        let p = kimura_fixation_prob(1000, 0.01, 0.5);
        assert!(p > 0.5, "positive selection should increase fixation");
        assert!(p < 1.0, "fixation probability should be < 1");
    }

    #[test]
    fn kimura_increases_with_n() {
        let p_small = kimura_fixation_prob(50, 0.01, 0.5);
        let p_large = kimura_fixation_prob(1000, 0.01, 0.5);
        assert!(
            p_large > p_small,
            "fixation prob should increase with N for s > 0"
        );
    }

    #[test]
    fn wf_deterministic() {
        let r1 = wright_fisher_fixation(100, 0.01, 0.5, 42);
        let r2 = wright_fisher_fixation(100, 0.01, 0.5, 42);
        assert_eq!(r1, r2, "same seed should give same result");
    }

    #[test]
    fn diversity_declines_under_drift() {
        let div = neutral_diversity_trajectory(10, 50, 200, 42);
        assert!(
            *div.last().expect("non-empty diversity trajectory") < div[0],
            "diversity should decline"
        );
    }

    #[test]
    fn larger_pop_preserves_diversity() {
        let div_small = neutral_diversity_trajectory(10, 50, 200, 42);
        let div_large = neutral_diversity_trajectory(10, 500, 200, 42);
        let final_large = *div_large.last().expect("non-empty large-pop trajectory");
        let final_small = *div_small.last().expect("non-empty small-pop trajectory");
        assert!(
            final_large > final_small,
            "larger populations should preserve more diversity"
        );
    }

    #[test]
    fn wf_batch_parity_cpu_sequential_vs_dispatch() {
        let pop = 50;
        let selection = 0.05;
        let freq = 0.5;
        let n_trials = 200;
        let base_seed = 42;

        let batch = wright_fisher_fixation_batch(pop, selection, freq, n_trials, base_seed);
        let cpu = wf_batch_cpu(pop, selection, freq, n_trials, base_seed);

        if cfg!(feature = "barracuda-gpu") {
            let kimura = kimura_fixation_prob(pop, selection, freq);
            let expected = (kimura * usize_f64(n_trials)).round();
            let tol = usize_f64(n_trials) * 0.2;
            assert!(
                (usize_f64(batch) - expected).abs() < tol,
                "GPU batch {batch} far from expected {expected} (Kimura: {kimura})",
            );
        } else {
            assert_eq!(batch, cpu, "without GPU, batch and CPU must match exactly");
        }
    }

    #[test]
    fn wf_batch_fixation_rate_reasonable() {
        let n_trials = 500;
        let fix_count = wright_fisher_fixation_batch(100, 0.05, 0.5, n_trials, 42);
        let rate = usize_f64(fix_count) / usize_f64(n_trials);
        let kimura = kimura_fixation_prob(100, 0.05, 0.5);
        assert!(
            (rate - kimura).abs() < 0.15,
            "fixation rate {rate} should be near Kimura {kimura}"
        );
    }
}
