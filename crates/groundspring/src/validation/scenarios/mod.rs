// SPDX-License-Identifier: AGPL-3.0-or-later

//! Validation scenarios absorbed from prokaryotic experiment binaries.
//!
//! Each scenario module exports a `pub const SCENARIO: Scenario` with metadata
//! and a run function. The `build_registry()` function assembles all scenarios
//! into a filterable `ScenarioRegistry`.

pub mod registry;

mod s_anderson;
mod s_bistable;
mod s_composition_parity;
mod s_decompose;
mod s_drift;
mod s_fao56;
mod s_freeze_out;
mod s_jackknife;
mod s_rarefaction;
mod s_seismic;

/// Build the complete scenario registry with all absorbed experiments.
#[must_use]
pub fn build_registry() -> registry::ScenarioRegistry {
    let mut reg = registry::ScenarioRegistry::new();

    reg.register(s_decompose::SCENARIO);
    reg.register(s_rarefaction::SCENARIO);
    reg.register(s_anderson::SCENARIO);
    reg.register(s_fao56::SCENARIO);
    reg.register(s_freeze_out::SCENARIO);
    reg.register(s_bistable::SCENARIO);
    reg.register(s_seismic::SCENARIO);
    reg.register(s_drift::SCENARIO);
    reg.register(s_jackknife::SCENARIO);
    reg.register(s_composition_parity::SCENARIO);

    reg
}
