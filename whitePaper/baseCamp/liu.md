# Liu — Statistical Resampling & Confidence

**Faculty**: Kevin Liu (CMSE, Michigan State University)
**Domain**: Phylogenetics, statistical resampling, bioinformatics algorithms
**groundSpring Connection**: Exp 003 (Monte Carlo error propagation) upgrade to modern resampling

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
  `prng` (weighted sampling), new `rawr` module
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
| Bootstrap resampling | `stats::bootstrap_*` (CPU) | Extend to RAWR weighted |
| Parallel weighted draws | PRNG + rejection sampling | GPU embarrassingly parallel |
| Percentile computation | CPU in groundSpring | GPU reduce for large samples |
| Tree topology scoring | Partial (via wetSpring) | For phylogenetic application |

## Three-Tier Control Plan

| Tier | Validation | Status |
|------|-----------|--------|
| CPU | Python RAWR matches Rust RAWR | Queued |
| GPU | barracuda parallel RAWR matches CPU | After CPU tier |
| metalForge | Cross-substrate for large-N resampling | After GPU tier |

## Cross-Spring

- **wetSpring**: Liu is a primary wetSpring faculty member (phylogenetics,
  RAWR, tree confidence). groundSpring adds the error propagation perspective.
- **neuralSpring**: Confidence estimation methods transfer to ML uncertainty.
