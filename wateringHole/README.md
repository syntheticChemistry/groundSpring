# groundSpring wateringHole — Cross-Primal Handoffs

**Purpose**: Handoff documents from groundSpring to ToadStool/BarraCUDA team,
following the wateringHole inter-primal standard.

**Last Updated**: February 27, 2026

## Active Handoffs

| Version | File | Date | Scope |
|---------|------|------|-------|
| V37 | [GROUNDSPRING_TOADSTOOL_V37_BARRACUDA_EVOLUTION_HANDOFF_FEB27_2026.md](handoffs/GROUNDSPRING_TOADSTOOL_V37_BARRACUDA_EVOLUTION_HANDOFF_FEB27_2026.md) | Feb 27, 2026 | Comprehensive BarraCUDA evolution summary: 32 active delegations (25 CPU + 7 GPU), 9 pending absorption, 19 workloads, 49 metalForge tests, 5 substrates, NAK f64 gap analysis, absorption priorities (3 grid search ops, 2 bio batch ops, DF64 default), cross-spring learnings |
| V35 | [GROUNDSPRING_TOADSTOOL_V35_TITANV_NAK_HANDOFF_FEB27_2026.md](handoffs/GROUNDSPRING_TOADSTOOL_V35_TITANV_NAK_HANDOFF_FEB27_2026.md) | Feb 27, 2026 | Titan V / NAK adaptive GPU dispatch: `GpuArch` detection, `NativeF64` capability, `AdaptiveBatch`, architecture-aware f64 routing, live GPU compute |

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
GROUNDSPRING_TOADSTOOL_V{N}_{TOPIC}_{DATE}.md
```

## Archive

Superseded handoffs are moved to `handoffs/archive/`.

| Version | Scope |
|---------|-------|
| V35 | Titan V / NAK adaptive GPU dispatch (companion to V37) |
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
