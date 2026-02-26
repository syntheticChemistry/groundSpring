# Exp 018: Band Edge Structure

**Domain**: Mathematical physics (spectral theory, condensed matter)
**Paper**: Filonov, Kachkovskiy (2018) Commun Math Phys 362:1101–1135
**Faculty**: Ilya Kachkovskiy (Mathematics, MSU)
**Question**: Can the transfer matrix method reproduce the band-gap structure
of a 1D tight-binding chain with periodic potentials, matching analytical
predictions for gap widths and band counts?

## Data Source

1D tight-binding Hamiltonian with nearest-neighbor hopping t=1.0.
Potentials: period-2 V=[1.0, −1.0], period-3 V=[1.5, 0.0, −0.5].
Energy scan E ∈ [−5.0, 5.0] with 10,001 points. Finite chain: 200 sites.

## Method

1. **Transfer matrix trace**: For each energy E, compute the half-trace
   τ(E) = ½ Tr(T_L · … · T_1) where T_j is the 2×2 transfer matrix at
   site j. Band states satisfy |τ| ≤ 1.
2. **Band edge detection**: Scan for sign changes of |τ|−1 to locate
   band-gap transitions with sub-step interpolation.
3. **Band counting**: Count connected intervals where |τ| ≤ 1. Period-p
   potential produces exactly p bands per Brillouin zone.
4. **Eigenvalue verification**: Build the full N×N tridiagonal Hamiltonian
   and compute eigenvalues. Fraction falling in the band regions should
   approach 1 as N → ∞.

## Key Result

**Periodic potentials open gaps at Brillouin zone boundaries.**
- Free lattice (V=0): single band [−2.0, 2.0], matching 2t cos(k)
- Period-2, V=[+1,−1]: gap of width 2.0 centered at E=0, exactly 2 bands
- Period-3, V=[1.5,0,−0.5]: exactly 3 bands per zone
- Gap width scales linearly with potential contrast ΔV
- Finite-system eigenvalues: >95% fall within transfer-matrix band regions
- Deterministic: bit-identical results across repeated runs

**Filonov & Kachkovskiy (2018) proved** precise asymptotics for band edge
behavior in multidimensional periodic operators. This 1D experiment
validates the core spectral machinery before extending to higher dimensions.

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | 8/8 | `control/band_edge/band_edge.py` |
| Phase 1 (Rust) | 10/10 | `validate-band-edge` |

## Barracuda Path

Transfer matrix multiplication is a sequential scan but trivially
parallelizable across energies — each energy point is independent.
Tridiagonal eigensolvers exist in barracuda's linear algebra module.
The energy-parallel structure maps naturally to GPU thread blocks.

## Modules

`band_structure` (`transfer_matrix_half_trace`, `find_band_edges`,
`count_bands`, `periodic_hamiltonian`, `eigenvalue_band_fraction`), `prng`
