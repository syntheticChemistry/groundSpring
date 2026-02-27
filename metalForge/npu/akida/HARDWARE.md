# AKD1000 Hardware Characterization — groundSpring

## Device

| Property | Value |
|----------|-------|
| Chip | BrainChip AKD1000 |
| Neural Processors | 80 NPs |
| On-chip SRAM | 10 MB |
| Interface | PCIe 2.0 x1 |
| Device Node | `/dev/akida0` |
| Driver | ToadStool `akida-driver` (pure Rust) |

## groundSpring Workloads

| Workload | Quantization | DMA Latency |
|----------|-------------|-------------|
| Anderson regime classification | int8 (3 features → 3 classes) | ~50 µs |
| Diversity saturation prediction | int8 (TBD) | TBD |

## Validation Status

- `validate-npu-anderson` (Exp 028): **9/9 PASS** — CPU classification + NPU DMA round-trip
- Hardware discovery: confirmed via `validate-metalforge-inventory`
- DMA write/read: functional at ~50 µs per inference round-trip

## Notes

- Current NPU path exercises DMA connectivity (write features → read output).
- Full spiking neural inference requires compiled SNN model via `ModelLoader` + `InferenceExecutor`.
- The AKD1000 supports online weight mutation — classifier swap without full reprogramming.
