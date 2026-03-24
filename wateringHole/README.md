# groundSpring wateringHole — Cross-Primal Handoffs

**Purpose**: Unidirectional handoff documents from groundSpring to the
toadStool/barraCuda/coralReef teams, following the ecoPrimals wateringHole
inter-primal standard.

**Last Updated**: March 24, 2026 (V123)

## What This Is

groundSpring writes handoffs; toadStool/barraCuda/coralReef read them.
Handoffs are unidirectional — no response expected. They document:
delegation state, evolution requests, cross-spring learnings, and
quality certificates. The toadStool/barraCuda team uses these to
prioritize absorption, evolve primitives, and validate GPU parity.

## Active Handoffs

| Version | File | Date | Scope |
|---------|------|------|-------|
| V123 Cross-Ecosystem Absorption | [GROUNDSPRING_V123_CROSS_ECOSYSTEM_ABSORPTION_PROVENANCE_HANDOFF_MAR24_2026.md](handoffs/GROUNDSPRING_V123_CROSS_ECOSYSTEM_ABSORPTION_PROVENANCE_HANDOFF_MAR24_2026.md) | Mar 24, 2026 | Full ecosystem review (7 springs + 10 primals), 6 upstream contract tolerance pins, provenance registry (29 baselines), `CastOverflowError`, 5 bitwise determinism tests, `SECURITY.md`, `rustfmt.toml` |
| V121 Deep Debt + Absorption | [GROUNDSPRING_V121_BARRACUDA_TOADSTOOL_DEEP_DEBT_HANDOFF_MAR23_2026.md](handoffs/GROUNDSPRING_V121_BARRACUDA_TOADSTOOL_DEEP_DEBT_HANDOFF_MAR23_2026.md) | Mar 23, 2026 | Tolerance centralization (10 new named constants), provenance hardening, benchmark round-trip test, `#[allow]`→`#[expect]` evolution, barraCuda v0.3.7 doc sync, evolution priorities for toadStool/barraCuda team. |

## Cross-Spring Documentation

| Document | Purpose |
|----------|---------|
| [CROSS_SPRING_SHADER_EVOLUTION.md](CROSS_SPRING_SHADER_EVOLUTION.md) | How hotSpring, wetSpring, and neuralSpring evolved barraCuda into what groundSpring delegates to |

## Conventions

**Naming**: `GROUNDSPRING_V{N}_{TOPIC}_HANDOFF_{MON}{DD}_{YYYY}.md`

**Flow**: groundSpring → barraCuda / toadStool / coralReef (unidirectional)

**Canonical location**: Also copied to `ecoPrimals/wateringHole/handoffs/`.

**Archive**: Superseded handoffs are moved to `handoffs/archive/`.

| Version | Scope |
|---------|-------|
| V122 Cast Evolution | Cast module extraction + `pub`, 20+ bare casts→named helpers, `u64_u32_truncate`, `eps::SSA_FLOOR` unconditional (superseded by V123, archived) |
| V120 Deep Audit Execution | Dispatch refactored, `#![forbid(unsafe_code)]` on 50 binaries, `DeviceCapabilities`, release-mode CI (superseded by V122, archived) |
| V119 Deep Audit + Absorption | Cross-ecosystem absorption: publish hygiene, MSRV 1.85, provenance registry, cast parity, IPC isolation (superseded by V121, archived) |
| V118 Deep Audit + RPC + PRNG | RPC expansion (16 capabilities), proptest, PRNG production, spectral_recon GPU GEMM, 110 delegations, provenance hardening (superseded by V119) |
| V117 All-Features + PRNG | All-features compilation fixed (tarpc-ipc), cargo deny modernised, PRNG DefaultRng feature-gated, validate_all meta-binary, clippy --all-features clean (superseded by V118) |
| V116 Typed Error Evolution | `DispatchError`, `EsnError`, `ResilienceError<E>`, `ValidationSink` trait, Format C/D capability parsing, `OnceLock` GPU probe, RAWR extraction, dispatch defaults named with provenance (superseded by V117) |
| V115 Deep Debt + Idiomatic API Evolution | `assert!` → `Result<T, InputError>`, CI hardened (nursery, `--all-features`), ecoBin compliance (14 C-deps banned), zero panicking public APIs (superseded by V116) |
| V114 Cross-Ecosystem Deep Absorption | safe_cast, health probes, resilient_call, OrExit evolution, FAMILY_ID discovery, primal composition guidance (superseded by V115) |
| V113 Ecosystem Resilience | GemmF64 transpose, exit_code constants, RetryPolicy + CircuitBreaker, 4-format capability parsing (superseded by V114) |
| V112 Deep Debt + OrExit | `OrExit<T>` + parse_benchmark() (28 binaries), generic socket_env_var(), provenance trio, thiserror BenchFieldError, tempdir test hygiene (superseded by V113) |
| V110 Cross-Ecosystem Absorption | `#[expect(reason)]` migration (95 files), Python tolerance mirror, tracing, toadStool direct dispatch, dual-format capability parsing, deny.toml, aarch64 CI (superseded by V112) |
| V109 Deep Debt + Smart Refactor | Zero-panic validation binaries (28 converted), smart module refactoring (regression/fao56/pipeline/validate-lib), named physical constants, Python dep pinning (superseded by V110) |
| V108 Deep Debt + Absorption | License AGPL-3.0-or-later, barracuda WelfordState CPU delegation, tolerance centralization, typed capability discovery, provenance enrichment, Result-based validate pattern (superseded by V109) |
| V107 License/Niche/Tolerance | AGPL-3.0-or-later (302 files), release profile, enriched niche.rs, tolerance provenance, bare literal removal (superseded by V108) |
| V105 Code Evolution | deny unwrap/expect, freeze_out 4-module refactor, typed tarpc IPC, platform-agnostic paths (superseded by V107) |
| V104 Deep Debt | Named constants with provenance, capability-based discovery, measurement.* surface (superseded by V105) |
| V103 Deep Debt Audit | Named constants with provenance, `biomeos/interaction.rs` extraction, `eps::LOG_FLOOR` centralized, batch primitive absorption opportunities (superseded by V104) |
| V102 Niche Deployment | Spring-as-Niche via biomeOS graph composition: UniBin, `measurement.*`, deploy graph, niche YAML, Provenance Trio, Neural API (superseded by V103) |
| V102 BarraCUDA/ToadStool Niche | Dispatch-through-delegation chain, method→library→delegation map, GPU evolution opportunities (superseded by V103) |
| V101 DRY Evolution | ESNConfig upstream sync, DRY extraction (chi², R², bootstrap), capability-based primal discovery, hardcoded primal names eliminated, doc sovereignty (superseded by V102) |
| V100 Deep Debt | Deep audit: build fix, silent fallback elimination, tolerance provenance, capability-based health. 908 tests, 102 delegations (superseded by V101) |
| V99 NUCLEUS Live | First live NUCLEUS connection, adaptive health, direct primal discovery, 40/40 NUCLEUS checks (superseded by V100) |
| V98 Upstream Rewire | barraCuda `a898dee`, toadStool S130+, coralReef Iteration 10, 936 tests, 102 delegations, three-tier parity intact (superseded by V99) |
| V97 GPU Smoke Test | Runtime f64 reduction smoke test, three-tier parity 29/29, 936 tests, P0 GpuDriverProfile fix request (superseded by V98) |
| V96 Upstream Rewire | PrecisionRoutingAdvice wired, barraCuda `2a6c072`, toadStool S130, coralReef Iter 7, 925 tests, 102 delegations (superseded by V97) |
| V95 coralReef Breakthrough | 907 tests, 102 delegations, coralReef Phase 11, sovereign GPU dispatch on Titan V, push buffer encoding fixed (superseded by V96) |
| V94 Evolution | 907 tests, 102 delegations, Shannon delegation, CorrelationFull API evolution, coralReef Phase 10 (superseded by V95) |
| V83 Pin Refresh | Dependency catch-up: barraCuda `0bd401f`, toadStool S96c, coralReef `1e048be`. 91 delegations verified compatible, 824 tests (superseded by V85) |
| V82 Delegation Expansion | Thornthwaite ET₀, fit_all regression, esn/fao56 smart-refactored, deep debt audit, 91 delegations (54 CPU + 37 GPU), 824 tests (superseded by V83) |
| V81 Modern Rewire | BootstrapMeanGpu GPU dispatch, freeze_out gate fix, coralReef (390 tests), 88 delegations (51 CPU + 37 GPU), barraCuda `a4c20a5`, toadStool S95 (superseded by V82) |
| V80 Fused Ops + Catch-Up | Fused `correlation_full` GPU, Welford single-pass CPU, barraCuda HEAD catch-up, toadStool S94b review, 87 delegations (superseded by V81) |
| V79 Exp 035 + Delegation | Exp 035 Multi-Method ET₀, seismic delegation, 85 delegations (superseded by V80) |
| V78 Modern Rewire | Fused mean+variance, 3 new ET₀ delegations, cross-spring benchmark evolution, 84 delegations (superseded by V79) |
| V77 wgpu 28 | wgpu 28 migration, barraCuda v0.3.3 sync, DF64 precision tiers, migration pattern reference (superseded by V78) |
| V76 Structural | Structural evolution, deep debt zero, NUCLEUS shared utilities, observation-gap parity chain (superseded by V77) |
| V76 Absorption | Absorption targets, evolution requests, ToadStool/BarraCUDA delegation patterns (superseded by V77) |
| V70 | barraCuda budding: rewired from phase1/toadstool to standalone barraCuda primal, zero code changes, akida-driver stays with toadStool (superseded by V71) |
| V69 | S87 pin, universal precision architecture audit, 76 delegations, cross-spring evolution parity (superseded by V70) |
| V68 | Comprehensive evolution: 76 delegations (44 CPU + 32 GPU), 30 metalForge workloads, GPU parity buildout, GPU→NPU PCIe bypass, NUCLEUS coordination, three-tier hardware matrix (superseded by V69) |
| V67 | ToadStool S86 catch-up: McEt0PropagateGpu, SeasonalPipelineF64, BatchedMultinomialConfig API fix, 73 delegations (43 CPU + 30 GPU), 28 metalForge workloads (superseded by V68) |
| V66 | Stats Tier A GPU (MAE, NSE, R²), bistable batch ODE GPU, 71 delegations (43 CPU + 28 GPU), 26 metalForge workloads, barracuda API usage review (superseded by V67) |
| V65 | Comprehensive absorption handoff: 67 delegations, paper queue × three-tier hardware matrix, PRNG alignment roadmap, zero-debt audit certificate (superseded by V66) |
| V64 | Deep audit: biomeos refactoring, `#[expect]` evolution, epsilon guard docs, tolerance comments, benchmark units, LICENSE fix, PRNG path docs — 67 delegations, 752 tests (superseded by V65) |
| V63 | Brain architecture + capability-based discovery + Paper 12 (tissue Anderson, 29/29) — 67 delegations (superseded by V64) |
| V62 | S79 catch-up: pollster eliminated, f64-capable device, redundant shaders removed, `Result`-based API, `f64::total_cmp()`, `#[expect]` — 710 tests (superseded by V63) |
| V61 | Mixed-hardware pipeline: PCIe topology, pipeline dispatch, NUCLEUS atomics (Tower/Node/Nest/Full), fallback chains, deep idiomatic debt pass, 668 tests (superseded by V62) |
| V60 | hotSpring cross-spring absorption: DriftMonitor, ClassificationUncertainty, concept edges, Nautilus dep, 620 tests (superseded by V61) |
| V59 | ToadStool S71+++ catch-up: jackknife GPU promoted, HargreavesBatchGpu, ComputeDispatch, DF64 transcendentals (superseded by V60) |
| V58 | Cross-spring evolution + deep-debt completion: 61 delegations, FAMILY_ID evolution (superseded by V59) |
| V56 | NUCLEUS integration: biomeOS Neural API, NestGate pipelines, Exp 029–032 (superseded by V58) |
| V55 | barracuda evolution review: 57-delegation inventory, cross-spring lineage (superseded by V56) |
| V54 | Full control validation: 283/283, 95/95 parity, Rust 11.6× (superseded by V55) |
| V53 | Complete rewiring, GPU grid adapters, cross-spring lineage (superseded by V54) |
| V52 | ToadStool S70+ catch-up: 4 new CPU delegations, 52 active (superseded by V53) |
| V51 | GPU stats dispatch, batch GPU APIs, 9 parity tests, 48 active (superseded by V52) |
| V47 | Library buildout: 7 new CPU delegations, 46 active (37 CPU + 9 GPU) (superseded by V51) |
| V44 | Deep-debt evolution: linalg module, typed InputError, 39 active delegations (superseded by V46) |
| V43 | Three-tier parity proven (27/27), pure GPU workloads (26/26), 39 active delegations (superseded by V44) |
| V39 | NUCLEUS integration, NestGate data pipeline, metalForge remote (superseded by V43) |
| V37 | Comprehensive barracuda evolution: 39 delegations, NAK f64 gap (superseded by V43) |
| V35 | Titan V / NAK adaptive GPU dispatch (superseded by V37/V39) |
| V33 | Delegation count expansion (32 active, 25 CPU + 7 GPU) (superseded by V35) |
| V32 | ToadStool S68+ catch-up (superseded by V33) |
| V31 | GPU dispatch wiring + metalForge expansion (superseded by V32) |
| V30 | biomeOS Neural API integration (superseded by V31) |
| V29 | Three-tier validation buildout + 3 new CPU delegations (superseded by V30) |
| V28 | Coverage evolution + PRNG readiness (superseded by V29) |
| V27 | Comprehensive barracuda review: 29 delegations, paper controls, three-tier validation (superseded by V28) |
| V26 | metalForge live hardware: NPU DMA on AKD1000, Exp 028 (superseded by V27) |
| V23 | Exp 019-021 buildout (jackknife, freeze-out, spectral recon) (superseded by V26) |
| V22 | Exp 016-018 buildout (rare biosphere, quasispecies, band edge), 3 new modules, absorption candidates (superseded by V23) |
| V21 | Complete barracuda rewiring: dual-mode CI, 225/225 pass both modes, 27 delegations (superseded by V22) |
| V20 | S68 catch-up: hill delegation #27, pin f0feb226, 27 delegations (superseded by V21) |
| V19 | Uncertainty bridge: Exp 015, 225 tests, 185/185 checks (superseded by V21) |
| V18 | Idiomatic Rust evolution: kinetics module, flat buffers, 225 tests (superseded by V20) |
| V17 | Deep debt evolution: delegation patterns, 9 action items, 26 delegations |
| V16 | S66 catch-up: rawr_mean delegation #26, V13–V15 consumption audit |
| V15 | Absorption request: 2 shaders, 3 semantic fixes, 25 delegations, cross-spring learnings |
| V14 | S65 revalidation: 25 delegations, 49.5× Exp 009, cross-spring documentation |
| V13 | Complete rewiring: 24 delegations, Sturm tridiag (50×), cross-spring S58-S65 |
| V12 | ToadStool S64 catch-up: 6 new delegations (20 total), 3 bug fixes |
| V11 | Full-suite parity + benchmarks: 14 delegations, 144/144 checks, 23.4× speedup |
| V10 | Definitive handoff: 5 absorption priorities, 119/119 checks, 11 delegations |
| V9 | Complete rewiring, benchmarks, cross-spring lineage |
| V8 | Sovereignty evolution, barracuda error handling, PRNG/GPU assessments |
| V7 | Deep audit, proptest, Python quality, coverage |
| V1–V6 | See ecoPrimals shared wateringHole archive |
