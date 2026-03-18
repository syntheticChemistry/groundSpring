# groundSpring V116 → Ecosystem Typed Error Evolution Handoff

**Date**: March 18, 2026
**From**: groundSpring V116
**To**: toadStool, barraCuda, coralReef, biomeOS, ecosystem
**Supersedes**: V115 (GROUNDSPRING_V115_TOADSTOOL_BARRACUDA_EVOLUTION_HANDOFF_MAR18_2026.md)
**Pins**: barraCuda v0.3.5, toadStool S158+, coralReef Iteration 55+
**License**: AGPL-3.0-or-later

## Executive Summary

V116 completes the typed error evolution started in V115. The entire dispatch
layer now uses structured `DispatchError` instead of opaque `String` errors,
the ESN classifier preserves the `BarracudaError` source chain, and
`resilient_call()` returns `ResilienceError<E>` that distinguishes circuit-open
from retry exhaustion. Capability parsing now handles all 4 known advertisement
formats (A–D) plus the `"methods"` wrapper. The `ValidationSink` trait
(absorbed from ludoSpring/rhizoCrypt/primalSpring) enables silent validation
for benchmarks. GPU probe caching via `OnceLock` prevents SIGSEGV in parallel
tests. RAWR resampling is cleanly extracted to its own module.

## Part 1: What V116 Changed

### Typed Error Evolution

| Module | Before (V115) | After (V116) |
|--------|--------------|--------------|
| `dispatch::dispatch()` | `Result<Value, String>` | `Result<Value, DispatchError>` |
| `serve_one()` / `handle_connection()` | `F: Fn -> Result<Value, String>` | `F: Fn -> Result<Value, E: Display>` |
| `EsnClassifier::new/train/classify` | `Result<_, String>` | `Result<_, EsnError>` |
| `resilient_call()` | `Result<T, String>` | `Result<T, ResilienceError<E>>` |

**`DispatchError` variants**: `MethodNotFound(String)`, `MissingParam(String)`,
`InvalidParam(String)`, `Input(#[from] InputError)`.

**`EsnError` variants**: `Init(BarracudaError)`, `Train(BarracudaError)`,
`Predict(BarracudaError)` — preserves source error chain via `#[source]`.

**`ResilienceError<E>` variants**: `CircuitOpen`, `RetriesExhausted { attempts, last_error }`.

**Impact on barraCuda/toadStool**: The `serve_one()` function is now generic
over `E: Display`, so callers using `Result<Value, String>` still work without
changes. The `EsnClassifier` API changes are behind `#[cfg(feature = "barracuda-gpu")]`.

### Capability Parsing (4-Format Support)

| Format | Key(s) | Example | Status |
|--------|--------|---------|--------|
| A | (flat string) | `"compute.execute"` | V114+ |
| B | `name`, `capability` | `{"name": "compute.execute"}` | V114+ |
| C | `method` | `{"method": "compute.execute", "description": "..."}` | **V116 new** |
| D | `semantic_method`, `method_name` | `{"semantic_method": "measurement.bootstrap"}` | **V116 new** |

Also added: `"methods"` wrapper key (biomeOS uses `{"methods": [...]}` in some responses).

### ValidationSink Trait

Absorbed from ludoSpring V22 / rhizoCrypt v0.13 / primalSpring validation patterns.

- `ValidationSink` trait: `record_pass`, `record_fail`, `section`, `write_summary`
- `StdoutSink` (default), `NullSink` (silent benchmarks), `WriteSink<W>` (custom)
- `ValidationHarness::silent()` constructor for zero-output mode
- `ValidationHarness::section()` for structured grouping
- Zero breaking change: `ValidationHarness::stdout()` signature unchanged

### OnceLock GPU Probe Cache

`metalForge/forge/src/probe.rs` caches `probe_gpus()` results in a
`static GPU_PROBE_CACHE: OnceLock<Vec<Substrate>>`. Prevents SIGSEGV from
concurrent `wgpu::Instance` creation in parallel tests (toadStool S158 finding).

### Smart Refactoring

- `rawr.rs` extracted from `bootstrap.rs` (669L → ~520L + ~180L)
- Backwards-compatible re-export: `bootstrap::rawr_mean` still resolves
- Shared infrastructure (`validate_bootstrap_inputs`, `percentile_ci`,
  `BootstrapResult`) stays in `bootstrap.rs` with `pub(crate)` visibility

### Named Dispatch Constants

All inline numeric defaults in dispatch method bodies now have named constants
with provenance comments:

| Constant | Value | Provenance |
|----------|-------|------------|
| `DEFAULT_SEED` | 42 | Ecosystem convention (hotSpring, wetSpring, airSpring) |
| `DEFAULT_ANDERSON_N_SITES` | 10_000 | Kachkovskiy Paper 2 finite-size scaling |
| `DEFAULT_ANDERSON_DISORDER` | 4.0 | 1D strongly localized regime |
| `DEFAULT_ANDERSON_REALIZATIONS` | 20 | Papers 2 & 3 averaging convention |
| `DEFAULT_N_BOOTSTRAP` | 10_000 | Efron & Tibshirani 1993 |
| `DEFAULT_N_OMEGA` | 50 | Exp 028 spectral reconstruction |

## Part 2: What Remains Unchanged

- 102 active barraCuda delegations (61 CPU + 41 GPU) — unchanged
- All validation binaries pass at all three tiers — unchanged
- `deny.toml` C-dependency banning — unchanged
- UniBin `--help`/`--version` flags — unchanged
- `#![forbid(unsafe_code)]` everywhere — unchanged

## Part 3: Quality Gates

| Metric | V115 | V116 |
|--------|------|------|
| Rust tests | 930+ | 960+ |
| Clippy (pedantic+nursery) | 0 | 0 |
| `Result<_, String>` in dispatch | 12+ sites | 0 |
| `format!` error flattening | ESN + resilience | 0 |
| Capability formats supported | A, B | A, B, C, D |
| GPU probe SIGSEGV risk | possible | eliminated (OnceLock) |

## Part 4: Recommendations for toadStool/barraCuda

1. **Adopt `DispatchError` pattern** — if toadStool has a similar dispatch layer
   returning `Result<Value, String>`, consider the same `thiserror` enum approach
2. **`OnceLock` for GPU singletons** — the SIGSEGV fix should be applied to any
   code that creates `wgpu::Instance` in test contexts
3. **Capability advertisement** — ensure Format C/D objects are used when
   advertising capabilities to biomeOS for richer metadata
