# Public Notebook Pattern — groundSpring

How to create public-facing notebooks for your spring. Adapted from
the wetSpring exemplar and primalSpring implementation.

## Directory Convention

```
your-spring/
  notebooks/
    NOTEBOOK_PATTERN.md          ← this file (copy to your spring)
    01-domain-validation.ipynb   ← flagship validation story
    02-benchmark-comparison.ipynb← Python vs Rust vs GPU
    03-paper-reproductions.ipynb ← per-researcher evidence
    04-cross-spring.ipynb        ← ecosystem connections
    05-domain-deep-dive.ipynb    ← your most compelling discovery
```

## Cell Structure

Every notebook follows the same structure:

1. **Title cell** (markdown): Title, one-paragraph context, data sources, "for other springs" adaptation note
2. **Imports + data loading** (code): Load from `../experiments/results/*.json`
3. **Domain-specific cells** (code + markdown): Visualization and analysis
4. **Summary cell** (markdown): Validation table, provenance note, links to primals.eco

## Data Loading Pattern

```python
import json
from pathlib import Path

RESULTS = Path('..') / 'experiments' / 'results'

def load(path):
    with open(RESULTS / path) as f:
        return json.load(f)

data = load('composition_validation.json')
```

Notebooks load **frozen data** (committed JSON artifacts), not live API responses.
This means they work without primals running.

## Frozen Data for groundSpring

| File | Contents |
|------|----------|
| `composition_validation.json` | Deploy graphs, capabilities, guideStone status, verb reconciliation |
| `test_suite_report.json` | Module-level test counts, timings, quality metrics |
| `experiment_catalog.json` | All 35 experiments across 10 domains with speedups |
| `security_gaps.json` | Gap registry, security posture, tolerance system stats |
| `cross_spring_matrix.json` | Primal consumption, ecosystem flows, patterns pioneered |
| `benchmark_timing.json` | Rust vs Python, three-mode benchmarks, delegation inventory |

## Visualization Standards

- Use `matplotlib` (available everywhere, renders to static PNG)
- Use `matplotlib.use('Agg')` for headless rendering
- Save figures to `/tmp/groundspring_<notebook>_<chart>.png`
- Color palette: `#2ecc71` (pass/ok), `#e74c3c` (fail), `#3498db` (info)
- Always include chart titles with key numbers

## Adapting for Your Spring

1. Copy this directory structure
2. Replace data paths with your `experiments/results/` JSONs
3. Update the narrative for your domain
4. Keep the cell structure (title → load → analyze → summary)
5. Add your spring to `shared/abg/commons/<spring>-public/notebooks/` symlink
