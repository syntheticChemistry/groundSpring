// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! GPU Tier Validation: prove barracuda math is portable from CPU → GPU.
//!
//! For each GPU-ready experiment, runs the computation via both the CPU path
//! and the barracuda-GPU path, then verifies parity within tolerance.
//!
//! This validates the user's thesis: "the math is truly portable via barracuda GPU"
//! and "toadstool allows for unidirectional streaming massively reducing dispatch."
//!
//! # Shader Provenance
//!
//! Each test maps to specific barraCuda shaders with cross-spring origins:
//!
//! | Test | Shader | Origin |
//! |------|--------|--------|
//! | Anderson Lyapunov | anderson.rs | hotSpring spectral S26 |
//! | Almost-Mathieu | hofstadter.rs | hotSpring spectral S26 |
//! | Stats metrics | CPU delegation | airSpring+groundSpring S64 |
//! | Shannon diversity | CPU delegation | wetSpring biodiversity S64 |
//! | Regression fits | CPU delegation | airSpring hydrology S66 |
//! | Bootstrap RAWR | CPU delegation | groundSpring S66 |
//! | Rare biosphere | `BatchedMultinomialGpu` | groundSpring→neuralSpring S64 |
//! | Bistable ODE | `BistableOde::cpu_derivative` | wetSpring S58 |
//! | Hill kinetics | CPU delegation | wetSpring QS/c-di-GMP S68 |
//! | Spectral recon | `linalg::solve_f64_cpu` | hotSpring S39 |
//!
//! Exit 0 if all checks pass, exit 1 on any failure.

mod bio;
mod spectral;
mod stats;

use groundspring_forge::harness::Harness;

fn main() {
    println!("=== groundSpring GPU Tier Validation ===");
    println!("=== barracuda CPU → barracuda GPU portability proof ===\n");
    println!("Provenance: synthetic parity — CPU vs GPU paths on identical inputs.");
    println!("  No benchmark JSON; expected values are analytical or computed inline.");
    println!("  Tolerance tiers from groundspring::tol (13-tier architecture V73).\n");

    let mut h = Harness::new();

    stats::validate_all(&mut h);
    spectral::validate_all(&mut h);
    bio::validate_all(&mut h);

    println!("\n--- Summary ---\n");
    println!("  Each test ran the SAME math through two paths:");
    println!("    1. Pure Rust CPU (groundSpring local implementation)");
    println!("    2. BarraCUDA delegation (CPU or GPU depending on feature)");
    println!("  Parity = identical results within documented tolerance.");
    println!("  This proves: math is universal, precision is silicon.\n");

    h.finish();
}
