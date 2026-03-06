# groundSpring V87 — barraCuda/toadStool Evolution Request

**Date**: March 6, 2026
**From**: groundSpring V87
**To**: barraCuda team, toadStool team, coralReef team
**Purpose**: Handoff of learnings, evolution requests, and absorption opportunities
**Pins**: barraCuda `e1184f3`, toadStool S96c (`d77fc546`), coralReef `849fedd`

---

## Executive Summary

groundSpring's Tier B delegation is fully resolved (93 active: 56 CPU + 37 GPU).
This handoff documents:
1. What groundSpring learned that benefits barraCuda evolution
2. Specific GPU pipeline issues needing investigation
3. Cross-spring shader patterns worth generalizing
4. PRNG alignment roadmap for Phase 2b

---

## 1. GPU Pipeline Investigation Request (Critical)

### 1.1 f64 Reduce Operations Return 0

**Symptom**: All f64 GPU reduce operations (sum, variance, bootstrap mean) return 0
on RTX 4070 (Ada Lovelace, consumer GPU).

**What we know**:
- `SumReduceF64`, `VarianceReduceF64`, `BootstrapMeanGpu` all return 0
- Both native f64 shaders AND DF64 variants return 0
- The DF64 wiring itself is architecturally correct (V86 confirmed)
- `BootstrapMeanGpu` doesn't use `var<workgroup> array<f64>`, so the issue isn't
  workgroup shared memory alone
- coralReef sovereign compilation produces valid SM70/SM89 SPIR-V (V85 confirmed)
- The issue is in `compile_shader_f64()` → shader module creation path

**Hypothesis**: The `compile_shader_f64()` function strips `enable f64;` and
passes through `ShaderTemplate::for_driver_profile()` which may be applying
transformations that break f64 arithmetic. The naga → SPIR-V pipeline may also
have issues with f64 buffer reads/writes on consumer hardware.

**Reproduction**:
```bash
cargo test --features gpu -p barracuda -- sum_reduce
cargo test --features gpu -p barracuda -- ops::variance_f64_wgsl::tests
```

**Impact**: Blocks all f64 GPU operations on consumer hardware. DF64 wiring (V86)
was designed to work around `var<workgroup> array<f64>` failures, but the root
cause is deeper.

### 1.2 Request: Minimal f64 GPU Diagnostic

A minimal shader that writes `42.0_f64` to a buffer and reads it back would
isolate whether f64 I/O or f64 arithmetic is the failure mode. If I/O works but
arithmetic doesn't, the issue is in naga's f64 instruction emission. If I/O
fails, the issue is in buffer layout or binding.

---

## 2. Learnings for barraCuda Evolution

### 2.1 `multinomial_sample_cpu` Signature

groundSpring successfully delegates to `barracuda::ops::bio::multinomial_sample_cpu`
using a closure RNG adapter:

```rust
let mut rng = Xorshift64::new(seed);
let counts = barracuda::ops::bio::multinomial_sample_cpu(
    &cumulative_probs,
    depth_u32,
    &mut || rng.next_f64(),
);
```

The closure-based RNG API is flexible — it lets callers inject any PRNG without
barraCuda needing to know about it. This pattern works well and should be
maintained in future bio op APIs.

### 2.2 CPU-by-Design Recognition

Two modules were formally identified as CPU-by-design (not delegation failures):

**`quasispecies_simulation`**: Single-locus Wright-Fisher model with per-generation
mutation thinning. The mutation step (binomial by Q=(1-μ)^L) requires a GPU→CPU
round-trip per generation. For O(N) scalar operations, GPU dispatch overhead
dominates. GPU is valuable only for the multi-locus, multi-population case
(pangenome selection, meta-population dynamics) — which `WrightFisherGpu` already
handles.

**`band_structure` coarse scan**: Evaluates transfer matrix half-trace at n_points
energies — data-dependent sequential matrix chains. Not expressible in current
barraCuda ops. The Brent refinement (via `barracuda::optimize::brent`) IS
delegated and provides the high-value precision improvement.

**Pattern**: Not all scientific computations benefit from GPU. The right delegation
decision considers per-dispatch data volume, not just algorithmic parallelism.

### 2.3 PRNG Alignment Status

| Context | PRNG | State Bits |
|---------|------|------------|
| groundSpring CPU reference | Xorshift64 | 64 |
| groundSpring GPU alignment | Xoshiro128StarStar | 256 |
| barraCuda `anderson_potential` | LcgRng | varies |
| barraCuda GPU shaders | xoshiro128** (WGSL) | 128 |

groundSpring delegates `anderson_potential` to barraCuda (LcgRng) and documents
the PRNG divergence. For full Phase 2b alignment, barraCuda could expose a
configurable PRNG for `anderson_potential` — or groundSpring could adopt LcgRng
for its own CPU reference. The second option is simpler.

### 2.4 Tolerance Architecture

groundSpring uses a 13-tier tolerance architecture (V73). wetSpring extended this
to 164 tiers. The pattern is:

```rust
pub mod tol {
    pub const EXACT: f64 = 1e-15;        // bitwise-identical ops
    pub const ANALYTICAL: f64 = 1e-12;   // closed-form solutions
    pub const NUMERICAL: f64 = 1e-10;    // iterative solvers
    pub const LITERATURE: f64 = 1e-3;    // published reference values
    // ... up to NORM_2PCT = 0.02 for stochastic convergence
}
```

barraCuda's `ValidationHarness` could adopt named tolerance tiers instead of
ad-hoc epsilon values.

---

## 3. Cross-Spring Absorption Opportunities

### 3.1 groundSpring Patterns Worth Absorbing

| Pattern | Where | Value |
|---------|-------|-------|
| `if let Ok` + CPU fallback | All 56 CPU delegations | Graceful degradation without silent failure |
| `#[cfg(feature)]` / `#[cfg(not(feature))]` mutual exclusion | All modules | Zero dead-code warnings, zero `#[allow(unreachable_code)]` |
| Tolerance tiers (`tol::EXACT` .. `tol::NORM_2PCT`) | `crates/groundspring/src/tol.rs` | Named, documented, per-operation tolerances |
| Feature-gated test parity | `cargo test` + `cargo test --features barracuda` | Both modes green — delegation doesn't break tests |

### 3.2 Shader Evolution Notes

| Shader | groundSpring Context | barraCuda Context |
|--------|---------------------|-------------------|
| `batched_multinomial_f64.wgsl` | Rare biosphere + rarefaction | Bio module — cross-spring from wetSpring |
| `wright_fisher_step_f64.wgsl` | Quasispecies (CPU-by-design) | Bio module — multi-locus for pangenome |
| `band_edges_parallel_f64.wgsl` | NOT used (algorithm mismatch) | Grid module — for pre-computed eigenvalues |
| `sum_reduce_f64_via_df64.wgsl` | Reduce (returns 0 — pipeline issue) | Reduce module — DF64 consumer GPU path |
| `variance_reduce_f64_via_df64.wgsl` | Reduce (returns 0 — pipeline issue) | Reduce module — Welford DF64 path |

### 3.3 coralReef Integration Notes

coralReef's sovereign compiler (V85) successfully compiles f64 reduction shaders
to native SM70/SM89 SPIR-V. The compiled shaders are structurally valid but
the execution path through `compile_shader_f64()` still produces zero results.
This suggests the issue may be in how the compiled shader module is passed to
wgpu — not in the shader compilation itself.

---

## 4. Delegation Inventory

### 4.1 Complete (93 Active)

56 CPU delegations + 37 GPU delegations. Full inventory in
`specs/BARRACUDA_EVOLUTION.md` Tier A table.

### 4.2 Remaining Tier B

| Module | Status | Notes |
|--------|--------|-------|
| `prng::Xorshift64` | Phase 2b | Align to xoshiro128** or adopt LcgRng |
| `band_structure` coarse scan | CPU by design | Data-dependent matrix chains |
| `quasispecies_simulation` | CPU by design | Per-gen mutation overhead |

### 4.3 Future GPU Opportunities

| Op | barraCuda Target | Blocker |
|----|-----------------|---------|
| `quasispecies` mutation kernel | Combined selection+mutation shader | Needs new WGSL kernel |
| `anderson_potential` GPU | GPU PRNG-based potential generation | Low priority (O(N) trivial) |
| Transport eigenvectors | Dense Jacobi GPU | `eigh_f64` exists but not wired |

---

## 5. Benchmark Data

### 5.1 CPU Delegation Performance (V79)

28-binary validation suite, release mode, i9-12900K:

| Mode | Time | Ratio |
|------|------|-------|
| Default (no barracuda) | 21.7s | 1.0× |
| barraCuda CPU delegation | 19.6s | **1.11×** (−10% faster) |

Notable: FAO-56 −78%, spectral-recon −81%, freeze-out −67%, jackknife −57%.

### 5.2 Rust vs Python (28 experiments)

| Metric | Value |
|--------|-------|
| Total Python | 104.49s |
| Total Rust | 20.35s |
| Overall speedup | **5.1×** |
| Best: Seismic | **53.5×** |

---

## 6. What groundSpring Needs Next from barraCuda

1. **GPU f64 pipeline fix** — Critical. Blocks all f64 GPU operations on consumer hardware.
2. **PRNG alignment decision** — Should groundSpring adopt LcgRng, or should barraCuda
   expose configurable PRNG in `anderson_potential`?
3. **Combined WF+mutation kernel** — Would enable GPU quasispecies for batched replicates
   (currently CPU-by-design due to per-generation round-trip).

---

## 7. Files Reference

| Document | Path |
|----------|------|
| Full delegation table | `specs/BARRACUDA_EVOLUTION.md` |
| Cross-spring evolution | `wateringHole/CROSS_SPRING_SHADER_EVOLUTION.md` |
| V87 handoff | `wateringHole/handoffs/GROUNDSPRING_V87_TIER_B_RESOLUTION_HANDOFF_MAR06_2026.md` |
| Tolerance tiers | `crates/groundspring/src/tol.rs` |
| GPU dispatch wiring | `crates/groundspring/src/gpu.rs` |
| Python baselines | `control/baseline_runner.py` |
| Benchmark scripts | `scripts/sovereign_pipeline_benchmark.py`, `scripts/full_stats_benchmark.py` |
