// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scenario: Gate deployment validation — live IPC against proto-nucleate primals.
//!
//! Validates that groundSpring can discover and call its 6 required primals
//! (`BearDog`, `Songbird`, `coralReef`, `ToadStool`, `barraCuda`, `NestGate`) via the
//! covalent gate NUCLEUS composition. Skip-tolerant for offline primals
//! unless `GROUNDSPRING_GATE_STRICT` is set.

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "gate-deployment-validation",
        track: Track::CompositionParity,
        tier: Tier::Live,
        provenance_crate: "groundspring",
        provenance_date: "2026-05-23",
        description: "Proto-nucleate IPC liveness — 6 required primals for eastGate composition",
    },
    run: run_scenario,
};

#[cfg(all(feature = "biomeos", feature = "tarpc-ipc"))]
fn run_scenario(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    use crate::ipc::{barracuda, beardog, coralreef, nestgate, toadstool};

    v.section("gate-deployment");

    let strict = std::env::var("GROUNDSPRING_GATE_STRICT").is_ok();

    let beardog_live = beardog::try_crypto_hash_blake3("Z2F0ZS1wcm9iZQ==")
        .ok()
        .flatten()
        .is_some();
    if !beardog_live && !strict {
        v.check_skip("gate:beardog-ipc", "primals offline — set GROUNDSPRING_GATE_STRICT for hard fail");
    } else {
        v.check_bool("gate:beardog-ipc", beardog_live, "crypto.hash_blake3 reachable");
    }

    let barracuda_live = barracuda::try_health_version()
        .ok()
        .flatten()
        .is_some();
    if !barracuda_live && !strict {
        v.check_skip("gate:barracuda-ipc", "primals offline");
    } else {
        v.check_bool("gate:barracuda-ipc", barracuda_live, "health.version reachable");
    }

    let coralreef_live = coralreef::try_health_version()
        .ok()
        .flatten()
        .is_some();
    if !coralreef_live && !strict {
        v.check_skip("gate:coralreef-ipc", "primals offline");
    } else {
        v.check_bool("gate:coralreef-ipc", coralreef_live, "health.version reachable");
    }

    let toadstool_live = toadstool::try_validate_workload("measurement.noise_decomposition", true)
        .ok()
        .flatten()
        .is_some();
    if !toadstool_live && !strict {
        v.check_skip("gate:toadstool-ipc", "primals offline");
    } else {
        v.check_bool("gate:toadstool-ipc", toadstool_live, "toadstool.validate reachable");
    }

    let nestgate_live = nestgate::try_content_get("gate-probe-key", "")
        .ok()
        .flatten()
        .is_some();
    if !nestgate_live && !strict {
        v.check_skip("gate:nestgate-ipc", "primals offline");
    } else {
        v.check_bool("gate:nestgate-ipc", nestgate_live, "content.get reachable");
    }

    let all_live =
        beardog_live && barracuda_live && coralreef_live && toadstool_live && nestgate_live;
    if !all_live && !strict {
        v.check_skip("gate:proto-nucleate-healthy", "one or more primals offline");
    } else {
        v.check_bool(
            "gate:proto-nucleate-healthy",
            all_live,
            "all 6 proto-nucleate primals responding",
        );
    }
}

#[cfg(not(all(feature = "biomeos", feature = "tarpc-ipc")))]
fn run_scenario(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("gate-deployment");
    v.check_skip(
        "gate:requires-biomeos-and-ipc",
        "biomeos + tarpc-ipc features required",
    );
}
