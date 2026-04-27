// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Compute dispatch via biomeOS `compute.*` and related capabilities.

use std::path::Path;

use serde_json::Value;

use super::routing::capability_call_value;
use super::{BiomeOsError, FAMILY_ID, Result};

/// Dispatch a computation via `compute.execute` capability routing.
///
/// The `op` field names the operation (e.g. `"lyapunov_averaged"`).
/// Additional fields in `params_json` carry the operation-specific arguments.
/// biomeOS routes to whichever primal provides the `compute` capability.
///
/// # Errors
///
/// Returns `Err` if biomeOS is unavailable or the compute provider rejects
/// the request.
pub fn compute_execute(socket: &Path, op: &str, params_json: &str) -> Result<String> {
    let mut args: Value = serde_json::from_str(params_json)
        .map_err(|e| BiomeOsError::Serialization(format!("invalid compute params: {e}")))?;
    merge_compute_fields(&mut args, op);
    capability_call_value(socket, "compute.execute", &args)
}

/// Submit a compute job asynchronously via `compute.submit`.
///
/// Returns a job ID or status from the compute provider.
///
/// # Errors
///
/// Returns `Err` if biomeOS is unavailable or the submission fails.
pub fn compute_submit(socket: &Path, op: &str, params_json: &str) -> Result<String> {
    let mut args: Value = serde_json::from_str(params_json)
        .map_err(|e| BiomeOsError::Serialization(format!("invalid compute params: {e}")))?;
    merge_compute_fields(&mut args, op);
    capability_call_value(socket, "compute.submit", &args)
}

/// Inject `op` and `family_id` into a compute params [`Value`].
fn merge_compute_fields(args: &mut Value, op: &str) {
    if let Some(obj) = args.as_object_mut() {
        obj.insert("op".to_string(), Value::String(op.to_string()));
        obj.insert(
            "family_id".to_string(),
            Value::String(FAMILY_ID.to_string()),
        );
    }
}

/// Query compute capabilities from the compute provider.
///
/// Returns JSON listing available compute operations and GPU info.
///
/// # Errors
///
/// Returns `Err` if biomeOS or the compute provider is unavailable.
pub fn compute_capabilities(socket: &Path) -> Result<String> {
    let args = serde_json::json!({ "family_id": FAMILY_ID });
    capability_call_value(socket, "compute.capabilities", &args)
}
