# groundSpring Specifications

**Last Updated**: March 7, 2026
**Status**: V98 — Phase 0 + Phase 1 + Phase 2a + Phase 4 (NUCLEUS) — 395/395 PASS (340 core + 55 NUCLEUS), 102 delegations (61 CPU + 41 GPU) — 936 workspace tests, 382 provenance tests, 187 metalForge checks, 35 experiments, 34 modules. Runtime f64 reduction smoke test + three-tier parity proven: 29/29 validation binaries PASS at default CPU, barracuda-CPU, and barracuda-GPU tiers
**Domain**: Measurement noise, inverse problems, sensing systems, uncertainty quantification

---

## Quick Status

| Metric | Value |
|--------|-------|
| Phase 0 (Python) | 28/28 experiments PASS across 9 scientific domains (~288 checks) |
| Phase 1 (Rust) | 395/395 PASS — 34 validation binaries (340 core + 55 NUCLEUS) |
| Phase 4 (NUCLEUS) | 55/55 PASS — 4 NUCLEUS validation binaries (Exp 029–032) |
| Total Validation | 395/395 PASS across 34 experiments |
| Mathematical Parity | 28/28 PROVEN (Python ⇌ Rust against shared benchmark JSONs) |
| Rust tests | 925 workspace + 261 Python = 1186+ total |
| metalForge | 2 production WGSL shaders (anderson_lyapunov, anderson_lyapunov_f32) |
| Exp 001 | Sensor noise decomposition — EC5 bias-dominated, CS616 mixed |
| Exp 002 | Observation gap ERA5 vs station — methodology validated |
| Exp 003 | Error propagation FAO-56 — humidity dominates |
| Exp 004 | Sequencing depth noise — genus saturation at 5,000 reads |
| Exp 005 | Seismic source inversion — regional accuracy demonstrated |
| Exp 006 | Signal specificity (c-di-GMP) — SNR scales with activation |
| Exp 007 | RAWR resampling — competitive or better than naive bootstrap |
| Exp 008 | Anderson localization — Thouless scaling verified |
| Exp 009 | Almost-Mathieu quasiperiodic — Aubry-André at λ=2 |
| Exp 010 | Bistable switching — noise-induced phenotypic transitions |
| Exp 011 | Multi-signal QS — dual signaling sharpens regulation |
| Exp 012 | Spin chain transport — ballistic→localized transition |
| Exp 013 | Resampling convergence — bootstrap width converges by 2000 |
| Exp 014 | Drift vs selection — N×s threshold determines drift/selection dominance |
| Exp 015 | Uncertainty bridge — sensor noise → Anderson ξ propagation |
| Exp 016 | Rare biosphere — sequencing depth determines rare taxa signal boundary |
| Exp 017 | Quasispecies threshold — Eigen's error threshold predicts information collapse |
| Exp 018 | Band edge structure — transfer matrix reproduces tight-binding band gaps |
| Exp 019 | Jackknife error estimation — subpercent precision (Bazavov 2025 Phys Rev D) |
| Exp 020 | Freeze-out inverse problem — inferring freeze-out conditions (Bazavov 2016) |
| Exp 021 | Spectral function reconstruction — signal recovery from noisy lattice data (Bazavov 2025) |
| Exp 022 | ET₀ → Anderson propagation — humidity-dominated ET₀ error → localization length CV |
| Exp 023 | No-Till vs Tilled sampling — saturation depth by soil management regime |
| Exp 024 | Aggregate stability noise — WSA measurement precision vs Anderson regime discrimination |
| Exp 025 | f32 vs f64 precision drift — Green-Kubo f32 accumulation bias fraction ~28% |
| Exp 026 | System-size convergence — transport coefficient finite-size extrapolation R² > 0.999 |
| Exp 027 | GPU vendor parity — cross-vendor transport coefficient agreement at 1e-12 relative |
| Exp 028 | NPU Anderson — Anderson regime classification on AKD1000 via int8 DMA |
| Exp 029 | Real GHCND ET₀ — Hargreaves vs Penman-Monteith on real/synthetic NOAA weather (NUCLEUS) |
| Exp 030 | Real NCBI 16S — rare biosphere detection on real/synthetic NCBI metagenomes (NUCLEUS) |
| Exp 031 | NUCLEUS Stack — full primal validation: Tower + Node + Squirrel + Nest |
| Exp 032 | IRIS Seismic — IRIS FDSN station geometry + travel times via NestGate (NUCLEUS) |
| Barracuda | 102 delegations (61 CPU + 41 GPU) — barraCuda `a898dee`. GPU grid adapters + batch APIs. 187 metalForge checks. PrecisionRoutingAdvice wired |
| NUCLEUS | biomeOS Neural API live: Tower, Node, Squirrel validated; NestGate data pipelines (NCBI, NOAA, IRIS) |
| Performance | 11.5× faster than Python (excl. LAPACK-bound); 5.1× overall |
| Faculty | Bazavov, Waters, Liu, Kachkovskiy, R. Anderson, Dolson, Gonzales |

---

## Specifications

### Validation & Reproduction

| Spec | Status | Description |
|------|--------|-------------|
| [PROVENANCE_SCHEMA.md](PROVENANCE_SCHEMA.md) | Active | Benchmark JSON provenance schema — required/optional fields, enforcement |
| [PAPER_REVIEW_QUEUE.md](PAPER_REVIEW_QUEUE.md) | Active | Papers to review/reproduce, prioritized by tier |
| [BARRACUDA_REQUIREMENTS.md](BARRACUDA_REQUIREMENTS.md) | Active | GPU kernel requirements and gap analysis |
| [BARRACUDA_EVOLUTION.md](BARRACUDA_EVOLUTION.md) | Active | Module → GPU promotion mapping (Tier A/B/C) |
| [PRIMAL_INTERACTION_EVOLUTION.md](PRIMAL_INTERACTION_EVOLUTION.md) | Active | NUCLEUS Neural API evolution (V0–V6), interaction map |
| [LAN_DEPLOYMENT_READINESS.md](LAN_DEPLOYMENT_READINESS.md) | Active | LAN HPC readiness assessment |

### NUCLEUS Evolution

| Spec | Status | Description |
|------|--------|-------------|
| [PRIMAL_INTERACTION_EVOLUTION.md](PRIMAL_INTERACTION_EVOLUTION.md) | Active | Socket discovery → capability routing → data pipelines → multi-primal |
| [CROSS_SPRING_EVOLUTION.md](CROSS_SPRING_EVOLUTION.md) | Active | How springs evolve barracuda, and NUCLEUS interaction patterns |

### GPU Evolution

| Spec | Status | Description |
|------|--------|-------------|
| metalForge/ABSORPTION_MANIFEST.md | Active | Write → Absorb → Lean inventory |
| metalForge/shaders/ | Active | Production WGSL shaders for Tier C absorption |

### Existing Documentation (in parent directories)

| Document | Location | Description |
|----------|----------|-------------|
| CONTROL_EXPERIMENT_STATUS.md | `../` | Detailed experiment logs and check counts |
| CHANGELOG.md | `../` | Release history and notable changes |
| whitePaper/STUDY.md | `../whitePaper/` | Full study with cross-domain synthesis |
| whitePaper/METHODOLOGY.md | `../whitePaper/` | Experimental design and acceptance criteria |

---

## Scope

### groundSpring IS:
- **Noise characterization** — decomposing measurement error into bias + random components
- **Inverse problems** — inferring hidden parameters from noisy observations
- **Error propagation** — tracking uncertainty through equation chains
- **Sensing system analysis** — what instruments tell us vs. what is true
- **The uncertainty budget** for all other springs

### groundSpring IS NOT:
- Machine learning (neuralSpring)
- Domain-specific pipelines (airSpring, wetSpring, hotSpring)
- GPU computation (ToadStool/BarraCUDA) — but writes production shaders for absorption

### groundSpring EXTENDS TO (via faculty):
- **Bazavov**: Spectral reconstruction, lattice QCD inverse problems, subpercent precision
- **Waters**: Biological signal specificity — quorum sensing as a noisy sensor network
- **Liu**: Statistical resampling for confidence (RAWR bootstrap, phylogenetic uncertainty)
- **Dolson**: Eco-evolutionary noise — emergence of organization from randomness

### groundSpring INFORMS:
- airSpring: which sensor matters most for ET₀ (humidity at 66%)
- wetSpring: minimum sequencing depth for genus-level taxonomy (5,000 reads)
- neuralSpring: uncertainty labels for transfer learning domain gap quantification
- hotSpring: noise floor expectations for GPU computation validation

---

## Reading Order

**New to groundSpring** (15 min):
1. This README (5 min)
2. `../whitePaper/README.md` — overview and key results (5 min)
3. PAPER_REVIEW_QUEUE.md — what's next (5 min)

**Deep dive** (1 hour):
`../whitePaper/STUDY.md` → BARRACUDA_EVOLUTION.md → `../metalForge/ABSORPTION_MANIFEST.md`

---

## License

**AGPL-3.0** — GNU Affero General Public License v3.0

All groundSpring code, data, and documentation are aggressively open science. See `../LICENSE` for full text. Any derivative work, including network-accessible services using groundSpring code, must publish source under the same license.
