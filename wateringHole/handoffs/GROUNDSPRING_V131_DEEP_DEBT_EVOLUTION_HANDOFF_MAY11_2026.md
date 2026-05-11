# groundSpring V131 — Deep Debt Evolution Handoff

**Date**: May 11, 2026
**From**: groundSpring
**To**: All primal and spring teams
**Status**: V131 — 1,101 tests, 395/395 validation checks, 140 metalForge checks, zero clippy, zero unsafe, 35 experiments

---

## What Changed in V131

### Smart Refactoring
- **guideStone binary**: 833→128 lines. Extracted 5 NUCLEUS layer modules into `src/guidestone/{bare,tower,node,nest,cross}.rs`. Binary is now a thin orchestrator importing library-level layer validators. Largest module is 194 lines.
- **Pattern**: Springs with large validation binaries should consider this modular extraction — it makes each NUCLEUS layer independently testable and reusable.

### Bug Fixes
- **bootstrap.rs doctest**: Percentile bootstrap with 500 replicates on uniform data placed estimate outside CI. Fixed: 2,000 replicates + robust assertions. Springs maintaining doctest examples with statistical operations should verify edge cases.
- **76 script binary name fixes**: All 6 benchmark/parity scripts had hyphenated names (`validate_freeze-out`) that don't match Cargo.toml underscored names (`validate_freeze_out`). Also fixed `test_three_tier_parity.py` (29 bins now, was 27).

### Benchmark Coverage
- `bench_barracuda_cpu_vs_python.py`: 11→28 experiments. Now covers all Python-paired experiments (001–027 + 035). NPU experiment (028) separated.

### Documentation
- Kokkos parity section added to `BARRACUDA_EVOLUTION.md`: CPU parity proven (value-level), GPU parity is timing-only (PRNG seed stride mismatch, Phase 2b).
- Anderson 2021 mSystems paper clarified as "Reference" (review paper, not numbered reproduction).
- Foundation `THREAD_INDEX.toml` bug fixed: Thread 7 (Anderson) was pointing to ag targets.

---

## Per-Team Guidance

### barraCuda Team
- **Guidestone pattern**: The 5-layer modular guidestone is now the template for NUCLEUS composition validation. If you add new capabilities, the layer modules show exactly where they'd be tested.
- **PRNG Phase 2b**: GPU Kokkos parity still blocked on seed stride alignment (`base_seed + r` vs `base_seed + r * 1000` in `lyapunov_averaged`). When Phase 2b ships, update the GPU bench to use the aligned stride.
- **Eigenvector gap**: Paper 17 — eigenvalues via Sturm on GPU, eigenvectors still CPU-only. This is the last GPU-blocked kernel.

### toadStool Team
- **No new shader requirements** from V131. The 110 delegations (67 CPU + 43 GPU) remain stable.
- **Absorption ready**: All groundSpring GPU paths compile and validate against toadStool S158+.

### bearDog / songbird / skunkBat
- **skunkBat IPC module (V130)**: `src/ipc/skunkbat.rs` is live with `emit_audit_event()`, `emit_validation_event()`, `emit_certification_event()`, `try_emit_audit_event()`. Pattern follows neuralSpring exemplar.
- **Discovery**: `primal_names::roles::AUDIT = "skunkbat"` for 5-tier socket discovery.

### NestGate
- **No changes from V131**. Exp 029–032 (NUCLEUS sovereign) remain stable. `validate_nestgate_ncbi.rs` already has full env-var discovery chain (NESTGATE_URL > NESTGATE_ADDRESS > NESTGATE_HOST+PORT > biomeOS lookup).

### coralReef
- **No changes from V131**. Shader capabilities and WGSL arch queries work through existing IPC paths.

---

## For Spring Teams

### Patterns to Absorb

1. **Modular guidestone**: Extract validation layers into library modules. Binary is orchestration only. This makes layers independently testable and enables library-level composition tests.

2. **Underscore binary naming**: Rust `Cargo.toml` binary names use underscores. Any scripts or tests referencing binaries must match exactly. We had 76 stale hyphenated references — check your scripts.

3. **Benchmark coverage**: Our `bench_barracuda_cpu_vs_python.py` template now covers 28 experiments with a clean `EXPERIMENTS` list pattern. Extend the pattern for your spring's experiments.

4. **Doctest robustness**: Statistical operations in doctest examples should use sufficient replicates and test invariants (finiteness, ordering) rather than fragile containment assertions.

5. **Kokkos parity documentation**: If your spring has reference implementations (Python, C++, MATLAB), document the parity level (value vs timing) and any PRNG/algorithmic divergences.

### NUCLEUS Composition Patterns

- **CompositionContext** remains the default IPC path. `barracuda` is `optional = true` with `local` feature for development.
- **Deploy graphs**: All 6 groundSpring deploy graphs include skunkBat (non-blocking, `fallback = "skip"`).
- **plasmidBin**: Release binary (`groundspring_unibin`, 1.1MB) is built and manifest updated. NUCLEUS workload references `$SPRINGS_ROOT/groundSpring/target/release/groundspring_unibin validate`.
- **Cell membrane**: NUCLEUS validation is intracellular. `groundspring_unibin validate` is the Tier 1 entry point for NUCLEUS science validation.

---

## Foundation Seeding Status

- **Thread 6 (ag)**: 18 sources anchored in `foundation/data/sources/thread06_ag.toml`
- **Thread 7 (Anderson)**: 18 targets anchored in `foundation/data/targets/thread07_anderson_targets.toml`
- **THREAD_INDEX fix**: Thread 7 `data_targets` corrected from `thread06_ag_targets.toml` to `thread07_anderson_targets.toml` (pushed upstream)
- **BLAKE3 hashes**: Pending — populated by `fetch_sources.sh` at download time
- **Dated validation run**: Not yet created (airSpring's `validation/ag-20260511/` is the exemplar pattern)

---

## Remaining Gaps (Unchanged from V130)

| ID | Description | Severity | Blocker |
|----|-------------|----------|---------|
| GAP-GS-001 | Squirrel crate missing `groundspring` metadata | Low | Upstream (squirrel) |
| GAP-GS-002 | coralReef `shader.compile.capabilities` wiring untested live | Low | Hardware-dependent |
| GAP-GS-003 | TensorSession generic typing for cross-spring reuse | Deferred | Design decision |
| GAP-GS-008 | barraCuda `eigh_f64` eigenvectors on GPU | Medium | Upstream (barraCuda) |
| GAP-GS-009 | toadStool daemon binary name mismatch | Low | Upstream (toadStool) |
| GAP-GS-011 | PRNG Xorshift64→Xoshiro128** alignment (Phase 2b) | Medium | Coordinated migration |

---

## CI Cross-Sync

- **primalSpring canonical**: 413 methods, 0 drift
- **groundSpring validated**: 16 `measurement.*` methods + composition methods
- **Registry sync test**: `>= 401` production methods (excludes test fixtures)

---

**Next targets**: Foundation dated validation run, PRNG Phase 2b coordination, Tier 1 dataset acquisition (PRJNA315684, LTEE, NOAA CDO real data).
