# Benchmark JSON Provenance Schema

**Last Updated**: March 6, 2026
**Status**: Active — enforced by `tests/test_baseline_integrity.py` (261 tests)
**Scope**: All `control/*/benchmark_*.json` files

---

## Purpose

Every benchmark JSON carries a machine-auditable provenance chain:
**paper → Python → JSON → Rust → pass/fail**. This schema defines
required and optional fields so that new experiments, primals, and
automation can produce and validate provenance without reading test code.

---

## Required Fields

| Field | Location | Type | Description |
|-------|----------|------|-------------|
| `_source` | Top-level | string | Human-readable experiment title (e.g. "groundSpring Exp 005 — Seismic Wave Propagation") |
| `_provenance` | Top-level | object | Provenance metadata block (see below) |
| `_provenance.baseline_date` | Nested | string | ISO-8601 date of last Python run (e.g. "2026-02-16") |
| `_provenance.baseline_commit` | Nested | string | Git SHA of the commit that produced these values (40-char hex or "unknown") |
| `_provenance.validation_script` | Nested | string | Relative path to the Python script (e.g. "control/seismic/seismic_inversion.py") |
| `_provenance.command` | Nested | string | Exact command to reproduce (e.g. "python3 control/seismic/seismic_inversion.py") |
| `_provenance.real_data_accession` | Nested | string | Public accession number or "N/A (analytical)" |
| `_doi` or `_doi_era5` or `_doi_ghcnd` | Top-level | string | DOI of the primary reference paper |

---

## Optional Fields

| Field | Location | Type | Description |
|-------|----------|------|-------------|
| `_provenance.generated_by` | Nested | string | Brief description of how values were produced |
| `_provenance.data_origin` | Nested | string | Where input data came from |
| `_provenance.notes` | Nested | string | Additional context |
| `_provenance.python_version` | Nested | string | Python version used for baseline |
| `_provenance.numpy_version` | Nested | string | NumPy version used for baseline |
| `_provenance.prng_algorithm` | Nested | string | PRNG identity and seed (required for stochastic experiments) |
| `_description` | Top-level | string | One-line experiment description |
| `_groundspring_question` | Top-level | string | The scientific question this experiment answers |
| `_references` | Top-level | array | List of reference citations |

---

## Enforcement

### Python: `tests/test_baseline_integrity.py`

- `test_has_source` — `_source` present
- `test_has_provenance_block` — `_provenance` object present
- `test_provenance_has_required_fields` — `baseline_date`, `baseline_commit`, `validation_script`, `command`, `real_data_accession`
- `test_baseline_commit_is_hex` — 40-char hex or "unknown"
- `test_has_doi` — at least one of `_doi`, `_doi_era5`, `_doi_ghcnd`
- `test_has_real_data_accession` — `real_data_accession` in `_provenance`
- `test_json_is_valid_utf8` — valid UTF-8 encoding

### Rust: `groundspring-validate` harness

- `print_provenance_header()` panics if `_source`, `baseline_commit`, or `baseline_date` are missing
- All validation binaries call `print_provenance_header()` before checks

### Drift Guard: `scripts/regenerate_benchmarks.sh`

- Re-runs all Python baselines
- Verifies `baseline_commit` matches `HEAD`
- `--stamp` updates `baseline_commit` and `baseline_date`
- Auto-discovers benchmarks via `find control -name 'benchmark_*.json'`

---

## Stochastic Experiments

Experiments using random sampling **must** include `_provenance.prng_algorithm`
with the algorithm name, version, and seed. Example:

```json
"prng_algorithm": "xorshift64 (Marsaglia 2003), seed=42"
```

When PRNG alignment occurs (e.g. xorshift64 → xoshiro128**), all
stochastic benchmark JSONs must be regenerated with updated provenance.

---

## Data Sources

All validation datasets must come from public repositories with documented
accession numbers:

| Source | Example Accession |
|--------|-------------------|
| NCBI/SRA | SRR identifier |
| NOAA GHCND | Station ID (e.g. USW00094860) |
| IRIS FDSN | Network.Station (e.g. NM.SIUC) |
| Zenodo | DOI |
| EPA | Accession or URL |
| Analytical | "N/A (analytical)" with formula reference |

---

## License

**AGPL-3.0-or-later** — All benchmark JSONs and provenance metadata are open science.
