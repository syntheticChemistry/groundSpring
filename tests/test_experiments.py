# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""Integration tests: run each experiment and verify exit code 0."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

CONTROL_DIR = Path(__file__).resolve().parent.parent / "control"


def _run_experiment(script: Path) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, str(script)],
        capture_output=True,
        text=True,
        timeout=120,
    )


class TestExperimentExitCodes:
    """Each experiment must return exit code 0 (all checks pass)."""

    def test_exp001_sensor_noise(self) -> None:
        result = _run_experiment(
            CONTROL_DIR / "sensor_noise" / "sensor_noise_decomposition.py"
        )
        assert result.returncode == 0, result.stdout + result.stderr

    @pytest.mark.skipif(
        not (CONTROL_DIR.parent / "data" / "observation_gap").exists(),
        reason="Requires cached or live data",
    )
    def test_exp002_observation_gap(self) -> None:
        result = _run_experiment(
            CONTROL_DIR / "observation_gap" / "observation_gap.py"
        )
        assert result.returncode == 0, result.stdout + result.stderr

    def test_exp003_error_propagation(self) -> None:
        airspring = CONTROL_DIR.parent.parent / "airSpring" / "control" / "fao56"
        if not (airspring / "penman_monteith.py").exists():
            pytest.skip("airSpring FAO-56 module not found")
        result = _run_experiment(
            CONTROL_DIR / "error_propagation" / "error_propagation_fao56.py"
        )
        assert result.returncode == 0, result.stdout + result.stderr

    def test_exp004_sequencing_noise(self) -> None:
        result = _run_experiment(
            CONTROL_DIR / "sequencing_noise" / "sequencing_noise.py"
        )
        assert result.returncode == 0, result.stdout + result.stderr

    def test_exp005_seismic_inversion(self) -> None:
        result = _run_experiment(
            CONTROL_DIR / "seismic" / "seismic_inversion.py"
        )
        assert result.returncode == 0, result.stdout + result.stderr

    def test_exp006_signal_specificity(self) -> None:
        result = _run_experiment(
            CONTROL_DIR / "signal_specificity" / "signal_specificity.py"
        )
        assert result.returncode == 0, result.stdout + result.stderr

    def test_exp007_rawr_resampling(self) -> None:
        result = _run_experiment(
            CONTROL_DIR / "rawr_resampling" / "rawr_resampling.py"
        )
        assert result.returncode == 0, result.stdout + result.stderr

    def test_exp008_anderson_localization(self) -> None:
        result = _run_experiment(
            CONTROL_DIR / "anderson_localization" / "anderson_localization.py"
        )
        assert result.returncode == 0, result.stdout + result.stderr
