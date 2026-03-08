# groundSpring V99 — Live NUCLEUS Integration Handoff

**Date**: March 8, 2026
**Author**: groundSpring validation team
**Supersedes**: V98 Upstream Rewire Handoff

## Summary

groundSpring achieved its **first live NUCLEUS connection**, with the biomeOS
Neural API and direct primal sockets both operational. The biomeos client was
evolved to handle protocol version mismatches adaptively and to discover
primal sockets directly when Neural API routing is unavailable.

## What Changed

### biomeOS Client Evolution (`crates/groundspring/src/biomeos/`)

| Function | Before | After |
|----------|--------|-------|
| `health()` | `topology.metrics` (single method) | Tries `neural_api.get_metrics` → `topology.metrics` (adaptive) |
| `auto_connect()` | Failed on live stack (method not found) | **CONNECTED** (uses evolved health) |
| `discover_primals()` | N/A | Scans `$XDG_RUNTIME_DIR/biomeos/` for primal sockets |
| `primal_health(name)` | N/A | Direct health check to individual primals |
| `direct_primal_rpc()` | N/A | Bypass Neural API, talk to primal sockets directly |
| `proprioception()` | N/A | Query Neural API self-awareness |
| `topology()` | N/A | Query Neural API primal connections |

### Protocol Finding: Neural API Binary Version

The running Neural API binary (`plasmidBin/primals/biomeos`) uses
`neural_api.*` namespaced methods (e.g. `neural_api.get_metrics`), not
the short aliases (`topology.metrics`). This means:

- `capability.call` is not available in this binary version
- groundSpring now falls back to direct primal socket calls
- The adaptive health check handles both naming conventions

**Action for biomeOS team**: When updating the Neural API binary, ensure both
`neural_api.get_metrics` and `topology.metrics` respond (backward compat).

### NestGate Binary Version Mismatch

The NestGate binary does not support the `daemon` subcommand used by
`start_nucleus.sh`. The socket never materializes.

**Action for NestGate team** (P1): Rebuild NestGate with `daemon` or
`service start --socket-only` support.

## Live NUCLEUS Results

```
NUCLEUS Full Mode
  Family ID:     8ff3b864a4bc589a
  Architecture:  x86_64
  Primals:       4 (beardog, songbird, toadstool, squirrel)
  
  Neural API:
    neural_api.get_metrics    ✅ (system: 41GB RAM, 12.9% CPU)
    neural_api.get_topology   ✅ (songbird→beardog, toadstool→songbird)
    neural_api.get_proprioception ✅ (can_deploy: true, can_execute_graphs: true)
  
  Direct Primal Health:
    BearDog   ✅ v0.9.0 (security, TCP :9900)
    ToadStool ✅ v0.1.0 (compute, JSON-RPC 2.0)
    Squirrel  ✅ (AI bridge, Anthropic+OpenAI loaded)
    Songbird  ✅ (network, TCP :3492 + IPC)
    NestGate  ⚠  socket pending (binary version mismatch)
```

## Validation Results

| Experiment | Checks | Result |
|-----------|--------|--------|
| Exp 029: GHCND ET₀ | 6/6 | PASS (synthetic fallback, NOAA via NestGate unavailable) |
| Exp 030: NCBI 16S | 9/9 | PASS (synthetic fallback, NCBI via NestGate unavailable) |
| Exp 031: NUCLEUS Stack | 16/16 | PASS (Neural API + direct primal + sovereign) |
| Exp 032: IRIS Seismic | 9/9 | PASS (synthetic fallback, IRIS via NestGate unavailable) |
| **Total** | **40/40** | **PASS** |

Full test suite: 936 tests PASS, clippy clean.

## Cross-Spring Evolution Notes

### How This Helps Other Springs

The adaptive health check pattern (`try evolved method → fall back to alias`)
and direct primal discovery (`scan socket dir → connect → check health`) can
be adopted by any spring's biomeOS client. The pattern handles:

- Binary version mismatches (old Neural API doesn't have short aliases)
- Partial deployments (some primals running, others not)
- NestGate absence (sovereign fallback with synthetic data)

### What groundSpring Needs from biomeOS

1. **P1**: NestGate binary with `daemon` subcommand
2. **P2**: Neural API binary with `capability.call` routing support
3. **P3**: `capability.register` endpoint to register groundSpring's
   7 science capabilities with the NUCLEUS registry

## Quality Gates

- 936 tests PASS (`--features biomeos`)
- clippy pedantic+nursery: 0 warnings
- 39/42 validation binaries PASS (3 pre-existing metalForge GPU tier issues)
- AGPL-3.0-only, SPDX headers on all files
