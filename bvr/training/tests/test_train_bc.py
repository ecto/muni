"""Tests for the train_bc module.

Tests the BCPolicy network architecture, export format, and training loop
using synthetic data (no real demonstrations needed).
"""

import json
import tempfile
from pathlib import Path

import numpy as np
import pytest
import torch

import sys

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from train_bc import BCPolicy, train


class TestBCPolicy:
    """Tests for the BCPolicy neural network."""

    def test_default_architecture(self):
        model = BCPolicy()
        assert model.param_count() == 4802  # 7*64+64 + 64*64+64 + 64*2+2

    def test_forward_shape(self):
        model = BCPolicy()
        x = torch.randn(16, 7)
        y = model(x)
        assert y.shape == (16, 2)

    def test_output_bounded(self):
        """All outputs should be in [-1, 1] due to final tanh."""
        model = BCPolicy()
        x = torch.randn(100, 7) * 10  # large inputs
        y = model(x)
        assert y.min().item() >= -1.0
        assert y.max().item() <= 1.0

    def test_custom_dimensions(self):
        model = BCPolicy(obs_dim=4, act_dim=3, hidden=32)
        x = torch.randn(8, 4)
        y = model(x)
        assert y.shape == (8, 3)

    def test_export_layers(self):
        model = BCPolicy()
        layers = model.export_layers()
        assert len(layers) == 3

        # Layer 0: 7 → 64
        assert len(layers[0]["weights"]) == 64
        assert len(layers[0]["weights"][0]) == 7
        assert len(layers[0]["biases"]) == 64

        # Layer 1: 64 → 64
        assert len(layers[1]["weights"]) == 64
        assert len(layers[1]["weights"][0]) == 64
        assert len(layers[1]["biases"]) == 64

        # Layer 2: 64 → 2
        assert len(layers[2]["weights"]) == 2
        assert len(layers[2]["weights"][0]) == 64
        assert len(layers[2]["biases"]) == 2

    def test_export_layers_json_serializable(self):
        model = BCPolicy()
        layers = model.export_layers()
        # Should not raise
        json.dumps(layers)

    def test_deterministic_with_seed(self):
        """Same seed produces same weights."""
        torch.manual_seed(42)
        m1 = BCPolicy()
        torch.manual_seed(42)
        m2 = BCPolicy()

        for p1, p2 in zip(m1.parameters(), m2.parameters()):
            assert torch.equal(p1, p2)


class TestTrain:
    """Integration tests for the training loop using synthetic data."""

    @pytest.fixture
    def synthetic_data(self, tmp_path):
        """Create a synthetic demos.npz with random data."""
        n = 500
        obs = np.random.randn(n, 7).astype(np.float32)
        act = np.random.randn(n, 2).astype(np.float32)
        act = np.clip(act, -1, 1)

        data_path = tmp_path / "demos.npz"
        np.savez(data_path, observations=obs, actions=act, sessions_used=3)
        return data_path

    def test_train_produces_json(self, synthetic_data, tmp_path):
        output = tmp_path / "policy.json"
        metrics = train(
            data_path=synthetic_data,
            output_path=output,
            epochs=5,
            batch_size=64,
        )

        assert output.exists()
        assert "mse" in metrics
        assert "r2" in metrics
        assert metrics["epochs"] == 5

    def test_output_json_structure(self, synthetic_data, tmp_path):
        output = tmp_path / "policy.json"
        train(data_path=synthetic_data, output_path=output, epochs=5)

        with open(output) as f:
            policy = json.load(f)

        assert policy["architecture"] == "mlp"
        assert policy["observation_size"] == 7
        assert policy["action_size"] == 2
        assert len(policy["layers"]) == 3
        assert "metrics" in policy

    def test_reproducible_with_seed(self, synthetic_data, tmp_path):
        """Same seed produces identical policy."""
        out1 = tmp_path / "p1.json"
        out2 = tmp_path / "p2.json"

        train(data_path=synthetic_data, output_path=out1, epochs=5, seed=123)
        train(data_path=synthetic_data, output_path=out2, epochs=5, seed=123)

        with open(out1) as f:
            p1 = json.load(f)
        with open(out2) as f:
            p2 = json.load(f)

        assert p1["layers"] == p2["layers"]
