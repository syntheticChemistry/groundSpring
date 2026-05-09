// SPDX-License-Identifier: AGPL-3.0-or-later

//! Validation scenario framework for groundSpring.
//!
//! Provides a registry of validation scenarios that can be filtered by
//! tier (Rust/Live) and track (measurement domain group). Each scenario
//! carries provenance metadata linking back to the original experiment.

pub mod scenarios;

pub use scenarios::build_registry;
pub use scenarios::registry::{Scenario, ScenarioMeta, ScenarioRegistry, Tier, Track};
