#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
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

# Auto-discover all benchmark JSONs and their companion Python scripts.
# Each control/<experiment>/benchmark_*.json is paired with the Python
# script listed in its _provenance.validation_script field.
mapfile -t BENCHMARKS < <(find control -name 'benchmark_*.json' -type f | sort)

if [[ ${#BENCHMARKS[@]} -eq 0 ]]; then
    echo "ERROR: No benchmark JSONs found under control/"
    exit 1
fi

BASELINES=()
for bench in "${BENCHMARKS[@]}"; do
    script="$(python3 -c "
import json, sys
with open('$bench') as f:
    d = json.load(f)
cmd = d.get('_provenance', {}).get('command', '')
if not cmd:
    script = d.get('_provenance', {}).get('validation_script', '')
    cmd = 'python3 ' + script if script else ''
print(cmd)
")"
    if [[ -z "$script" ]]; then
        echo "WARNING: No command in _provenance for $bench — skipping"
        continue
    fi
    BASELINES+=("$script")
done

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
