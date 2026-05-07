#!/usr/bin/env python3
"""Generate publication-grade Jupyter notebooks from groundSpring Python baselines.

Reads each control/<experiment>/<script>.py and converts it into a structured
.ipynb notebook in notebooks/baselines/ with:
  - Markdown title cell (from docstring: title, questions, method, references)
  - Setup cell (imports, path wiring, benchmark JSON loading)
  - Science cells (functions + validation, split by section separators)
  - Visualization cell (matplotlib charts of key results)
  - Summary cell (provenance, links)

Usage:
    python scripts/generate_baseline_notebooks.py
"""

from __future__ import annotations

import json
import re
import textwrap
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CONTROL = REPO / "control"
OUT = REPO / "notebooks" / "baselines"

PALETTE = {"pass": "#2ecc71", "fail": "#e74c3c", "info": "#3498db", "warn": "#f39c12"}

BASELINES = [
    (1, "sensor_noise", "sensor_noise_decomposition.py", "Sensor Noise Characterization", "measurement", "Dong, Miller, Kelley (2020) Agriculture 10(12), 598"),
    (2, "observation_gap", "observation_gap.py", "Observation Gap Analysis", "measurement", "ERA5/NOAA reanalysis comparison"),
    (3, "error_propagation", "error_propagation_fao56.py", "Error Propagation FAO-56", "hydrology", "Allen et al. (1998) FAO Irrigation and Drainage Paper 56"),
    (4, "sequencing_noise", "sequencing_noise.py", "Sequencing Noise Characterization", "genomics", "Synthetic community benchmarks"),
    (5, "seismic", "seismic_inversion.py", "Seismic Wave Propagation", "geophysics", "New Madrid Seismic Zone synthetic model"),
    (6, "signal_specificity", "signal_specificity.py", "Signal Specificity in Quorum Sensing", "biochemistry", "Massie et al. (2012) PNAS, Gillespie SSA"),
    (7, "rawr_resampling", "rawr_resampling.py", "RAWR Bootstrap Resampling", "statistics", "Liu et al. — weighted bootstrap"),
    (8, "anderson_localization", "anderson_localization.py", "Anderson Localization", "condensed_matter", "Anderson (1958) Phys Rev 109:1492; Bourgain & Kachkovskiy (2018)"),
    (9, "quasiperiodic", "quasiperiodic_localization.py", "Almost-Mathieu Quasiperiodic Localization", "condensed_matter", "Aubry-André model; Jitomirskaya (1999)"),
    (10, "bistable_switching", "bistable_switching.py", "Bistable Phenotypic Switching", "biochemistry", "Fernandez, Waters et al. (2020) PNAS 117:26058"),
    (11, "multisignal_qs", "multisignal_qs.py", "Multi-Signal Quorum Sensing Integration", "biochemistry", "Hammer & Bassler (2007) Mol Microbiol 64:547"),
    (12, "spin_transport", "spin_chain_transport.py", "Spin Chain Transport", "condensed_matter", "Anderson localization in 1D spin chains"),
    (13, "resampling_convergence", "resampling_convergence.py", "Resampling Convergence Analysis", "statistics", "CLT convergence for bootstrap resampling"),
    (14, "drift_selection", "drift_selection.py", "Drift vs Selection in Microbial Populations", "population_genetics", "Wright-Fisher + Moran models"),
    (15, "uncertainty_bridge", "uncertainty_bridge.py", "Uncertainty Bridge: Sensor Noise → Localization", "cross_domain", "Cross-domain error propagation"),
    (16, "rare_biosphere", "rare_biosphere.py", "Rare Biosphere Signal Detection", "genomics", "R. Anderson — deep subsurface microbiology"),
    (17, "quasispecies_threshold", "quasispecies_threshold.py", "Quasispecies Error Threshold", "evolutionary_biology", "Eigen (1971) error catastrophe; Dolson"),
    (18, "band_edge", "band_edge.py", "Band Edge Structure", "condensed_matter", "Kachkovskiy — spectral gap detection"),
    (19, "jackknife_estimation", "jackknife_estimation.py", "Jackknife Error Estimation", "statistics", "Bazavov — QCD systematic error estimation"),
    (20, "freeze_out_inverse", "freeze_out_inverse.py", "Freeze-Out Inverse Problem", "lattice_qcd", "Bazavov — QCD freeze-out temperature"),
    (21, "spectral_recon", "spectral_recon.py", "Spectral Reconstruction", "lattice_qcd", "Bazavov — Tikhonov regularization of lattice QCD correlators"),
    (22, "et0_anderson_propagation", "et0_anderson_propagation.py", "ET₀-Anderson Error Propagation", "hydrology", "FAO-56 ET₀ error → localization length"),
    (23, "notill_sampling", "notill_sampling.py", "No-Till vs Tilled 16S Sampling", "soil_science", "Cross-spring: wetSpring 16S pipeline"),
    (24, "aggregate_stability", "aggregate_stability.py", "Aggregate Stability Noise Analysis", "soil_science", "Cross-spring: airSpring soil structure"),
    (25, "precision_drift", "precision_drift.py", "f32 vs f64 Precision Drift", "numerical_methods", "WDM float precision analysis"),
    (26, "size_convergence", "size_convergence.py", "System-Size Convergence", "numerical_methods", "WDM system-size convergence analysis"),
    (27, "vendor_parity", "vendor_parity.py", "GPU Vendor Parity", "gpu_validation", "WDM cross-vendor GPU comparison"),
    (28, "npu_anderson", "npu_anderson.py", "NPU Anderson Classification", "neuromorphic", "BrainChip AKD1000 Anderson classification"),
    (29, "et0_methods", "et0_methods.py", "Multi-Method ET₀ Comparison", "hydrology", "PM, Hargreaves, Makkink, Turc, Hamon"),
]

FACULTY_MAP = {
    1: "Dong et al.", 2: None, 3: "Allen et al. (FAO)", 4: None, 5: None,
    6: "Waters Lab (MSU)", 7: "Liu Lab (MSU)", 8: "Kachkovskiy (MSU)",
    9: "Kachkovskiy (MSU)", 10: "Waters Lab (MSU)", 11: "Waters Lab (MSU)",
    12: "Kachkovskiy / Gonzales", 13: None, 14: "R. Anderson (Carleton)",
    15: "Cross-domain bridge", 16: "R. Anderson (Carleton)", 17: "Dolson (MSU)",
    18: "Kachkovskiy (MSU)", 19: "Bazavov (MSU)", 20: "Bazavov (MSU)",
    21: "Bazavov (MSU)", 22: "Cross-domain (airSpring)", 23: "Cross-spring (wetSpring)",
    24: "Cross-spring (airSpring)", 25: "WDM", 26: "WDM", 27: "WDM",
    28: "metalForge (NPU)", 29: "Cross-spring (airSpring)",
}


def parse_docstring(source: str) -> tuple[str, str]:
    """Extract the module docstring and return (title_line, body)."""
    match = re.search(r'"""(.*?)"""', source, re.DOTALL)
    if not match:
        return ("Untitled", "")
    doc = match.group(1).strip()
    lines = doc.split("\n")
    title = lines[0] if lines else "Untitled"
    body = "\n".join(lines[1:]).strip() if len(lines) > 1 else ""
    return (title, body)


def extract_sections(source: str) -> list[tuple[str, str]]:
    """Split source into (header, code) pairs by # ---- separators."""
    lines = source.split("\n")
    sections: list[tuple[str, str]] = []
    current_header = "Setup"
    current_lines: list[str] = []

    for line in lines:
        if re.match(r"^# -{40,}", line):
            if current_lines:
                sections.append((current_header, "\n".join(current_lines)))
                current_lines = []
        elif re.match(r"^# \w", line) and not line.startswith("# SPDX") and not line.startswith("# Copyright"):
            header = line.lstrip("# ").strip()
            if header and len(header) > 5:
                current_header = header
        else:
            current_lines.append(line)

    if current_lines:
        sections.append((current_header, "\n".join(current_lines)))

    return sections


def extract_main_parts(source: str) -> list[tuple[str, str]]:
    """Extract Part/Step/numbered sections from main() function."""
    parts = []
    current_name = "Initialization"
    current_lines: list[str] = []

    in_main = False
    for line in source.split("\n"):
        if "def main()" in line:
            in_main = True
            continue
        if not in_main:
            continue
        if line and not line[0].isspace() and line[0] != "#" and in_main and "if __name__" not in line:
            break

        # Match: "Part N:", "Step N:", "--- Part N:", "--- Step N:"
        part_match = re.search(r'(?:Part|Step)\s+\d+[:\s]+(.+?)(?:\s*---|\s*$)', line)
        if part_match:
            if current_lines:
                parts.append((current_name, "\n".join(current_lines)))
                current_lines = []
            current_name = part_match.group(1).strip().rstrip('"\')')
            continue

        # Match numbered sections: print("1. Title") or print("\n2. Title")
        num_match = re.search(r'print\(["\']\\?n?(\d+)\.\s+(.+?)["\']', line)
        if num_match and not re.search(r'print\(f["\']', line):
            if current_lines:
                parts.append((current_name, "\n".join(current_lines)))
                current_lines = []
            current_name = num_match.group(2).strip().rstrip('"\')')
            continue

        # Match: "--- Validation Checks ---" or similar standalone section
        val_match = re.search(r'---\s+(.+?)\s+---', line)
        if val_match and "Part" not in line and "Step" not in line:
            name = val_match.group(1).strip()
            if len(name) > 3:
                if current_lines:
                    parts.append((current_name, "\n".join(current_lines)))
                    current_lines = []
                current_name = name
                continue

        # Match inline comment section markers: # --- Some Section ---
        inline_match = re.match(r'\s+# -+\s*$', line)
        if inline_match:
            continue
        inline_header = re.match(r'\s+# (.{5,})', line)
        if inline_header and not line.strip().startswith("# Tol") and not line.strip().startswith("# noqa"):
            header_text = inline_header.group(1).strip()
            if re.match(r'^[A-Z]', header_text) and len(header_text) < 60 and ":" not in header_text:
                if current_lines and len(current_lines) > 3:
                    parts.append((current_name, "\n".join(current_lines)))
                    current_lines = []
                    current_name = header_text
                    continue

        key_match = re.search(r'KEY FINDINGS', line)
        if key_match:
            if current_lines:
                parts.append((current_name, "\n".join(current_lines)))
                current_lines = []
            current_name = "Key Findings"
            continue

        current_lines.append(line)

    if current_lines:
        parts.append((current_name, "\n".join(current_lines)))

    return parts


def make_md_cell(source: str) -> dict:
    return {
        "cell_type": "markdown",
        "metadata": {},
        "source": [line + "\n" for line in source.split("\n")]
    }


def make_code_cell(source: str) -> dict:
    return {
        "cell_type": "code",
        "execution_count": None,
        "metadata": {},
        "outputs": [],
        "source": [line + "\n" for line in source.split("\n")]
    }


def strip_main_wrapper(code: str) -> str:
    """Remove the def main() wrapper and un-indent the body."""
    lines = code.split("\n")
    result = []
    in_main = False
    for line in lines:
        if "def main()" in line:
            in_main = True
            continue
        if in_main and line.startswith("    "):
            result.append(line[4:])
        elif in_main and line.strip() == "":
            result.append("")
        elif "if __name__" in line:
            break
        elif not in_main:
            pass
    return "\n".join(result)


def build_notebook(exp_id: int, dir_name: str, script_name: str,
                   title: str, domain: str, reference: str) -> dict:
    """Build a complete notebook dict for one experiment."""
    script_path = CONTROL / dir_name / script_name
    source = script_path.read_text()

    doc_title, doc_body = parse_docstring(source)
    faculty = FACULTY_MAP.get(exp_id)

    # --- Cell 0: Title ---
    title_md = f"# Experiment {exp_id:03d} — {title}\n\n"
    if doc_body:
        for line in doc_body.split("\n"):
            line = line.strip()
            if line.startswith("Key questions:") or line.startswith("Method:") or line.startswith("Reference:") or line.startswith("Cross-spring"):
                title_md += f"\n**{line}**\n"
            elif line.startswith("  "):
                title_md += f"{line}\n"
            elif line:
                title_md += f"{line}\n"
    title_md += f"\n**Domain**: {domain.replace('_', ' ').title()}\n"
    if faculty:
        title_md += f"**Faculty**: {faculty}\n"
    title_md += f"**Reference**: {reference}\n"
    title_md += f"\n**Data source**: `control/{dir_name}/{script_name}` + `benchmark_*.json`\n"
    title_md += f"\n---\n\n*This notebook is the publication-grade Python baseline for Experiment {exp_id:03d}. "
    title_md += "The identical computations are validated in Rust (see `validate_*` binary) "
    title_md += "and delegated to barraCuda for GPU acceleration.*"

    cells = [make_md_cell(title_md)]

    # --- Cell 1: Setup & imports ---
    setup_imports = source.split("def ")[0] if "def " in source else source[:500]
    setup_lines = []
    for line in setup_imports.split("\n"):
        if line.startswith("#!") or line.startswith("# SPDX") or line.startswith("# Copyright"):
            continue
        if line.startswith('"""') or (line.strip().startswith('"') and 'groundSpring' in line):
            continue
        if '"""' in line:
            continue
        setup_lines.append(line)

    # Clean up: remove leading empty lines and docstring body
    cleaned = "\n".join(setup_lines)
    cleaned = re.sub(r'""".*?"""', '', cleaned, flags=re.DOTALL)
    cleaned_lines = cleaned.split("\n")
    while cleaned_lines and not cleaned_lines[0].strip():
        cleaned_lines.pop(0)

    import_code = "import json\n"
    import_code += "import math\n"
    import_code += "import sys\n"
    import_code += "from pathlib import Path\n\n"
    import_code += "import numpy as np\n"

    if "from scipy" in source:
        scipy_imports = [l.strip() for l in source.split("\n") if l.strip().startswith("from scipy")]
        for si in scipy_imports:
            import_code += f"{si}\n"

    import_code += "import matplotlib\n"
    import_code += "matplotlib.use('Agg')\n"
    import_code += "import matplotlib.pyplot as plt\n\n"

    import_code += f"# Wire path to groundSpring control/ for common utilities\n"
    import_code += f"CONTROL = Path('..') / '..' / 'control'\n"
    import_code += f"sys.path.insert(0, str(CONTROL))\n"
    import_code += f"from common import *  # noqa: F403 — validation harness\n\n"
    import_code += f"# Load benchmark data\n"

    # Find the benchmark JSON name
    bench_match = re.search(r'"(benchmark_\w+\.json)"', source)
    if not bench_match:
        bench_match = re.search(r'/ "(benchmark[^"]+)"', source)
    bench_name = bench_match.group(1) if bench_match else f"benchmark_{dir_name}.json"

    import_code += f"benchmark_path = CONTROL / '{dir_name}' / '{bench_name}'\n"
    import_code += f"with open(benchmark_path) as f:\n"
    import_code += f"    benchmark = json.load(f)\n\n"
    import_code += f"PASS_COLOR = '{PALETTE['pass']}'\n"
    import_code += f"FAIL_COLOR = '{PALETTE['fail']}'\n"
    import_code += f"INFO_COLOR = '{PALETTE['info']}'\n"
    import_code += f"\n"
    import_code += f"print(f'Loaded benchmark: {bench_name}')\n"
    import_code += f"print(f'Provenance: {{benchmark.get(\"_provenance\", {{}})}}')"

    cells.append(make_code_cell(import_code))

    # --- Cells 2+: Function definitions ---
    func_blocks = re.split(r'\n# -{50,}\n', source)
    functions = []
    for block in func_blocks:
        block = block.strip()
        if not block or block.startswith("#!") or block.startswith('"""'):
            continue
        if "def " in block and "def main" not in block:
            header_match = re.search(r'^# (.+)$', block, re.MULTILINE)
            header = header_match.group(1).strip() if header_match else "Model Functions"
            func_lines = []
            for line in block.split("\n"):
                if line.startswith("# -"):
                    continue
                func_lines.append(line)
            code = "\n".join(func_lines).strip()
            if code and "def " in code:
                functions.append((header, code))

    if functions:
        cells.append(make_md_cell("## Model Implementation"))
        for header, code in functions:
            if header != "Model Functions":
                cells.append(make_md_cell(f"### {header}"))
            cells.append(make_code_cell(code))

    # --- Main validation cells ---
    cells.append(make_md_cell("## Validation"))

    main_body = strip_main_wrapper(source)
    parts = extract_main_parts(source)

    if parts:
        for part_name, part_code in parts:
            part_code = part_code.strip()
            if not part_code:
                continue
            lines = part_code.split("\n")
            code_lines = []
            for line in lines:
                if "return print_summary" in line:
                    code_lines.append(f"# Results: {{pass_count()}}/{{total_count()}} checks passed")
                    code_lines.append("print_summary" + line.split("print_summary")[1])
                    continue
                code_lines.append(line)

            cleaned_code = "\n".join(code_lines).strip()
            if cleaned_code:
                cells.append(make_md_cell(f"### {part_name}"))
                cells.append(make_code_cell(textwrap.dedent(cleaned_code)))
    else:
        cells.append(make_code_cell("reset_counters()\n\n" + main_body))

    # --- Visualization cell ---
    cells.append(make_md_cell("## Visualization"))

    viz_code = f"# Publication-grade summary chart for Exp {exp_id:03d}\n"
    viz_code += f"fig, ax = plt.subplots(figsize=(8, 4))\n\n"
    viz_code += f"p, f_count, t = pass_count(), fail_count(), total_count()\n"
    viz_code += f"ax.barh(['Pass', 'Fail'], [p, f_count], color=[PASS_COLOR, FAIL_COLOR])\n"
    viz_code += f"ax.set_xlim(0, max(t * 1.15, 1))\n"
    viz_code += f"ax.set_title('Exp {exp_id:03d}: {title} — Validation Results')\n"
    viz_code += f"ax.set_xlabel('Check Count')\n"
    viz_code += f"for i, v in enumerate([p, f_count]):\n"
    viz_code += f"    if v > 0:\n"
    viz_code += f"        ax.text(v + 0.3, i, str(v), va='center', fontweight='bold')\n\n"
    viz_code += f"plt.tight_layout()\n"
    viz_code += f"plt.savefig(f'/tmp/groundspring_exp{exp_id:03d}.png', dpi=150, bbox_inches='tight')\n"
    viz_code += f"plt.show()\n"
    viz_code += f"print(f'\\nResult: {{p}}/{{t}} PASS, {{f_count}}/{{t}} FAIL')"

    cells.append(make_code_cell(viz_code))

    # --- Summary cell ---
    summary_md = f"## Provenance & Summary\n\n"
    summary_md += f"| Field | Value |\n|-------|-------|\n"
    summary_md += f"| Experiment | {exp_id:03d} — {title} |\n"
    summary_md += f"| Domain | {domain.replace('_', ' ').title()} |\n"
    summary_md += f"| Reference | {reference} |\n"
    if faculty:
        summary_md += f"| Faculty | {faculty} |\n"
    summary_md += f"| Python baseline | `control/{dir_name}/{script_name}` |\n"
    summary_md += f"| Benchmark JSON | `control/{dir_name}/{bench_name}` |\n"
    summary_md += f"| Rust validator | `validate_*` binary (exit-code protocol) |\n"
    summary_md += f"| Rust speedup | See benchmark comparison notebook |\n"
    summary_md += f"| License | AGPL-3.0-or-later |\n\n"
    summary_md += f"**Provenance chain**: Python baseline → Rust validation → barraCuda GPU → "
    summary_md += f"metalForge cross-substrate → primal IPC composition\n\n"
    summary_md += f"See [primals.eco](https://primals.eco) for rendered lab notebooks."

    cells.append(make_md_cell(summary_md))

    return {
        "cells": cells,
        "metadata": {
            "kernelspec": {
                "display_name": "Python 3",
                "language": "python",
                "name": "python3"
            },
            "language_info": {
                "name": "python",
                "version": "3.12.0"
            }
        },
        "nbformat": 4,
        "nbformat_minor": 4,
    }


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    generated = []

    for exp_id, dir_name, script_name, title, domain, reference in BASELINES:
        script_path = CONTROL / dir_name / script_name
        if not script_path.exists():
            print(f"  SKIP exp {exp_id:03d}: {script_path} not found")
            continue

        nb = build_notebook(exp_id, dir_name, script_name, title, domain, reference)
        out_path = OUT / f"exp-{exp_id:03d}-{dir_name.replace('_', '-')}.ipynb"
        with open(out_path, "w") as f:
            json.dump(nb, f, indent=1)
        cells = nb["cells"]
        md = sum(1 for c in cells if c["cell_type"] == "markdown")
        code = sum(1 for c in cells if c["cell_type"] == "code")
        print(f"  OK  exp-{exp_id:03d}-{dir_name}: {len(cells)} cells ({md} md + {code} code)")
        generated.append((exp_id, dir_name, title, domain, len(cells)))

    print(f"\nGenerated {len(generated)} notebooks in {OUT}")

    # Write index
    index_md = "# groundSpring Baseline Notebooks\n\n"
    index_md += "Publication-grade Python baselines for all 29 groundSpring experiments.\n"
    index_md += "Each notebook is executable, self-contained, and produces charts for the website.\n\n"
    index_md += "| # | Notebook | Domain | Cells |\n"
    index_md += "|---|----------|--------|-------|\n"
    for exp_id, dir_name, title, domain, n_cells in generated:
        fname = f"exp-{exp_id:03d}-{dir_name.replace('_', '-')}.ipynb"
        index_md += f"| {exp_id:03d} | [{title}]({fname}) | {domain.replace('_', ' ').title()} | {n_cells} |\n"
    index_md += f"\n## Conventions\n\n"
    index_md += "- All notebooks load frozen benchmark data from `control/<experiment>/benchmark_*.json`\n"
    index_md += "- Charts use ecosystem palette: `#2ecc71` (pass), `#e74c3c` (fail), `#3498db` (info)\n"
    index_md += "- Each notebook ends with a provenance summary\n"
    index_md += "- Notebooks are executable in CI via `jupyter nbconvert --execute`\n"

    with open(OUT / "README.md", "w") as f:
        f.write(index_md)
    print(f"Wrote {OUT / 'README.md'}")


if __name__ == "__main__":
    main()
