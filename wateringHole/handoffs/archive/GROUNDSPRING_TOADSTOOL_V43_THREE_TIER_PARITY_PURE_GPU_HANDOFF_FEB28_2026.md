# groundSpring → ToadStool V43: Three-Tier Parity Proven + Pure GPU Workload Validation

**Date**: February 28, 2026
**groundSpring HEAD**: `main`
**ToadStool pin**: S68+

---

## Summary

V43 validates the full hardware progression that proves groundSpring's math
is universal and portable:

```
Python (interpreted)     107.1s   ─── math correctness (open data + open systems)
  │  5.2× faster
Rust (compiled)           20.5s   ─── pure safe Rust, same math (28/28 parity)
  │  ~0% overhead
barracuda-CPU             22.8s   ─── delegation proves portability (27/27 parity)
  │  2.2× faster
barracuda-GPU              9.8s   ─── GPU proves the math is truly portable
  │                                    (47.4× peak, via hotSpring Sturm eigensolver)
  │
metalForge                        ─── cross-system: GPU → NPU → CPU per-workload
                                       19 workloads, 3 substrates, sovereign fallback
```

---

## What V43 Validates

### Three-Tier Parity Certificate

**27/27 experiments: THREE-TIER PARITY PROVEN**

Every validation binary produces identical pass counts across:
- Tier 1: `default` (pure Rust, no barracuda)
- Tier 2: `--features barracuda` (CPU delegations)
- Tier 3: `--features barracuda-gpu` (CPU + GPU delegations)

Certificate: `data/three_tier_parity_report.json`
Script: `scripts/three_tier_parity_report.sh`
Unit tests: `three_tier_parity.rs` (44 tests × 3 modes)

### GPU Tier Validation

**`validate-gpu-tier`: 39/39 checks × 3 modes**

Tests 11 subsystems from 5 cross-spring origins:

| Test | Provenance | Checks |
|------|-----------|--------|
| Stats metrics | airSpring+groundSpring S64 | 7 |
| Regression | airSpring S66 | 5 |
| Bootstrap RAWR | groundSpring S66 | 2 |
| Shannon diversity | wetSpring S64 | 3 |
| Hill kinetics | wetSpring S68 | 3 |
| Anderson localization | hotSpring S26 | 4 |
| Almost-Mathieu | hotSpring S26 | 3 |
| Bistable ODE | wetSpring S58 | 2 |
| Spectral reconstruction | hotSpring S39 | 3 |
| Rare biosphere | groundSpring→neuralSpring S64 | 3 |
| Band structure | hotSpring S26 | 4 |

### Pure GPU Workload Validation

**`validate-pure-gpu-workloads`: 26/26 checks**

Validates the complete pipeline: hardware discovery → dispatch routing → computation → parity.

| Category | Result |
|----------|--------|
| Hardware discovery | 3 GPUs + 1 CPU |
| Dispatch routing | 17/19 → Titan V (NVK GV100) |
| Anderson parity | γ bitwise identical |
| Stats parity | RMSE, NSE, R² bitwise identical |
| Bootstrap parity | RAWR CI bitwise identical |
| Diversity parity | Shannon, evenness correct |
| Spectral parity | 50 eigenvalues bitwise identical |
| Regression parity | slope=2.5, intercept=1.0, R²=1.0 |
| Rare biosphere | occupancy ordering correct |
| Green-Kubo | D*=0.431 (analytical: 0.431) |
| Timing | Anderson 5ms, RAWR 152ms |

### Full Test Suite

| Mode | Tests | Warnings | Failures |
|------|-------|----------|----------|
| `barracuda-gpu` | 462 | 0 | 0 |
| `default` | 410 | 0 | 0 |
| `biomeos` | 442 | 0 | 0 |

---

## Delegation Inventory (unchanged from V42)

**39 active** (30 CPU + 9 GPU) + **7 pending** ToadStool.

### New Validation Binaries (V43)

| Binary | Location | Checks | Scope |
|--------|----------|--------|-------|
| `validate-gpu-tier` | `metalForge/forge/src/bin/` | 39 | Cross-spring subsystem parity |
| `validate-pure-gpu-workloads` | `metalForge/forge/src/bin/` | 26 | End-to-end GPU pipeline |

---

## Hardware Substrates Validated

| Substrate | Identity | Capabilities | f64 |
|-----------|----------|-------------|-----|
| GPU | NVIDIA TITAN V (NVK GV100) | f32, f64, shader, reduce, native-f64 | **Native 1:2** |
| GPU | NVIDIA GeForce RTX 4070 | f32, f64, shader, reduce | DF64 (emulated) |
| GPU | NVIDIA GeForce RTX 4070/PCIe/SSE2 | f32, shader | f32 only |
| CPU | 12th Gen Intel i9-12900K | f64, f32, simd (AVX2) | Native |

---

## Road Ahead

### Immediate (no blockers)

- Run `scripts/three_tier_parity_report.sh` on every `ToadStool` absorption to detect regressions
- barracuda CPU already proven pure math — ready for `baseCamp` integration benchmarks

### Pending ToadStool (7 delegations)

| Function | Target | Priority |
|----------|--------|----------|
| `daily_et0` | `stats::hydrology::fao56_et0` | LOW (scalar) |
| `kimura_fixation_prob` | `stats::kimura_fixation` | MEDIUM |
| `grid_search_inversion` | `ops::grid::grid_search_3d_f64` | HIGH (GPU) |
| `find_band_edges` | `spectral::band_edges_parallel` | MEDIUM |
| `grid_fit_2d` | `ops::grid::grid_fit_2d_f64` | HIGH (GPU) |
| `jackknife_mean_variance` | `stats::jackknife_mean_variance` | MEDIUM |
| `quasispecies batched GPU` | `ops::bio::WrightFisherGpu` (batched) | MEDIUM |

### metalForge Next Steps

- Pure GPU benchmark binary (unidirectional streaming): data → GPU → result (no round-trips)
- Cross-substrate pipeline: GPU compute → NPU classify → CPU aggregate
- Remote substrate discovery for distributed workloads

---

*This handoff supersedes V42. Archive V42 to `wateringHole/handoffs/archive/`.*
