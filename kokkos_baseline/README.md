# Kokkos Tier 1 Validation Baseline

> Python validates correctness. Kokkos validates competitiveness.

This directory implements the same algorithms as groundSpring's Rust library
using [Kokkos](https://github.com/kokkos/kokkos) (Sandia National
Laboratories) parallel primitives. The purpose is to establish a **Tier 1
performance baseline** — if BarraCuda's WGSL shaders match or beat Kokkos
on the same physics, that's a meaningful result.

## Evolution Path

```
Tier 0: Python baseline          — correctness reference
Tier 1: Kokkos baseline          — performance reference (this directory)
Tier 2: Rust + BarraCuda (WGSL)  — sovereign implementation
```

## Algorithms Implemented

| Kernel | Kokkos Pattern | groundSpring Module | Physics |
|--------|---------------|---------------------|---------|
| Anderson Lyapunov | `parallel_reduce` over realizations | `anderson.rs` | 1D localization, transfer matrix |
| Mean / Variance | `parallel_reduce` | `stats/metrics.rs` | Descriptive statistics |
| Pearson r | `parallel_reduce` (3 accumulators) | `stats/correlation.rs` | Linear correlation |
| Bootstrap CI | `parallel_for` + host sort | `bootstrap.rs` | Percentile confidence interval |

These cover the three main Kokkos dispatch patterns:
- `parallel_for` — independent work items (bootstrap sampling)
- `parallel_reduce` — aggregation with accumulator (statistics, Lyapunov)
- `View` memory management — device-resident arrays

## PRNG Parity

The Kokkos implementation uses the same `Xorshift64` PRNG as
`groundspring::prng::Xorshift64`, with identical state transitions. Given
the same seed, the Kokkos and Rust implementations produce the same
pseudorandom sequence. This ensures that numerical differences in results
are due to floating-point evaluation order, not input divergence.

Note: BarraCuda GPU uses `xoshiro128**` — seed alignment between the three
tiers (Python, Kokkos, BarraCuda) is tracked in
`specs/BARRACUDA_EVOLUTION.md` as a Phase 2b item.

## Build

### Prerequisites

- CMake >= 3.22
- C++17 compiler (GCC >= 9, Clang >= 11)
- (Optional) CUDA Toolkit for GPU backend
- (Optional) OpenMP for threaded CPU backend

Kokkos is fetched automatically via CMake `FetchContent` — no manual
installation needed.

### Serial (single-thread CPU)

```bash
cmake -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build -j$(nproc)
./build/kokkos_baseline
```

### OpenMP (multi-thread CPU)

```bash
cmake -B build -DCMAKE_BUILD_TYPE=Release -DKokkos_ENABLE_OPENMP=ON
cmake --build build -j$(nproc)
OMP_NUM_THREADS=8 ./build/kokkos_baseline
```

### CUDA (NVIDIA GPU)

```bash
cmake -B build -DCMAKE_BUILD_TYPE=Release \
    -DKokkos_ENABLE_CUDA=ON \
    -DKokkos_ARCH_AMPERE86=ON   # adjust for your GPU
cmake --build build -j$(nproc)
./build/kokkos_baseline
```

### HIP (AMD GPU)

```bash
cmake -B build -DCMAKE_BUILD_TYPE=Release \
    -DKokkos_ENABLE_HIP=ON \
    -DKokkos_ARCH_VEGA90A=ON
cmake --build build -j$(nproc)
./build/kokkos_baseline
```

## Compare Against Rust

The binary emits JSON with provenance at the end of its output. Pipe it
through the comparison script:

```bash
./build/kokkos_baseline | python3 scripts/compare_kokkos_rust.py
```

To compare against Rust results, generate Rust benchmark JSON first
(validation binary output), then:

```bash
./build/kokkos_baseline | python3 scripts/compare_kokkos_rust.py --rust-json rust_results.json
```

## For Other Springs

This directory is the **reference pattern** for adding Kokkos Tier 1
baselines to other springs. To adapt for your spring:

1. **Copy this directory** to `your_spring/kokkos_baseline/`
2. **Replace the kernels** in `src/main.cpp` with your spring's key algorithms
3. **Keep the same structure**:
   - Same `Xorshift64` PRNG (or document your PRNG)
   - Same JSON output format with `_provenance`
   - Same `CMakeLists.txt` pattern (FetchContent, no manual Kokkos install)
4. **Map your algorithms** to Kokkos patterns:

| Your Pattern | Kokkos Equivalent |
|-------------|-------------------|
| Element-wise transform | `parallel_for` |
| Sum / mean / variance | `parallel_reduce` |
| Prefix sum / CDF | `parallel_scan` |
| Sparse matrix-vector | `parallel_for` with `View<int*>` + `View<double*>` |
| Monte Carlo batch | `parallel_for` with per-thread PRNG |
| ODE batch | `parallel_for` over initial conditions |

### Spring-Specific Targets

| Spring | Priority Kokkos Kernels |
|--------|------------------------|
| **hotSpring** | Yukawa pair force, PPPM charge spread, HMC leapfrog, plaquette |
| **wetSpring** | Gillespie SSA batch, multinomial sampling, Bray-Curtis |
| **airSpring** | FAO-56 batch ET₀, Hargreaves, seasonal pipeline |
| **neuralSpring** | ESN reservoir update, HMM forward, batch fitness |

## Key Differences: Kokkos vs BarraCuda

| Aspect | Kokkos | BarraCuda |
|--------|--------|-----------|
| Dispatch | Compile-time backend | Runtime WGSL JIT |
| Vendor SDK | Required (CUDA/ROCm/oneAPI) | Not required (Vulkan only) |
| Binary | One per backend | One for all backends |
| Precision | Native f64 | f64 + DF64 hybrid |
| Ecosystem | LAMMPS, Trilinos, Cabana | ecoPrimals springs |

The performance question: can runtime WGSL JIT match compile-time
backend-specific codegen? Our DF64 results on lattice QCD suggest yes for
compute-bound kernels. This baseline lets us measure it.

## References

- Edwards et al., "Kokkos: Enabling manycore performance portability,"
  JPDC 74(12), 2014
- Trott et al., "Kokkos 3: Programming Model Extensions for the Exascale
  Era," IEEE CiSE 24(4), 2022
- Kokkos Tutorials: https://github.com/kokkos/kokkos-tutorials

## License

AGPL-3.0-only. See top-level LICENSE.
