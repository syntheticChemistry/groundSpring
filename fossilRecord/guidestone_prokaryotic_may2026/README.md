# Fossil: groundspring_guidestone Binary (Prokaryotic Era)

**Fossilized**: May 9, 2026
**From**: `crates/groundspring-validate/src/groundspring_guidestone.rs`
**Superseded by**: `crates/groundspring/src/certification/` + `groundspring_unibin certify`

## What This Was

A standalone binary that validated groundSpring's NUCLEUS deployability
through 5 bare guideStone properties and 3 NUCLEUS composition layers
(L2-L4). Required the `guidestone` feature flag and primalSpring v0.9.25.

## Why It Was Superseded

The certification organelle pattern absorbs the guidestone logic into a
library module (`certification/mod.rs`, `certification/bare.rs`,
`certification/composition.rs`). The library function `certify(max_layer)`
is called by the UniBin's `certify` subcommand.

Benefits:
- Library-callable certification (not just a binary)
- Layer-gated execution (`--layer N`, `--bare`)
- Testable organelle (unit tests can call `validate_deterministic` directly)
- Single binary deployment

## Architecture at Fossilization

```
groundspring_guidestone.rs (841 lines)
├── main()
│   ├── Bare guideStone: Properties 1-5
│   │   ├── validate_deterministic()
│   │   ├── validate_traceable()
│   │   ├── validate_self_verifying()
│   │   ├── validate_env_agnostic()
│   │   └── validate_tolerance_documented()
│   └── NUCLEUS Composition: Layers 2-4
│       ├── L2: Discovery + Atomic Health
│       ├── L3: Tower (BearDog + Songbird)
│       ├── L3: Node (barraCuda + coralReef + toadStool)
│       ├── L3: Nest (NestGate + provenance trio)
│       └── L4: Cross-Atomic Pipeline
```
