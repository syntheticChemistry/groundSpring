# groundSpring V139 — Tier 2 Wiring Handoff

**Date**: May 12, 2026
**From**: groundSpring
**For**: primalSpring coordination, toadStool, barraCuda teams
**Trigger**: Ecosystem Wave Sync — toadstool.validate + barracuda.precision.route now live

---

## What Shipped

### Tier 2 IPC Wiring (3 new methods)

| Method | File | Pattern | Discovery Role |
|--------|------|---------|----------------|
| `toadstool.validate` | `ipc/toadstool.rs` | tarpc trait + biomeOS JSON-RPC | `roles::COMPUTE` ("toadstool") |
| `toadstool.list_workloads` | `ipc/toadstool.rs` | tarpc trait + biomeOS JSON-RPC | `roles::COMPUTE` ("toadstool") |
| `barracuda.precision.route` | `ipc/barracuda.rs` | tarpc trait + biomeOS JSON-RPC | `roles::GPU_MATH` ("barracuda") |

### API Contracts Implemented

**`toadstool.validate`**:
- Params: `{"workload_path": string, "dry_run": bool}`
- Returns: `{"valid": bool, "gpu_available": bool, "precision_tier": string, "estimated_dispatch_time_ms": int, "warnings": [], "required_capabilities": []}`
- Graceful degradation: `try_validate_workload()` returns `Ok(None)` if ToadStool not discovered

**`toadstool.list_workloads`**:
- Params: `{}`
- Returns: workload descriptor array
- Graceful degradation: `try_list_workloads()` returns `Ok(None)` if ToadStool not discovered

**`barracuda.precision.route`**:
- Params: `{"domain": string, "hardware_hint": string}`
- Returns: `{"recommended_tier": string, "fma_safe": bool, "requires_compiler": bool, "hardware_hint": string}`
- Graceful degradation: `try_precision_route()` returns `Ok(None)` if barraCuda not discovered

### Bug Fix: Role Constant Assertion

`ipc/barracuda.rs` had a test asserting `roles::COMPUTE == "barracuda"` — this was wrong (`roles::COMPUTE` is `"toadstool"`). Fixed by:
1. Adding `roles::GPU_MATH = "barracuda"` to `primal_names.rs`
2. Correcting the test to assert `roles::GPU_MATH == "barracuda"`

### Doc Cleanup

Backtick-formatted all primal names in IPC module doc comments to satisfy `clippy::doc_markdown` when `tarpc-ipc` + `biomeos` features are enabled.

---

## Verification

- `cargo clippy --workspace -- -D warnings`: PASS (zero warnings)
- `cargo test --workspace`: 1,123 tests, all passing
- `cargo fmt --check`: no diff

---

## Ecosystem Position

| Capability | groundSpring Status |
|------------|-------------------|
| `--format json` | DONE (all 38 binaries) |
| `toadstool.validate` | WIRED (tarpc + biomeOS) |
| `toadstool.list_workloads` | WIRED (tarpc + biomeOS) |
| `barracuda.precision.route` | WIRED (tarpc + biomeOS) |
| LTEE B1-B4 | COMPLETE |
| Tier 4 IPC-first | COMPLIANT (default = []) |
| guideStone | Level 4 |

groundSpring is now Tier 2 ready. Live validation against deployed ToadStool/barraCuda will exercise these paths when NUCLEUS composition graphs include groundSpring workloads.

---

## Open Items (not blocking Tier 2)

| Gap | Status |
|-----|--------|
| GAP-GS-002: coralReef IPC | Stub, awaiting SM rebuild |
| GAP-GS-011: PRNG Phase 2b | Deferred, barraCuda team deliverable |
| GAP-GS-008: Ionic runtime | Blocked upstream (BearDog) |
| GAP-GS-009: BTSP session crypto | Blocked upstream (BearDog/barraCuda) |
