# LAN HPC Deployment Readiness Checklist

**Date**: February 28, 2026
**Status**: Eastgate NUCLEUS validated, awaiting 10G Cat6a cables

---

## Eastgate (Primary NUCLEUS) — VALIDATED

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

### Validation Results

- 498+ Rust tests: ALL PASS (default + biomeos features)
- 347/347 experiment checks: ALL PASS (292 core + 55 NUCLEUS)
- 44 three-tier parity tests: ALL PASS
- metalForge GPU: 11/11
- metalForge spectral: 6/6
- metalForge mathieu: 7/7
- metalForge Titan V: 13/13
- metalForge inventory: 11/14 (3 expected NPU-only failures)
- NestGate NCBI validation: 9/9
- clippy::pedantic: CLEAN

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
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
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
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
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
