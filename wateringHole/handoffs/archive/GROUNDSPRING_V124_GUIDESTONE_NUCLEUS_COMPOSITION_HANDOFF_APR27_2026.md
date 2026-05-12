# groundSpring V124 — guideStone Level 3 + NUCLEUS Composition Handoff

**Date**: April 27, 2026
**From**: groundSpring V124
**To**: All primal teams, all spring teams, primalSpring, biomeOS
**barraCuda**: v0.3.12 | **toadStool**: S158+ | **coralReef**: Iteration 55+
**guideStone**: Level 3 (bare scaffold + IPC wiring)

---

## Executive Summary

groundSpring advances from guideStone Level 0 to **Level 3** — the bare
guideStone scaffold is implemented with IPC wiring to the NUCLEUS composition
API. All deploy graphs, niche YAML, and capability registrations are now
synchronized and reconciled. This handoff documents: (1) the guideStone
implementation and what it validates, (2) composition patterns discovered,
(3) gaps found and handed back, (4) evolution items per team.

**Quality certificate**: 965+ tests, 0 failures, 0 clippy warnings,
0 unsafe, 0 TODO/FIXME, 0 production mocks, 0 hardcoded primal addresses.
All tests pass. All deploy graph verbs match IPC contracts.

---

## 1. guideStone Implementation

### 1a. Binary: `groundspring_guidestone`

Located at `crates/groundspring-validate/src/groundspring_guidestone.rs`.
Built with `cargo build --bin groundspring_guidestone --features guidestone`.
Requires `primalspring` path dependency (`../../../primalSpring/ecoPrimal`).

### 1b. Bare Properties (always pass, no NUCLEUS needed)

| Property | What It Validates | Status |
|----------|-------------------|--------|
| **1. Deterministic** | `decompose_error` and `stats::mean` produce bitwise-identical results across runs | PASS |
| **2. Reference-Traceable** | `provenance_registry` entries match experiment count, niche capabilities match code | PASS |
| **3. Self-Verifying** | `validation/CHECKSUMS` BLAKE3 manifest covers 12 critical source files | PASS |
| **4. Environment-Agnostic** | No `HOME`/`USER`/`HOSTNAME` dependency; results identical with cleared env | PASS |
| **5. Tolerance-Documented** | All `tol::*` and `eps::*` constants have provenance comments and match registry | PASS |

### 1c. NUCLEUS Additive Checks (require live primals via IPC)

| Check | Capability | What It Does |
|-------|------------|-------------|
| **Scalar Parity** | `tensor` | `stats::mean` local vs IPC `tensor.reduce_mean` |
| **Vector Parity** | `tensor` | Local matmul vs IPC `tensor.matmul` |
| **Decomposition E2E** | `tensor` | `decompose_error` local vs IPC round-trip |
| **Storage Round-Trip** | `storage` | NestGate `storage.put` → `storage.get` |
| **Crypto Witness** | `security` | BearDog `crypto.hash` of guideStone output |
| **Compute Dispatch** | `compute` | ToadStool `compute.execute` for Lyapunov |

When no NUCLEUS primals are discovered, the binary reports bare-only
certification and exits with a skip-aware code.

---

## 2. Composition Patterns Discovered

### 2a. Capability Discovery with Fallback

```rust
let ctx = CompositionContext::from_live_discovery_with_fallback();
let alive = validate_liveness(&mut ctx, &mut v, &["tensor", "compute", "storage", "security"]);
```

The `from_live_discovery_with_fallback()` pattern is the correct way to
build a composition context — it tries live Songbird discovery, then falls
back to `FAMILY_ID`-derived socket paths, then to environment variables.
Springs should never hardcode socket paths.

### 2b. Parity Validation Pattern

```rust
validate_parity(&mut ctx, &mut v, "stats.mean", local_result, &params, tol);
validate_parity_vec(&mut ctx, &mut v, "tensor.matmul", &local_vec, &params, tol);
```

This is the canonical way to validate that local Rust computation matches
IPC composition — call the same operation both ways and compare within
tolerance. The `primalspring` API handles serialization, IPC, and
tolerance comparison.

### 2c. Skip-Aware Exit Codes for NUCLEUS

The guideStone uses `exit_code_skip_aware()` which returns:
- **0**: all checks pass
- **1**: at least one check failed
- **2**: no primals discovered (bare certification only)

CI should treat exit 2 as "skip" not "fail" for NUCLEUS checks.

### 2d. Deploy Graph → Code Alignment Discipline

We found 4 verb mismatches between deploy graphs and actual IPC contracts
(see §4). Springs should validate that every `capability` field in their
TOML graphs corresponds to an actual method in their dispatch table.

---

## 3. For primalSpring (Upstream)

### 3a. Gaps Handed Back (docs/PRIMAL_GAPS.md)

| GAP | Primal | Severity | Status |
|-----|--------|----------|--------|
| GAP-GS-001 | Squirrel | Low | Deferred (AI additive, not required) |
| GAP-GS-002 | coralReef | Low | Deferred (shader compiler API not stable) |
| GAP-GS-003 | barraCuda | Low | Deferred (TensorSession not stable) |
| GAP-GS-006 | self | Low | Active (metalForge tolerance duplication) |
| GAP-GS-008 | BearDog | Medium | Blocked upstream (ionic bonding not implemented) |
| GAP-GS-009 | barraCuda | Medium | Blocked upstream (BTSP session crypto) |
| GAP-GS-011 | barraCuda | Low | Deferred (PRNG rebaseline) |

### 3b. Resolved Gaps

| GAP | Resolution |
|-----|-----------|
| GAP-GS-004 | Niche YAML synced from 8 → 16 capabilities |
| GAP-GS-005 | All deploy graph verbs fixed to match IPC contracts |
| GAP-GS-007 | barraCuda version refs updated to v0.3.12 |
| GAP-GS-010 | `compute_capabilities()` fixed to call `compute.capabilities` |

### 3c. Downstream Manifest Alignment

`downstream_manifest.toml` entry for groundSpring lists 6 validation_capabilities:
`tensor.matmul`, `stats.mean`, `compute.dispatch`, `storage.store`,
`storage.retrieve`, `crypto.hash`.

**Issue**: `storage.store` should be `storage.put` (matches NestGate API),
`storage.retrieve` should be `storage.get`. Please update the manifest.

---

## 4. For All Spring Teams

### 4a. guideStone Scaffold Pattern

If you're at guideStone Level 0, use groundSpring's implementation as a
template. Key files:

| File | Purpose |
|------|---------|
| `crates/groundspring-validate/src/groundspring_guidestone.rs` | guideStone binary (~500 LOC) |
| `crates/groundspring-validate/Cargo.toml` | `guidestone` feature flag + `primalspring` dependency |
| `validation/CHECKSUMS` | BLAKE3 manifest for self-verification |
| `scripts/generate-checksums.sh` | Manifest generation |
| `scripts/build-guidestone.sh` | Build + run helper |

### 4b. Deploy Graph Reconciliation Checklist

We found these verb mismatches — check your own graphs:

| Wrong Verb | Correct Verb | Why |
|------------|-------------|-----|
| `measurement.validate_suite` | Does not exist | Use actual dispatch methods |
| `measurement.parity_report` | Does not exist | Use `measurement.uncertainty_budget` |
| `storage.store` | `storage.put` | NestGate API |
| `registry.register` | `capability.register` | Songbird API |

### 4c. Niche YAML Sync Checklist

Ensure your niche YAML `capabilities:` list matches your `niche.rs`
`CAPABILITIES` array exactly. We found 8 capabilities missing from YAML
that were present in code. biomeOS uses the YAML for deployment.

### 4d. Hardcoding Elimination Patterns

| Pattern | Before | After |
|---------|--------|-------|
| Plausibility ranges | `(0.0..=15.0).contains(&v)` | `(ET0_PLAUSIBLE_MIN_MM..=ET0_PLAUSIBLE_MAX_MM).contains(v)` |
| Default ports | `format!("http://{h}:8090")` | `format!("http://{h}:{NESTGATE_DEFAULT_PORT}")` |
| Heuristic multipliers | `boot_widths[0] * 1.1` | `boot_widths[0] * CONVERGENCE_FACTOR_GAUSSIAN` |
| Capability calls | `"resource.health.check"` | `"compute.capabilities"` (match actual API) |

---

## 5. For barraCuda / toadStool Team

### 5a. Version Reference Update

All active groundSpring specs and graphs now reference barraCuda v0.3.12.
Historical tolerance pins in `tol.rs` retain their original v0.3.7
annotations (documenting when contracts were established).

### 5b. `default-features = false` Alignment

metalForge forge Cargo.toml now uses `default-features = false` for
barracuda (ecoBin compliance). Other springs should adopt this pattern.

### 5c. Evolution Priorities (unchanged from V123)

| Priority | What | Why |
|----------|------|-----|
| P0 | PRNG alignment (xoshiro128**) | Baseline regeneration blocked |
| P0 | Sparse SpMV for Anderson 2D/3D | Lanczos at scale |
| P1 | Named tolerances in barraCuda test suite | Match spring contract pin pattern |
| P2 | TensorSession stabilization | Multi-op fusion for measurement pipelines |

---

## 6. For biomeOS Team

### 6a. Neural API Deployment Patterns

groundSpring's guideStone validates the full NUCLEUS composition via:

1. `validate_liveness` — capability-based health check of tensor/compute/storage/security
2. `validate_parity` — scalar IPC round-trip comparison
3. `validate_parity_vec` — vector IPC round-trip comparison
4. `call` / `call_f64` / `hash_bytes` — direct capability invocation

This exercises the biomeOS routing layer end-to-end. The patterns are
reusable by any spring implementing guideStone Level 3+.

### 6b. Socket Discovery Chain

The validated discovery order is:
1. `FAMILY_ID`-qualified sockets (`{capability}-{family}.sock`)
2. Plain capability sockets (`{capability}.sock`)
3. Environment variable overrides (`NESTGATE_URL`, etc.)
4. biomeOS socket registry (`socket-registry.json`)

Protocol tolerance: primals responding with HTTP framing on UDS are
classified as reachable-but-incompatible (SKIP, not FAIL).

---

## 7. Quality Certificate

| Metric | Value |
|--------|-------|
| `cargo check --workspace` | PASS (0 warnings) |
| `cargo clippy --workspace` | PASS (0 warnings) |
| `cargo test --workspace` | PASS (965+ tests, 0 failures) |
| Validation checks | 395/395 PASS |
| metalForge checks | 140/140 PASS |
| Math parity | 29/29 PROVEN |
| guideStone Level | **3** (bare + IPC wired) |
| Measurement capabilities | 16 (synced YAML/graph/code) |
| Deploy graph verb mismatches | **0** (4 fixed) |
| Hardcoded primal addresses | **0** |
| Production mocks | **0** |
| `unsafe` in production | **0** (`#![forbid(unsafe_code)]`) |
| `#[allow]` in production | **0** |
| TODO/FIXME in production | **0** |
| Library coverage | ≥92% |
| barraCuda version | v0.3.12 |

---

## 8. Next Steps (groundSpring → Level 4)

1. **Deploy NUCLEUS from plasmidBin** — `nucleus_launcher.sh` with `FAMILY_ID=groundspring-validation`
2. **Run guideStone externally** — `./groundspring_guidestone` against live primals
3. **Achieve 6/6 NUCLEUS additive checks** — scalar parity, vector parity, decomposition, storage, crypto, compute
4. **Document any primal misbehavior** — hand back to primalSpring via `docs/PRIMAL_GAPS.md`
5. **Certify Level 4** — all bare + all NUCLEUS checks pass against live composition
