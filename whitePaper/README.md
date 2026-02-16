# groundSpring White Paper

## The Dirty Differences: Characterizing Measurement Noise Across Scientific Domains

### Purpose

This white paper documents groundSpring's systematic approach to quantifying the gap between what models predict and what instruments actually measure. Where airSpring validates clean FAO-56 equations and wetSpring validates taxonomy pipelines, groundSpring asks: **"how confident are we in these numbers?"**

### Status

Phase 0 baselines completed: **71/71 quantitative checks passed** across 5 experiments spanning 4 scientific domains.

### Key Results

| Experiment | Domain | Tests | Key Finding |
|------------|--------|-------|-------------|
| 001: Sensor Noise | Agricultural sensors | 32/32 | EC5 bias-dominated (77%); CS616 mixed noise structure |
| 002: Observation Gap | Meteorology | 5/5 | ERA5 vs station: methodology validated (real NOAA data pending) |
| 003: Error Propagation | ET0 uncertainty | 8/8 | Humidity dominates ET0 variance (66%); MC/analytical agree to 1% |
| 004: Sequencing Noise | Microbiome | 16/16 | Genus saturation at 5000 reads; Shannon converges by 500 reads |
| 005: Seismic Inversion | Geophysics | 10/10 | ±0.5s noise → 2km location uncertainty; depth poorly constrained |

### Key Research Questions Answered

1. **How much sensor error is correctable?** 50-80% of total soil moisture sensor error is systematic bias that can be removed with site-specific calibration (Exp 001).

2. **Which measurement matters most for ET0?** Humidity sensor accuracy dominates ET0 uncertainty (66% of variance), followed by radiation (20%) and temperature (10%) (Exp 003).

3. **When does more sequencing stop helping?** Above 5000 reads, genus discovery yields diminishing returns. Shannon diversity stabilizes by 500 reads (Exp 004).

4. **How does noise propagate through an inverse problem?** ±0.5s arrival time noise produces ~2km horizontal location uncertainty but ~8.5km depth uncertainty — the classic tradeoff between well-constrained and poorly-constrained parameters (Exp 005).

### Documents

- [METHODOLOGY.md](METHODOLOGY.md) — Experimental design and validation approach
- [STUDY.md](STUDY.md) — Detailed results and analysis
