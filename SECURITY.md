# Security Policy

SPDX-License-Identifier: AGPL-3.0-or-later

## Supported Versions

| Version | Supported |
|---------|-----------|
| V135+ (current) | Yes |

## Security Model

groundSpring is a scientific validation Spring — it does not handle user
authentication, network-facing services, or sensitive data directly. Its
security posture derives from the ecoPrimals sovereign stack:

- **Pure Rust**: `#![forbid(unsafe_code)]` across all workspace crates.
  Zero C dependencies in application code (ecoBin compliant).
- **cargo-deny**: Continuous advisory scanning via RustSec database,
  license compliance, and source provenance checks (`deny.toml`).
- **No vendor lock-in**: Zero proprietary dependencies. All computation
  is sovereign (runs on your hardware, no cloud calls).
- **IPC isolation**: JSON-RPC 2.0 over Unix domain sockets with
  capability-based discovery. No shared memory, no global state.
- **Deterministic validation**: Fixed seeds, named tolerances, provenance
  tracing. All 1,125 tests are rerun-identical.
- **NDJSON output hardening**: Structured validation output (NDJSON sink)
  escapes all string fields per RFC 8259 to prevent JSON injection.

## Reporting a Vulnerability

If you discover a security issue:

1. **Do not** open a public issue.
2. Contact the ecoPrimals maintainers via the repository's security
   advisory feature (GitHub → Security → Advisories → New draft).
3. Include: affected component, reproduction steps, potential impact.
4. Expected response time: 72 hours for acknowledgment.

## Dependencies

All dependencies are pure Rust and audited via `cargo-deny`. The dependency
tree is reviewed on every commit. Optional transitive `-sys` crates (from
`wgpu` GPU HAL behind `barracuda-gpu` feature) are infrastructure-level
and do not process untrusted input.

## Data Provenance

All datasets used in validation are from public repositories (SRA, Zenodo,
EPA, PDB, NOAA CDO, IRIS FDSN) with documented accession numbers in
`specs/PROVENANCE_SCHEMA.md`. No proprietary or sensitive data is included
in this repository.
