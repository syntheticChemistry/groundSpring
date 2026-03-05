# groundSpring V80 — Fused Ops Rewire + barraCuda/ToadStool Catch-Up

**Date**: March 5, 2026
**Supersedes**: V79 (Exp 035 Multi-Method ET₀ + Delegation Strengthening)
**barraCuda pin**: v0.3.3+ (`15d3774`)
**toadStool pin**: S94b (`9d359814`)
**Tests**: 812 workspace + 390 Python = 1202
**Delegations**: 87 active (51 CPU + 36 GPU)
**Validation**: 395/395 PASS across 34 binaries (340 core + 55 NUCLEUS)

---

## What Changed

### 1. Fused `correlation_full` GPU Dispatch (NEW)

Wired barraCuda's `CorrelationF64::correlation_full()` — a 5-accumulator
single-pass GPU shader that returns mean_x, mean_y, var_x, var_y, and
Pearson r in one kernel launch (no intermediate readbacks).

New public API:

- `stats::pearson_full(x, y) -> CorrelationFull` — full statistics from one dispatch
- `stats::CorrelationFull` struct — mirrors barraCuda's `CorrelationResult`
- `stats::covariance()` — GPU path now derives covariance from `correlation_full`
  (was CPU-only via `barracuda::stats::correlation::covariance`)

Delegation chain:
```
pearson_full() → GPU: CorrelationF64::correlation_full → correlation_full_f64.wgsl
                                                        (or correlation_full_df64.wgsl on consumer GPUs)
                  CPU: Single-pass Welford co-moment algorithm
```

### 2. Welford Single-Pass CPU Stats (EVOLUTION)

Replaced two-pass mean-then-variance CPU fallback with Welford's online
algorithm. Affects `std_dev`, `sample_std_dev`, and `mean_and_std_dev` CPU
paths. Same results, one pass through data, numerically stable.

Before: `mean()` → iterate once; `variance = Σ(x-m)²/N` → iterate again.
After: `welford_population()` → single pass, online mean + M2 accumulation.

No API change. CPU fallback only (GPU path already used fused Welford shader).

### 3. barraCuda Catch-Up to HEAD (`15d3774`)

groundSpring's path dependency automatically tracks barraCuda HEAD.
Since V79 (`4629bdd`), barraCuda gained:

| Commit | Change |
|--------|--------|
| `15d3774` | `chi_squared`: gate `WORKGROUP_SIZE_1D` behind `gpu` feature |
| `4629bdd` | DF64 naga rewriter: NAK compound assignment fix |
| `7797d90` | DF64 precision tier: 15 ops gain `Fp64Strategy` selection |
| `66e2774` | Subgroup detection + 10 more ops to TensorContext |
| `0b6ebef` | Fused reduction shaders + TensorContext stats migration |

All 812 tests pass with barraCuda HEAD. No code changes required —
free upgrades via path dependency.

### 4. ToadStool S94b Review

ToadStool S94b completed full primal decoupling. Key findings:

- barraCuda is fully standalone (extracted from toadstool S89)
- Embedded `crates/barracuda/` removed, fossilized to `ecoPrimals/fossil/`
- DF64 precision ownership transferred to barraCuda (S93)
- groundSpring's V68 work fully absorbed: L-BFGS GPU, tridiag eigenvectors,
  anderson_4d, wegner_block_4d, SeasonalGpuParams, RAWR, metalForge topology

No action needed — groundSpring already correctly depends on standalone barraCuda.

### 5. coralNAK Ecosystem Awareness

New ecosystem primal at `ecoPrimals/coralNAK/` — sovereign Rust NVIDIA
shader compiler (Mesa NAK fork). Phase 2: NAK sources wired, 17 compile
errors remaining. groundSpring is assigned Level 4 work (userspace GPU
driver, memory management, command buffer builder) per
`wateringHole/SOVEREIGN_COMPUTE_EVOLUTION.md`.

No immediate action — coralNAK is in early phases.

---

## Pipeline Status for ToadStool/BarraCUDA Team

| Stage | Status | Delegations |
|-------|--------|-------------|
| **Python baseline** | 29 experiments, 390 checks PASS | — |
| **Rust validation** | 34 binaries, 395/395 PASS | — |
| **barracuda CPU** (pure Rust math) | 51 CPU delegations, 11.6× faster than Python | 51 |
| **barracuda GPU** (portable math) | 36 GPU delegations, 25 of 34 papers wired | 36 |
| **Pure GPU workload** | metalForge 30 workloads (24 GPU + 2 NPU + 2 CPU-only) | — |
| **metalForge cross-system** | 187 checks PASS, PCIe topology validated | — |

### What ToadStool/BarraCUDA Should Absorb

1. **`correlation_full` Welford co-moment CPU**: The single-pass CPU algorithm
   in `stats::correlation::pearson_full_cpu` could be promoted to `barracuda::stats`
   as a CPU reference implementation for `correlation_full`.

2. **ET₀ multi-method batch dispatch** (from V79): Batch GPU dispatch for all 5
   ET₀ methods via `BatchedElementwiseF64` is still pending.

3. **Covariance from correlation_full**: The GPU covariance derivation
   (r × √(var_x × var_y) × N/(N-1)) could be formalized in barraCuda as a
   `covariance_gpu` op that reuses the `correlation_full` shader.

---

## Cross-Spring Evolution Notes

### Shader Provenance (new in V80)

| Shader/Op | Origin | Version | groundSpring Use |
|-----------|--------|---------|-----------------|
| `CorrelationF64::correlation_full` | hotSpring DF64 | barraCuda v0.3.3+ | `stats::pearson_full`, `stats::covariance` (GPU) |
| `correlation_full_df64.wgsl` | hotSpring DF64 | barraCuda v0.3.3+ | Auto-selected on consumer GPUs via `Fp64Strategy::Hybrid` |

### What groundSpring Learned (for other springs)

- **Welford co-moment is the right CPU pattern**: Single-pass, numerically stable,
  mirrors the GPU shader algorithm. Springs should use this instead of two-pass
  mean-then-variance.

- **`correlation_full` GPU eliminates multi-dispatch stats**: Instead of
  `mean(x)` + `mean(y)` + `correlation(x,y)` (3 dispatches), `correlation_full`
  gives everything in one. This is material for large arrays.

- **Path dependency = free upgrades**: barraCuda's DF64 precision tiers,
  TensorContext, and chi_squared fix all inherited automatically with zero
  groundSpring code changes.

---

## Quality Gates

- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `cargo doc --workspace --no-deps`: PASS
- `cargo test --workspace`: PASS (812 tests)
- Python Phase 0: 29 experiments, 390 checks PASS
- Rust Phase 1: 34 binaries, 395/395 PASS
- Zero TODO/FIXME/unsafe/unwrap in production
- All files < 1000 lines
- Deep debt zero maintained

---

## Files Changed (V80)

| File | Change |
|------|--------|
| `crates/groundspring/src/stats/correlation.rs` | `CorrelationFull`, `pearson_full()`, `covariance_gpu()` via `correlation_full` |
| `crates/groundspring/src/stats/metrics.rs` | `welford_population()`, single-pass CPU for `std_dev`, `sample_std_dev`, `mean_and_std_dev` |
| `crates/groundspring/src/stats/mod.rs` | Export `CorrelationFull`, `pearson_full` |
| `CHANGELOG.md` | V80 entry |
| `README.md` | V80 status line |
| `wateringHole/handoffs/` | New V80 handoff, V79 archived |
