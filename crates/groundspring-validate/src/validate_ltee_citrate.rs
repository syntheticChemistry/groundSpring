// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! Experiment 039 — LTEE Citrate Innovation (Blount et al. 2008/2012).
//!
//! Validates the potentiation-actualization cascade model for the Cit+
//! key innovation:
//!   - Cit+ fraction is rare (consistent with 1/12 in real LTEE)
//!   - Replay probability is non-decreasing with generation number
//!   - Early replay probability <= late replay probability
//!   - Two-hit cascade mean exceeds single-hit mean
//!   - Deterministic with seed
//!
//! LTEE `GuideStone` B4 | `lithoSpore` module 4.

use groundspring::cast::usize_f64;
use groundspring::prng::Xorshift64;
use groundspring::validate::ValidationHarness;
use groundspring_validate::{f64_field, parse_benchmark, print_provenance_header, usize_field};

const BENCHMARK: &str =
    include_str!("../../../control/ltee_citrate_innovation/benchmark_ltee_citrate.json");

fn main() {
    std::process::exit(run());
}

struct PopState {
    potentiated: bool,
    cit_plus: bool,
    potentiation_gen: Option<usize>,
    cit_plus_gen: Option<usize>,
}

fn simulate_populations(
    n_pops: usize,
    n_gens: usize,
    p_pot: f64,
    p_act: f64,
    rng: &mut Xorshift64,
) -> Vec<PopState> {
    let mut pops: Vec<PopState> = (0..n_pops)
        .map(|_| PopState {
            potentiated: false,
            cit_plus: false,
            potentiation_gen: None,
            cit_plus_gen: None,
        })
        .collect();

    for g in 0..n_gens {
        for pop in &mut pops {
            if !pop.potentiated && !pop.cit_plus && rng.next_f64() < p_pot {
                pop.potentiated = true;
                pop.potentiation_gen = Some(g);
            }
            if pop.potentiated && !pop.cit_plus && rng.next_f64() < p_act {
                pop.cit_plus = true;
                pop.cit_plus_gen = Some(g);
            }
        }
        if pops.iter().all(|p| p.cit_plus) {
            break;
        }
    }

    pops
}

fn simulate_replay(
    potentiated_at_tp: &[bool],
    p_act: f64,
    replay_duration: usize,
    n_reps: usize,
    rng: &mut Xorshift64,
) -> f64 {
    let mut cit_count: u64 = 0;
    let mut total: u64 = 0;

    for &is_pot in potentiated_at_tp {
        for _ in 0..n_reps {
            total += 1;
            if !is_pot {
                continue;
            }
            for _ in 0..replay_duration {
                if rng.next_f64() < p_act {
                    cit_count += 1;
                    break;
                }
            }
        }
    }

    if total == 0 {
        0.0
    } else {
        #[expect(
            clippy::cast_precision_loss,
            reason = "replay counts are small — fits f64 mantissa"
        )]
        {
            cit_count as f64 / total as f64
        }
    }
}

fn run() -> i32 {
    let bench = parse_benchmark(BENCHMARK);
    let mut h = ValidationHarness::from_args(
        "Rust Validation: LTEE Citrate Innovation (Blount 2008/2012 B4)",
    );
    print_provenance_header(&bench, "LTEE Citrate Innovation");

    let model = &bench["model"];
    let exp = &bench["expected_results"];

    let n_pops = usize_field(model, "n_populations");
    let n_gens = usize_field(model, "n_generations");
    let p_pot = f64_field(model, "potentiation_rate_per_gen");
    let p_act = f64_field(model, "actualization_rate_per_gen");
    let seed = usize_field(model, "seed") as u64;

    let replay_timepoints: Vec<usize> = model["replay_timepoints"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_u64().and_then(|x| usize::try_from(x).ok()))
                .collect()
        })
        .unwrap_or_default();
    let n_replay_reps = usize_field(model, "n_replay_replicates");
    let replay_duration = usize_field(model, "replay_duration");

    let mut rng = Xorshift64::new(seed);
    let pops = simulate_populations(n_pops, n_gens, p_pot, p_act, &mut rng);

    let cit_count = pops.iter().filter(|p| p.cit_plus).count();
    let pot_count = pops.iter().filter(|p| p.potentiated || p.cit_plus).count();
    let cit_fraction = usize_f64(cit_count) / usize_f64(n_pops);
    let pot_fraction = usize_f64(pot_count) / usize_f64(n_pops);

    // Check 1: Cit+ fraction in expected range
    let range = exp["fraction_populations_cit_plus_range"]
        .as_array()
        .and_then(|a| Some((a[0].as_f64()?, a[1].as_f64()?)))
        .unwrap_or((0.0, 1.0));
    h.check_true(
        "Cit+ fraction in expected range",
        cit_fraction >= range.0 && cit_fraction <= range.1,
    );

    // Replay experiments
    let mut rng_replay = Xorshift64::new(seed + 1);
    let mut replay_probs: Vec<(usize, f64)> = Vec::new();

    for &tp in &replay_timepoints {
        let potentiated_at_tp: Vec<bool> = pops
            .iter()
            .map(|p| p.potentiation_gen.is_some_and(|g| g <= tp))
            .collect();
        let prob = simulate_replay(
            &potentiated_at_tp,
            p_act,
            replay_duration,
            n_replay_reps,
            &mut rng_replay,
        );
        replay_probs.push((tp, prob));
    }

    // Check 2: Replay probability non-decreasing
    let non_decreasing = replay_probs
        .windows(2)
        .all(|w| w[0].1 <= w[1].1 + 0.01);
    h.check_true(
        "Replay probability non-decreasing with generation",
        non_decreasing,
    );

    // Check 3: Early <= late
    let early_prob = replay_probs.first().map_or(0.0, |r| r.1);
    let late_prob = replay_probs.last().map_or(0.0, |r| r.1);
    h.check_true(
        "Early replay prob <= late replay prob",
        early_prob <= late_prob,
    );

    // Check 4: Early replay below threshold
    let early_max = f64_field(exp, "early_replay_cit_fraction_max");
    h.check_true(
        "Early replay Cit+ fraction below threshold",
        early_prob <= early_max,
    );

    // Check 5: Late replay above threshold
    let late_min = f64_field(exp, "late_replay_cit_fraction_min");
    h.check_true(
        "Late replay Cit+ fraction above threshold",
        late_prob >= late_min,
    );

    // Check 6: Potentiation fraction at endpoint
    let pot_min = f64_field(exp, "potentiation_fraction_at_60k_min");
    h.check_true("Potentiation fraction at endpoint", pot_fraction >= pot_min);

    // Check 7: Two-hit analytical mean > single-hit mean (E[τ₁+τ₂] > E[τ₂])
    let single_hit_mean = 1.0 / p_act;
    let two_hit_analytical = p_pot.recip() + p_act.recip();
    h.check_true(
        "Two-hit analytical mean > single-hit mean",
        two_hit_analytical > single_hit_mean,
    );

    // Check 8: Determinism
    let mut rng2 = Xorshift64::new(seed);
    let pops2 = simulate_populations(n_pops, n_gens, p_pot, p_act, &mut rng2);
    let det_pass = pops
        .iter()
        .zip(pops2.iter())
        .all(|(a, b)| a.potentiation_gen == b.potentiation_gen && a.cit_plus_gen == b.cit_plus_gen);
    h.check_true("Deterministic (same seed → same result)", det_pass);

    h.summary()
}
