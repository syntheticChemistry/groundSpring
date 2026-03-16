// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Primal name constants — single source of truth for IPC identifiers.
//!
//! All primal names used in IPC discovery, registration, and capability
//! routing are defined here. No hardcoded primal name strings elsewhere
//! in library code. groundSpring discovers other primals at runtime via
//! capability-based discovery; these constants are identifiers, not
//! assumptions about what is running.

/// This niche's canonical identifier.
pub const SELF_ID: &str = "groundspring";

/// biomeOS orchestrator.
pub const BIOMEOS: &str = "biomeos";

/// `Songbird` discovery mesh.
pub const SONGBIRD: &str = "songbird";

/// `NestGate` content-addressed storage.
pub const NESTGATE: &str = "nestgate";

/// `BearDog` security foundation.
pub const BEARDOG: &str = "beardog";

/// `ToadStool` compute orchestrator.
pub const TOADSTOOL: &str = "toadstool";

/// `coralReef` sovereign shader compiler.
pub const CORALREEF: &str = "coralreef";

/// `petalTongue` visualization.
pub const PETALTONGUE: &str = "petaltongue";

/// Squirrel AI assistant.
pub const SQUIRREL: &str = "squirrel";

/// Socket directory name for biomeOS IPC mesh.
pub const BIOMEOS_SOCKET_DIR: &str = "biomeos";
