# Experiment 023: No-Till vs Tilled 16S Sampling Design

**Domain**: Cross-spring (microbial ecology + soil management)
**Paper**: Anderson, Sogin, Baross (2015) FEMS Microbiol Ecol 91:fiu016
**Phase 0**: 7/7 PASS (Python)
**Phase 1**: 7/7 PASS (Rust)
**Barracuda**: Uses rarefaction + rare_biosphere modules

## Question

Does the saturation depth differ between no-till (high diversity) and tilled (low diversity) soil communities?

## Method

1. Pre-computed synthetic communities: no-till (150 genera, log-normal) and tilled (100 genera, more dominant species)
2. Rarefaction at 6 depths: 100, 500, 1000, 5000, 10000, 50000 reads
3. Shannon diversity and Chao1 richness at each depth
4. Saturation depth (5% convergence threshold)
5. Community distinguishability analysis

## Key Results

- No-till Shannon: 3.88 (150 genera), Tilled Shannon: 1.57 (100 genera)
- No-till Chao1: 149 at D=50000, Tilled Chao1: 99 at D=50000
- Both saturate at ~500 reads (5% threshold)
- Communities distinguishable at 1000 reads

## Files

| File | Description |
|------|-------------|
| `control/notill_sampling/notill_sampling.py` | Python baseline |
| `control/notill_sampling/benchmark_notill_sampling.json` | Benchmark config (includes pre-computed communities) |
| `crates/groundspring-validate/src/validate_notill_sampling.rs` | Rust validation binary |

## Cross-Spring

Extends Exp 004 (sequencing noise) and Exp 016 (rare biosphere).
Contributes sampling design for baseCamp Sub-thesis 06.
