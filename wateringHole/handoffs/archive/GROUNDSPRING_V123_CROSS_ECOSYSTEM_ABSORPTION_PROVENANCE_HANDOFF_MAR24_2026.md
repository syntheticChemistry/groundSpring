# groundSpring V123 → Cross-Ecosystem Absorption + Provenance Handoff

**Date**: March 24, 2026
**From**: groundSpring V123
**To**: barraCuda team, toadStool team, all springs
**Pins**: barraCuda v0.3.7, toadStool S158+, coralReef Iteration 55+

---

## Summary

groundSpring V123 completes a **cross-ecosystem absorption and provenance
hardening** cycle. Pulled and reviewed all 7 sibling springs, 10 primals,
barraCuda v0.3.7, and coralReef Iter 63 — synthesized absorption
opportunities and executed the high-value items.

---

## 1. Ecosystem Standards Absorbed

| Standard | Source | What we absorbed |
|----------|--------|-----------------|
| `SECURITY.md` | neuralSpring S174, healthSpring V42 | Vulnerability reporting, audit posture, data provenance doc |
| `rustfmt.toml` | neuralSpring S173 | Edition 2024, max_width 100, consistent formatting |
| Upstream contract tolerance pins | neuralSpring V124 | 6 constants binding to barraCuda v0.3.7 behaviour |
| Provenance registry | neuralSpring S174 (49 baselines) | `provenance_registry.rs` — 29 baselines, typed lookups |
| Bitwise determinism tests | healthSpring V42 | 5 new PRNG identity assertions |
| `CastOverflowError` | ecosystem trend | Typed error replacing `Result<u32, String>` |

---

## 2. Upstream Contract Tolerance Pins (new pattern)

Six new constants in `tol.rs` that pin groundSpring to specific barraCuda
v0.3.7 APIs. If upstream changes its numerical behaviour, the pinned
validation tests will fail immediately — early detection of contract drift.

| Pin | barraCuda API | Tolerance | Validated by |
|-----|--------------|-----------|-------------|
| `UPSTREAM_SPECTRAL_EIGH` | `spectral::find_all_eigenvalues` | `ANALYTICAL` (1e-10) | `validate_anderson`, `validate_transport` |
| `UPSTREAM_BIO_MULTINOMIAL` | `ops::bio::multinomial_sample_cpu` | `DETERMINISM` (1e-15) | `validate_rarefaction`, `validate_rare_biosphere` |
| `UPSTREAM_OPTIMIZE_BRENT` | `optimize::brent` | `ANALYTICAL` (1e-10) | `validate_fao56`, `validate_et0_anderson` |
| `UPSTREAM_SPECTRAL_ANDERSON` | `spectral::anderson_sweep_averaged` | `LITERATURE` (0.001) | `validate_anderson`, `validate_uncertainty_bridge` |
| `UPSTREAM_SPECTRAL_ALMOST_MATHIEU` | `spectral::almost_mathieu_hamiltonian` | `EXACT` (1e-12) | `validate_quasiperiodic` |
| `UPSTREAM_SPECTRAL_BANDS` | `spectral::detect_bands` | `ANALYTICAL` (1e-10) | `validate_band_edge` |

**Recommendation for all springs**: adopt this pattern. Pin your critical
upstream dependencies to explicit tolerance values so you detect drift at
test time, not in production.

---

## 3. Provenance Registry (new module)

`groundspring::provenance_registry` — centralized source of truth for all
29 Python baselines:

```rust
pub struct BaselineEntry {
    pub exp_id: u32,
    pub name: &'static str,
    pub script: &'static str,
    pub benchmark_json: &'static str,
    pub validator: &'static str,
    pub domain: &'static str,
}
```

Lookups: `by_exp_id()`, `by_name()`, `by_validator()`, `by_domain()`,
`domains()`. 14 tests ensuring uniqueness and correctness.

---

## 4. Ecosystem Review Findings

### What groundSpring is ahead on
- Cast module (V122): centralized, documented, 15 functions — sibling
  springs credit groundSpring as origin
- Zero `#[allow]` discipline (V121): first spring to reach zero
- Named tolerance system with provenance citations
- Benchmark JSON provenance (`validation_script`, `command` fields)

### What groundSpring absorbed from siblings
- neuralSpring: upstream contract pins, provenance headers, `SECURITY.md`
- healthSpring: bitwise determinism tests, metalForge forge split pattern
- wetSpring: validation harness decomposition (already had `NdjsonSink`)
- airSpring: three-tier discovery pattern (already aligned)
- ludoSpring: CI/coverage template pattern, MCP tooling reference
- primalSpring: `NdjsonSink` evolution, skip-aware exit codes reference

### Confirmed: already aligned
- `ValidationSink` / `NdjsonSink` (V121)
- `validate_all` meta-runner (V122)
- `primal_names.rs` centralization
- Capability-based discovery with env overrides
- `IpcError::is_recoverable()` (V120)
- `capability_registry.toml`

---

## 5. Audit Results

| Area | Result |
|------|--------|
| Production mocks | **Zero** — all test doubles behind `#[cfg(test)]` |
| TODO/FIXME/HACK | **Zero** in all Rust sources |
| Direct `-sys` crates | **Zero** — transitive only from wgpu/rustix |
| Hardcoded paths | **Centralized** in `primal_names.rs` with env overrides |
| Large files (>500 LOC) | **11 reviewed** — all well-structured; `rare_biosphere.rs` has optional GPU extraction (tracked) |

---

## 6. Continuing Evolution Priorities

### P0 — Critical alignment (from V122)
- PRNG alignment (xoshiro128** WGSL vs xorshift64 Rust) — open
- Lanczos at scale (N > 4096) — open

### P1 — Important
- Sparse SpMV for Anderson 2D/3D — open
- `GpuDriverProfile` deprecation — open
- Named tolerances in barraCuda test suite — open
- Cast module absorption into barraCuda — NEW (V122 handoff)
- MCP `tools.list` / `tools.call` surface — NEW (V123, Tier B)
- `rare_biosphere.rs` GPU extraction — optional (V123)

### P2 — Desired
- RAWR GPU dispatch — open
- Matrix exponentiation GPU — open
- Sobol sensitivity indices — open
- Provenance trio integration (sweetGrass/rhizoCrypt/loamSpine) — tracked

---

## 7. Quality Certificate

| Gate | Result |
|------|--------|
| `cargo check --workspace` | PASS (0 warnings) |
| `cargo clippy --workspace` | PASS (0 warnings) |
| `cargo fmt --check` | PASS (0 diffs) |
| `cargo doc --workspace` | PASS (0 warnings) |
| `cargo test --workspace` | PASS (1020+ tests, 0 failures) |
| `cargo deny check` | PASS |
| Validation checks | 395/395 PASS |
| metalForge checks | 140/140 PASS |
| Math parity | 29/29 PROVEN |
| Library coverage | ≥92% |
| `unsafe` in production | Zero (`#![forbid(unsafe_code)]`) |
| `#[allow]` in production | Zero |
| TODO/FIXME in production | Zero |
| `unwrap`/`expect` in production | Zero (workspace deny) |
| Production mocks | Zero |
| Direct `-sys` dependencies | Zero |

---

## 8. New Files

| File | Purpose | LOC |
|------|---------|-----|
| `SECURITY.md` | Security policy (ecosystem standard) | 41 |
| `rustfmt.toml` | Formatter config (ecosystem consistency) | 8 |
| `src/provenance_registry.rs` | Centralized baseline registry (29 experiments) | ~220 |

---

## 9. Delegation Inventory

**110 active delegations** (67 CPU + 43 GPU), unchanged from V122.
No new delegations — this was an absorption and hardening cycle.
