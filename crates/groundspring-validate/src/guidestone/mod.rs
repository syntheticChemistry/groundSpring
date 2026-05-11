// SPDX-License-Identifier: AGPL-3.0-or-later

//! guideStone layer modules — NUCLEUS composition parity validation.
//!
//! Each module implements one architectural layer of the guideStone
//! certification:
//!
//! - [`bare`] — Properties 1-5 (deterministic, traceable, self-verifying,
//!   environment-agnostic, tolerance-documented). No primals needed.
//! - [`tower`] — Layer 3 Tower Atomic (BearDog + Songbird security/discovery).
//! - [`node`] — Layer 3 Node Atomic (barraCuda + coralReef + toadStool compute).
//! - [`nest`] — Layer 3 Nest Atomic (NestGate storage + provenance trio).
//! - [`cross`] — Layer 4 Cross-Atomic Pipeline (Tower hash → Nest roundtrip).

pub mod bare;
pub mod cross;
pub mod nest;
pub mod node;
pub mod tower;
