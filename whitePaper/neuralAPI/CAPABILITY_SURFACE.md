# groundSpring Capability Surface

> Semantic capabilities for the biomeOS capability registry.

**Status**: V139 (May 12, 2026)
**Domain**: `measurement`

## Capabilities Provided

These are the capabilities groundSpring registers with the biomeOS Neural API.
Other primals can invoke them via `capability.call`.

### `measurement.noise_decomposition`

Bias-variance decomposition of sensor measurements (RMSE² = MBE² + σ²).

```json
{
  "capability": "measurement",
  "operation": "noise_decomposition",
  "args": {
    "observed": [1.2, 1.5, 1.3, 1.8],
    "modeled": [1.1, 1.4, 1.35, 1.7]
  }
}
```

**Returns**: `{ "rmse": 0.091, "mbe": 0.0625, "bias_fraction": 0.472, "noise_fraction": 0.528 }`

**Module**: `groundspring::decompose`
**Tier**: CPU (local), barraCuda CPU, barraCuda GPU

---

### `measurement.anderson_validation`

Anderson localization with Lyapunov exponents for 1D tight-binding chains.

```json
{
  "capability": "measurement",
  "operation": "anderson_validation",
  "args": {
    "n_sites": 10000,
    "disorder": 4.0,
    "energy": 0.0,
    "n_realizations": 20,
    "seed": 42
  }
}
```

**Returns**: `{ "gamma": 0.123, "localization_length": 8.13, "n_sites": 10000, "disorder": 4.0 }`

**Module**: `groundspring::anderson`
**Tier**: CPU → barraCuda GPU (embarrassingly parallel over realizations)

---

### `measurement.parity_check`

Validate CPU/GPU mathematical parity for a set of values within a tolerance.

```json
{
  "capability": "measurement",
  "operation": "parity_check",
  "args": {
    "cpu_values": [1.0, 2.0, 3.0],
    "gpu_values": [1.0, 2.0, 3.0],
    "tolerance": 1e-12
  }
}
```

**Returns**: `{ "parity": true, "max_difference": 0.0, "tolerance": 1e-12, "n_values": 3 }`

**Module**: `groundspring::dispatch`
**Tier**: CPU only (validation is inherently sequential)

---

### `measurement.et0_propagation`

FAO-56 Penman-Monteith reference evapotranspiration.

```json
{
  "capability": "measurement",
  "operation": "et0_propagation",
  "args": {
    "temperature_max": 30.0,
    "temperature_min": 18.0,
    "wind_speed": 2.0,
    "sunshine_hours": 8.5,
    "latitude": 43.0,
    "day_of_year": 180
  }
}
```

**Returns**: `{ "et0_mm_day": 4.82, "method": "FAO-56 Penman-Monteith" }`

**Module**: `groundspring::fao56`
**Tier**: CPU → barraCuda CPU (`daily_et0` delegated), barraCuda GPU (batch ET₀)

---

### `measurement.regime_classification`

Rule-based Anderson regime classification from eigenvalue spectra.

```json
{
  "capability": "measurement",
  "operation": "regime_classification",
  "args": {
    "eigenvalues": [0.1, 0.3, 0.5, 0.7, 0.9],
    "margin": 0.1
  }
}
```

**Returns**: `{ "label": "Extended", "mean_spacing_ratio": 0.53, "spectral_rigidity": 0.8, "ipr": 1.8 }`

**Module**: `groundspring::esn`
**Tier**: CPU, barraCuda GPU (ESN reservoir update)

---

### `measurement.uncertainty_budget`

Combined bootstrap + jackknife uncertainty estimation.

```json
{
  "capability": "measurement",
  "operation": "uncertainty_budget",
  "args": {
    "data": [1.0, 2.0, 3.0, 4.0, 5.0],
    "confidence": 0.95,
    "n_bootstrap": 10000,
    "seed": 42
  }
}
```

**Returns**: `{ "bootstrap": { "estimate": 3.0, "ci_lower": 1.8, "ci_upper": 4.2, "std_error": 0.63 }, "jackknife": { "estimate": 3.0, "variance": 2.5, "std_error": 1.58 } }`

**Module**: `groundspring::bootstrap`, `groundspring::jackknife`
**Tier**: CPU, barraCuda CPU, barraCuda GPU

---

### `measurement.spectral_features`

Spectral function reconstruction via Tikhonov regularization.

```json
{
  "capability": "measurement",
  "operation": "spectral_features",
  "args": {
    "correlator": [1.0, 0.8, 0.5, 0.3, 0.1],
    "n_omega": 50,
    "regularization": 1e-4
  }
}
```

**Returns**: `{ "spectral_function": [...], "peak_index": 12, "residual_rmse": 0.001, "n_omega": 50 }`

**Module**: `groundspring::spectral_recon`
**Tier**: CPU, barraCuda CPU (Cholesky), barraCuda GPU (matrix solve)

---

### `measurement.freeze_out`

Freeze-out curve chi-squared fitting (Bazavov et al. 2016).

```json
{
  "capability": "measurement",
  "operation": "freeze_out",
  "args": {
    "observed": [155.0, 153.0, 150.0],
    "mu_b": [0.0, 100.0, 200.0],
    "sigma": 1.0
  }
}
```

**Returns**: `{ "t0": 155.2, "kappa2": 0.013, "chi_squared": 0.42, "chi2_per_dof": 0.42 }`

**Module**: `groundspring::freeze_out`
**Tier**: CPU (grid search), barraCuda GPU (L-BFGS + Nelder-Mead multi-start)

---

## Capabilities Consumed

These are capabilities groundSpring requests from other biomeOS primals
via capability-based discovery (no compile-time primal knowledge).

### `compute.execute` (discovered at runtime)

GPU compute for barraCuda delegations. groundSpring sends pure-math workloads
through the `compute` capability when biomeOS routing is enabled.

### `storage.put` / `storage.get` (discovered at runtime)

Benchmark JSON storage and provenance tracking via content-addressed storage.

---

## Registry Format

For biomeOS `capability_registry.toml`:

```toml
[[capabilities]]
name = "measurement.noise_decomposition"
provider = "groundspring"
version = "0.1.0"
substrate = ["cpu", "gpu"]

[[capabilities]]
name = "measurement.anderson_validation"
provider = "groundspring"
version = "0.1.0"
substrate = ["cpu", "gpu"]

[[capabilities]]
name = "measurement.parity_check"
provider = "groundspring"
version = "0.1.0"
substrate = ["cpu"]

[[capabilities]]
name = "measurement.et0_propagation"
provider = "groundspring"
version = "0.1.0"
substrate = ["cpu", "gpu"]

[[capabilities]]
name = "measurement.regime_classification"
provider = "groundspring"
version = "0.1.0"
substrate = ["cpu", "gpu"]

[[capabilities]]
name = "measurement.uncertainty_budget"
provider = "groundspring"
version = "0.1.0"
substrate = ["cpu", "gpu"]

[[capabilities]]
name = "measurement.spectral_features"
provider = "groundspring"
version = "0.1.0"
substrate = ["cpu", "gpu"]

[[capabilities]]
name = "measurement.freeze_out"
provider = "groundspring"
version = "0.1.0"
substrate = ["cpu", "gpu"]
```
