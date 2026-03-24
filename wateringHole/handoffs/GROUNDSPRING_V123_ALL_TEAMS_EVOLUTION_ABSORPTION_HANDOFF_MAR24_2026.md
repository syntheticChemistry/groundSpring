# groundSpring V123 → All-Teams Evolution & Absorption Handoff

**Date**: March 24, 2026
**From**: groundSpring V123
**To**: All spring teams, all primal teams
**Pins**: barraCuda v0.3.7, toadStool S158+, coralReef Iteration 55+

---

## Executive Summary

groundSpring V123 completes a full cross-ecosystem review (7 springs, 10
primals, barraCuda/coralReef) and executes high-value absorption items.
This handoff documents: (1) what groundSpring now offers for absorption,
(2) what patterns we pioneered that other teams can adopt, (3) what we
learned from reviewing your codebases, and (4) evolution recommendations
per team.

**Quality certificate**: 1020+ tests, 0 failures, 0 clippy warnings,
0 unsafe, 0 TODOs, 0 production mocks, 0 direct -sys crates,
≥92% library coverage, `cargo deny check` PASS.

---

## 1. What groundSpring Offers for Absorption

### 1a. Patterns Ready for Ecosystem Adoption

| Pattern | Module | What It Does | Recommended For |
|---------|--------|--------------|-----------------|
| **Upstream contract tolerance pins** | `tol.rs` | Named constants that pin to specific barraCuda v0.3.7 API behaviour; tests fail immediately if upstream drifts | All springs |
| **Provenance registry** | `provenance_registry.rs` | Centralized mapping of all Python baselines → scripts, JSONs, validators, domains. Typed lookups | All springs with Python baselines |
| **Cast module** | `cast.rs` | 15 named numeric cast helpers with documented safety. Eliminates `#[allow(clippy::cast_*)]` | All Rust projects (already absorbed by neuralSpring, airSpring) |
| **Typed validation errors** | `cast::CastOverflowError` | Replaces `Result<T, String>` with typed errors. `thiserror` + `Display` + `Clone + PartialEq` | All crates using `Result<T, String>` |
| **Named tolerance system** | `tol.rs` + `eps.rs` | 13 tolerance tiers + 6 upstream pins + 4 production epsilons. Each with mathematical provenance | All springs doing numerical validation |
| **ValidationSink trait** | `validate.rs` | Trait abstraction for validation output: `StdoutSink`, `NullSink`, `NdjsonSink`. CI-parseable NDJSON | All springs with validation harnesses |
| **validate_all meta-runner** | `validate_all.rs` | Single binary that runs all validation binaries; core + optional with skip-aware exit codes | All springs with multiple validation binaries |
| **Zero-#[allow] discipline** | workspace lints | Replaced all `#[allow]` with `#[expect(lint, reason = "...")]` — self-documenting, compiler-verified | All Rust projects |
| **SECURITY.md** | root | Vulnerability reporting, audit posture, data provenance statement | All repos |
| **rustfmt.toml** | root | Explicit edition 2024, max_width 100, field_init/try shorthand | All repos |
| **Bitwise PRNG determinism tests** | `prng.rs` | Known-sequence pins + 1000-round bitwise identity + cross-generator parity | All springs with stochastic algorithms |
| **Capability-based discovery** | `primal_names.rs` | Centralized primal name constants with env overrides. Zero hardcoded paths in production | All primals doing socket discovery |

### 1b. barraCuda Delegations (110 total)

| barraCuda Module | CPU Delegations | GPU Delegations | Key Operations |
|-----------------|-----------------|-----------------|----------------|
| `stats` | 28 | 8 | RMSE, MAE, MBE, NSE, R², IA, WIA, hit rate, mean, std, percentile, Pearson, Spearman, covariance, diversity, Shannon, Simpson, Chao1, Kimura, Hill, Monod, bootstrap |
| `spectral` | 12 | 8 | Anderson (1D/2D/3D/4D sweep, Lyapunov), Lanczos, Almost-Mathieu, band detection, eigenvalues, eigenvectors |
| `ops` | 10 | 18 | Gillespie, Wright-Fisher, multinomial, grid search, FFT, GEMM, sum/variance reduce, fused map-reduce, correlation, covariance, autocorrelation, peak detect, batched ODE, batched elementwise |
| `optimize` | 4 | 3 | Brent, L-BFGS, batched Nelder-Mead |
| `numerical` | 3 | 1 | Trapezoid, BistableOde, MultiSignalOde |
| `linalg` | 4 | 2 | Solve, Cholesky, ridge, eigh |
| `esn_v2` | 2 | 2 | ESN train, predict, reservoir |
| `pipeline` | 2 | 0 | Seasonal pipeline, water balance |
| `special` | 2 | 1 | Anderson transport, localization length |
| **Total** | **67** | **43** | **110** |

---

## 2. For barraCuda / toadStool Team

### 2a. Upstream Contract Pins (NEW — act on these)

groundSpring now pins 6 tolerance constants to barraCuda v0.3.7 behaviour.
**If you change the numerical output of these APIs, our tests will fail.
This is intentional.** It gives both teams early detection of contract drift.

Please add equivalent pins in your test suite for these APIs:

| API | groundSpring pin | Purpose |
|-----|-----------------|---------|
| `spectral::find_all_eigenvalues` | `ANALYTICAL` (1e-10) | Jacobi eigenvalues |
| `ops::bio::multinomial_sample_cpu` | `DETERMINISM` (1e-15) | Rarefaction diversity |
| `optimize::brent` | `ANALYTICAL` (1e-10) | ET₀ root finding |
| `spectral::anderson_sweep_averaged` | `LITERATURE` (0.001) | Disorder sweep |
| `spectral::almost_mathieu_hamiltonian` | `EXACT` (1e-12) | Hofstadter butterfly |
| `spectral::detect_bands` | `ANALYTICAL` (1e-10) | Band gap detection |

### 2b. Cast Module Absorption

groundSpring V122 created `cast.rs` (15 functions). neuralSpring S173 and
airSpring V010 have credited and absorbed the pattern. **Recommend barraCuda
absorb a canonical `barracuda::cast` module** so springs can `use barracuda::cast::*`
instead of maintaining parallel copies.

### 2c. Evolution Gaps (what we need from you)

| Priority | What | Why |
|----------|------|-----|
| P0 | PRNG alignment (xoshiro128** CPU → match WGSL) | Baseline regeneration blocked |
| P0 | Sparse SpMV for Anderson 2D/3D (N > 4096) | Lanczos at scale |
| P1 | `GpuDriverProfile` deprecation path | Consumer Ampere/Ada f64 routing |
| P1 | Named tolerances in barraCuda test suite | Match spring contract pin pattern |
| P2 | Matrix exponentiation GPU op | Spin chain transport |
| P2 | RAWR GPU dispatch | RAWR bootstrap acceleration |
| P2 | Sobol sensitivity indices | FAO-56 sensitivity analysis |

---

## 3. For All Spring Teams

### 3a. Patterns to Adopt (prioritized)

**Tier A — High value, low effort:**
1. **`SECURITY.md`** — 41 lines, matches ecosystem standard
2. **`rustfmt.toml`** — 8 lines, prevents formatting drift
3. **Upstream tolerance pins** — pin your critical barraCuda APIs
4. **Bitwise PRNG determinism tests** — if you use stochastic algorithms

**Tier B — Medium value, medium effort:**
5. **Provenance registry module** — centralize your Python baseline mappings
6. **`CastOverflowError` typed errors** — if you still have `Result<T, String>`
7. **`NdjsonSink`** — if your CI needs machine-readable validation output

**Tier C — Future alignment:**
8. **MCP `tools.list` / `tools.call` surface** — for AI agent discovery
9. **`validate_all` meta-runner** — if you have multiple validation binaries

### 3b. What We Learned from Your Codebases

| Spring | What We Absorbed | What We Noticed |
|--------|-----------------|-----------------|
| **hotSpring** (v0.6.32) | Ember pipeline pattern, validation harness decomposition, three-tier parity architecture | 848+ tests, 74 experiments — most mature spring; GCN5/RTX 5060 coverage impressive |
| **neuralSpring** (V124) | Upstream tolerance pins, provenance headers, `SECURITY.md`, `rustfmt.toml` | 1385+ tests, 232+ named tolerances — strongest provenance discipline |
| **wetSpring** (V135) | `NdjsonSink` / `StreamItem` NDJSON pattern, absorption handoff structure | 1891 tests, 355 binaries — largest binary count; drug NMF delegation notable |
| **airSpring** (v0.10.0) | Three-tier capability discovery, `PRIMAL_NAME`-derived RPC | 1316 tests, 91 binaries — strongest forge feature alignment |
| **healthSpring** (V42) | Bitwise determinism tests, metalForge forge split, NLME shader lineage | 83 experiments — most domain-specific; WGSL shader count growing |
| **ludoSpring** (V30) | CI/coverage template pattern, MCP tooling reference | Game mechanics domain — useful MCP pattern reference |
| **primalSpring** (0.7.0) | `NdjsonSink` evolution, skip-aware exit codes, multi-node bonding | Coordination domain — NUCLEUS composition patterns |

---

## 4. For biomeOS / NUCLEUS Team

### 4a. Capabilities groundSpring Exposes

16 `measurement.*` capabilities via JSON-RPC 2.0:

```
measurement.noise_decomposition    measurement.anderson_validation
measurement.parity_check           measurement.et0_propagation
measurement.regime_classification  measurement.uncertainty_budget
measurement.spectral_features      measurement.freeze_out
measurement.bootstrap              measurement.rarefaction
measurement.drift                  measurement.band_edge
measurement.rare_biosphere         measurement.gillespie
measurement.bistable               measurement.quasispecies
```

### 4b. Capabilities groundSpring Consumes

```
compute.execute        compute.submit         storage.put
storage.get            data.ncbi_search       data.ncbi_fetch
data.noaa_ghcnd        data.iris_stations     data.iris_events
crypto.sign            crypto.verify          discovery.find_primals
discovery.query
```

Plus provenance trio: `provenance.session_create`, `provenance.session_dehydrate`,
`contribution.recordDehydration`.

Plus registration: `capability.register`, `capability.deregister`.

### 4c. CONSUMED_CAPABILITIES Gap

`niche.rs::CONSUMED_CAPABILITIES` does not include the provenance trio
or registration capabilities. Recommend aligning the declared surface
with actual code usage for orchestration tooling.

---

## 5. For coralReef Team

groundSpring has **2 unique WGSL reference shaders** remaining in
`metalForge/shaders/` (`anderson_lyapunov_*.wgsl`). All other shaders
absorbed upstream. The Titan V / Volta SM70 path is validated but
consumer Ada/Ampere f64 routing needs `GpuDriverProfile` evolution
from toadStool to be safe for non-workstation GPUs.

---

## 6. For Phase 1 Primals (toadStool, Squirrel, NestGate, Songbird, BearDog)

| Primal | groundSpring Interaction | Recommendation |
|--------|------------------------|----------------|
| **toadStool** | `compute.execute/submit/status` via capability routing; `akida-driver` path dep for NPU | See section 2 for barraCuda gap items |
| **Squirrel** | Copyright headers only; `roles::ASSISTANT` | MCP tool surface for groundSpring measurement capabilities — future item |
| **NestGate** | `storage.put/get` + data pipelines (NCBI/NOAA/IRIS) via capability routing | Working well; no issues |
| **Songbird** | `roles::DISCOVERY`; socket discovery chain | Working well; no issues |
| **BearDog** | `roles::SECURITY`; `crypto.sign/verify` consumed | Working well; no issues |

---

## 7. For Phase 2 Primals (biomeOS, petalTongue, sweetGrass, rhizoCrypt, loamSpine)

| Primal | groundSpring Interaction | Recommendation |
|--------|------------------------|----------------|
| **biomeOS** | Full NUCLEUS client: registration, routing, health, Tower/Node/Squirrel/Nest validation | `PRIMAL_REGISTRY.md` is stale for groundSpring (shows older version) — please update |
| **petalTongue** | `roles::VISUALIZATION` only | No active interaction yet |
| **sweetGrass** | `contribution.recordDehydration` via capability routing | Working; provenance attribution path validated |
| **rhizoCrypt** | `provenance.session_create/dehydrate` via capability routing | Working; session lifecycle validated |
| **loamSpine** | Implicit via provenance trio chain | No direct interaction |

---

## 8. Quality Certificate

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
| `unsafe` | Zero (`#![forbid(unsafe_code)]`) |
| `#[allow]` | Zero |
| TODO/FIXME | Zero |
| Production mocks | Zero |
| Direct `-sys` crates | Zero |
