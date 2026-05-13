// SPDX-License-Identifier: AGPL-3.0-or-later

//! IPC interface for `coralReef` sovereign shader compilation.
//!
//! Maps `shader.*` JSON-RPC methods to typed tarpc traits.
//! `coralReef` provides the sovereign WGSL compilation pipeline that
//! replaces `wgpu`'s bundled `naga` compiler with a verified,
//! deterministic shader-to-ISA path.
//!
//! # Status
//!
//! **Wired** — `coralReef` FECS stability proof shipped (Sprint 7, 4,790 tests).
//! `groundSpring` tracks residual gaps via GAP-GS-002 in `docs/PRIMAL_GAPS.md`.
//!
//! # Capability surface
//!
//! - `shader.compile.wgsl` — compile WGSL to target ISA (PTX, SPIR-V)
//! - `shader.targets` — list available compilation targets
//! - `shader.validate` — validate a WGSL module without output

/// Sovereign shader compilation capabilities via `coralReef`.
#[tarpc::service]
pub trait ShaderCompile {
    /// Compile a WGSL shader to the target ISA.
    ///
    /// `target` specifies the output format (`"ptx"`, `"spirv"`).
    /// `sm_version` is the shader model version (e.g. `70` for SM 7.0).
    /// Upstream: `shader.compile.wgsl` (Wave 8, Sprint 7 FECS proof).
    async fn compile_wgsl(
        source: String,
        target: String,
        sm_version: u32,
    ) -> Result<String, String>;

    /// Query available shader compilation targets.
    async fn targets() -> Result<String, String>;

    /// Validate a WGSL module without producing output.
    async fn validate(source: String) -> Result<String, String>;
}

/// Compile a WGSL shader via `coralReef` JSON-RPC.
///
/// # Errors
///
/// Returns `BiomeOsError` if `coralReef` is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn compile_wgsl(
    socket: &std::path::Path,
    source: &str,
    target: &str,
    sm_version: u32,
) -> crate::biomeos::Result<serde_json::Value> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "shader.compile.wgsl",
        "params": {
            "source": source,
            "target": target,
            "sm_version": sm_version,
        },
        "id": 1
    })
    .to_string();
    let response = crate::biomeos::raw_rpc_call(socket, &request)?;
    parse_jsonrpc_response(&response)
}

/// Attempt to discover `coralReef` and compile a WGSL shader.
///
/// Returns `Ok(None)` if `coralReef` is not available (graceful degradation).
///
/// # Errors
///
/// Returns `BiomeOsError` if the IPC call fails after successful discovery.
#[cfg(feature = "biomeos")]
pub fn try_compile_wgsl(
    source: &str,
    target: &str,
    sm_version: u32,
) -> crate::biomeos::Result<Option<serde_json::Value>> {
    crate::primal_names::discover_socket(crate::primal_names::roles::COMPILER).map_or_else(
        || {
            tracing::debug!("coralReef not discovered — shader compilation skipped");
            Ok(None)
        },
        |socket| compile_wgsl(&socket, source, target, sm_version).map(Some),
    )
}

/// Extract `result` or `error` from a JSON-RPC 2.0 response.
#[cfg(feature = "biomeos")]
fn parse_jsonrpc_response(response: &str) -> crate::biomeos::Result<serde_json::Value> {
    let parsed: serde_json::Value = serde_json::from_str(response)
        .map_err(|e| crate::biomeos::BiomeOsError::Protocol(format!("invalid JSON: {e}")))?;
    if let Some(err) = parsed.get("error") {
        return Err(crate::biomeos::BiomeOsError::Protocol(err.to_string()));
    }
    Ok(parsed.get("result").cloned().unwrap_or(parsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tarpc_trait_compiles() {
        fn _assert_service<T: ShaderCompile>() {}
    }

    #[test]
    fn compiler_role_is_coralreef() {
        assert_eq!(crate::primal_names::roles::COMPILER, "coralreef");
    }
}
