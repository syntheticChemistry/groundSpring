# groundSpring → ToadStool/BarraCUDA Handoff V39

**Date**: February 27, 2026
**From**: groundSpring (Eastgate)
**To**: ToadStool / BarraCUDA core team
**Covers**: NUCLEUS integration, NestGate data pipeline, metalForge remote discovery, baseCamp synchronization
**License**: AGPL-3.0-or-later
**Previous**: V37 (BarraCUDA Evolution) — remains active (companion)

---

## Executive Summary

V39 extends groundSpring from a single-machine validation framework into a
NUCLEUS-ready science primal. Three new capabilities:

1. **NestGate data pipeline** — NCBI genome search/fetch and NOAA weather data via biomeOS, with provenance storage and cache-through
2. **metalForge remote substrate discovery** — parse and merge remote NUCLEUS node inventories for multi-gate GPU/NPU dispatch
3. **NUCLEUS pipeline graphs** — Tower bootstrap, Node atomic, and cross-substrate validation orchestrated through biomeOS

All 32 active delegations (25 CPU + 7 GPU) unchanged. 9 pending ToadStool
absorptions unchanged. No new barracuda API requirements.

---

## Part 1: Current BarraCUDA State (unchanged from V37)

### Active Delegations: 32

| Tier | Count | Functions |
|------|-------|-----------|
| CPU (`barracuda`) | 25 | pearson_r, spearman_r, covariance, norm_cdf, norm_ppf, chi2_statistic, rmse, mae, nash_sutcliffe, mbe, r_squared, index_of_agreement, hit_rate, mean, sample_std_dev, percentile, fit_linear, bootstrap_mean, rawr_mean, analytical_localization_length, shannon_diversity, evenness, hill, bistable_derivative, multisignal_derivative |
| GPU (`barracuda-gpu`) | 7 | lyapunov_exponent, lyapunov_averaged, level_spacing_ratio, almost_mathieu_hamiltonian, almost_mathieu_eigenvalues, detect_band_ranges, tikhonov_solve |

### Pending ToadStool Absorption: 9

| Function | Target | Blocker |
|----------|--------|---------|
| `kimura_fixation_prob` | `stats::kimura_fixation` | Not yet in barracuda |
| `jackknife_mean_variance` | `stats::jackknife_mean_variance` | Not yet in barracuda |
| `daily_et0` | `stats::hydrology::fao56_et0` | Not yet in barracuda |
| `grid_fit_2d` | `ops::grid::grid_fit_2d_f64` | Not yet in barracuda |
| `grid_search_3d` | `ops::grid::grid_search_3d_f64` | Not yet in barracuda |
| `band_edges_parallel` | `spectral::band_edges_parallel` | Not yet in barracuda |
| `wright_fisher_simulate` | `ops::bio::wright_fisher_simulate` | Not yet in barracuda |
| `batched_multinomial_occupancy` | `ops::bio::batched_multinomial_occupancy` | Not yet in barracuda |
| `batched_multinomial_tier_rate` | `ops::bio::batched_multinomial_tier_rate` | Not yet in barracuda |

### Production WGSL Shaders (ready for absorption)

| Shader | Lines | Purpose |
|--------|-------|---------|
| `batched_multinomial.wgsl` | 112 | Multinomial sampling with xoshiro PRNG |
| `mc_et0_propagate.wgsl` | 149 | Monte Carlo ET₀ propagation |

---

## Part 2: What's New in V39

### 2.1 NestGate Data Pipeline (`crates/groundspring/src/nestgate.rs`)

New module behind `biomeos` feature that wires NestGate live data providers
into groundSpring's experiment flow.

**Provenance key schema** (for ToadStool/NestGate coordination):

```
groundspring:results:exp{NNN}:{run_id}     — validation results
groundspring:data:{source}:{query_id}       — cached live data
groundspring:parity:exp{NNN}:{substrate}    — cross-substrate parity
groundspring:tower:{event}:{timestamp}      — NUCLEUS lifecycle events
```

**Data provider functions** (route through biomeOS `direct_rpc_call`):

| Function | NestGate Method | Use Case |
|----------|-----------------|----------|
| `ncbi_search` | `data.ncbi_search` | Search NCBI genomes (Exp 004/014/016) |
| `ncbi_fetch` | `data.ncbi_fetch` | Fetch sequences by accession |
| `noaa_ghcnd` | `data.noaa_ghcnd` | Daily weather observations (Exp 002) |
| `noaa_fao56_variables` | `data.noaa_ghcnd` | FAO-56 specific vars (TMAX, TMIN, AWND, RHAV) |
| `fetch_cached` | `storage.store` / `storage.retrieve` | Cache-through with NestGate storage |

**ToadStool relevance**: When NestGate absorbs these as server-side methods,
groundSpring will be the first science primal with live data provenance.

### 2.2 metalForge Remote Substrate Discovery (`metalForge/forge/src/remote.rs`)

New module that discovers substrates on remote NUCLEUS nodes via biomeOS
capability routing (`metalforge.discover`).

**Key types**:

| Type | Purpose |
|------|---------|
| `RemoteOrigin` | Node ID, LAN flag, estimated latency |
| `RemoteSubstrate` | `Substrate` + `RemoteOrigin` |
| `parse_remote_inventory` | Parse JSON response from remote node |
| `merge_remote` | Prefix names with `@{node_id}` and merge into local |

**Example**: After merge, a Titan V on biomeGate appears as `TITAN V@biomegate`
in the inventory. Dispatch can then route f64 workloads there.

**ToadStool relevance**: ToadStool's `ComputeDispatch` will eventually route
through this same discovery layer. The `metalforge.discover` capability call
needs a server-side handler in biomeOS that forwards to each node's local
metalForge probe.

### 2.3 NUCLEUS Pipeline Graphs

Four biomeOS pipeline graphs in `graphs/`:

| Graph | Atomic | Purpose |
|-------|--------|---------|
| `groundspring_tower_bootstrap.toml` | Tower | BearDog + Songbird on Eastgate |
| `groundspring_nucleus_node.toml` | Node | Tower + ToadStool for 28-experiment GPU validation |
| `groundspring_cross_substrate.toml` | Node + Nest | CPU → GPU → NPU parity with provenance |
| `groundspring_validation.toml` | Node (optional Nest) | Anderson localization through biomeOS |

---

## Part 3: Absorption Priorities (unchanged)

### Priority 1: Grid Search Operations

| Op | groundSpring Module | WGSL Ready? |
|----|---------------------|-------------|
| `grid_search_3d_f64` | `seismic::grid_search_inversion` | No (dispatch block only) |
| `grid_fit_2d_f64` | `freeze_out::grid_fit_2d` | No (dispatch block only) |

### Priority 2: Bio Batch Operations

| Op | groundSpring Module | WGSL Ready? |
|----|---------------------|-------------|
| `wright_fisher_simulate` | `quasispecies::quasispecies_simulation` | No |
| `batched_multinomial_occupancy` | `rare_biosphere::abundance_occupancy` | Partial (`batched_multinomial.wgsl`) |
| `batched_multinomial_tier_rate` | `rare_biosphere::tier_detection_rate` | Partial (`batched_multinomial.wgsl`) |

### Priority 3: Statistics

| Op | groundSpring Module | Notes |
|----|---------------------|-------|
| `kimura_fixation` | `drift::kimura_fixation_prob` | Scalar — low priority |
| `jackknife_mean_variance` | `jackknife::jackknife_mean_variance` | Embarrassingly parallel |
| `fao56_et0` | `fao56::daily_et0` | Already absorbed as Op; needs stats wrapper |

---

## Part 4: Cross-Spring Learnings for ToadStool

### NAK f64 Gap (confirmed V35)

NAK advertises `SHADER_F64` but ALU lowering is not implemented
(`from_nir.rs:1092: assert bit_size == 32`). All current NVIDIA GPUs
(consumer and workstation via NVK) need DF64 emulation for f64 workloads.
ToadStool's DF64 (double-float on f32 cores, ~50-bit precision) is the
bridge. This affects every spring's f64 pipeline.

### `chao1` Formula Divergence

groundSpring's `chao1` uses classic Chao 1984: `S_obs + f₁²/(2f₂)`.
BarraCUDA's `chao1` uses bias-corrected Chao & Chiu 2016:
`S_obs + f₁(f₁−1)/(2(f₂+1))`. Both are valid; groundSpring keeps local
to match its Python baseline. If ToadStool standardizes on one formula,
document which — downstream springs depend on it.

### Sovereign Fallback Pattern

All 32 delegations and all NestGate calls use the same pattern: `if let Ok`
with CPU/local fallback always compiled. This means groundSpring works
identically with or without barracuda, with or without biomeOS. ToadStool
should preserve this — no science primal should require any upstream
dependency to produce correct results.

---

## Part 5: Validation State

| Metric | Value |
|--------|-------|
| Experiments | 28 |
| Validation checks | 288/288 PASS |
| Rust tests (biomeos) | 498+ |
| Python tests | 320 |
| Active delegations | 32 (25 CPU + 7 GPU) |
| Pending ToadStool | 9 (3 CPU + 6 GPU) |
| metalForge workloads | 19 |
| metalForge tests | 49+ |
| Discovered substrates | 5+ (local) + remote via NUCLEUS |
| WGSL shaders | 2 production-ready |
| Pipeline graphs | 4 (Tower, Node, cross-substrate, validation) |
| clippy warnings | 0 |
| `cargo fmt` | clean |

---

## Part 6: Files Changed Since V37

| File | Change |
|------|--------|
| `crates/groundspring/src/nestgate.rs` | **NEW** — NestGate data pipeline |
| `crates/groundspring/src/biomeos.rs` | Added `escape_json_pub()` |
| `crates/groundspring/src/lib.rs` | Registered `nestgate` module |
| `metalForge/forge/src/remote.rs` | **NEW** — Remote substrate discovery |
| `metalForge/forge/src/lib.rs` | Registered `remote` module |
| `metalForge/forge/src/inventory.rs` | Added `merge_remote()` |
| `metalForge/ABSORPTION_MANIFEST.md` | Remote discovery marked complete |
| `graphs/groundspring_tower_bootstrap.toml` | **NEW** — Tower atomic graph |
| gen3/baseCamp/06_notill_anderson.md | Added Exp 022-024 |
| gen3/baseCamp/07_sovereign_wdm.md | Added Exp 025-027 (Section 6.3) |
| gen3/baseCamp/README.md | Expansion paragraph updated |
| whitePaper/baseCamp/anderson.md | Three-tier table updated |
| whitePaper/baseCamp/bazavov.md | GPU tier status updated |
| whitePaper/baseCamp/README.md | Cross-Spring Impact + Sub-thesis 07 |
