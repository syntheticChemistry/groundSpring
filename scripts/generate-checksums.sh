#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Generate BLAKE3 CHECKSUMS manifest for guideStone Property 3 (self-verifying).
# Run from the groundSpring repo root.
set -euo pipefail

MANIFEST="validation/CHECKSUMS"

FILES=(
    "crates/groundspring/src/lib.rs"
    "crates/groundspring/src/decompose.rs"
    "crates/groundspring/src/niche.rs"
    "crates/groundspring/src/tol.rs"
    "crates/groundspring/src/eps.rs"
    "crates/groundspring/src/validate/mod.rs"
    "crates/groundspring/src/validate/sink.rs"
    "crates/groundspring/src/validate/harness.rs"
    "crates/groundspring/src/primal_names.rs"
    "crates/groundspring/src/provenance_registry.rs"
    "crates/groundspring/src/ipc/mod.rs"
    "crates/groundspring/src/ipc/coralreef.rs"
    "crates/groundspring-validate/src/groundspring_guidestone.rs"
    "crates/groundspring-validate/src/validate_ltee_clonal.rs"
    "crates/groundspring-validate/src/tolerances.rs"
    "Cargo.toml"
    "deny.toml"
)

if ! command -v b3sum &>/dev/null; then
    echo "ERROR: b3sum not found. Install via: cargo install b3sum" >&2
    exit 1
fi

: > "$MANIFEST"

for file in "${FILES[@]}"; do
    if [[ -f "$file" ]]; then
        hash=$(b3sum --no-names "$file")
        echo "${hash}  ${file}" >> "$MANIFEST"
    else
        echo "WARNING: $file not found, skipping" >&2
    fi
done

echo "Generated $MANIFEST with $(wc -l < "$MANIFEST") entries"
