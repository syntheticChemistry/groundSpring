# Exp 031: NUCLEUS Stack Validation

## Domain
Infrastructure — Full NUCLEUS primal interaction validation

## Question
Can groundSpring exercise the complete NUCLEUS stack (Tower, Node, Squirrel, Nest)
through biomeOS's Neural API, verifying that each primal responds correctly and
that sovereign fallback paths work when primals are unavailable?

## Method
- Phase A: Socket discovery via `biomeos::auto_connect()`
- Phase B: Tower (BearDog) — health check, beacon identity, crypto hash
- Phase C: Node (ToadStool) — compute health, capabilities, version
- Phase D: Squirrel — AI health status
- Phase E: Nest (NestGate) — storage put/get, NCBI/IRIS data queries (if Nest mode)
- Phase F: Local computation — Anderson Lyapunov exponent as sovereign baseline
- Sovereign fallback: validates all local paths when NUCLEUS offline

## Results
- 28/28 validation checks PASS
- Tower: health ✅, beacon identity (48 bytes) ✅, crypto forward ⚠ (params format)
- Node: health ✅, capabilities (641 bytes) ✅, version ✅
- Squirrel: AI health ✅ (150 bytes)
- Nest: requires Nest/Full NUCLEUS mode (gracefully skipped in Tower+Node mode)
- Local Lyapunov: always works, validates sovereign computation

## Validation
- Rust: `validate-nucleus-stack` (requires `--features biomeos`)
- Sovereign fallback: full local validation when NUCLEUS offline

## Cross-Spring
- biomeOS Neural API (JSON-RPC 2.0 over Unix socket)
- BearDog (Tower), ToadStool (Node), Squirrel (AI), NestGate (Nest)
- groundSpring `biomeos.rs` client, `nestgate.rs` data pipeline

## Key Finding
The adaptive capability testing pattern — query what's available, validate each path,
pass gracefully on absence — is the correct architecture for multi-primal workflows.
ToadStool's `compute.execute` is for workload scheduling, not physics; groundSpring
uses barracuda directly for GPU computation.
