# groundSpring → ToadStool V58 Handoff: Cross-Spring Evolution + Deep-Debt Completion

**Date**: March 1, 2026
**From**: groundSpring (V58)
**To**: ToadStool / BarraCUDA team
**ToadStool pin**: S70+++ (`1dd7e338`)
**License**: AGPL-3.0-or-later
**Supersedes**: V56 (NUCLEUS Integration)

---

## Executive Summary

- **61 active barracuda delegations** (38 CPU + 19 GPU + 4 cross-spring S59+)
- **4 new cross-spring capabilities wired**: ESN regime classification, Lanczos sparse
  eigensolver, 2D/3D Anderson eigenvalues, decomposed chi-squared analysis
- **Deep-debt audit complete**: zero unsafe, zero production mocks, zero production
  unwrap/panic, zero primal coupling in code logic, zero TODO/FIXME in source
- **FAMILY_ID evolution**: all hardcoded family identifiers replaced with single constant
- **613 workspace tests**, 486 library tests, 85 metalForge tests, all PASS

---

## Part 1: New Cross-Spring Capabilities (V57)

### 1.1 Barracuda APIs Absorbed

| groundSpring Function | barracuda API | Cross-Spring Lineage | Feature Gate |
|----------------------|---------------|---------------------|--------------|
| `anderson::disorder_sweep` | `spectral::anderson_sweep_averaged` | hotSpring Lanczos → barracuda S59 | `barracuda` (CPU fallback available) |
| `anderson::anderson_2d_eigenvalues` | `spectral::anderson_2d` + `lanczos` | hotSpring 2D Anderson → barracuda S59 | `barracuda-gpu` |
| `anderson::anderson_3d_eigenvalues` | `spectral::anderson_3d` + `lanczos` | hotSpring 3D Anderson → barracuda S59 | `barracuda-gpu` |
| `freeze_out::chi2_analysis` | `stats::chi2::chi2_decomposed_weighted` | hotSpring nuclear fit quality → barracuda S62 | `barracuda` (CPU fallback available) |

### 1.2 New Modules

| Module | Purpose | barracuda Dependency |
|--------|---------|---------------------|
| `esn` | Echo State Network regime classification (Extended/Critical/Localized) | `barracuda::esn_v2::ESN` (GPU), rule-based CPU fallback |
| `lanczos` | Sparse eigensolver for CSR matrices from 2D/3D Anderson | `barracuda::spectral::lanczos`, `lanczos_eigenvalues` |

### 1.3 Cross-Spring Shader Provenance

| Shader/Op | Origin Spring | Path Through ToadStool | groundSpring Use |
|-----------|--------------|----------------------|-----------------|
| `spmv_csr_f64.wgsl` | hotSpring spectral → S59 | Sparse matrix-vector for Lanczos iteration | `lanczos::sparse_eigenvalues` |
| `anderson_2d.wgsl` | hotSpring Anderson → S59 | 2D Anderson Hamiltonian CSR generation | `anderson::anderson_2d_eigenvalues` |
| `anderson_3d.wgsl` | hotSpring Anderson → S59 | 3D Anderson Hamiltonian CSR generation | `anderson::anderson_3d_eigenvalues` |
| `anderson_sweep.wgsl` | hotSpring precision → S59 | Disorder-averaged Lyapunov sweep | `anderson::disorder_sweep` |
| `chi2_decomposed.wgsl` | hotSpring nuclear → S62 | Per-datum chi-squared contributions | `freeze_out::chi2_analysis` |
| `esn_reservoir.wgsl` | wetSpring ESN → S64 | Echo state network reservoir update | `esn::EsnClassifier` |
| `esn_readout.wgsl` | wetSpring ESN → S64 | ESN readout layer with Tikhonov | `esn::EsnClassifier::train` |

---

## Part 2: Deep-Debt Audit Results (V58)

### 2.1 Comprehensive Audit Findings

| Category | Count | Status |
|----------|-------|--------|
| `unsafe` blocks | 0 | Clean |
| Production `unwrap()` | 0 | Clean |
| Production `panic!()` | 0 | Clean |
| Production `assert!`/`assert_eq!` | 14 | All documented with `/// # Panics` |
| Production mocks/stubs | 0 | Clean |
| `Vec<Vec<...>>` | 0 | Clean |
| TODO/FIXME/HACK/XXX in src | 0 | Clean |
| Primal names in code logic | 0 | Runtime discovery only |

### 2.2 Hardcoding → Agnostic Evolution

- `biomeos::FAMILY_ID` made public; all `nestgate.rs` key generation and RPC params
  now derive from this constant (9 hardcoded literals eliminated)
- Socket discovery is fully runtime: env var → XDG → Linux fallback → temp fallback
- No compile-time coupling to other primals in any production code path

### 2.3 DRY Refactoring

- `merge_compute_params()` extracted in `biomeos.rs` (deduplicates `compute_execute`
  and `compute_submit`)
- `direct_rpc_call()` simplified to delegate to `capability_call()` (12 lines of
  duplicate request formatting removed)

### 2.4 External Dependencies

| Dependency | Type | Assessment |
|-----------|------|-----------|
| `wgpu` 22 | GPU compute | Required; no alternative |
| `pollster` 0.4 | Blocking async | Minimal; appropriate for simple GPU blocking |
| `bytemuck` 1 | Safe buffer casting | Safer than manual `unsafe` casts |
| `serde_json` 1.0 | JSON parsing | Required for validation; standard |
| `proptest` 1 | Property testing (dev) | No std alternative |
| `tempfile` 3.26 | Temp files (dev) | Standard; test-only |

All dependencies are necessary, minimal, and cannot be replaced with pure-Rust std equivalents.

---

## Part 3: Barracuda Evolution Recommendations

### 3.1 For ToadStool to Absorb

groundSpring has validated these patterns that could benefit barracuda:

1. **FAMILY_ID pattern**: Springs should have a single `const FAMILY_ID` that all IPC
   and provenance key generation references. barracuda could expose this as a trait.

2. **CPU fallback pattern**: All barracuda-gpu functions in groundSpring have CPU
   fallbacks. The pattern is:
   ```rust
   #[cfg(feature = "barracuda-gpu")]
   { if let Some(r) = gpu_path() { return r; } }
   #[cfg(feature = "barracuda")]
   if let Ok(r) = barracuda::cpu_path() { return r; }
   local_cpu_fallback()
   ```
   ToadStool could provide a `dispatch!` macro for this.

3. **Chi2Analysis struct**: groundSpring wraps `chi2_decomposed_weighted` into a
   `Chi2Analysis` struct with derived quantities (per_dof, p_value). barracuda could
   provide this struct natively.

4. **ESN regime classification**: The rule-based `classify_by_spacing_ratio` and
   `classify_by_lyapunov` functions provide fast CPU-only classification alongside
   the GPU ESN. barracuda could provide both paths.

### 3.2 Remaining Evolution Candidate

| Function | barracuda Status | Gap |
|----------|-----------------|-----|
| `band_structure::find_band_edges` | `spectral::band_edges_parallel` exists | Algorithm mismatch: groundSpring uses transfer matrix; barracuda uses eigenvalue threshold. Parity proven for simple cases but diverges for complex potentials. |

### 3.3 What groundSpring Learned for barracuda

1. **Anderson 2D/3D requires Lanczos**: Direct diagonalization is O(N³); Lanczos gives
   the first k eigenvalues in O(N·k) via SpMV. The `SpectralCsrMatrix` → `lanczos`
   pipeline is the right abstraction.

2. **ESN needs both GPU and CPU paths**: The `EsnClassifier` (GPU) is for training/bulk
   classification; rule-based `classify_by_spacing_ratio` (CPU) is for single-point
   decisions in sweep analysis. Both are needed.

3. **Chi2 decomposition is universally useful**: Every spring that does fitting (hotSpring
   nuclear, groundSpring freeze-out, airSpring ET₀) benefits from per-datum chi-squared
   contributions. Consider promoting to a first-class barracuda type.

4. **`pollster::block_on` is the right pattern for sync API over async GPU**: All
   barracuda-gpu functions in groundSpring use `pollster::block_on`. This avoids forcing
   async on the caller while still using wgpu's async API internally.

---

## Part 4: Complete Delegation Inventory (61 Active)

### Tier A — Lean (CPU delegated, 38 functions)

| # | groundSpring Function | barracuda Function |
|---|----------------------|-------------------|
| 1–38 | See `specs/BARRACUDA_EVOLUTION.md` V57 for complete table |

### Tier A — Lean (GPU dispatched, 19 functions)

| # | groundSpring Function | barracuda Op/Kernel |
|---|----------------------|-------------------|
| 39–57 | See `specs/BARRACUDA_EVOLUTION.md` V57 for complete table |

### Tier A — Cross-Spring S59+ (4 functions)

| # | Function | barracuda API | Lineage |
|---|---------|---------------|---------|
| 58 | `anderson::disorder_sweep` | `spectral::anderson_sweep_averaged` | hotSpring → S59 |
| 59 | `anderson::anderson_2d_eigenvalues` | `spectral::anderson_2d` + `lanczos` | hotSpring → S59 |
| 60 | `anderson::anderson_3d_eigenvalues` | `spectral::anderson_3d` + `lanczos` | hotSpring → S59 |
| 61 | `freeze_out::chi2_analysis` | `stats::chi2::chi2_decomposed_weighted` | hotSpring → S62 |

---

## Part 5: Quality State

| Gate | Status |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy -- -D warnings` | 0 warnings |
| `cargo clippy -W clippy::pedantic` | 0 warnings |
| `cargo doc --no-deps` | clean |
| `cargo test -p groundspring` | 486 PASS |
| `cargo test -p groundspring-validate` | 33 binaries, all PASS |
| `cargo test -p groundspring-forge` | 85 PASS |
| Workspace total | 613 PASS |
| `unsafe` blocks | 0 |
| Production mocks | 0 |
| Production `unwrap` | 0 |
| TODO/FIXME | 0 |

---

*groundSpring V58 — Cross-Spring Evolution + Deep-Debt Completion — March 1, 2026*
