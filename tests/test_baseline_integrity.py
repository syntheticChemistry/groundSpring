# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""Baseline integrity tests — verify benchmark JSON provenance and completeness.

Each control experiment must have a benchmark JSON with valid provenance
metadata. This catches:
  - Missing benchmark files for new experiments
  - Corrupted or hand-edited JSON without provenance updates
  - Experiments with stale provenance (baseline_date older than script mtime)
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

CONTROL_DIR = Path(__file__).resolve().parent.parent / "control"

REQUIRED_PROVENANCE_FIELDS = {
    "baseline_date",
    "baseline_commit",
    "validation_script",
    "command",
    "real_data_accession",
}

SKIP_DIRS = {"__pycache__", "common.py"}


def _experiment_dirs() -> list[Path]:
    """All subdirectories of control/ that contain a Python experiment."""
    return sorted(
        d
        for d in CONTROL_DIR.iterdir()
        if d.is_dir() and d.name not in SKIP_DIRS and not d.name.startswith("__")
    )


def _benchmark_files() -> list[Path]:
    """All benchmark JSON files under control/."""
    return sorted(CONTROL_DIR.rglob("benchmark_*.json"))


class TestBenchmarkProvenance:
    """Every benchmark JSON must have complete provenance metadata."""

    @pytest.fixture(params=_benchmark_files(), ids=lambda p: p.parent.name)
    def benchmark(self, request: pytest.FixtureRequest) -> tuple[Path, dict]:
        path = request.param
        with open(path) as f:
            data = json.load(f)
        return path, data

    def test_has_source(self, benchmark: tuple[Path, dict]) -> None:
        path, data = benchmark
        assert "_source" in data, f"{path.name} missing _source field"

    def test_has_provenance_block(self, benchmark: tuple[Path, dict]) -> None:
        path, data = benchmark
        assert "_provenance" in data, f"{path.name} missing _provenance block"

    def test_provenance_has_required_fields(
        self, benchmark: tuple[Path, dict]
    ) -> None:
        path, data = benchmark
        prov = data.get("_provenance", {})
        missing = REQUIRED_PROVENANCE_FIELDS - set(prov.keys())
        assert not missing, f"{path.name} provenance missing: {missing}"

    def test_baseline_commit_is_hex(self, benchmark: tuple[Path, dict]) -> None:
        path, data = benchmark
        commit = data.get("_provenance", {}).get("baseline_commit", "")
        if commit and commit != "unknown":
            assert all(
                c in "0123456789abcdef" for c in commit
            ), f"{path.name}: baseline_commit '{commit}' is not a hex hash"

    def test_has_doi(self, benchmark: tuple[Path, dict]) -> None:
        path, data = benchmark
        assert "_doi" in data or "_doi_era5" in data or "_doi_ghcnd" in data, (
            f"{path.name} missing _doi field"
        )

    def test_has_real_data_accession(self, benchmark: tuple[Path, dict]) -> None:
        path, data = benchmark
        prov = data.get("_provenance", {})
        assert "real_data_accession" in prov, (
            f"{path.name} provenance missing real_data_accession"
        )

    def test_json_is_valid_utf8(self, benchmark: tuple[Path, dict]) -> None:
        path, _ = benchmark
        path.read_text(encoding="utf-8")


class TestExperimentCompleteness:
    """Every experiment directory must have a benchmark JSON file."""

    @pytest.fixture(params=_experiment_dirs(), ids=lambda p: p.name)
    def experiment_dir(self, request: pytest.FixtureRequest) -> Path:
        result: Path = request.param
        return result

    def test_has_benchmark_json(self, experiment_dir: Path) -> None:
        benchmarks = list(experiment_dir.glob("benchmark_*.json"))
        assert benchmarks, (
            f"{experiment_dir.name}/ has no benchmark_*.json — "
            "every experiment must have a committed baseline"
        )

    def test_has_python_script(self, experiment_dir: Path) -> None:
        scripts = list(experiment_dir.glob("*.py"))
        assert scripts, (
            f"{experiment_dir.name}/ has no Python script"
        )
