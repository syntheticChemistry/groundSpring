#!/usr/bin/env bash
# groundSpring — Run All Phase 0 Baselines
#
# Usage:
#   bash scripts/run_all_baselines.sh
#
# Runs all five groundSpring experiments and reports summary.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"

cd "$ROOT"

TOTAL_PASS=0
TOTAL_FAIL=0
EXPERIMENTS=()
RESULTS=()

run_experiment() {
    local name="$1"
    local script="$2"

    echo ""
    echo "================================================================"
    echo "  Running: $name"
    echo "================================================================"

    if python3 "$script"; then
        RESULTS+=("PASS")
        echo ""
        echo "  >>> $name: PASS"
    else
        RESULTS+=("FAIL")
        echo ""
        echo "  >>> $name: FAIL"
    fi
    EXPERIMENTS+=("$name")
}

echo "================================================================"
echo "  groundSpring — Phase 0 Baseline Validation Suite"
echo "  Date: $(date -Iseconds)"
echo "================================================================"

# Exp 001: Sensor Noise Characterization
run_experiment \
    "Exp 001: Sensor Noise Characterization" \
    "control/sensor_noise/sensor_noise_decomposition.py"

# Exp 002: Weather Model vs Observation Gap
run_experiment \
    "Exp 002: Weather Model vs Observation Gap" \
    "control/observation_gap/observation_gap.py"

# Exp 003: Error Propagation Through FAO-56
run_experiment \
    "Exp 003: Error Propagation Through FAO-56" \
    "control/error_propagation/error_propagation_fao56.py"

# Exp 004: Sequencing Depth & Taxonomic Noise
run_experiment \
    "Exp 004: Sequencing Depth & Taxonomic Noise" \
    "control/sequencing_noise/sequencing_noise.py"

# Exp 005: Seismic Wave Propagation
run_experiment \
    "Exp 005: Seismic Wave Propagation" \
    "control/seismic/seismic_inversion.py"

# Optional: Download IRIS station metadata
echo ""
echo "================================================================"
echo "  Optional: IRIS Station Metadata"
echo "================================================================"
if command -v python3 &> /dev/null; then
    python3 scripts/download_iris.py --stations 2>&1 || echo "  [SKIP] IRIS download failed (may need network)"
else
    echo "  [SKIP] python3 not found"
fi

# Summary
echo ""
echo "================================================================"
echo "  GRAND SUMMARY"
echo "================================================================"

N_PASS=0
N_FAIL=0
for i in "${!EXPERIMENTS[@]}"; do
    status="${RESULTS[$i]}"
    echo "  ${EXPERIMENTS[$i]}: $status"
    if [ "$status" = "PASS" ]; then
        ((N_PASS++)) || true
    else
        ((N_FAIL++)) || true
    fi
done

echo ""
echo "  Total: $N_PASS PASS, $N_FAIL FAIL out of ${#EXPERIMENTS[@]} experiments"
echo "================================================================"

if [ "$N_FAIL" -gt 0 ]; then
    exit 1
fi
