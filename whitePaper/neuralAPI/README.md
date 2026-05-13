# groundSpring × biomeOS — Neural API Integration Concept

> groundSpring as a **validation science primal** in the biomeOS ecosystem.

**Status**: V140 live (May 13, 2026) — NUCLEUS connection validated, --format json, LTEE B1-B4

## Role

groundSpring provides noise characterization, mathematical parity validation,
and cross-domain signal analysis to the biomeOS ecosystem. Where wetSpring
produces biodiversity metrics and hotSpring models nuclear processes,
groundSpring validates that the underlying mathematics are correct —
identical results across CPU, GPU, and NPU substrates.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   biomeOS Tower Node                     │
│  ┌──────────┐  ┌──────────┐  ┌────────────────────┐    │
│  │ BearDog  │  │ Songbird │  │   Neural API Router │    │
│  │(Security)│  │(Network) │  │  (capability.call)  │    │
│  └────┬─────┘  └────┬─────┘  └─────────┬──────────┘    │
│       └──────────────┴──────────────────┘               │
└─────────────────────┬───────────────────────────────────┘
                      │ JSON-RPC 2.0 (Unix socket)
          ┌───────────┼───────────┐
          │           │           │
     ┌────▼────┐ ┌────▼────┐ ┌───▼──────┐
     │ToadStool│ │NestGate │ │  Springs  │
     │ (GPU)   │ │(Storage)│ │(Science)  │
     └─────────┘ └─────────┘ └──────────┘
                               │
                  ┌────────────┼────────────┐
                  │            │            │
            groundSpring  wetSpring   hotSpring
            (validation)  (biodiv)   (nuclear)
```

## Three-Tier Flow

groundSpring's biomeOS integration follows the same three-tier progression
as its local validation:

| Tier | Local | biomeOS |
|------|-------|---------|
| Phase 0 | Python baseline + benchmark JSON | — |
| Phase 1 | Rust CPU validation | `capability.call` routed through Neural API |
| Phase 2a | Barracuda CPU | `compute.execute` → ToadStool CPU path |
| Phase 2b | Barracuda GPU | `compute.execute` → ToadStool GPU shaders |
| Phase 3 | metalForge cross-substrate | Neural API → metalForge → GPU/NPU/CPU dispatch |

At every tier, the mathematical result must be identical within documented
tolerances. biomeOS routing adds ecosystem integration (provenance tracking,
cross-spring experiments, shared compute) without changing the math.

## Consumption Model

### What groundSpring PROVIDES

groundSpring exposes validation science capabilities through the Neural API:

- `science.noise_decomposition` — bias-variance decomposition
- `science.anderson_validation` — Anderson localization with Lyapunov exponents
- `science.parity_check` — validate Python/Rust mathematical parity
- `science.three_tier_validate` — run a validation across all substrate tiers
- `science.et0_propagation` — FAO-56 error propagation analysis

### What groundSpring CONSUMES

- `compute.execute` (ToadStool) — GPU compute for barracuda delegations (barracuda from `barraCuda` primal at `ecoPrimals/barraCuda/`)
- `storage.put/get` (NestGate) — benchmark JSON storage and provenance
- `science.diversity` (wetSpring) — Shannon diversity for cross-spring experiments

## Protocol

JSON-RPC 2.0, newline-delimited, over Unix domain socket. Follows the same
protocol wetSpring's NestGate client established.

Socket discovery:
1. `GROUNDSPRING_BIOMEOS_SOCKET` env var (explicit override)
2. `$XDG_RUNTIME_DIR/biomeos/neural-api-default.sock`
3. `<temp_dir>/biomeos-neural-api.sock` (platform-agnostic fallback)

## Sovereign Fallback

All groundSpring functions work without biomeOS. The `biomeos` feature gate
is optional — when the socket is unavailable, every operation falls back to
local computation. This is the same sovereign pattern wetSpring's NestGate
client follows: biomeOS enhances but never gates.

## Implementation

- Client: `crates/groundspring/src/biomeos.rs` (feature-gated `biomeos`)
- Pipeline graph: `graphs/groundspring_validation.toml`
- Capability surface: `whitePaper/neuralAPI/CAPABILITY_SURFACE.md`
- Integration tests: `crates/groundspring/tests/biomeos_integration.rs`

## See Also

- [CAPABILITY_SURFACE.md](CAPABILITY_SURFACE.md) — semantic capability definitions
- [../../specs/BARRACUDA_EVOLUTION.md](../../specs/BARRACUDA_EVOLUTION.md) — Barracuda delegation roadmap
- [../../graphs/groundspring_validation.toml](../../graphs/groundspring_validation.toml) — pipeline graph
