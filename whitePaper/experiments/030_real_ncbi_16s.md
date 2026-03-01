# Exp 030: Real NCBI 16S Rare Biosphere Detection

## Domain
Biological (NCBI) — Rare biosphere detection on real metagenome data

## Question
Can groundSpring's rare biosphere analysis (Chao1, detection power, rarefaction)
correctly characterize community structure when driven by real NCBI 16S metagenomic
data obtained through NestGate's NCBI E-utilities provider?

## Method
- Query NCBI SRA for 16S metagenome datasets via NestGate `data.ncbi_search` (if NUCLEUS live)
- Fall back to synthetic community (log-normal rank-abundance) when NUCLEUS unavailable
- Compute: Chao1 richness, Shannon diversity, detection power for rare taxa
- Validate: rarefaction sub-sampling preserves diversity ordering

## Results
- 9/9 validation checks PASS
- Chao1 estimate ≥ observed taxa count (expected: rare taxa inflate estimator)
- Detection power monotonically increases with sequencing depth
- Rarefied community retains fewer observed taxa than full community
- Shannon diversity > 0 (non-degenerate community)
- Sovereign fallback to synthetic community works seamlessly

## Validation
- Rust: `validate-real-ncbi-16s` (requires `--features biomeos`)
- Sovereign fallback: synthetic log-normal community when NUCLEUS offline

## Cross-Spring
- Uses `rarefaction::multinomial_sample`, `rare_biosphere::detection_power`, `rare_biosphere::chao1`
- NestGate NCBI E-utilities data provider via biomeOS Neural API
- wetSpring 16S methodology, groundSpring rare biosphere analysis

## Key Finding
Sovereign fallback produces communities with realistic rank-abundance structure that
exercise the same statistical paths as real NCBI data, validating the code without
requiring network access.
