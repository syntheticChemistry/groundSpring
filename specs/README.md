# groundSpring Specifications

**Last Updated**: February 25, 2026
**Status**: Phase 0 (Python) + Phase 1a (Rust) complete
**Domain**: Measurement noise, inverse problems, sensing systems, uncertainty quantification

---

## Quick Status

| Metric | Value |
|--------|-------|
| Phase 0 (Python) | 5/5 experiments PASS across 4 scientific domains |
| Phase 1a (Rust) | 60/60 PASS — 3 validation binaries (decompose, rarefaction, seismic) |
| pytest | 31/31 PASS — unit, determinism, integration tests |
| Exp 001 | Sensor noise decomposition — EC5 bias-dominated, CS616 mixed |
| Exp 002 | Observation gap ERA5 vs station — methodology validated |
| Exp 003 | Error propagation FAO-56 — humidity dominates |
| Exp 004 | Sequencing depth noise — genus saturation at 5,000 reads |
| Exp 005 | Seismic source inversion — regional accuracy demonstrated |
| Faculty | Bazavov (CMSE + Physics, MSU), Waters (MMG, MSU), Liu (CMSE, MSU) |

---

## Specifications

### Validation & Reproduction

| Spec | Status | Description |
|------|--------|-------------|
| [PAPER_REVIEW_QUEUE.md](PAPER_REVIEW_QUEUE.md) | Active | Papers to review/reproduce, prioritized by tier |
| [BARRACUDA_REQUIREMENTS.md](BARRACUDA_REQUIREMENTS.md) | Active | GPU kernel requirements and gap analysis |
| [BARRACUDA_EVOLUTION.md](BARRACUDA_EVOLUTION.md) | Active | Module → GPU promotion mapping (Tier A/B/C) |

### Existing Documentation (in parent directories)

| Document | Location | Description |
|----------|----------|-------------|
| CONTROL_EXPERIMENT_STATUS.md | `../` | Detailed experiment logs and check counts |
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
- GPU computation (ToadStool/BarraCUDA)

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
`../whitePaper/STUDY.md` → BARRACUDA_REQUIREMENTS.md → cross-spring connections in whitePaper

---

## License

**AGPL-3.0** — GNU Affero General Public License v3.0

All groundSpring code, data, and documentation are aggressively open science. See `../LICENSE` for full text. Any derivative work, including network-accessible services using groundSpring code, must publish source under the same license.
