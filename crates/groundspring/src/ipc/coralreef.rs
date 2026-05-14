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
//! - `shader.compile.gemm` — compile tensor-core GEMM kernel (SM80+ `mma.sync`)
//! - `shader.targets` — list available compilation targets
//! - `shader.validate` — validate a WGSL module without output
//! - `health.version` — build identity for post-upgrade verification

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

    /// Compile a tensor-core GEMM kernel for SM80+ `mma.sync` dispatch.
    ///
    /// Upstream: `shader.compile.gemm` (Sprint 11, tarpc + JSON-RPC).
    async fn compile_gemm(
        m: u32,
        n: u32,
        k: u32,
        precision: String,
        arch: String,
    ) -> Result<String, String>;

    /// Query available shader compilation targets.
    async fn targets() -> Result<String, String>;

    /// Validate a WGSL module without producing output.
    async fn validate(source: String) -> Result<String, String>;

    /// Build identity for post-upgrade verification.
    ///
    /// Returns session ID, build hash, version, and primal name.
    /// Trio-consistent with `barraCuda` and `toadStool` `health.version`.
    async fn health_version() -> Result<String, String>;
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

/// Compile a tensor-core GEMM kernel via `coralReef` JSON-RPC.
///
/// Requests `mma.sync` GEMM compilation for SM80+ targets with specified
/// matrix dimensions and precision. Upstream: `shader.compile.gemm` (Sprint 11).
///
/// # Errors
///
/// Returns `BiomeOsError` if `coralReef` is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn compile_gemm(
    socket: &std::path::Path,
    m: u32,
    n: u32,
    k: u32,
    precision: &str,
    arch: &str,
) -> crate::biomeos::Result<serde_json::Value> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "shader.compile.gemm",
        "params": {
            "m": m,
            "n": n,
            "k": k,
            "precision": precision,
            "arch": arch,
        },
        "id": 1
    })
    .to_string();
    let response = crate::biomeos::raw_rpc_call(socket, &request)?;
    parse_jsonrpc_response(&response)
}

/// Attempt to discover `coralReef` and compile a GEMM kernel.
///
/// Returns `Ok(None)` if `coralReef` is not available (graceful degradation).
#[cfg(feature = "biomeos")]
pub fn try_compile_gemm(
    m: u32,
    n: u32,
    k: u32,
    precision: &str,
    arch: &str,
) -> crate::biomeos::Result<Option<serde_json::Value>> {
    crate::primal_names::discover_socket(crate::primal_names::roles::COMPILER).map_or_else(
        || {
            tracing::debug!("coralReef not discovered — GEMM compilation skipped");
            Ok(None)
        },
        |socket| compile_gemm(&socket, m, n, k, precision, arch).map(Some),
    )
}

/// Query `coralReef` build identity via JSON-RPC.
///
/// Returns session ID, build hash, version, and primal name.
/// Trio-consistent with `barraCuda` and `toadStool` `health.version`.
///
/// # Errors
///
/// Returns `BiomeOsError` if `coralReef` is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn health_version(socket: &std::path::Path) -> crate::biomeos::Result<serde_json::Value> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.version",
        "params": {},
        "id": 1
    })
    .to_string();
    let response = crate::biomeos::raw_rpc_call(socket, &request)?;
    parse_jsonrpc_response(&response)
}

/// Attempt to discover `coralReef` and query build identity.
///
/// Returns `Ok(None)` if `coralReef` is not available (graceful degradation).
#[cfg(feature = "biomeos")]
pub fn try_health_version() -> crate::biomeos::Result<Option<serde_json::Value>> {
    crate::primal_names::discover_socket(crate::primal_names::roles::COMPILER).map_or_else(
        || {
            tracing::debug!("coralReef not discovered — health.version skipped");
            Ok(None)
        },
        |socket| health_version(&socket).map(Some),
    )
}

/// List available shader compilation targets via `coralReef` JSON-RPC.
///
/// Returns targets supported by the discovered `coralReef` instance
/// (e.g. `"ptx"`, `"spirv"`, ISA variants).
///
/// # Errors
///
/// Returns `BiomeOsError` if `coralReef` is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn shader_targets(socket: &std::path::Path) -> crate::biomeos::Result<serde_json::Value> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "shader.targets",
        "params": {},
        "id": 1
    })
    .to_string();
    let response = crate::biomeos::raw_rpc_call(socket, &request)?;
    parse_jsonrpc_response(&response)
}

/// Attempt to discover `coralReef` and list shader targets.
///
/// Returns `Ok(None)` if `coralReef` is not available (graceful degradation).
#[cfg(feature = "biomeos")]
pub fn try_shader_targets() -> crate::biomeos::Result<Option<serde_json::Value>> {
    crate::primal_names::discover_socket(crate::primal_names::roles::COMPILER).map_or_else(
        || {
            tracing::debug!("coralReef not discovered — shader targets skipped");
            Ok(None)
        },
        |socket| shader_targets(&socket).map(Some),
    )
}

/// Validate a WGSL module via `coralReef` JSON-RPC without producing output.
///
/// Useful for pre-flight checks before compilation.
///
/// # Errors
///
/// Returns `BiomeOsError` if `coralReef` is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn validate_shader(
    socket: &std::path::Path,
    source: &str,
) -> crate::biomeos::Result<serde_json::Value> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "shader.validate",
        "params": {
            "source": source,
        },
        "id": 1
    })
    .to_string();
    let response = crate::biomeos::raw_rpc_call(socket, &request)?;
    parse_jsonrpc_response(&response)
}

/// Attempt to discover `coralReef` and validate a WGSL module.
///
/// Returns `Ok(None)` if `coralReef` is not available (graceful degradation).
#[cfg(feature = "biomeos")]
pub fn try_validate_shader(source: &str) -> crate::biomeos::Result<Option<serde_json::Value>> {
    crate::primal_names::discover_socket(crate::primal_names::roles::COMPILER).map_or_else(
        || {
            tracing::debug!("coralReef not discovered — shader validation skipped");
            Ok(None)
        },
        |socket| validate_shader(&socket, source).map(Some),
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
