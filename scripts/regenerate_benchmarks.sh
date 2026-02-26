#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
#
# groundSpring — Benchmark Drift Guard
#
# Re-runs every Python baseline and verifies the benchmark JSON provenance
# fields match the current repo state. Use after modifying any experiment
# or benchmark file to ensure no silent baseline drift.
#
# Usage:
#   bash scripts/regenerate_benchmarks.sh          # verify only
#   bash scripts/regenerate_benchmarks.sh --stamp  # update baseline_commit + date
#
# Exit code:
#   0 — all baselines pass and provenance is current
#   1 — at least one baseline failed or provenance is stale

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"
cd "$ROOT"

STAMP=false
if [[ "${1:-}" == "--stamp" ]]; then
    STAMP=true
fi

HEAD_SHA="$(git rev-parse HEAD)"
TODAY="$(date +%Y-%m-%d)"
FAIL=0

echo "================================================================"
echo "  groundSpring — Benchmark Drift Guard"
echo "  HEAD: $HEAD_SHA"
echo "  Date: $TODAY"
echo "================================================================"
echo ""

BENCHMARKS=(
    "control/sensor_noise/benchmark_sensor_noise.json"
    "control/observation_gap/benchmark_observation_gap.json"
    "control/error_propagation/benchmark_error_propagation.json"
    "control/sequencing_noise/benchmark_sequencing_noise.json"
    "control/seismic/benchmark_seismic.json"
    "control/signal_specificity/benchmark_signal_specificity.json"
    "control/rawr_resampling/benchmark_rawr_resampling.json"
    "control/anderson_localization/benchmark_anderson_localization.json"
    "control/quasiperiodic/benchmark_quasiperiodic.json"
    "control/bistable_switching/benchmark_bistable.json"
    "control/multisignal_qs/benchmark_multisignal.json"
    "control/spin_transport/benchmark_spin_transport.json"
    "control/resampling_convergence/benchmark_resampling_convergence.json"
    "control/drift_selection/benchmark_drift_selection.json"
    "control/uncertainty_bridge/benchmark_uncertainty_bridge.json"
)

BASELINES=(
    "python3 control/sensor_noise/sensor_noise_decomposition.py"
    "python3 control/observation_gap/observation_gap.py"
    "python3 control/error_propagation/error_propagation_fao56.py"
    "python3 control/sequencing_noise/sequencing_noise.py"
    "python3 control/seismic/seismic_inversion.py"
    "python3 control/signal_specificity/signal_specificity.py"
    "python3 control/rawr_resampling/rawr_resampling.py"
    "python3 control/anderson_localization/anderson_localization.py"
    "python3 control/quasiperiodic/quasiperiodic_localization.py"
    "python3 control/bistable_switching/bistable_switching.py"
    "python3 control/multisignal_qs/multisignal_qs.py"
    "python3 control/spin_transport/spin_chain_transport.py"
    "python3 control/resampling_convergence/resampling_convergence.py"
    "python3 control/drift_selection/drift_selection.py"
    "python3 control/uncertainty_bridge/uncertainty_bridge.py"
)

echo "--- Phase 1: Re-run all Python baselines ---"
echo ""

for i in "${!BASELINES[@]}"; do
    cmd="${BASELINES[$i]}"
    bench="${BENCHMARKS[$i]}"
    name="$(basename "$bench" .json)"
    printf "  %-44s " "$name"
    if eval "$cmd" > /dev/null 2>&1; then
        echo "PASS"
    else
        echo "FAIL"
        ((FAIL++)) || true
    fi
done

echo ""
echo "--- Phase 2: Provenance check ---"
echo ""

for bench in "${BENCHMARKS[@]}"; do
    name="$(basename "$bench" .json)"
    stored_sha="$(python3 -c "
import json, sys
with open('$bench') as f:
    d = json.load(f)
prov = d.get('_provenance', {})
print(prov.get('baseline_commit', 'MISSING'))
")"
    printf "  %-44s " "$name"
    if [[ "$stored_sha" == "$HEAD_SHA" ]]; then
        echo "current"
    elif [[ "$stored_sha" == "MISSING" || "$stored_sha" == "pending" ]]; then
        echo "MISSING ($stored_sha)"
        ((FAIL++)) || true
    else
        echo "STALE (${stored_sha:0:12}…)"
        ((FAIL++)) || true
    fi
done

if $STAMP; then
    echo ""
    echo "--- Stamping provenance ---"
    echo ""
    for bench in "${BENCHMARKS[@]}"; do
        python3 -c "
import json
with open('$bench') as f:
    d = json.load(f)
prov = d.get('_provenance', {})
prov['baseline_commit'] = '$HEAD_SHA'
prov['baseline_date'] = '$TODAY'
d['_provenance'] = prov
with open('$bench', 'w') as f:
    json.dump(d, f, indent=2)
    f.write('\n')
"
        echo "  Stamped $(basename "$bench")"
    done
fi

echo ""
if [[ "$FAIL" -gt 0 ]]; then
    echo "  RESULT: $FAIL issue(s) found. Run with --stamp after fixes."
    exit 1
else
    echo "  RESULT: All benchmarks pass. Provenance is current."
fi
