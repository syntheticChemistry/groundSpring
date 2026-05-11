# groundSpring Validation Tiers

**Date**: May 11, 2026
**primalSpring**: v0.9.25 | **groundSpring**: V135

---

## Tier 1 — Rust (Pure Structural Validation)

No IPC, no running primals, no sockets. Safe for CI.

### What Tier 1 Covers

- Bias-variance decomposition determinism and Pythagorean identity
- Rarefaction curve monotonicity and boundary conditions
- Anderson localization Lyapunov exponent positivity
- FAO-56 Penman-Monteith ET₀ plausibility
- Freeze-out chi-squared non-negativity
- Bistable phenotypic switching convergence
- Seismic travel-time computation
- Wright-Fisher drift fixation bounds
- Jackknife resampling variance estimation
- Certification L0 (bare guideStone Properties 1-5)

### How to Run

```bash
groundspring_unibin validate --tier rust
groundspring_unibin certify --bare
```

### Scenarios in This Tier

| ID | Track | Provenance |
|----|-------|-----------|
| decompose-bias-variance | noise-decomposition | validate_decompose |
| rarefaction-curves | ecology | validate_rarefaction |
| anderson-localization | condensed-matter | validate_anderson |
| fao56-et0-penman-monteith | agricultural-science | validate_fao56 |
| freeze-out-chi2 | statistical-fitting | validate_freeze_out |
| bistable-phenotypic-switch | dynamical-systems | validate_bistable |
| seismic-travel-time | geophysics | validate_seismic |
| drift-wright-fisher | population-genetics | validate_drift |
| jackknife-delete-one | resampling | validate_jackknife |

---

## Tier 2 — Live (NUCLEUS Composition Validation)

Requires deployed primals from plasmidBin ecobins. Exercises live
composition behavior via `CompositionContext`.

### What Tier 2 Covers

- Full NUCLEUS composition parity (Tower + Node + Nest + cross-atomic)
- Certification L2-L4 (atomic health, capability parity, cross-atomic pipeline)
- Live IPC round-trip tolerance verification
- Deploy graph validation against real compositions

### How to Run

```bash
# Deploy primals first
plasmidBin deploy --composition nucleus-local

# Then validate
groundspring_unibin validate --tier live
groundspring_unibin certify --layer 4
```

### Scenarios in This Tier

| ID | Track | Provenance |
|----|-------|-----------|
| nucleus-composition-parity | composition-parity | exp094_composition_parity |

### Graceful Degradation

When primals are not available, Tier 2 scenarios use `check_skip` rather
than `check_fail`. This means:
- **SKIP ≠ FAIL** — missing primals produce skip, not failure
- CI runs can include Tier 2 scenarios without false negatives
- The certification engine exits with code `2` (bare-only) rather than `1` (fail)

---

## Running Both Tiers

```bash
groundspring_unibin validate --tier all    # or --tier both
groundspring_unibin validate --list         # show all scenarios
```

---

## Deprecated Patterns

| Old Pattern | Replacement |
|-------------|------------|
| `PrimalClient::connect(path)` | `CompositionContext::from_live_discovery_with_fallback()` |
| `discover_primal("name")` | `ctx.call("capability", "method", params)` |
| `AtomicHarness::spawn(config)` | Deploy via plasmidBin + biomeOS |
| `probe_primal(addr)` | `ctx.health_check("capability")` |
| `neural_api_healthy()` | `NeuralBridge::discover()` + `ctx.call(...)` |

---

## License

AGPL-3.0-or-later
