# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""Determinism tests: verify that seeded stochastic operations are rerun-identical."""

from __future__ import annotations

import sys
from pathlib import Path

import numpy as np
import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "control"))


class TestSensorNoiseDeterminism:
    def test_noise_samples_identical_across_runs(self) -> None:
        from sensor_noise.sensor_noise_decomposition import generate_sensor_noise_samples

        run1 = generate_sensor_noise_samples(mbe=-0.01, random_std=0.014, rng_seed=42)
        run2 = generate_sensor_noise_samples(mbe=-0.01, random_std=0.014, rng_seed=42)
        np.testing.assert_array_equal(run1, run2)

    def test_different_seeds_differ(self) -> None:
        from sensor_noise.sensor_noise_decomposition import generate_sensor_noise_samples

        run1 = generate_sensor_noise_samples(mbe=-0.01, random_std=0.014, rng_seed=42)
        run2 = generate_sensor_noise_samples(mbe=-0.01, random_std=0.014, rng_seed=99)
        assert not np.array_equal(run1, run2)


class TestSequencingDeterminism:
    def test_rarefaction_identical_across_runs(self) -> None:
        from sequencing_noise.sequencing_noise import (
            generate_reference_community,
            rarefaction_at_depth,
        )

        community_config = {
            "dominant_phyla": [
                {"name": "TestPhylum", "n_genera": 10, "relative_abundance": 1.0}
            ]
        }
        comm = generate_reference_community(community_config, seed=42)

        r1 = rarefaction_at_depth(comm, depth=1000, n_replicates=10, seed=42)
        r2 = rarefaction_at_depth(comm, depth=1000, n_replicates=10, seed=42)

        assert r1["genera_detected"]["mean"] == r2["genera_detected"]["mean"]
        assert r1["shannon"]["mean"] == r2["shannon"]["mean"]


class TestShannonDeterminism:
    def test_known_value(self) -> None:
        """Shannon of a uniform distribution of S species = ln(S)."""
        from sequencing_noise.sequencing_noise import compute_shannon

        counts = np.array([100, 100, 100, 100])
        expected = np.log(4)
        assert compute_shannon(counts) == pytest.approx(expected, abs=1e-10)

    def test_single_species(self) -> None:
        from sequencing_noise.sequencing_noise import compute_shannon

        counts = np.array([1000, 0, 0, 0])
        assert compute_shannon(counts) == pytest.approx(0.0, abs=1e-10)


class TestSeismicDeterminism:
    def test_haversine_known_value(self) -> None:
        """New York to London ≈ 5570 km."""
        from seismic.seismic_inversion import haversine_km

        d = haversine_km(40.7128, -74.0060, 51.5074, -0.1278)
        assert d == pytest.approx(5570, abs=50)

    def test_haversine_zero_distance(self) -> None:
        from seismic.seismic_inversion import haversine_km

        assert haversine_km(37.5, -89.0, 37.5, -89.0) == pytest.approx(0.0)

    def test_travel_time_proportional_to_distance(self) -> None:
        from seismic.seismic_inversion import travel_time_1d

        t1 = travel_time_1d(100, 10, 6.0)
        t2 = travel_time_1d(200, 10, 6.0)
        assert t2 > t1
