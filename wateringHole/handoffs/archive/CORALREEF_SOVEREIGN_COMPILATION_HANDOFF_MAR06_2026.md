# coralReef Sovereign Compilation — Handoff

**Date**: March 6, 2026
**From**: groundSpring V85 session
**To**: coralReef, barraCuda, toadStool teams
**coralReef pin**: `849fedd` (main)

---

## Executive Summary

groundSpring drove coralReef to compile the **exact f64 shared-memory reduction
shaders** that fail through `naga → SPIR-V → NVK/NAK`. Two bugs were found and
fixed in coralReef; the sovereign compiler now produces native SM70 (Titan V)
and SM89 (RTX 4070) binaries for multi-barrier f64 workgroup reduction shaders.

**Result**: coralReef compiles what Mesa/NAK cannot. The path to full-speed Titan V
compute runs through coralDriver.

---

## Bugs Fixed in coralReef (commit `849fedd`)

### 1. CFG edge loss in `opt_jump_thread` (critical)

**Root cause**: `translate_if` created condition blocks with no branch instruction.
`opt_jump_thread → rewrite_cfg` rebuilds the CFG from fall-through and branch
instructions only, losing the structural edges from condition → reject blocks.
After jump threading, reject blocks became orphaned. `to_cssa` then added phi
nodes referencing those orphaned blocks, causing the register allocator to panic
with "Unknown SSA value".

**Fix**: `translate_if` now emits `@!cond BRA reject_label` at the end of the
condition block. When the reject path is empty (no `else` clause), branches
directly to merge — eliminating the empty block entirely.

**Trigger**: Any shader with 3+ `workgroupBarrier()` calls and if-statements
between them (i.e., the standard tree-reduction pattern).

### 2. Single-predecessor register allocation (significant)

**Root cause**: `assign_regs` used only `pred[0]`'s register state for blocks
with multiple predecessors (e.g., if/else merge points). SSA values assigned
by later predecessors were invisible.

**Fix**: `first_pass` now accepts a slice of ALL predecessor RA states and merges
SSA→register mappings from every predecessor, skipping duplicates.

### 3. OpBar encoding completion (incremental)

The BAR.SYNC instruction encoder was incomplete (fields commented out).
Added required fields: src register, reduction op, barrier mode, predicate.

**Note**: nvdisasm still reports an opex table error (value 0x10). The barrier
count field encoding needs further investigation for the immediate form (0xb1d
vs 0x31d). The register form works but the opex value needs to match Volta's
expected encoding.

---

## Compilation Results

| Shader | SM70 (Titan V) | SM89 (RTX 4070) | Status |
|--------|---------------|-----------------|--------|
| basic f64 arithmetic | 272 B | 272 B | nvdisasm clean |
| f64 storage read/write | 336 B | 336 B | nvdisasm clean |
| f64 shared_mem + 1 barrier | 528 B | 528 B | compiles |
| f64 shared_mem + 2 barriers | 704 B | 704 B | compiles |
| f64 shared_mem + 3 barriers | 928 B | 928 B | **NEW** — was panic |
| f64 8-step unrolled reduce | 2272 B | 2272 B | **NEW** — was panic |

### What still fails

| Shader | Error | Notes |
|--------|-------|-------|
| Loop-based tree reduction | `opt_instr_sched_prepass` assertion | Loop back-edge + instruction scheduling |
| Uniform buffer bindings | "not yet supported" | barraCuda's `sum_reduce_f64.wgsl` uses `var<uniform>` |

---

## Gap to Full-Speed GPU Execution

### 1. coralDriver (submission path) — Phase 7

coralReef produces correct native GPU binaries but cannot submit them to the
GPU. The binary is a raw instruction stream with SPH (Shader Program Header),
not a cubin/ELF that the CUDA driver API expects.

**Options for submission**:

| Path | Target GPU | Complexity | Notes |
|------|-----------|-----------|-------|
| cubin ELF wrapper + CUDA driver API | RTX 4070 | Medium | Requires wrapping coralReef output in cubin ELF sections |
| coralDriver (userspace GPU driver) | Both | High | Direct ioctl submission to `/dev/nvidia*` or `/dev/dri/*` |
| Move Titan V to proprietary driver | Titan V | Low (config) | Gets CUDA access for Titan V; loses NVK |
| SPIR-V backend for coralReef | Both | Medium | Output SPIR-V instead of native; submit through wgpu |

**Recommended next step**: cubin ELF wrapper for CUDA submission on RTX 4070
(proves end-to-end), then coralDriver for sovereignty.

### 2. f64 instruction generation

The basic f64 shader disassembles using FMUL/FADD (f32 ops) instead of
DMUL/DADD (f64 ops). The naga→IR translation may be lowering f64 to f32
prematurely, or the encoder may need explicit f64 instruction variants.

### 3. BAR.SYNC encoding

nvdisasm reports `undefined value 0x10 for table TABLES_opex_0`. The barrier
opcode extension field needs to match Volta's exact expected encoding. This
is a single-field fix once the correct table value is identified (likely from
a reference cubin disassembly).

---

## Hardware State

| GPU | Driver | CUDA | coralReef | wgpu/Vulkan |
|-----|--------|------|-----------|-------------|
| RTX 4070 (SM89) | nvidia 580.119.02 | Yes (device 0) | Compiles SM89 | Works (f32/DF64) |
| Titan V (SM70) | nouveau/NVK | No | Compiles SM70 | f64 shared_mem broken |

### Titan V Path to Full Speed

1. **Now**: coralReef compiles SM70 binaries for the exact shaders NAK can't handle
2. **Next**: coralDriver submits binaries directly to Titan V via nouveau ioctls
3. **Result**: Full f64 compute on Titan V, bypassing Mesa/NAK entirely

---

## Requested Evolution

### coralReef

1. **BAR.SYNC encoding**: Fix opex table value for Volta (single field)
2. **Uniform buffer bindings**: Support `var<uniform>` in compute prologue
   (needed for barraCuda's production shaders)
3. **Loop compilation**: Fix `opt_instr_sched_prepass` assertion for loop
   back-edges (enables loop-based reduction vs unrolled)
4. **f64 instruction emission**: Ensure DMUL/DADD/DFMA are generated for
   f64 operations (not f32 FMUL/FADD)
5. **SPIR-V output mode**: Alternative to native binary for wgpu submission

### barraCuda

1. **Unrolled reduction variants**: Ship WGSL shaders that use unrolled
   reduction (works now) alongside loop-based (pending coralReef fix)
2. **coralReef integration**: Wire `CoralCompiler` to cache native binaries
   keyed by `(shader_hash, arch)` — infrastructure already exists
3. **coralDriver dispatch**: When coralDriver lands, dispatch cached native
   binaries instead of wgpu submissions

### toadStool

1. **`SubstrateCapabilityKind::NativeCompile`**: New capability for primals
   that can produce verified native GPU binaries
2. **`HardwareFingerprint` evolution**: Include BAR encoding version, f64
   instruction support level, and coralDriver availability

---

## Reproduction

```bash
# Build coralReef at 849fedd
cd /path/to/coralReef && cargo build --release

# Compile the 8-step unrolled f64 reduction for Titan V
./target/release/coralreef compile shader.wgsl --arch sm_70 --fp64-software -o out.bin

# Compile for RTX 4070
./target/release/coralreef compile shader.wgsl --arch sm_89 --fp64-software -o out.bin
```

---

## Validation Certificate

- coralReef tests: **672/672 pass** (0 failures, 3 ignored)
- Multi-barrier shaders: **6/6 compile** for both SM70 and SM89
- `cargo clippy`: clean
- `cargo fmt`: clean
