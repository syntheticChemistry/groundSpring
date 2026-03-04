# groundSpring wateringHole — Cross-Primal Handoffs

**Purpose**: Handoff documents from groundSpring to ToadStool/BarraCUDA team,
following the wateringHole inter-primal standard.

**Last Updated**: March 4, 2026

## Active Handoffs

| Version | File | Date | Scope |
|---------|------|------|-------|
| V74 Absorption | [GROUNDSPRING_V74_TOADSTOOL_BARRACUDA_ABSORPTION_HANDOFF_MAR04_2026.md](handoffs/GROUNDSPRING_V74_TOADSTOOL_BARRACUDA_ABSORPTION_HANDOFF_MAR04_2026.md) | Mar 4, 2026 | Absorption targets, evolution requests, cross-spring learnings, performance data |
| V74 Catch-Up | [GROUNDSPRING_BARRACUDA_V74_DEEP_DEBT_CATCHUP_HANDOFF_MAR04_2026.md](handoffs/GROUNDSPRING_BARRACUDA_V74_DEEP_DEBT_CATCHUP_HANDOFF_MAR04_2026.md) | Mar 4, 2026 | Deep debt + clippy pedantic CI + ToadStool S93/barraCuda v0.3.1 full catch-up audit |
| V73 | [GROUNDSPRING_BARRACUDA_V73_TOLERANCE_ARCHITECTURE_HANDOFF_MAR04_2026.md](handoffs/GROUNDSPRING_BARRACUDA_V73_TOLERANCE_ARCHITECTURE_HANDOFF_MAR04_2026.md) | Mar 4, 2026 | 13-tier tolerance architecture (`tol::`, `eps::`), ~170 bare literals → named constants |
| Guide | [GROUNDSPRING_TOADSTOOL_BARRACUDA_ABSORPTION_GUIDE_MAR04_2026.md](handoffs/GROUNDSPRING_TOADSTOOL_BARRACUDA_ABSORPTION_GUIDE_MAR04_2026.md) | Mar 4, 2026 | Delegation patterns, feature gating, tolerance tiers, evolution requests |

## Cross-Spring Documentation

| Document | Purpose |
|----------|---------|
| [CROSS_SPRING_SHADER_EVOLUTION.md](CROSS_SPRING_SHADER_EVOLUTION.md) | How hotSpring, wetSpring, and neuralSpring evolved BarraCUDA into what groundSpring delegates to |

## Canonical Location

The authoritative copy of each handoff also lives at
`ecoPrimals/wateringHole/handoffs/GROUNDSPRING_TOADSTOOL_V{N}_*.md` (the shared
ecoPrimals wateringHole). This local copy is for convenience
and groundSpring-centric context.

## Naming Convention

```
GROUNDSPRING_BARRACUDA_V{N}_{TOPIC}_{DATE}.md
```

## Archive

Superseded handoffs are moved to `handoffs/archive/`.

| Version | Scope |
|---------|-------|
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
