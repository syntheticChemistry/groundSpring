# groundSpring Capability Surface

> Semantic capabilities for the biomeOS capability registry.

**Status**: V30 (Feb 27, 2026)

## Capabilities Provided

These are the capabilities groundSpring registers with the biomeOS Neural API.
Other primals can invoke them via `capability.call`.

### `science.noise_decomposition`

Bias-variance decomposition of sensor measurements.

```json
{
  "capability": "science.noise_decomposition",
  "params": {
    "observed": [1.2, 1.5, 1.3],
    "predicted": [1.1, 1.4, 1.35],
    "method": "triple_collocation"
  }
}
```

**Returns**: `{ "bias": 0.083, "variance": 0.0042, "rmse": 0.091 }`

**Module**: `groundspring::decompose`
**Tier**: CPU (local), Barracuda CPU, Barracuda GPU

---

### `science.anderson_validation`

Anderson localization with Lyapunov exponents for 1D tight-binding chains.

```json
{
  "capability": "science.anderson_validation",
  "params": {
    "n_sites": 10000,
    "disorder_strengths": [0.5, 1.0, 2.0, 4.0, 8.0],
    "energy": 0.0,
    "n_realizations": 20,
    "seed": 42
  }
}
```

**Returns**: `{ "lyapunov_exponents": [...], "localization_lengths": [...], "thouless_coefficient": 96.2 }`

**Module**: `groundspring::anderson`
**Tier**: CPU → Barracuda GPU (embarrassingly parallel over realizations)

---

### `science.parity_check`

Validate Python/Rust mathematical parity for a benchmark JSON.

```json
{
  "capability": "science.parity_check",
  "params": {
    "experiment_id": "exp008_anderson_localization",
    "benchmark_json_key": "groundspring:benchmarks:exp008"
  }
}
```

**Returns**: `{ "passed": 12, "failed": 0, "total": 12, "status": "PASS" }`

**Module**: `groundspring::validate`
**Tier**: CPU only (validation is inherently sequential)

---

### `science.three_tier_validate`

Run a validation binary in default, barracuda-cpu, and barracuda-gpu modes,
comparing results for mathematical equivalence.

```json
{
  "capability": "science.three_tier_validate",
  "params": {
    "experiment": "anderson",
    "modes": ["default", "barracuda", "barracuda-gpu"]
  }
}
```

**Returns**: `{ "default": "PASS", "barracuda": "PASS", "barracuda_gpu": "PASS", "parity": true }`

**Module**: `groundspring::validate` + three-tier harness
**Tier**: Multi-tier by definition

---

### `science.et0_propagation`

FAO-56 Penman-Monteith reference evapotranspiration with Monte Carlo error propagation.

```json
{
  "capability": "science.et0_propagation",
  "params": {
    "temperature": 25.0,
    "humidity": 0.6,
    "wind_speed": 2.0,
    "solar_radiation": 22.0,
    "n_monte_carlo": 10000,
    "seed": 42
  }
}
```

**Returns**: `{ "et0_mm_day": 4.82, "uncertainty_mm_day": 0.34, "cv_percent": 7.1 }`

**Module**: `groundspring::fao56`
**Tier**: CPU → Barracuda CPU (`daily_et0` delegated), Barracuda GPU (batch ET₀)

---

## Capabilities Consumed

These are capabilities groundSpring requests from other biomeOS primals.

### `compute.execute` (ToadStool)

GPU compute for Barracuda delegations. groundSpring sends pure-math workloads
to ToadStool when the `barracuda` feature is active and biomeOS routing is enabled.

```json
{
  "capability": "compute.execute",
  "params": {
    "op": "lyapunov_averaged",
    "n_sites": 10000,
    "disorder": 2.0,
    "energy": 0.0,
    "n_realizations": 20,
    "seed": 42
  }
}
```

**Provider**: ToadStool
**Dispatch**: Neural API → ToadStool → WGSL shader (Barracuda GPU) or CPU fallback

---

### `storage.put` / `storage.get` (NestGate)

Benchmark JSON storage and provenance tracking. groundSpring stores validation
results and retrieves benchmark data through NestGate.

```json
{
  "capability": "storage.put",
  "params": {
    "key": "groundspring:results:exp008",
    "value": "{\"passed\":12,\"failed\":0}",
    "family_id": "groundspring"
  }
}
```

**Provider**: NestGate
**Dispatch**: Neural API → NestGate → content-addressed storage

---

### `science.diversity` (wetSpring)

Shannon diversity for cross-spring experiments (Exp 022–024). groundSpring
consumes wetSpring's diversity metrics to validate noise decomposition
across biological signal types.

```json
{
  "capability": "science.diversity",
  "params": {
    "metrics": ["shannon", "simpson", "observed_features"],
    "input_key": "wetspring:sequences:cached"
  }
}
```

**Provider**: wetSpring
**Dispatch**: Neural API → wetSpring → diversity pipeline

---

## Registry Format

For biomeOS `capability_registry.toml`:

```toml
[[capabilities]]
name = "science.noise_decomposition"
provider = "groundspring"
version = "0.1.0"
substrate = ["cpu", "gpu"]

[[capabilities]]
name = "science.anderson_validation"
provider = "groundspring"
version = "0.1.0"
substrate = ["cpu", "gpu"]

[[capabilities]]
name = "science.parity_check"
provider = "groundspring"
version = "0.1.0"
substrate = ["cpu"]

[[capabilities]]
name = "science.three_tier_validate"
provider = "groundspring"
version = "0.1.0"
substrate = ["cpu", "gpu", "npu"]

[[capabilities]]
name = "science.et0_propagation"
provider = "groundspring"
version = "0.1.0"
substrate = ["cpu", "gpu"]
```
