# Liu — Statistical Resampling & Confidence

**Faculty**: Kevin Liu (CMSE, Michigan State University)
**Domain**: Phylogenetics, statistical resampling, bioinformatics algorithms
**groundSpring Connection**: Exp 007 (RAWR bootstrap) — DONE. Exp 003 (Monte Carlo error propagation) upgrade to modern resampling.

---

## Why This Matters for groundSpring

groundSpring Exp 003 uses naive Monte Carlo (N=10,000 random draws) to propagate
sensor uncertainties through the FAO-56 equation chain. Liu's RAWR (Resampling
Approximate Weighted Resampling) is a modern alternative that achieves tighter
confidence intervals with fewer samples by using weighted resampling instead of
uniform random draws. Adopting RAWR could improve both the efficiency and
accuracy of groundSpring's error propagation framework across all experiments.

## Papers for Reproduction

### Tier 1 (Priority)

**Paper #12**: Wang et al. (2021) "Build a better bootstrap and the RAWR shall
beat a random path to your door." Bioinformatics (ISMB) 37:i111-i119.
DOI: 10.1093/bioinformatics/btab263

- **Open Data**: Simulation parameters fully specified in Methods; phylogenetic
  test datasets available from TreeBASE and Dryad
- **Open Code**: RAWR implementation available as open-source (C++ / Python)
- **groundSpring Modules**: `stats` (percentile, bootstrap extension),
  `prng` (weighted sampling), `bootstrap` module (RAWR in `bootstrap.rs`)
- **BarraCUDA Needs**: Parallel resampling (embarrassingly parallel — good GPU target),
  weighted sampling kernel
- **Control Plan**: Python RAWR baseline → Rust CPU → barracuda GPU

### Tier 2

**Paper #13**: Lee & Liu (2024) "A Statistical Optimization Technique to Inform
Statistical Resampling Assessments." IEEE BIBM 2024.

- **Open Data**: Simulation framework documented; test cases reproducible
- **Method**: Meta-statistical optimization — improving the resampling strategy itself
- **groundSpring Modules**: `stats` extension for adaptive resampling

## BarraCUDA Kernel Requirements

| Primitive | Status | Notes |
|-----------|--------|-------|
| Bootstrap resampling | `bootstrap_mean` + `rawr_mean` delegated | RAWR in `bootstrap.rs`; `rawr_mean` delegates to `barracuda::stats::rawr_mean` (S66) |
| Parallel weighted draws | PRNG + rejection sampling | GPU embarrassingly parallel |
| Percentile computation | CPU in groundSpring | GPU reduce for large samples |
| Tree topology scoring | Partial (via wetSpring) | For phylogenetic application |

## Three-Tier Control Plan

| Tier | Validation | Status |
|------|-----------|--------|
| CPU | Python RAWR matches Rust RAWR | **DONE** (11/11 PASS, 7.3× faster) |
| GPU | barracuda parallel RAWR matches CPU | `bootstrap_mean` + `rawr_mean` delegated (S66) |
| metalForge | Cross-substrate for large-N resampling | After GPU tier |

## Cross-Spring

- **wetSpring**: Liu is a primary wetSpring faculty member (phylogenetics,
  RAWR, tree confidence). groundSpring adds the error propagation perspective.
- **neuralSpring**: Confidence estimation methods transfer to ML uncertainty.

## Extension Roadmap (historical — written at V114)

### V115 Capabilities

- `bootstrap_mean`, `rawr_mean`, `bootstrap_median`, `bootstrap_std` now return
  `Result<BootstrapResult, InputError>` — zero panicking public APIs
- New error-path tests: empty data, out-of-range confidence, insufficient samples
- CI: `--all-features` doc/test, nursery lint enforcement, biomeOS validation jobs
- ecoBin: 14 C-dependency crates banned in `deny.toml`, UniBin flags

### V114 Capabilities

- `.expect()` → `OrExit` in validation binaries (validate-rawr, validate-resampling-conv)
- `cast::` helpers in bootstrap module — checked numeric conversions
- `health.liveness`/`health.readiness` for NUCLEUS deployment
- `resilient_call()` for IPC to ToadStool GPU dispatch

### Extension Opportunities

- **RAWR GPU kernel**: `rawr_mean` not yet in barraCuda — parallel weighted resampling is an ideal GPU target
- **Exp 003 upgrade**: Replace naive Monte Carlo in FAO-56 error propagation with RAWR for tighter CIs
- **Adaptive resampling**: Lee & Liu (2024) meta-statistical optimization — choose resampling strategy dynamically
- **Phylogenetic application**: RAWR on tree topologies from TreeBASE/Dryad (via wetSpring)
- **Cross-spring**: neuralSpring confidence estimation methods can adopt RAWR for ML uncertainty quantification

### Compute Budget

| Workload | Single GPU (RTX 4070) | LAN |
|----------|-----------------------|-----|
| RAWR 10K replicates × 1K samples | ~10s (CPU) | N/A |
| RAWR 100K replicates × 10K samples | ~5min GPU | N/A |
| Phylogenetic RAWR (TreeBASE) | ~1h | ~10min |

### New Experiments (Planned)

- **Exp 036+**: RAWR-enhanced FAO-56 error propagation (upgrade Exp 003)
- **Exp 037+**: Adaptive resampling strategy selection per dataset characteristics
- `rawr_mean` GPU kernel → barraCuda absorption target (P2 request in V114 handoff)

### Primal Wiring

- ToadStool: `compute.execute` for GPU RAWR when kernel is available
- NestGate: TreeBASE/Dryad phylogenetic dataset storage
- Node Atomic sufficient (Tower + ToadStool) — compute-bound
