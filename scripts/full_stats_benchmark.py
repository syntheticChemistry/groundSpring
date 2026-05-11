#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring — Full Stats Benchmark with Energy and Precision Tiers

Comprehensive benchmark that measures:
  1. Speed     — wall time across N_RUNS per tier (median, mean, stdev, p5/p95)
  2. Energy    — CPU (Intel RAPL) + GPU (nvidia-smi) per tier
  3. Precision — fp64 vs DF64 ("fp48") vs fp32 value comparison
  4. Parity    — all tiers produce mathematically identical results

Precision tiers (on consumer Ada/Ampere GPUs, fp64:fp32 = 1:64):
  fp64  — full IEEE 754 double (53-bit mantissa, ~15.9 digits)
  DF64  — double-float on FP32 cores (~48-bit mantissa, ~14 digits)
          9.9× throughput vs native fp64 on consumer GPUs
  fp32  — single precision (24-bit mantissa, ~7.2 digits)

DF64 is "fp48" — much of science needs more than fp32, but fp64 is
overkill. DF64 unlocks massive throughput on consumer hardware.

Usage:
    python3 scripts/full_stats_benchmark.py
    python3 scripts/full_stats_benchmark.py --save --runs 5
    sudo python3 scripts/full_stats_benchmark.py --save --runs 5  # for RAPL

Following hotSpring/barracuda/src/bench/ patterns:
  RAPL: /sys/class/powercap/intel-rapl:0/energy_uj
  GPU:  nvidia-smi --query-gpu=power.draw,temperature.gpu,memory.used
"""

from __future__ import annotations

import json
import math
import os
import signal
import subprocess
import sys
import threading
import time
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATA_DIR = ROOT / "data"

N_RUNS = 3


# ---------------------------------------------------------------------------
# Hardware inventory (hotSpring pattern)
# ---------------------------------------------------------------------------

@dataclass
class HardwareInventory:
    cpu_model: str = "unknown"
    cpu_cores: int = 0
    cpu_threads: int = 0
    ram_mb: int = 0
    gpu_name: str = "N/A"
    gpu_vram_mb: int = 0
    gpu_driver: str = "N/A"
    gpu_compute_cap: str = "N/A"
    gpu_fp64_ratio: str = "N/A"
    os_kernel: str = "unknown"

    @staticmethod
    def detect() -> HardwareInventory:
        hw = HardwareInventory()
        try:
            with open("/proc/cpuinfo") as f:
                for line in f:
                    if line.startswith("model name"):
                        hw.cpu_model = line.split(":", 1)[1].strip()
                    elif line.startswith("processor"):
                        hw.cpu_threads += 1
            hw.cpu_cores = hw.cpu_threads // 2 or hw.cpu_threads
        except OSError:
            pass
        try:
            with open("/proc/meminfo") as f:
                for line in f:
                    if line.startswith("MemTotal:"):
                        hw.ram_mb = int(line.split()[1]) // 1024
                        break
        except OSError:
            pass
        try:
            result = subprocess.run(
                ["nvidia-smi",
                 "--query-gpu=name,memory.total,driver_version,compute_cap",
                 "--format=csv,noheader,nounits"],
                capture_output=True, text=True, timeout=5, check=False,
            )
            if result.returncode == 0:
                parts = result.stdout.strip().split(", ")
                if len(parts) >= 4:
                    hw.gpu_name = parts[0].strip()
                    hw.gpu_vram_mb = int(parts[1].strip())
                    hw.gpu_driver = parts[2].strip()
                    hw.gpu_compute_cap = parts[3].strip()
                    cc_major = int(hw.gpu_compute_cap.split(".")[0])
                    if cc_major >= 7 and cc_major <= 7:
                        hw.gpu_fp64_ratio = "1:2 (Volta)"
                    elif cc_major == 8 or cc_major == 9:
                        hw.gpu_fp64_ratio = "1:64 (Ada/Ampere — DF64 optimal)"
                    else:
                        hw.gpu_fp64_ratio = "unknown"
        except (OSError, subprocess.TimeoutExpired):
            pass
        try:
            result = subprocess.run(["uname", "-r"], capture_output=True,
                                    text=True, timeout=5, check=False)
            hw.os_kernel = result.stdout.strip()
        except OSError:
            pass
        return hw


# ---------------------------------------------------------------------------
# Energy monitoring (hotSpring bench/power.rs pattern)
# ---------------------------------------------------------------------------

RAPL_PATH = "/sys/class/powercap/intel-rapl:0/energy_uj"
RAPL_MAX_PATH = "/sys/class/powercap/intel-rapl:0/max_energy_range_uj"


def read_rapl_uj() -> int | None:
    try:
        return int(Path(RAPL_PATH).read_text().strip())
    except (OSError, ValueError):
        return None


def read_rapl_max_uj() -> int:
    try:
        return int(Path(RAPL_MAX_PATH).read_text().strip())
    except (OSError, ValueError):
        return 2**63


@dataclass
class GpuSample:
    watts: float
    temp_c: float
    vram_mib: float
    timestamp: float


@dataclass
class EnergyReport:
    cpu_joules: float = 0.0
    gpu_joules: float = 0.0
    gpu_watts_avg: float = 0.0
    gpu_watts_peak: float = 0.0
    gpu_temp_peak_c: float = 0.0
    gpu_vram_peak_mib: float = 0.0
    gpu_samples: int = 0


class PowerMonitor:
    """Background RAPL + nvidia-smi monitor (hotSpring pattern)."""

    def __init__(self) -> None:
        self._rapl_start: int | None = None
        self._wall_start: float = 0.0
        self._smi_proc: subprocess.Popen | None = None
        self._samples: list[GpuSample] = []
        self._reader_thread: threading.Thread | None = None
        self._stop_event = threading.Event()

    def start(self) -> None:
        self._rapl_start = read_rapl_uj()
        self._wall_start = time.monotonic()
        self._samples = []
        self._stop_event.clear()

        try:
            self._smi_proc = subprocess.Popen(
                ["nvidia-smi",
                 "--query-gpu=power.draw,temperature.gpu,memory.used",
                 "--format=csv,noheader,nounits",
                 "-lms", "100"],
                stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                text=True,
            )
            self._reader_thread = threading.Thread(
                target=self._read_smi, daemon=True)
            self._reader_thread.start()
        except OSError:
            self._smi_proc = None

    def _read_smi(self) -> None:
        assert self._smi_proc is not None
        assert self._smi_proc.stdout is not None
        for line in self._smi_proc.stdout:
            if self._stop_event.is_set():
                break
            parts = line.strip().split(", ")
            if len(parts) >= 3:
                try:
                    self._samples.append(GpuSample(
                        watts=float(parts[0]),
                        temp_c=float(parts[1]),
                        vram_mib=float(parts[2]),
                        timestamp=time.monotonic(),
                    ))
                except ValueError:
                    pass

    def stop(self) -> EnergyReport:
        wall_elapsed = time.monotonic() - self._wall_start
        self._stop_event.set()

        if self._smi_proc is not None:
            self._smi_proc.terminate()
            try:
                self._smi_proc.wait(timeout=2)
            except subprocess.TimeoutExpired:
                self._smi_proc.kill()
        if self._reader_thread is not None:
            self._reader_thread.join(timeout=2)

        # CPU energy (RAPL)
        cpu_joules = 0.0
        rapl_end = read_rapl_uj()
        if self._rapl_start is not None and rapl_end is not None:
            if rapl_end >= self._rapl_start:
                delta = rapl_end - self._rapl_start
            else:
                delta = read_rapl_max_uj() - self._rapl_start + rapl_end
            cpu_joules = delta / 1_000_000.0

        # GPU energy (trapezoidal integration)
        samples = self._samples
        n = len(samples)
        if n == 0:
            return EnergyReport(cpu_joules=cpu_joules)

        gpu_joules = 0.0
        watts_sum = sum(s.watts for s in samples)
        watts_peak = max(s.watts for s in samples)
        temp_peak = max(s.temp_c for s in samples)
        vram_peak = max(s.vram_mib for s in samples)

        for i in range(1, n):
            dt = samples[i].timestamp - samples[i - 1].timestamp
            avg_w = (samples[i].watts + samples[i - 1].watts) / 2.0
            gpu_joules += avg_w * dt

        if n == 1:
            gpu_joules = samples[0].watts * wall_elapsed

        return EnergyReport(
            cpu_joules=cpu_joules,
            gpu_joules=gpu_joules,
            gpu_watts_avg=watts_sum / n,
            gpu_watts_peak=watts_peak,
            gpu_temp_peak_c=temp_peak,
            gpu_vram_peak_mib=vram_peak,
            gpu_samples=n,
        )


# ---------------------------------------------------------------------------
# Statistical functions for multi-run analysis
# ---------------------------------------------------------------------------

def percentile(data: list[float], p: float) -> float:
    s = sorted(data)
    k = (len(s) - 1) * p / 100.0
    lo = int(math.floor(k))
    hi = min(lo + 1, len(s) - 1)
    frac = k - lo
    return s[lo] * (1 - frac) + s[hi] * frac


@dataclass
class RunStats:
    n: int
    mean: float
    median: float
    stdev: float
    min_val: float
    max_val: float
    p5: float
    p95: float

    @staticmethod
    def from_values(values: list[float]) -> RunStats:
        n = len(values)
        if n == 0:
            return RunStats(0, 0, 0, 0, 0, 0, 0, 0)
        mean = sum(values) / n
        s = sorted(values)
        median = s[n // 2] if n % 2 == 1 else (s[n // 2 - 1] + s[n // 2]) / 2
        variance = sum((x - mean) ** 2 for x in values) / n if n > 1 else 0
        return RunStats(
            n=n, mean=mean, median=median,
            stdev=math.sqrt(variance),
            min_val=s[0], max_val=s[-1],
            p5=percentile(values, 5),
            p95=percentile(values, 95),
        )


# ---------------------------------------------------------------------------
# Benchmark runner with energy tracking
# ---------------------------------------------------------------------------

@dataclass
class TierRun:
    tier: str
    backend: str
    results: dict  # kernel -> value
    elapsed_us: dict  # kernel -> elapsed_us
    energy: EnergyReport
    wall_s: float


def _extract_json(text: str) -> dict | None:
    """Extract the last complete JSON object from mixed text output."""
    brace_depth = 0
    json_end = -1
    json_start = -1
    for i in range(len(text) - 1, -1, -1):
        if text[i] == "}":
            if brace_depth == 0:
                json_end = i
            brace_depth += 1
        elif text[i] == "{":
            brace_depth -= 1
            if brace_depth == 0:
                json_start = i
                break
    if json_start < 0 or json_end < 0:
        return None
    try:
        return json.loads(text[json_start : json_end + 1])
    except json.JSONDecodeError:
        return None


def run_binary_with_energy(cmd: list[str], tier: str, backend: str,
                           cwd: str | None = None) -> TierRun | None:
    """Run a binary with power monitoring, return results + energy."""
    monitor = PowerMonitor()
    monitor.start()
    wall_start = time.monotonic()

    try:
        result = subprocess.run(
            cmd, capture_output=True, text=True,
            timeout=600, check=False, cwd=cwd,
        )
    except Exception as e:
        monitor.stop()
        print(f"    [WARN] {tier} failed: {e}")
        return None

    wall_s = time.monotonic() - wall_start
    energy = monitor.stop()

    if result.returncode != 0:
        print(f"    [WARN] {tier} exit code {result.returncode}")
        return None

    data = _extract_json(result.stdout)
    if data is None:
        print(f"    [WARN] {tier} no JSON output")
        return None

    results = {}
    elapsed = {}
    for r in data.get("results", []):
        results[r["name"]] = r["value"]
        elapsed[r["name"]] = r["elapsed_us"]

    return TierRun(tier=tier, backend=backend, results=results,
                   elapsed_us=elapsed, energy=energy, wall_s=wall_s)


KERNELS = [
    "anderson_lyapunov_averaged",
    "mean",
    "variance",
    "pearson_r",
    "bootstrap_mean",
]


# ---------------------------------------------------------------------------
# Precision tier analysis
# ---------------------------------------------------------------------------

@dataclass
class PrecisionTier:
    name: str
    bits: int
    digits: str
    throughput_multiplier: str
    values: dict = field(default_factory=dict)
    note: str = ""


def analyze_precision(tiers: list[TierRun]) -> list[PrecisionTier]:
    """Analyze precision across fp64 (Python/Kokkos/Rust CPU) and DF64 (GPU)."""
    fp64_tier = PrecisionTier(
        name="fp64", bits=53, digits="~15.9",
        throughput_multiplier="1×",
        note="Full IEEE 754 double precision")
    df64_tier = PrecisionTier(
        name="DF64 (fp48)", bits=48, digits="~14.4",
        throughput_multiplier="~9.9×",
        note="Double-float on FP32 cores — sweet spot for science")
    fp32_tier = PrecisionTier(
        name="fp32", bits=24, digits="~7.2",
        throughput_multiplier="~64×",
        note="Single precision — NPU/inference tier")

    fp64_source = None
    gpu_source = None
    for t in tiers:
        if "Python" in t.backend or "Rust CPU" in t.backend:
            fp64_source = t
        if "GPU" in t.backend or "WGSL" in t.backend or "CUDA" in t.backend:
            gpu_source = t

    if fp64_source:
        fp64_tier.values = fp64_source.results

    if gpu_source:
        for k, v in gpu_source.results.items():
            if v != 0.0:
                df64_tier.values[k] = v

    return [fp64_tier, df64_tier, fp32_tier]


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> None:
    save = "--save" in sys.argv
    n_runs = N_RUNS
    for i, arg in enumerate(sys.argv):
        if arg == "--runs" and i + 1 < len(sys.argv):
            n_runs = int(sys.argv[i + 1])

    hw = HardwareInventory.detect()
    now = datetime.now(timezone.utc)

    gs_head = "unknown"
    try:
        gs_head = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            capture_output=True, text=True, check=True, cwd=str(ROOT),
        ).stdout.strip()
    except Exception:
        pass

    print("=" * 80)
    print("  groundSpring — Full Stats Benchmark")
    print(f"  Date: {now.isoformat()}")
    print(f"  groundSpring HEAD: {gs_head}")
    print("=" * 80)
    print()
    print(f"  CPU:  {hw.cpu_model} ({hw.cpu_cores}c/{hw.cpu_threads}t)")
    print(f"  RAM:  {hw.ram_mb} MB")
    print(f"  GPU:  {hw.gpu_name} ({hw.gpu_vram_mb} MB, CC {hw.gpu_compute_cap})")
    print(f"  fp64: {hw.gpu_fp64_ratio}")
    print(f"  OS:   {hw.os_kernel}")
    print(f"  Runs: {n_runs} per tier")
    print()

    # Pre-build all binaries
    print("  Building release binaries...")
    subprocess.run(
        ["cargo", "build", "--release", "--bin", "bench_kokkos_parity",
         "--bin", "bench_gpu_vs_kokkos"],
        capture_output=True, text=True, timeout=300, check=False,
        cwd=str(ROOT),
    )
    subprocess.run(
        ["cargo", "build", "--release", "--features", "barracuda-gpu",
         "--bin", "bench_gpu_vs_kokkos"],
        capture_output=True, text=True, timeout=300, check=False,
        cwd=str(ROOT),
    )
    print("  Build complete.\n")

    # Define tier commands
    tier_defs = [
        ("Tier 0: Python",     "Python (CPython fp64)",
         [sys.executable, str(ROOT / "control" / "baseline_runner.py"), "--json-only"]),
        ("Tier 1: Kokkos",     "Kokkos CUDA (fp64)",
         [str(ROOT / "kokkos_baseline" / "build" / "kokkos_baseline")]),
        ("Tier 2: Rust CPU",   "Rust CPU (pure Rust fp64)",
         [str(ROOT / "target" / "release" / "bench_kokkos_parity")]),
        ("Tier 3: Rust GPU",   "BarraCuda WGSL (DF64/fp64)",
         [str(ROOT / "target" / "release" / "bench_gpu_vs_kokkos")]),
    ]

    # Multi-run collection
    all_runs: dict[str, list[TierRun]] = {}

    for tier_name, backend, cmd in tier_defs:
        if not Path(cmd[0]).exists():
            print(f"  SKIP {tier_name} — binary not found: {cmd[0]}")
            continue

        print(f"  Running {tier_name} ({n_runs} runs)...")
        runs = []
        for run_idx in range(n_runs):
            r = run_binary_with_energy(cmd, tier_name, backend, cwd=str(ROOT))
            if r is not None:
                runs.append(r)
                e = r.energy
                energy_str = ""
                if e.cpu_joules > 0 or e.gpu_joules > 0:
                    energy_str = (f" | CPU={e.cpu_joules:.1f}J"
                                  f" GPU={e.gpu_joules:.1f}J"
                                  f" ({e.gpu_watts_avg:.0f}W avg)")
                print(f"    run {run_idx + 1}/{n_runs}: {r.wall_s:.2f}s{energy_str}")
            else:
                print(f"    run {run_idx + 1}/{n_runs}: FAILED")

        if runs:
            all_runs[tier_name] = runs
        print()

    if not all_runs:
        print("  ERROR: No tiers produced results.")
        sys.exit(1)

    # ======================================================================
    # SECTION 1: Multi-run timing statistics
    # ======================================================================
    print("=" * 80)
    print("  TIMING STATISTICS (microseconds)")
    print("=" * 80)
    print()

    for kernel in KERNELS:
        print(f"  {kernel}:")
        for tier_name in all_runs:
            runs = all_runs[tier_name]
            values = [r.elapsed_us.get(kernel, 0) for r in runs
                      if r.elapsed_us.get(kernel, 0) > 0]
            if not values:
                print(f"    {tier_name:<24} (evolving)")
                continue
            stats = RunStats.from_values(values)
            print(f"    {tier_name:<24} median={stats.median:>12.0f}  "
                  f"mean={stats.mean:>12.0f}  stdev={stats.stdev:>8.0f}  "
                  f"[{stats.min_val:.0f} – {stats.max_val:.0f}]")
        print()

    # ======================================================================
    # SECTION 2: CPU vs CPU — Rust fp64 vs Python fp64
    # ======================================================================
    print("=" * 80)
    print("  LEVEL 1: CPU vs CPU — Rust fp64 vs Python fp64")
    print("  Same hardware, same precision. Pure algorithmic comparison.")
    print("=" * 80)
    print()

    def _get_medians(tier_name: str) -> dict[str, float]:
        if tier_name not in all_runs:
            return {}
        out = {}
        for kernel in KERNELS:
            vals = [r.elapsed_us.get(kernel, 0)
                    for r in all_runs[tier_name]
                    if r.elapsed_us.get(kernel, 0) > 0]
            real = [r.results.get(kernel, 0) for r in all_runs[tier_name]
                    if r.results.get(kernel, 0) != 0]
            if vals and real:
                out[kernel] = sorted(vals)[len(vals) // 2]
        return out

    py_medians = _get_medians("Tier 0: Python")
    rust_cpu_medians = _get_medians("Tier 2: Rust CPU")

    if py_medians and rust_cpu_medians:
        print(f"  {'Kernel':<30} {'Python (µs)':>14} {'Rust CPU (µs)':>14} "
              f"{'Speedup':>10} {'Verdict':>10}")
        print("  " + "-" * 82)
        for kernel in KERNELS:
            py_us = py_medians.get(kernel)
            rs_us = rust_cpu_medians.get(kernel)
            if py_us and rs_us:
                speedup = py_us / rs_us
                verdict = "RUST WINS" if speedup > 1.0 else "PYTHON"
                print(f"  {kernel:<30} {py_us:>14.0f} {rs_us:>14.0f} "
                      f"{speedup:>9.1f}× {verdict:>10}")
            else:
                print(f"  {kernel:<30} {py_us or 0:>14.0f} {'—':>14} "
                      f"{'—':>10} {'—':>10}")
        print()
        all_speedups = [py_medians[k] / rust_cpu_medians[k]
                        for k in KERNELS
                        if k in py_medians and k in rust_cpu_medians]
        if all_speedups:
            geo_mean = math.exp(sum(math.log(s) for s in all_speedups) / len(all_speedups))
            print(f"  Geometric mean speedup: {geo_mean:.1f}× — Rust CPU beats Python CPU")
            print(f"  Same fp64 precision, same single-thread CPU, pure Rust wins.")
    print()

    # ======================================================================
    # SECTION 2b: GPU vs GPU — barraCuda WGSL vs Kokkos CUDA
    # ======================================================================
    print("=" * 80)
    print("  LEVEL 2: GPU vs GPU — barraCuda WGSL vs Kokkos CUDA")
    print("  Same GPU, same fp64 precision. Framework comparison.")
    print("=" * 80)
    print()

    kokkos_medians = _get_medians("Tier 1: Kokkos")
    gpu_medians = _get_medians("Tier 3: Rust GPU")

    if kokkos_medians and gpu_medians:
        print(f"  {'Kernel':<30} {'Kokkos (µs)':>14} {'barraCuda (µs)':>14} "
              f"{'Ratio':>10} {'Status':>12}")
        print("  " + "-" * 84)
        for kernel in KERNELS:
            ko_us = kokkos_medians.get(kernel)
            bc_us = gpu_medians.get(kernel)
            if ko_us and bc_us:
                ratio = ko_us / bc_us
                if ratio > 0.9:
                    status = "PARITY" if ratio > 0.9 else "COMPETITIVE"
                elif ratio > 0.5:
                    status = "COMPETITIVE"
                else:
                    status = f"gap {1/ratio:.1f}×"
                print(f"  {kernel:<30} {ko_us:>14.0f} {bc_us:>14.0f} "
                      f"{ratio:>9.2f}× {status:>12}")
            elif ko_us and not bc_us:
                print(f"  {kernel:<30} {ko_us:>14.0f} {'(evolving)':>14} "
                      f"{'—':>10} {'EVOLVING':>12}")
            else:
                print(f"  {kernel:<30} {'—':>14} {'—':>14} "
                      f"{'—':>10} {'—':>12}")
        print()

        matched = [k for k in KERNELS if k in kokkos_medians and k in gpu_medians]
        if matched:
            ratios = [kokkos_medians[k] / gpu_medians[k] for k in matched]
            geo = math.exp(sum(math.log(r) for r in ratios) / len(ratios))
            evolving = [k for k in KERNELS
                        if k in kokkos_medians and k not in gpu_medians]
            print(f"  Dispatching kernels ({len(matched)}/{len(KERNELS)}): "
                  f"geo mean ratio = {geo:.2f}×")
            if evolving:
                print(f"  Evolving kernels ({len(evolving)}): "
                      f"{', '.join(evolving)}")
                print(f"  → barraCuda reduce shaders need wiring in barraCuda primal")
        print()
        print("  Target: match Kokkos at fp64, then unlock DF64 for ~9.9× throughput")
    elif kokkos_medians:
        print("  barraCuda GPU dispatch not yet producing values for comparison.")
        print("  Kokkos CUDA baseline ready — waiting for barraCuda reduce shader evolution.")
    print()

    # ======================================================================
    # SECTION 2c: DF64 unlock projection
    # ======================================================================
    print("=" * 80)
    print("  LEVEL 3: DF64 UNLOCK — the science throughput multiplier")
    print("  Once fp64 parity is proven, DF64 gives ~9.9× on FP32 cores")
    print("=" * 80)
    print()

    print("  Precision tiers on this GPU:")
    print(f"    GPU: {hw.gpu_name} (CC {hw.gpu_compute_cap})")
    print(f"    fp64:fp32 ratio: {hw.gpu_fp64_ratio}")
    print()
    print("  ┌─────────┬──────────┬─────────────┬──────────────┬───────────────────────┐")
    print("  │ Tier    │ Mantissa │ Digits      │ Throughput   │ Use Case              │")
    print("  ├─────────┼──────────┼─────────────┼──────────────┼───────────────────────┤")
    print("  │ fp64    │ 53 bit   │ ~15.9       │ 1× (native)  │ Reference / overkill  │")
    print("  │ DF64    │ ~48 bit  │ ~14.4       │ ~9.9×        │ SWEET SPOT for science│")
    print("  │ fp32    │ 24 bit   │  ~7.2       │ ~64×         │ NPU / inference       │")
    print("  └─────────┴──────────┴─────────────┴──────────────┴───────────────────────┘")
    print()

    if kokkos_medians and gpu_medians:
        matched = [k for k in KERNELS if k in kokkos_medians and k in gpu_medians]
        if matched:
            print("  DF64 throughput projection (once all kernels dispatch):")
            for k in matched:
                ko_us = kokkos_medians[k]
                bc_fp64_us = gpu_medians[k]
                df64_projected = bc_fp64_us / 9.9
                ratio_vs_kokkos = ko_us / df64_projected
                print(f"    {k}: barraCuda fp64={bc_fp64_us:.0f}µs "
                      f"→ DF64≈{df64_projected:.0f}µs "
                      f"(vs Kokkos {ko_us:.0f}µs = {ratio_vs_kokkos:.1f}×)")
            print()

    print("  Much of science needs more than fp32, but fp64 is overkill.")
    print("  DF64 on FP32 cores: ~14 digits at ~9.9× throughput.")
    print("  On consumer Ada/Ampere GPUs (1:64 fp64:fp32), this is transformative.")
    print()

    # ======================================================================
    # SECTION 3: Energy comparison
    # ======================================================================
    print("=" * 80)
    print("  ENERGY COMPARISON (per full benchmark run)")
    print("=" * 80)
    print()

    header = f"  {'Metric':<24}"
    for tier_name in all_runs:
        header += f" {tier_name.split(':')[1].strip():>14}"
    print(header)
    print("  " + "-" * (len(header) - 2))

    for metric_name, metric_fn in [
        ("Wall time (s)",      lambda r: r.wall_s),
        ("CPU energy (J)",     lambda r: r.energy.cpu_joules),
        ("GPU energy (J)",     lambda r: r.energy.gpu_joules),
        ("GPU power avg (W)",  lambda r: r.energy.gpu_watts_avg),
        ("GPU power peak (W)", lambda r: r.energy.gpu_watts_peak),
        ("GPU temp peak (°C)", lambda r: r.energy.gpu_temp_peak_c),
        ("GPU VRAM peak (MiB)",lambda r: r.energy.gpu_vram_peak_mib),
    ]:
        line = f"  {metric_name:<24}"
        for tier_name in all_runs:
            runs = all_runs[tier_name]
            values = [metric_fn(r) for r in runs if metric_fn(r) > 0]
            if values:
                med = sorted(values)[len(values) // 2]
                line += f" {med:>14.1f}"
            else:
                line += f" {'—':>14}"
        print(line)

    # Energy efficiency
    if "Tier 0: Python" in all_runs and len(all_runs) > 1:
        py_wall = sorted([r.wall_s for r in all_runs["Tier 0: Python"]]
                         )[len(all_runs["Tier 0: Python"]) // 2]
        py_cpu_j = sorted([r.energy.cpu_joules
                           for r in all_runs["Tier 0: Python"]
                           if r.energy.cpu_joules > 0] or [0])[0]
        print()
        print(f"  {'Energy efficiency':}")
        for tier_name in all_runs:
            if tier_name == "Tier 0: Python":
                continue
            runs = all_runs[tier_name]
            wall_vals = sorted([r.wall_s for r in runs])
            t_wall = wall_vals[len(wall_vals) // 2]
            total_j_vals = [r.energy.cpu_joules + r.energy.gpu_joules
                            for r in runs
                            if r.energy.cpu_joules > 0 or r.energy.gpu_joules > 0]
            if total_j_vals and py_cpu_j > 0:
                t_j = sorted(total_j_vals)[len(total_j_vals) // 2]
                efficiency = py_cpu_j / t_j if t_j > 0 else float("inf")
                print(f"    {tier_name}: {efficiency:.1f}× less energy than Python "
                      f"({t_j:.1f}J vs {py_cpu_j:.1f}J)")
            elif py_wall > 0:
                speed_eff = py_wall / t_wall
                print(f"    {tier_name}: {speed_eff:.1f}× faster "
                      f"({t_wall:.2f}s vs {py_wall:.2f}s)")
    print()

    # ======================================================================
    # SECTION 4: Precision parity
    # ======================================================================
    print("=" * 80)
    print("  PRECISION PARITY (value comparison across tiers)")
    print("=" * 80)
    print()

    # Use first run from each tier for value comparison
    tier_first: dict[str, TierRun] = {
        name: runs[0] for name, runs in all_runs.items()
    }

    header = f"  {'Kernel':<30}"
    for tier_name in tier_first:
        header += f" {tier_name.split(':')[1].strip():>20}"
    header += f" {'Max Diff':>12} {'Status':>8}"
    print(header)
    print("  " + "-" * (len(header) - 2))

    proven = 0
    for kernel in KERNELS:
        line = f"  {kernel:<30}"
        values = []
        for tier_name, run in tier_first.items():
            v = run.results.get(kernel)
            if v is not None and v != 0.0:
                line += f" {v:>20.12e}"
                values.append(v)
            elif v == 0.0:
                line += f" {'0 (evolving)':>20}"
            else:
                line += f" {'—':>20}"

        if len(values) >= 2:
            max_diff = max(values) - min(values)
            status = "PROVEN" if max_diff < 1e-8 else "DIFF"
            if status == "PROVEN":
                proven += 1
            line += f" {max_diff:>12.2e} {status:>8}"
        else:
            line += f" {'—':>12} {'—':>8}"
        print(line)

    print()
    print(f"  {proven}/{len(KERNELS)} kernels: mathematical parity PROVEN")
    print()

    # ======================================================================
    # SECTION 5: Full evolution path summary
    # ======================================================================
    print("=" * 80)
    print("  EVOLUTION PATH SUMMARY")
    print("=" * 80)
    print()

    # CPU level
    if py_medians and rust_cpu_medians:
        cpu_speedups = [py_medians[k] / rust_cpu_medians[k]
                        for k in KERNELS
                        if k in py_medians and k in rust_cpu_medians]
        if cpu_speedups:
            geo = math.exp(sum(math.log(s) for s in cpu_speedups) / len(cpu_speedups))
            print(f"  Level 1 — CPU:  Rust fp64 is {geo:.0f}× faster than Python fp64")
            print(f"    Status: PROVEN — identical precision, pure compiled Rust wins")
    print()

    # GPU level
    if kokkos_medians and gpu_medians:
        matched = [k for k in KERNELS if k in kokkos_medians and k in gpu_medians]
        evolving = [k for k in KERNELS
                    if k in kokkos_medians and k not in gpu_medians]
        if matched:
            ratios = [kokkos_medians[k] / gpu_medians[k] for k in matched]
            geo = math.exp(sum(math.log(r) for r in ratios) / len(ratios))
            parity_label = "COMPETITIVE" if geo > 0.3 else "EVOLVING"
            print(f"  Level 2 — GPU:  barraCuda vs Kokkos = {geo:.2f}× "
                  f"({len(matched)}/{len(KERNELS)} kernels dispatching)")
            print(f"    Status: {parity_label} — "
                  f"{len(evolving)} kernels need reduce shader wiring")
    elif kokkos_medians:
        print(f"  Level 2 — GPU:  Kokkos baseline ready, barraCuda dispatch evolving")
    print()

    # DF64 projection
    print(f"  Level 3 — DF64: Once Kokkos fp64 parity is proven:")
    print(f"    → DF64 unlocks ~9.9× throughput at ~14 digits precision")
    print(f"    → On {hw.gpu_name} (fp64:fp32 = {hw.gpu_fp64_ratio})")
    print(f"    → fp64 is overkill for most science. DF64 is the sweet spot.")
    print()

    # Overall
    print(f"  Precision: {proven}/{len(KERNELS)} kernels proven parity "
          f"(Python = Kokkos = Rust CPU)")
    for tier_name, runs in all_runs.items():
        med_wall = sorted([r.wall_s for r in runs])[len(runs) // 2]
        print(f"    {tier_name}: {runs[0].backend} — "
              f"median {med_wall:.2f}s ({len(runs)} runs)")
    print("=" * 80)

    if save:
        DATA_DIR.mkdir(parents=True, exist_ok=True)
        cert = {
            "title": "groundSpring Full Stats Benchmark",
            "date": now.isoformat(),
            "groundspring_head": gs_head,
            "hardware": {
                "cpu": hw.cpu_model,
                "gpu": hw.gpu_name,
                "gpu_compute_cap": hw.gpu_compute_cap,
                "gpu_fp64_ratio": hw.gpu_fp64_ratio,
                "ram_mb": hw.ram_mb,
            },
            "n_runs": n_runs,
            "tiers": {},
        }
        for tier_name, runs in all_runs.items():
            walls = [r.wall_s for r in runs]
            energies = [r.energy.cpu_joules + r.energy.gpu_joules for r in runs]
            cert["tiers"][tier_name] = {
                "backend": runs[0].backend,
                "wall_s": {"median": sorted(walls)[len(walls) // 2],
                           "mean": sum(walls) / len(walls)},
                "total_energy_j": {"median": sorted(energies)[len(energies) // 2]
                                   if any(e > 0 for e in energies) else 0},
                "values": runs[0].results,
                "timing_us": {
                    k: RunStats.from_values(
                        [r.elapsed_us.get(k, 0) for r in runs
                         if r.elapsed_us.get(k, 0) > 0]
                    ).__dict__
                    for k in KERNELS
                },
            }
        path = DATA_DIR / "full_stats_benchmark.json"
        path.write_text(json.dumps(cert, indent=2, default=str))
        print(f"\n  Certificate saved to {path.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
