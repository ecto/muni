#!/usr/bin/env python3
"""Train an MLP policy via behavioral cloning on teleop demonstrations.

This is the second step in the behavioral cloning pipeline. It takes the
dataset produced by extract_demos.py and trains a small MLP to imitate the
teleop operator's velocity commands.

Pipeline position:
    extract_demos.py  →  demos.npz  →  [THIS SCRIPT]  →  policy.json  →  rover

Input:
    demos.npz — NumPy archive with:
        observations: (N, 7) float32 — normalized observation vectors
        actions: (N, 2) float32 — normalized velocity commands
        sessions_used: int — number of source sessions

Output:
    JSON policy file directly loadable by the Rust policy crate
    (bvr/firmware/crates/policy). The file contains the MLP weights,
    biases, architecture metadata, and training metrics.

Architecture:
    7 → 64 (tanh) → 64 (tanh) → 2 (tanh)

    - Input: 7-dim observation (pose, velocity, goal — see extract_demos.py)
    - Hidden: two 64-unit layers with tanh activation
    - Output: 2-dim action (linear vel, angular vel), tanh-bounded to [-1, 1]
    - Total parameters: 4802
        Layer 0: 7×64 + 64     = 512
        Layer 1: 64×64 + 64    = 4160
        Layer 2: 64×2 + 2      = 130
                                  ----
                                  4802

    Tanh is used everywhere (not ReLU) because:
    1. The Rust inference engine uses tanh — matching activations avoids
       train/deploy mismatch.
    2. Bounded outputs are desirable for velocity commands.
    3. The network is small enough that vanishing gradients aren't an issue.

Training:
    - Loss: MSE between predicted and demonstrated actions
    - Optimizer: Adam with default betas, lr=1e-3
    - Batch size: 256
    - Epochs: 200
    - Validation: 10% held out, best model selected by val MSE
    - Deterministic: seeded (default seed=42) for reproducibility

Output JSON format:
    The output JSON is designed to be loaded by Policy::load() in the Rust
    crate without any conversion step. Key fields:

    {
        "architecture": "mlp",
        "observation_size": 7,
        "action_size": 2,
        "layers": [
            {"weights": [[...]], "biases": [...]},  // 64x7 + 64
            {"weights": [[...]], "biases": [...]},  // 64x64 + 64
            {"weights": [[...]], "biases": [...]}   // 2x64 + 2
        ],
        "metrics": {"mse": ..., "r2": ..., ...}
    }

    The "weights" and "biases" top-level fields are empty arrays (they exist
    only for backward compatibility with linear policies).

Usage:
    python train_bc.py --data demos.npz --output bc-teleop-v0.1.0.json
    python train_bc.py --data demos.npz --epochs 500 --lr 5e-4

Requirements:
    pip install numpy torch   (see requirements.txt)
"""

from __future__ import annotations

import argparse
import json
import time
from pathlib import Path

import numpy as np
import torch
import torch.nn as nn
from torch.utils.data import DataLoader, TensorDataset


class BCPolicy(nn.Module):
    """MLP policy network for behavioral cloning.

    A simple feedforward network: obs → hidden → hidden → action, with tanh
    activations on every layer (including the output). The architecture is
    intentionally small — it needs to run in real-time on the rover's Jetson
    Orin NX via the Rust policy crate's forward_mlp() function.

    The network structure mirrors the Rust inference engine exactly:
    each nn.Linear layer corresponds to one MlpLayer in the Rust crate,
    and tanh is applied after every layer (there is no final linear output).

    Attributes:
        net: Sequential stack of Linear → Tanh layers.
    """

    def __init__(self, obs_dim: int = 7, act_dim: int = 2, hidden: int = 64):
        """Initialize the BC policy network.

        Args:
            obs_dim: Observation vector size (default 7: pose + velocity + goal).
            act_dim: Action vector size (default 2: linear vel + angular vel).
            hidden: Hidden layer width (default 64). Both hidden layers use
                the same width.
        """
        super().__init__()
        self.net = nn.Sequential(
            nn.Linear(obs_dim, hidden),
            nn.Tanh(),
            nn.Linear(hidden, hidden),
            nn.Tanh(),
            nn.Linear(hidden, act_dim),
            nn.Tanh(),
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """Forward pass: observation tensor → action tensor.

        Args:
            x: Batch of observations, shape (batch_size, obs_dim).

        Returns:
            Batch of actions, shape (batch_size, act_dim), values in [-1, 1].
        """
        return self.net(x)

    def export_layers(self) -> list[dict]:
        """Export weights and biases for the Rust policy JSON format.

        Iterates through the Sequential modules, extracts weight matrices
        and bias vectors from each nn.Linear layer, and returns them as
        plain Python lists (JSON-serializable).

        The weight matrix shape is (output_dim, input_dim), matching the
        Rust MlpLayer::weights convention (and PyTorch's native layout).

        Returns:
            List of dicts, one per linear layer, each with:
                "weights": list of lists (output_dim × input_dim)
                "biases": list (output_dim)
        """
        layers = []
        for module in self.net:
            if isinstance(module, nn.Linear):
                layers.append(
                    {
                        "weights": module.weight.detach().cpu().numpy().tolist(),
                        "biases": module.bias.detach().cpu().numpy().tolist(),
                    }
                )
        return layers

    def param_count(self) -> int:
        """Total number of trainable parameters."""
        return sum(p.numel() for p in self.parameters())


def train(
    data_path: Path,
    output_path: Path,
    epochs: int = 200,
    batch_size: int = 256,
    lr: float = 1e-3,
    val_split: float = 0.1,
    seed: int = 42,
) -> dict:
    """Train the behavioral cloning policy and export to JSON.

    Loads the demonstration dataset, splits into train/validation, trains
    the MLP with MSE loss and Adam optimizer, selects the best model by
    validation loss, and exports it as a Rust-compatible policy JSON file.

    The training loop uses early-stopping-like behavior: it tracks the best
    validation MSE across all epochs and restores those weights before
    exporting. All epochs are run (no early termination) to keep the
    interface simple.

    Args:
        data_path: Path to demos.npz (output of extract_demos.py).
        output_path: Where to write the policy JSON file.
        epochs: Number of training epochs (default 200).
        batch_size: Mini-batch size for SGD (default 256).
        lr: Adam learning rate (default 1e-3).
        val_split: Fraction of data for validation (default 0.1).
        seed: Random seed for reproducibility (default 42).

    Returns:
        Dict of training metrics (mse, r2, mae, sample counts, etc.).
        These same metrics are embedded in the output JSON.
    """
    torch.manual_seed(seed)
    np.random.seed(seed)

    # Load data
    data = np.load(data_path)
    observations = data["observations"]  # (N, 7)
    actions = data["actions"]  # (N, 2)
    sessions_used = int(data.get("sessions_used", 0))

    n = len(observations)
    print(f"Loaded {n} samples")
    print(f"Observations: {observations.shape}, Actions: {actions.shape}")

    # Train/val split
    indices = np.random.permutation(n)
    val_n = max(1, int(n * val_split))
    val_idx = indices[:val_n]
    train_idx = indices[val_n:]

    train_obs = torch.from_numpy(observations[train_idx])
    train_act = torch.from_numpy(actions[train_idx])
    val_obs = torch.from_numpy(observations[val_idx])
    val_act = torch.from_numpy(actions[val_idx])

    train_loader = DataLoader(
        TensorDataset(train_obs, train_act),
        batch_size=batch_size,
        shuffle=True,
    )

    # Model
    model = BCPolicy()
    print(f"Model parameters: {model.param_count()}")

    optimizer = torch.optim.Adam(model.parameters(), lr=lr)
    loss_fn = nn.MSELoss()

    best_val_loss = float("inf")
    best_state = None
    t0 = time.time()

    for epoch in range(epochs):
        # Train
        model.train()
        train_loss = 0.0
        for obs_batch, act_batch in train_loader:
            pred = model(obs_batch)
            loss = loss_fn(pred, act_batch)
            optimizer.zero_grad()
            loss.backward()
            optimizer.step()
            train_loss += loss.item() * len(obs_batch)
        train_loss /= len(train_idx)

        # Validate
        model.eval()
        with torch.no_grad():
            val_pred = model(val_obs)
            val_loss = loss_fn(val_pred, val_act).item()

        if val_loss < best_val_loss:
            best_val_loss = val_loss
            best_state = {k: v.clone() for k, v in model.state_dict().items()}

        if (epoch + 1) % 20 == 0 or epoch == 0:
            print(
                f"  Epoch {epoch+1:3d}/{epochs}  "
                f"train_mse={train_loss:.5f}  val_mse={val_loss:.5f}"
            )

    elapsed = time.time() - t0
    print(f"\nTraining complete in {elapsed:.1f}s")
    print(f"Best validation MSE: {best_val_loss:.5f}")

    # Restore best weights
    if best_state is not None:
        model.load_state_dict(best_state)
    model.eval()

    # Compute final metrics
    with torch.no_grad():
        val_pred = model(val_obs)
        per_dim_mae = (val_pred - val_act).abs().mean(dim=0).numpy()
        val_mse = ((val_pred - val_act) ** 2).mean().item()

        # R² score
        ss_res = ((val_act - val_pred) ** 2).sum().item()
        ss_tot = ((val_act - val_act.mean(dim=0)) ** 2).sum().item()
        r2 = 1.0 - ss_res / ss_tot if ss_tot > 0 else 0.0

    demo_hours = n / (10 * 3600)  # 10Hz samples -> hours

    metrics = {
        "mse": round(val_mse, 6),
        "r2": round(r2, 4),
        "mae_linear": round(float(per_dim_mae[0]), 6),
        "mae_angular": round(float(per_dim_mae[1]), 6),
        "training_samples": n,
        "demo_hours": round(demo_hours, 2),
        "sessions_used": sessions_used,
        "epochs": epochs,
        "best_val_mse": round(best_val_loss, 6),
    }

    # Export to policy JSON format
    policy = {
        "version": "0.1.0",
        "name": "bc-teleop",
        "description": "Behavioral cloning policy trained on teleop demonstrations",
        "observation_size": 7,
        "action_size": 2,
        "architecture": "mlp",
        "weights": [],
        "biases": [],
        "layers": model.export_layers(),
        "metrics": metrics,
    }

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w") as f:
        json.dump(policy, f, indent=2)

    print(f"\nPolicy saved to {output_path}")
    print(f"Metrics: {json.dumps(metrics, indent=2)}")

    return metrics


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Train behavioral cloning policy"
    )
    parser.add_argument(
        "--data",
        type=Path,
        required=True,
        help="Path to demos.npz",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("bc-teleop-v0.1.0.json"),
        help="Output policy JSON path",
    )
    parser.add_argument("--epochs", type=int, default=200)
    parser.add_argument("--batch-size", type=int, default=256)
    parser.add_argument("--lr", type=float, default=1e-3)
    parser.add_argument("--seed", type=int, default=42)
    args = parser.parse_args()

    train(
        data_path=args.data,
        output_path=args.output,
        epochs=args.epochs,
        batch_size=args.batch_size,
        lr=args.lr,
        seed=args.seed,
    )


if __name__ == "__main__":
    main()
