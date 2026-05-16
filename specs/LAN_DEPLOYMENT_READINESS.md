# LAN HPC Deployment Readiness Checklist

**Date**: May 8, 2026
**Status**: Eastgate NUCLEUS validated (V143 / guideStone L4), awaiting 10G Cat6a cables

---

## Eastgate (Primary NUCLEUS) — VALIDATED (V117)

| Component | Status | Details |
|-----------|--------|---------|
| BearDog (crypto) | OPERATIONAL | PID active, socket `beardog.sock`, TCP 9900 |
| Songbird (network) | OPERATIONAL | PID active, socket `songbird.sock`, TCP 3492 |
| ToadStool (compute) | OPERATIONAL | PID active, socket `toadstool.sock`, JSON-RPC `toadstool.jsonrpc.sock` |
| NestGate (storage) | OPERATIONAL | PID active, HTTP API `127.0.0.1:8090`, tarpc `8091` |
| Squirrel (AI) | OPERATIONAL | PID active, socket `squirrel.sock`, 2 providers |
| Neural API | OPERATIONAL | PID active, socket `neural-api.sock` |
| `.family.seed` | PRESENT | Canonical at `~/.family.seed` + biomeOS copies |
| `.beacon.seed` | PRESENT | |
| `.lineage.seed` | PRESENT | |
| Family ID | `8ff3b864a4bc589a` | Derived from `.family.seed[0..8]` |

### V115 Validation Results

- 930+ Rust tests: ALL PASS (default + biomeos features)

### V116 Validation Results

- 1,123 Rust tests: ALL PASS (default + biomeos features)
- 427/427 experiment checks: ALL PASS (340 core + 55 NUCLEUS + 32 LTEE)
- 44 three-tier parity tests: ALL PASS
- metalForge GPU: 11/11
- metalForge spectral: 6/6
- metalForge mathieu: 7/7
- metalForge Titan V: 13/13
- metalForge inventory: 11/14 (3 expected NPU-only failures)
- NestGate NCBI validation: 9/9
- clippy pedantic + nursery: ZERO WARNINGS
- `cargo fmt`: CLEAN
- Zero `#[allow()]` in production code
- Zero unsafe code
- Zero mocks outside `#[cfg(test)]`

### V114 Evolution (since V113)

| Evolution | Impact |
|-----------|--------|
| `.expect()` → `OrExit` | 29 validation binaries, zero-panic exits |
| `cast::` module | Checked numeric conversions in 7 modules |
| `resilient_call()` | RetryPolicy + CircuitBreaker for NestGate pipeline |
| `health.liveness`/`health.readiness` | NUCLEUS health probes registered |
| `primal_names::roles` | Runtime discovery, zero hardcoded names |
| `#[expect(reason)]` | All lint exceptions documented |
| Deploy graphs V114 | All 3 graphs updated, health probes added |

---

## Physical Prerequisites

| Item | Status | Action |
|------|--------|--------|
| 10G switch | INSTALLED | Netgear XS508M |
| 10G NICs | INSTALLED | In all primary gates |
| Cat6a cables | **PENDING** | Need 5× runs (Eastgate↔switch, Northgate↔switch, Westgate↔switch, Southgate↔switch, Strandgate↔switch) |

---

## Gate Deployment Plan (post-cable)

### Phase 1: Westgate Nest Atomic (76TB ZFS cold storage)

```bash
# From Eastgate:
cd "$ECOPRIMALS_ROOT/phase2/biomeOS"
./livespore-usb/x86_64/scripts/deploy_to_gate.sh <westgate-ip> westgate

# On Westgate:
cd /tmp/biomeos-livespore
NODE_ID=westgate NESTGATE_JWT_SECRET=$(openssl rand -base64 48) \
  ./scripts/start_tower.sh  # Tower first
# Then manually start NestGate with ZFS backend
```

**Westgate specialization**: Nest Atomic (Tower + NestGate + Squirrel). 76TB ZFS pool for content-addressed blob storage, NestGate cold storage, SRA FASTQ archives.

### Phase 2: Northgate Node Atomic (RTX 5090 heavy GPU)

```bash
# From Eastgate:
./livespore-usb/x86_64/scripts/deploy_to_gate.sh <northgate-ip> northgate

# On Northgate:
NODE_ID=northgate ./start.sh
```

**Northgate specialization**: Node Atomic (Tower + ToadStool). RTX 5090 (32GB VRAM) for large Anderson lattices (L=14-20, 8000×8000 matrices), WDM MD sweeps, and any workload exceeding RTX 4070 VRAM.

### Phase 3: Strandgate Full NUCLEUS (bioinformatics hub)

**Strandgate specialization**: Full NUCLEUS. Dual EPYC 7313 (128 threads), RTX 3090 + RX 6950 XT (multi-vendor GPU parity), 2× Akida NPU, 256GB ECC. CPU-bound bioinformatics (Kraken2, alignment), multi-vendor GPU validation.

### Phase 4: biomeGate Node Atomic (Titan V f64)

**biomeGate specialization**: Node Atomic. Titan V (5120 CUDA, native f64), RTX 3090, Akida NPU. f64 GPU dispatch target for precision-critical workloads (WDM transport, spectral reconstruction).

### Phase 5: Remaining Gates

- **Southgate**: Node Atomic, RTX 3090 additional capacity
- **FlockGate/KinGate/Swiftgate**: Tower Atomic relay nodes
- **3× Intel NUC**: Always-on Tower Atomic beacons

---

## What Can Run Now (Before LAN)

With local NUCLEUS on eastGate, groundSpring V117 can execute all Tier 0-1
dataset extensions without LAN infrastructure:

| Tier | Datasets | Compute | Status |
|------|----------|---------|--------|
| 0 | EMP 30K synthetic, NCBI Protein QS, analytical baselines | Minutes | Ready |
| 1 | Cold seep metadata, LTEE, real GHCND, IRIS, symbiotic metagenomes | ~3h total | Ready via NestGate |

LAN is needed only for Tier 2+ (KBS LTER 200GB, EMP 30K real 50GB,
Tara Oceans 100GB, HMP 50GB) and Tier 3 (multi-TB SRA surveys).

---

## Phased Gate Deployment Plan (V117)

### Immediate (Now — Local NUCLEUS on eastGate)

```
eastGate [Tower + Node + Nest]
  ├── BearDog (crypto/identity)
  ├── Songbird (discovery)
  ├── ToadStool (GPU: RTX 4070, 12GB VRAM)
  ├── NestGate (NCBI + NOAA + IRIS + local storage)
  ├── Squirrel (AI: ESN/LSTM classification)
  └── groundSpring V117 (measurement.* capabilities + health probes)
```

**Actionable now:**
1. Fetch real GHCND weather → Exp 036 (ET₀ → Anderson on real Ohio data)
2. Fetch NCBI Protein QS genes → extend Exp 140-142
3. Fetch cold seep PRJNA315684 metadata → new Exp 036 (regime classification)
4. Fetch symbiotic metagenomes → new Exp 036 (cross-species QS)

### Phase 1: westGate Nest Atomic (after 10G cables)

**Priority: FIRST** — 76TB ZFS enables Tier 2 dataset storage.

```
westGate [Tower + NestGate]
  ├── BearDog + Songbird
  └── NestGate (76TB ZFS: cold storage for SRA FASTQ, EMP, KBS LTER)
```

**Unlocks:**
- EMP 30K real samples (~50GB) stored on ZFS
- KBS LTER 30yr soil data (~200GB) archived
- Cold seep FASTQ bulk download (~170GB) cached
- NestGate SRA evolution target (bulk FASTQ via SRA Toolkit)

### Phase 2: northGate Node Atomic (after 10G cables)

```
northGate [Tower + ToadStool]
  ├── BearDog + Songbird
  └── ToadStool (GPU: RTX 5090, 32GB VRAM)
```

**Unlocks:**
- Large Anderson lattices (L=14-20, 3D) for Paper 01/12
- EMP full pipeline at scale (~30min vs ~4h on eastGate)
- Parallel parameter sweeps via biomeOS distributed dispatch

### Phase 3: strandGate Full NUCLEUS (bioinformatics hub)

```
strandGate [Tower + Node + Nest + Squirrel (Full)]
  ├── BearDog + Songbird
  ├── ToadStool (GPU: RTX 3090 + RX 6950 XT — multi-vendor)
  ├── NestGate (local storage)
  ├── Squirrel (2× Akida NPU)
  └── CPU: Dual EPYC 7313 (128 threads)
```

**Unlocks:**
- Multi-vendor GPU parity validation (NVIDIA + AMD)
- CPU-bound bioinformatics (Kraken2, DADA2 on 128 threads)
- NPU sentinel pipeline (Akida × 2 for ESN classification)

### Phase 4: biomeGate Node Atomic (precision GPU)

```
biomeGate [Tower + ToadStool]
  ├── BearDog + Songbird
  └── ToadStool (GPU: 2× Titan V HBM2 + 2× MI50 + Akida NPU)
```

**Unlocks:**
- Native f64 GPU compute (Titan V: 5120 CUDA cores with HBM2)
- DF64 validation (MI50: 16GB HBM2)
- Large-lattice 3D Anderson at full f64 precision
- Paper 07 WDM precision-critical workloads

### Phase 5: Full LAN Mesh

All gates connected via 10G backbone. biomeOS Plasmodium collective
discovery enables distributed dispatch across all GPU/NPU resources:

| Gate | VRAM | Storage | Specialization |
|------|------|---------|----------------|
| eastGate | 12GB (RTX 4070) | 2TB NVMe | Primary orchestrator |
| westGate | — | 76TB ZFS | Cold storage + SRA archive |
| northGate | 32GB (RTX 5090) | 2TB NVMe | Heavy GPU compute |
| strandGate | 24GB + 16GB | 4TB NVMe | Multi-vendor + bioinformatics |
| biomeGate | 24GB + 32GB + 32GB | 2TB NVMe | Precision f64 + NPU |
| southGate | 24GB (RTX 3090) | 2TB NVMe | Additional capacity |
| **Total** | **~164GB** | **~88TB** | |

---

## Compute Budget by Tier (V117)

| Tier | Data | eastGate (single GPU) | Full LAN | Blocking |
|------|------|-----------------------|----------|----------|
| 0 | In-memory | Minutes | N/A | Nothing |
| 1 | ~200GB | ~4h | N/A (local sufficient) | Nothing |
| 2 | ~400GB | ~10h | ~1h | 10G cables |
| 3 | Multi-TB | Days | ~12h | 10G cables + NestGate SRA |

---

## Plasmodium Validation (post-deployment)

Once 2+ gates are connected:

1. `biomeos plasmodium status` — verify collective discovery
2. Cross-gate determinism: same eigenvalues from Eastgate CPU vs Northgate GPU
3. metalForge `merge_remote()` discovers remote NUCLEUS substrates
4. End-to-end science pipeline: NestGate data (Westgate) → ToadStool compute (Northgate) → provenance (Westgate)

---

## Deployment Commands Reference

```bash
# Start full NUCLEUS on Eastgate:
cd "$ECOPRIMALS_ROOT/phase2/biomeOS"
NODE_ID=eastgate ./scripts/start_nucleus.sh full

# Start NestGate separately (HTTP API, requires JWT):
NESTGATE_JWT_SECRET=$(openssl rand -base64 48) \
  ./primals/nestgate service start --port 8090 --daemon

# Deploy to a LAN gate:
./livespore-usb/x86_64/scripts/deploy_to_gate.sh <ip> <node-id>

# Verify primal health:
echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | nc -U /run/user/1000/biomeos/beardog.sock -w 3 -q 1

# Run groundSpring validation with NUCLEUS:
cargo test --workspace --features biomeos
cargo run -p groundspring-forge --features biomeos --bin validate-nestgate-ncbi
```
