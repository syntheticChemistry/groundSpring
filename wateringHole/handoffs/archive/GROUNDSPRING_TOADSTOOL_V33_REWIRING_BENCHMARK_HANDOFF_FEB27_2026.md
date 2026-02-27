# groundSpring → ToadStool Handoff: V33 Complete Rewiring + Three-Mode Benchmark

> **From**: groundSpring V33 (Feb 27, 2026)
> **To**: ToadStool S68+ (e96576ee)
> **Supersedes**: V32 S68 Catch-Up Handoff

---

## Summary

V33 completes the barracuda rewiring cycle that V32 started. V32 cleaned 9
forward declarations that referenced unimplemented ToadStool functions. V33
adds 3 **new** delegations using functions ToadStool **already has**, runs the
full three-mode benchmark (default / barracuda CPU / barracuda GPU), and
documents cross-spring evolution provenance for every delegation.

---

## New Delegations (V33)

| # | groundSpring fn | barracuda fn | Tier | Origin |
|---|----------------|-------------|------|--------|
| 30 | `stats::mae` | `barracuda::stats::mae` | CPU | airSpring V009 → S64 |
| 31 | `stats::nash_sutcliffe` | `barracuda::stats::nash_sutcliffe` | CPU | airSpring V009 → S64 |
| 32 | `band_structure::detect_band_ranges` | `barracuda::spectral::detect_bands` | GPU | hotSpring v0.6 → S26 |

All three compile cleanly, have unit tests, and have three-tier parity tests.

---

## Three-Mode Benchmark Results

**279/279 checks pass in ALL three modes. 28/28 Python↔Rust parity proven.**

| Mode | Total Runtime | vs Default |
|------|-------------|-----------|
| Default (no barracuda) | 22,030 ms | — |
| Barracuda CPU | 22,828 ms | +3.6% overhead |
| **Barracuda GPU** | **9,798 ms** | **−55% (2.2× faster)** |

### GPU Speedup Leaders

| Experiment | Default | GPU | Speedup | Cross-Spring Source |
|-----------|---------|-----|---------|-------------------|
| Exp 009 Quasiperiodic | 11,376ms | 240ms | **47.4×** | hotSpring Sturm tridiag eigensolver (S26) |
| Exp 019 Jackknife | 410ms | 100ms | **4.1×** | barracuda jackknife (S64) |
| Exp 020 Freeze-Out | 219ms | 127ms | **1.7×** | barracuda chi² grid fit (S64) |
| Exp 026 Size Convergence | 176ms | 111ms | **1.6×** | barracuda regression (S66) |

---

## Current Delegation Inventory

### Active: 32 (25 CPU + 7 GPU)

**CPU Tier (25)** — `#[cfg(feature = "barracuda")]`:
1. `pearson_correlation` 2. `spearman_correlation` 3. `std_dev` 4. `covariance`
5. `norm_cdf` 6. `norm_ppf` 7. `chi2_decomposed` 8. `bootstrap_mean`
9. `localization_length` 10. `BistableOde::cpu_derivative`
11. `MultiSignalOde::cpu_derivative` 12. `rmse` 13. `mbe` 14. `r_squared`
15. `index_of_agreement` 16. `hit_rate` 17. `shannon` 18. `mean`
19. `percentile` 20. `pielou_evenness` 21. `rawr_mean` 22. `hill`
23. `fit_linear` 24. `mae` *(new V33)* 25. `nash_sutcliffe` *(new V33)*

**GPU Tier (7)** — `#[cfg(feature = "barracuda-gpu")]`:
1. `lyapunov_exponent` 2. `lyapunov_averaged` 3. `level_spacing_ratio`
4. `almost_mathieu_hamiltonian` 5. `find_all_eigenvalues` 6. `solve_f64_cpu`
7. `detect_bands` *(new V33, from hotSpring v0.6 spectral theory)*

### Pending ToadStool Absorption: 9 (3 CPU + 6 GPU)

**CPU** (commented out with `TODO(toadstool)`):
1. `stats::kimura_fixation` — analytical fixation probability
2. `stats::jackknife_mean_variance` — delete-one jackknife
3. `stats::fao56_et0` — Penman-Monteith scalar ET₀

**GPU** (commented out with `TODO(toadstool)`):
4. `ops::grid::grid_fit_2d_f64` — 2D chi² grid search
5. `ops::grid::grid_search_3d_f64` — 3D grid search
6. `spectral::band_edges_parallel` — per-energy transfer matrix
7. `ops::bio::wright_fisher_simulate` — multi-generation WF wrapper
8. `ops::bio::batched_multinomial_occupancy` — occupancy from counts
9. `ops::bio::batched_multinomial_tier_rate` — tier detection rate

---

## Cross-Spring Evolution Map

Every delegation traces back to a specific Spring contribution:

```
hotSpring (nuclear physics)     → f64 precision, spectral theory, DF64, Sturm eigensolver
  └─ #9-12, #23-24, #28, #32          (47.4× GPU speedup on Exp 009)

wetSpring (metagenomics)        → bio-stats, Shannon, ODE systems, Gillespie
  └─ #13-14, #20, #23, #25

neuralSpring (ML/agents)        → spectral density, dispatch patterns, xoshiro PRNG
  └─ #2 (Spearman), spectral diagnostics

airSpring (agriculture)         → error metrics, hydrology, regression
  └─ #15-19, #21-22, #29-31           (MAE + NSE from airSpring ET₀ validation)

groundSpring (noise validation) → validation patterns, bootstrap, RAWR
  └─ #8, #24, three-mode CI pattern
```

---

## ToadStool Action Items (unchanged from V32)

The 9 pending delegations still need ToadStool to implement:

1. **`stats::kimura_fixation(pop_size, selection, initial_freq) -> f64`**
2. **`stats::jackknife_mean_variance(data) -> (f64, f64)`**
3. **`stats::fao56_et0(...)  -> f64`** (scalar Penman-Monteith)
4. **`ops::grid::grid_fit_2d_f64(...)` / `grid_search_3d_f64(...)`**
5. **`spectral::band_edges_parallel(...)`**
6. **`ops::bio::wright_fisher_simulate(...)` multi-generation wrapper**
7. **`ops::bio::batched_multinomial_occupancy(...)` → occupancy fractions**
8. **`ops::bio::batched_multinomial_tier_rate(...)` → detection rates**

Additionally, ToadStool may want to note that groundSpring now uses
`spectral::detect_bands` (delegation #32) — this proves the hotSpring
spectral module is cross-spring useful beyond nuclear physics.

---

## Verification

```
cargo check --features barracuda       → PASS
cargo check --features barracuda-gpu   → PASS
cargo test --workspace                 → 0 failures
cargo clippy --all-features -D warnings → 0 warnings
three_mode_benchmark.sh               → 279/279 × 3 modes
parity_report.py                      → 28/28 PROVEN
```
