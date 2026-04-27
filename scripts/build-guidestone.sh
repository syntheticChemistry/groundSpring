#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Build and optionally run the groundSpring guideStone binary.
# Run from the groundSpring repo root.
set -euo pipefail

echo "=== Building groundspring_guidestone ==="
cargo build --release -p groundspring-validate --bin groundspring_guidestone --features guidestone

echo ""
echo "=== Generating CHECKSUMS ==="
bash scripts/generate-checksums.sh

if [[ "${1:-}" == "--run" ]]; then
    echo ""
    echo "=== Running groundspring_guidestone ==="
    cargo run --release -p groundspring-validate --bin groundspring_guidestone --features guidestone
fi
