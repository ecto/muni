#!/usr/bin/env python3
"""Train an MLP policy via behavioral cloning on teleop demonstrations.

Input: demos.npz (from extract_demos.py)
Output: JSON policy file compatible with the Rust policy crate.

Architecture: 7 -> 64 (tanh) -> 64 (tanh) -> 2 (tanh)  (4802 params)
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
    """Simple MLP for behavioral cloning."""

    def __init__(self, obs_dim: int = 7, act_dim: int = 2, hidden: int = 64):
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
        return self.net(x)

    def export_layers(self) -> list[dict]:
        """Export weights/biases as list of layer dicts for Rust policy format."""
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
    """Train the BC policy and return metrics."""
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
