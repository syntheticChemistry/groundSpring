# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""Shared fixtures for groundSpring tests."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

CONTROL_DIR = Path(__file__).resolve().parent.parent / "control"
sys.path.insert(0, str(CONTROL_DIR))


@pytest.fixture
def benchmark_dir() -> Path:
    return CONTROL_DIR
