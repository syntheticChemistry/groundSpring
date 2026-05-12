// SPDX-License-Identifier: AGPL-3.0-or-later

//! IPC interface for coralReef sovereign shader compilation.
//!
//! Maps `shader.*` JSON-RPC methods to typed tarpc traits.
//! coralReef provides the sovereign WGSL compilation pipeline that
//! replaces `wgpu`'s bundled `naga` compiler with a verified,
//! deterministic shader-to-SPIR-V path.
//!
//! # Status
//!
//! **Stub** — coralReef is undergoing an SM (shader model) rebuild.
//! groundSpring tracks this via GAP-GS-002 in `docs/PRIMAL_GAPS.md`.
//! When coralReef ships the rebuilt API, this module will wire up
//! `shader.compile.wgsl` and related methods.

/// Sovereign shader compilation capabilities via coralReef.
#[tarpc::service]
pub trait ShaderCompile {
    /// Compile a WGSL shader module to SPIR-V.
    async fn compile_wgsl(source: String, entry_point: String) -> Result<Vec<u8>, String>;

    /// Query available shader compilation targets.
    async fn targets() -> Result<String, String>;

    /// Validate a WGSL module without producing output.
    async fn validate(source: String) -> Result<String, String>;
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
