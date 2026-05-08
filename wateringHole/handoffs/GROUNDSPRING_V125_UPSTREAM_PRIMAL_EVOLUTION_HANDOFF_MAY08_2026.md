# groundSpring V125 — Upstream Primal Evolution Handoff

**Date**: May 8, 2026
**From**: groundSpring V125 (guideStone Level 4)
**To**: barraCuda, BearDog, Songbird, ToadStool, NestGate, coralReef, Squirrel, primalSpring
**barraCuda**: v0.3.13 | **toadStool**: S158+ | **coralReef**: Iteration 55+

---

## Purpose

This handoff documents what groundSpring learned during its L3→L4 evolution
that is relevant to upstream primal teams. It covers: open gaps blocked on
primal evolution, patterns discovered during NUCLEUS composition wiring,
neuralAPI/biomeOS deployment learnings, and recommendations for the ecosystem.

---

## 1. Open Gaps Blocked on Upstream Primals

From `docs/PRIMAL_GAPS.md` — 6 gaps remain, 4 blocked upstream:

### GAP-GS-008: IONIC-RUNTIME Cross-Family GPU Lease (BearDog + Songbird)

groundSpring's GPU compute goes through ToadStool. For cross-family NUCLEUS
deployment (e.g. groundSpring measurement dispatched through a different family's
ToadStool), we need IONIC-RUNTIME GPU lease negotiation mediated by BearDog
(crypto) and Songbird (discovery). Currently there is no protocol for a primal
to lease GPU time from a ToadStool instance that belongs to a different family.

**Blocked on**: BearDog cross-family identity, Songbird GPU resource discovery.

### GAP-GS-009: BTSP Session Crypto for barraCuda IPC

barraCuda delegations are currently unencrypted over Unix sockets. For
production NUCLEUS deployment, all IPC should be authenticated via BTSP
(BearDog Transport Security Protocol). groundSpring is ready to adopt this
once barraCuda implements BTSP session establishment.

**Blocked on**: barraCuda BTSP integration.

### GAP-GS-001: Squirrel Not Wired

Squirrel (sovereign AI assistant) is referenced in ecosystem docs but not
wired into groundSpring's niche YAML, deploy graphs, or `CONSUMED_CAPABILITIES`.
Once Squirrel defines its capability surface, groundSpring should add it as
an optional organism.

**Blocked on**: Squirrel capability surface definition.

### GAP-GS-002: coralReef Shader Compile Not Exposed

coralReef sovereign shader compilation is currently internal to barraCuda.
groundSpring cannot directly invoke coralReef for shader-to-SPIR-V compilation.
This matters for metalForge cross-substrate validation where we want to test
compilation paths independently.

**Blocked on**: coralReef capability surface exposure.

### GAP-GS-003: barraCuda TensorSession Not Adopted

barraCuda's `TensorSession` API (persistent GPU memory across multiple ops)
would benefit groundSpring's multi-step pipelines (e.g. freeze-out grid search
→ L-BFGS refinement → spectral reconstruction). Currently each dispatch
allocates/deallocates independently.

**Blocked on**: TensorSession stabilization in barraCuda.

### GAP-GS-011: PRNG Rebaseline

groundSpring uses Xorshift64 PRNG. barraCuda GPU shaders use xoshiro128**.
Three-tier parity tests currently account for this divergence in tolerances,
but a proper rebaseline to xoshiro across all tiers would tighten parity and
simplify tolerance reasoning.

**Blocked on**: Ecosystem PRNG alignment decision.

---

## 2. barraCuda Evolution Priorities

### Make barraCuda optional = true (all springs)

groundSpring's `Cargo.toml` has `barracuda = { ..., optional = true }` but
`default = ["barracuda"]`. This means `cargo build` requires barraCuda on disk.
For sovereign NUCLEUS deployment from plasmidBin, springs should not require
any other primal's source code at compile time. IPC-first is the target.

**Recommendation**: All springs should evolve toward `default-features = false`
with barraCuda behind an explicit feature flag. Compute delegation should go
through biomeOS `compute.execute` capability calls, not compile-time linking.

### barraCuda v0.3.13 Pin

groundSpring is pinned to barraCuda v0.3.13 via path dependency. The following
delegations are active:

- **67 CPU**: stats, regression, fao56, rarefaction, bootstrap, jackknife,
  gillespie, ODE, diversity, transport, drift, freeze_out, kimura
- **43 GPU**: Anderson Lyapunov, spectral, Sturm tridiag, batch multinomial,
  fao56 batch, Hargreaves batch, grid search, stats reduction, correlation

---

## 3. Patterns Learned — For All Primal Teams

### Tolerance unification via canonical constants

**Problem**: metalForge's `ToleranceTier` enum duplicated tolerance values
from `groundspring::tol`. Version drift was inevitable.

**Solution**: `ToleranceTier::relative_tolerance()` now delegates to
`groundspring::tol::{EXACT, ANALYTICAL, STOCHASTIC, QUANTIZED}`. The forge
crate imports from the library crate — single source of truth.

**Recommendation for primal teams**: If you define tolerance constants, put
them in one canonical module and have all consumers import from there.

### tracing over log

**Problem**: `metalForge/forge` used the `log` crate while the rest of
groundSpring used `tracing`. This made structured diagnostics inconsistent.

**Solution**: Replaced `log::warn!` with `tracing::warn!` in `probe.rs` and
`nucleus.rs`. Dropped `log` from `forge/Cargo.toml`.

**Recommendation**: Standardize on `tracing` ecosystem-wide. It's a superset
of `log` and supports structured fields, spans, and subscriber composition.

### Platform guards for Linux-only paths

**Problem**: `/run/user/` UID enumeration in `nucleus.rs` would fail on macOS
or other platforms.

**Solution**: `#[cfg(target_os = "linux")]` guard on the UID discovery block.
The function falls back to environment variable discovery on non-Linux.

**Recommendation**: All primals using `/proc/`, `/run/`, or other Linux-specific
filesystem paths should gate them with `#[cfg(target_os)]`.

### primal_names::roles::* over literal strings

**Problem**: Socket registry lookup used `.contains("nestgate")` — a bare
string that could silently break if the role name changed.

**Solution**: `groundspring::primal_names::roles::STORAGE` constant used
instead. The constant is defined once in the library crate.

**Recommendation**: Use `primal_names` module constants for all primal role
references. Never use literal strings in socket lookups.

---

## 4. NUCLEUS Composition Patterns for neuralAPI/biomeOS

### Capability-based routing

groundSpring's biomeOS integration (`crates/groundspring/src/biomeos/`) uses
pure capability-based routing:
- `compute.*` → dispatched to whichever primal registered the capability
- `storage.*` → NestGate (discovered via `primal_names::roles::STORAGE`)
- `crypto.*` → BearDog (discovered via role)
- `discovery.*` → Songbird (direct)

No primal names are hardcoded in the routing layer. Discovery is always through
Songbird's `discovery.find_primals` or `discovery.query`.

### Socket discovery chain

The canonical discovery order for biomeOS sockets:
1. Environment variable (`BIOMEOS_SOCKET`, `NESTGATE_URL`, etc.)
2. Socket registry file (via `primal_names::discover_socket()`)
3. Default fallback (e.g. `NESTGATE_DEFAULT_PORT`)

### Health/liveness/readiness contract

groundSpring exposes three health endpoints:
- `health.liveness` — basic process alive check
- `health.readiness` — full capability verification
- `health_check` (legacy) — combined health report

The liveness/readiness split follows Kubernetes conventions and is recommended
for all primals deploying via biomeOS.

### Deploy graph patterns

groundSpring maintains 6 deploy graphs, each for a different deployment context:
- `groundspring_deploy.toml` — Canonical niche deploy
- `groundspring_tower_bootstrap.toml` — Minimal Tower with full 16-cap registration
- `groundspring_nucleus_local.toml` — Full local NUCLEUS with all primals
- `groundspring_nucleus_node.toml` — Node atomic GPU validation
- `groundspring_validation.toml` — Anderson-style pipeline validation
- `groundspring_cross_substrate.toml` — metalForge CPU/GPU/NPU parity

All graphs use `capability.register` (not `registry.register`) for capability
registration. Provenance blocks use `provenance.session_create` for lineage.

---

## 5. Registry Cross-Sync Recommendation

No spring currently tests its registered capabilities against primalSpring's
canonical `config/capability_registry.toml` (389 methods). This is a gap.

**Proposed CI check**: Each spring's CI should:
1. Parse its own `capability_registry.toml`
2. Fetch primalSpring's canonical registry
3. Verify all local methods are a subset of the canonical registry
4. Fail if any method string doesn't match

This prevents capability drift and ensures all springs speak the same language.
