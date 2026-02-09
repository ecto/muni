"""Tests for the extract_demos module.

Tests the core data processing functions without requiring .rrd files or the
rerun SDK — we test the pure math and normalization logic.
"""

import math

import numpy as np
import pytest

# We import from the parent package.  Add parent to path so we can import
# the module directly even without `pip install -e`.
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from extract_demos import (
    MAX_ANGULAR_VEL,
    MAX_LINEAR_VEL,
    MAX_POSITION,
    transform_to_robot_frame,
)


class TestTransformToRobotFrame:
    """Tests for the world→robot frame coordinate transform."""

    def test_identity_heading_zero(self):
        """With heading=0, robot frame == world frame."""
        rx, ry = transform_to_robot_frame(1.0, 0.0, 0.0)
        assert pytest.approx(rx, abs=1e-9) == 1.0
        assert pytest.approx(ry, abs=1e-9) == 0.0

    def test_heading_zero_lateral(self):
        """With heading=0, world +y maps to robot +y (left)."""
        rx, ry = transform_to_robot_frame(0.0, 1.0, 0.0)
        assert pytest.approx(rx, abs=1e-9) == 0.0
        assert pytest.approx(ry, abs=1e-9) == 1.0

    def test_heading_pi_half(self):
        """With heading=pi/2 (facing north), world +x maps to robot +y."""
        rx, ry = transform_to_robot_frame(1.0, 0.0, math.pi / 2)
        assert pytest.approx(rx, abs=1e-6) == 0.0
        assert pytest.approx(ry, abs=1e-6) == -1.0

    def test_heading_pi(self):
        """With heading=pi (facing west), world +x maps to robot -x."""
        rx, ry = transform_to_robot_frame(1.0, 0.0, math.pi)
        assert pytest.approx(rx, abs=1e-6) == -1.0
        assert pytest.approx(ry, abs=1e-6) == 0.0

    def test_zero_displacement(self):
        """Zero displacement stays zero regardless of heading."""
        rx, ry = transform_to_robot_frame(0.0, 0.0, 1.234)
        assert rx == 0.0
        assert ry == 0.0

    def test_diagonal_heading_zero(self):
        """Diagonal displacement with heading=0."""
        rx, ry = transform_to_robot_frame(3.0, 4.0, 0.0)
        assert pytest.approx(rx, abs=1e-9) == 3.0
        assert pytest.approx(ry, abs=1e-9) == 4.0

    def test_roundtrip_magnitude(self):
        """Transform preserves vector magnitude (it's a rotation)."""
        dx, dy = 3.0, 4.0
        for theta in [0.0, 0.5, 1.0, 2.0, -1.0, math.pi]:
            rx, ry = transform_to_robot_frame(dx, dy, theta)
            original_mag = math.sqrt(dx**2 + dy**2)
            result_mag = math.sqrt(rx**2 + ry**2)
            assert pytest.approx(result_mag, abs=1e-9) == original_mag


class TestNormalizationConstants:
    """Verify normalization constants match expected values.

    These must stay in sync with the Rust policy crate's
    NormalizationConfig::default().
    """

    def test_max_position(self):
        assert MAX_POSITION == 50.0

    def test_max_linear_vel(self):
        assert MAX_LINEAR_VEL == 2.0

    def test_max_angular_vel(self):
        assert MAX_ANGULAR_VEL == 2.0

    def test_normalization_clips_to_unit(self):
        """Values at the max should normalize to exactly 1.0."""
        assert np.clip(MAX_POSITION / MAX_POSITION, -1, 1) == 1.0
        assert np.clip(MAX_LINEAR_VEL / MAX_LINEAR_VEL, -1, 1) == 1.0
        assert np.clip(MAX_ANGULAR_VEL / MAX_ANGULAR_VEL, -1, 1) == 1.0
