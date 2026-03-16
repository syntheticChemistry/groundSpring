# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""Unit tests for control/common.py — analytical known-value tests."""

from __future__ import annotations

import math
import sys
from pathlib import Path

import numpy as np
import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "control"))
from common import (
    bias_variance_decompose,
    compute_ia,
    compute_mbe,
    compute_r2,
    compute_rmse,
    decompose_error,
    noise_floor_reduction,
)


class TestDecomposeError:
    """Analytical known-value tests for bias-variance decomposition."""

    def test_pure_bias_no_noise(self) -> None:
        """If RMSE == |MBE|, all error is bias and random_std == 0."""
        result = decompose_error(mbe=0.05, rmse=0.05)
        assert result["random_std"] == pytest.approx(0.0, abs=1e-12)
        assert result["bias_fraction"] == pytest.approx(1.0, abs=1e-12)
        assert result["noise_fraction"] == pytest.approx(0.0, abs=1e-12)

    def test_pure_noise_no_bias(self) -> None:
        """If MBE == 0, all error is noise and bias_fraction == 0."""
        result = decompose_error(mbe=0.0, rmse=0.03)
        assert result["random_std"] == pytest.approx(0.03, abs=1e-12)
        assert result["bias_fraction"] == pytest.approx(0.0, abs=1e-12)
        assert result["noise_fraction"] == pytest.approx(1.0, abs=1e-12)

    def test_pythagorean_identity(self) -> None:
        """RMSE² = MBE² + random_std² must hold for any decomposition."""
        for mbe, rmse in [(0.01, 0.017), (-0.03, 0.039), (0.03, 0.038)]:
            result = decompose_error(mbe, rmse)
            reconstructed = math.sqrt(result["bias_sq"] + result["variance"])
            assert reconstructed == pytest.approx(rmse, abs=1e-10)

    def test_bias_fraction_bounds(self) -> None:
        """Bias fraction must be in [0, 1]."""
        for mbe, rmse in [(0.0, 0.1), (0.05, 0.05), (-0.02, 0.039)]:
            result = decompose_error(mbe, rmse)
            assert 0.0 <= result["bias_fraction"] <= 1.0

    def test_zero_rmse(self) -> None:
        """Zero RMSE should not divide by zero."""
        result = decompose_error(mbe=0.0, rmse=0.0)
        assert result["bias_fraction"] == 0.0
        assert result["random_std"] == 0.0

    def test_dong2020_cs616_sand(self) -> None:
        """Reproduce Dong et al. (2020) CS616 sand decomposition."""
        result = decompose_error(mbe=-0.01, rmse=0.017)
        assert result["random_std"] == pytest.approx(0.0137, abs=0.001)
        assert result["bias_fraction"] == pytest.approx(0.346, abs=0.005)


class TestNoiseFloorReduction:
    def test_improvement_expected(self) -> None:
        result = noise_floor_reduction(factory_rmse=0.039, corrected_rmse=0.012)
        assert result["removed_error"] > 0
        assert result["reduction_pct"] > 0
        assert result["noise_floor"] == 0.012

    def test_no_improvement(self) -> None:
        result = noise_floor_reduction(factory_rmse=0.01, corrected_rmse=0.01)
        assert result["reduction_pct"] == pytest.approx(0.0)

    def test_pythagorean_holds(self) -> None:
        result = noise_floor_reduction(factory_rmse=0.039, corrected_rmse=0.012)
        reconstructed = math.sqrt(result["removed_error"] ** 2 + result["noise_floor"] ** 2)
        assert reconstructed == pytest.approx(0.039, abs=1e-10)


class TestStatisticalMetrics:
    """Known-value tests for statistical functions."""

    def test_rmse_zero_for_identical(self) -> None:
        x = np.array([1.0, 2.0, 3.0])
        assert compute_rmse(x, x) == pytest.approx(0.0)

    def test_rmse_known_value(self) -> None:
        obs = np.array([1.0, 2.0, 3.0])
        mod = np.array([1.1, 2.1, 3.1])
        assert compute_rmse(obs, mod) == pytest.approx(0.1, abs=1e-10)

    def test_mbe_positive_for_overestimate(self) -> None:
        obs = np.array([1.0, 2.0, 3.0])
        mod = np.array([1.5, 2.5, 3.5])
        assert compute_mbe(obs, mod) == pytest.approx(0.5)

    def test_mbe_negative_for_underestimate(self) -> None:
        obs = np.array([1.0, 2.0, 3.0])
        mod = np.array([0.5, 1.5, 2.5])
        assert compute_mbe(obs, mod) == pytest.approx(-0.5)

    def test_r2_perfect(self) -> None:
        obs = np.array([1.0, 2.0, 3.0])
        assert compute_r2(obs, obs) == pytest.approx(1.0)

    def test_r2_zero_for_mean_model(self) -> None:
        obs = np.array([1.0, 2.0, 3.0])
        mod = np.full(3, np.mean(obs))
        assert compute_r2(obs, mod) == pytest.approx(0.0)

    def test_ia_perfect(self) -> None:
        obs = np.array([1.0, 2.0, 3.0, 4.0])
        assert compute_ia(obs, obs) == pytest.approx(1.0)

    def test_ia_constant_observation(self) -> None:
        obs = np.array([5.0, 5.0, 5.0])
        mod = np.array([5.0, 5.0, 5.0])
        assert compute_ia(obs, mod) == pytest.approx(0.0)


class TestBiasVarianceDecompose:
    def test_consistency_with_decompose_error(self) -> None:
        obs = np.array([1.0, 2.0, 3.0, 4.0, 5.0])
        mod = np.array([1.1, 2.2, 3.0, 3.9, 5.1])
        bv = bias_variance_decompose(obs, mod)

        mbe = compute_mbe(obs, mod)
        rmse = compute_rmse(obs, mod)
        de = decompose_error(mbe, rmse)

        assert bv["bias_fraction"] == pytest.approx(de["bias_fraction"])
        assert bv["random_std"] == pytest.approx(de["random_std"])
