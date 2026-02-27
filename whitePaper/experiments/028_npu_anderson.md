# Exp 028: NPU Anderson Regime Classification

## Domain
Hardware (NPU) — BrainChip AKD1000 neuromorphic inference

## Question
Can Anderson localization regimes (Localized / Critical / Extended) be classified
via int8-quantized features on a neuromorphic processor, proving mathematical
portability from CPU to NPU?

## Method
- Analytical ξ(W, E=0) = C/W² assigns ground truth regime per disorder value
- Features (W, E, L) quantized to int8 ([0, 127]) for NPU dispatch
- Centroid classifier trained from 100 random disorder values
- Classification verified on 10 test disorder values across all three regimes
- NPU inference via DMA write/read on BrainChip AKD1000

## Results (CPU)
- 10/10 disorder values correctly classified
- Quantization round-trip error < 25%
- Classifier accuracy ≥ 90% on training data
- All three regime classes covered

## Results (NPU)
- AKD1000 discovered: 80 NPs, 10 MB SRAM
- DMA round-trip latency: ~51 µs/inference
- Classifier weights loaded (9 bytes)
- DMA connectivity proven (all 10 values produce valid class labels)

## Validation
- Python: `control/npu_anderson/npu_anderson.py` — 7/7 PASS
- Rust: `validate-npu-anderson` — 9/9 PASS (7 CPU + 2 NPU live hardware)
- Benchmark: `control/npu_anderson/benchmark_npu_anderson.json`

## Cross-Spring
- Uses ToadStool `akida-driver` (pure Rust, zero mocks)
- Follows wetSpring's proven NPU integration pattern
- groundSpring's `npu` feature mirrors wetSpring's `npu` feature

## Paper
Anderson 1958; Derrida-Gardner 1984; BrainChip AKD1000 datasheet
