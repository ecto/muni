#!/usr/bin/env python3
"""Evaluate a trained behavioral cloning policy and generate a metrics report.

This is the final step in the behavioral cloning pipeline. It loads a
trained policy JSON and the demonstration dataset, runs inference on the
validation split, and computes detailed accuracy metrics.

Pipeline position:
    extract_demos.py → demos.npz → train_bc.py → policy.json → [THIS SCRIPT]

The evaluation uses the same validation split as training (same seed and
val_split ratio) so the reported metrics match the training script's final
validation numbers. This script adds additional metrics beyond what the
training loop computes: per-dimension correlation, prediction range analysis,
and a formatted summary report.

Metrics computed:
    - Validation MSE (mean squared error across both action dimensions)
    - R² score (coefficient of determination — 1.0 is perfect)
    - Per-dimension MAE (mean absolute error for linear and angular velocity)
    - Per-dimension Pearson correlation (linear and angular velocity)
    - Prediction range (min/max of predicted actions)

The forward pass is implemented in pure NumPy (no PyTorch dependency) to
verify that the exported JSON weights produce correct outputs independently
of the training framework. This NumPy implementation mirrors the Rust
forward_mlp() function in bvr/firmware/crates/policy/src/lib.rs.

Usage:
    python evaluate.py --policy bc-teleop-v0.1.0.json --data demos.npz
    python evaluate.py --policy policy.json --data demos.npz --output metrics.json

Requirements:
    pip install numpy   (see requirements.txt)
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np


def load_policy_layers(policy_path: Path) -> tuple[list[dict], dict]:
    """Load MLP layers and training metrics from a policy JSON file.

    Args:
        policy_path: Path to the policy JSON (output of train_bc.py).

    Returns:
        Tuple of (layers, metrics) where layers is a list of dicts with
        "weights" and "biases" keys, and metrics is the training metrics
        dict (may be empty for manually created policies).
    """
    with open(policy_path) as f:
        policy = json.load(f)
    return policy["layers"], policy.get("metrics", {})


def forward_mlp(layers: list[dict], obs: np.ndarray) -> np.ndarray:
    """Run MLP forward pass in pure NumPy.

    This mirrors the Rust policy crate's Policy::forward_mlp() exactly:
    for each layer, compute y = tanh(W @ x + b). The weight matrix W has
    shape (output_dim, input_dim), so we compute x @ W.T + b.

    This implementation exists independently of PyTorch to verify that the
    exported JSON weights produce correct results without the training
    framework.

    Args:
        layers: List of layer dicts, each with "weights" (list of lists)
            and "biases" (list). Output of BCPolicy.export_layers() or
            loaded from a policy JSON.
        obs: Single observation vector, shape (obs_dim,).

    Returns:
        Action vector, shape (act_dim,), values in [-1, 1].
    """
    x = obs.copy()
    for layer in layers:
        w = np.array(layer["weights"], dtype=np.float32)  # (out, in)
        b = np.array(layer["biases"], dtype=np.float32)  # (out,)
        x = np.tanh(x @ w.T + b)
    return x


def evaluate(
    policy_path: Path,
    data_path: Path,
    val_split: float = 0.1,
    seed: int = 42,
) -> dict:
    """Evaluate the policy on the validation split and compute metrics.

    Uses the same seed and val_split as training to produce the same
    validation set. Runs the NumPy forward pass on each validation sample
    and computes MSE, R², MAE, correlation, and prediction range.

    Args:
        policy_path: Path to the trained policy JSON file.
        data_path: Path to demos.npz (same file used for training).
        val_split: Fraction of data used for validation (must match training).
        seed: Random seed for the train/val split (must match training).

    Returns:
        Dict of evaluation metrics. Keys:
            validation_mse: float — mean squared error on val set
            r2_score: float — coefficient of determination
            mae_linear_vel: float — mean absolute error, linear dimension
            mae_angular_vel: float — mean absolute error, angular dimension
            correlation_linear: float — Pearson r, linear dimension
            correlation_angular: float — Pearson r, angular dimension
            pred_linear_range: [min, max] — predicted linear vel range
            pred_angular_range: [min, max] — predicted angular vel range
            total_samples: int — total dataset size
            validation_samples: int — number of validation samples
            demo_hours: float — total demo time in hours (at 10Hz)
            sessions_used: int — number of source .rrd sessions
    """
    np.random.seed(seed)

    # Load data
    data = np.load(data_path)
    observations = data["observations"]
    actions = data["actions"]
    sessions_used = int(data.get("sessions_used", 0))
    n = len(observations)

    # Same val split as training
    indices = np.random.permutation(n)
    val_n = max(1, int(n * val_split))
    val_obs = observations[indices[:val_n]]
    val_act = actions[indices[:val_n]]

    # Load policy
    layers, train_metrics = load_policy_layers(policy_path)

    # Run inference
    val_pred = np.array([forward_mlp(layers, obs) for obs in val_obs])

    # --- Metrics ---
    # MSE
    mse = float(np.mean((val_pred - val_act) ** 2))

    # Per-dimension MAE
    mae = np.mean(np.abs(val_pred - val_act), axis=0)
    mae_linear = float(mae[0])
    mae_angular = float(mae[1])

    # R² score
    ss_res = np.sum((val_act - val_pred) ** 2)
    ss_tot = np.sum((val_act - val_act.mean(axis=0)) ** 2)
    r2 = 1.0 - ss_res / ss_tot if ss_tot > 0 else 0.0

    # Correlation
    corr_linear = float(np.corrcoef(val_act[:, 0], val_pred[:, 0])[0, 1])
    corr_angular = float(np.corrcoef(val_act[:, 1], val_pred[:, 1])[0, 1])

    # Prediction range
    pred_lin_range = (float(val_pred[:, 0].min()), float(val_pred[:, 0].max()))
    pred_ang_range = (float(val_pred[:, 1].min()), float(val_pred[:, 1].max()))

    demo_hours = n / (10 * 3600)

    metrics = {
        "validation_mse": round(mse, 6),
        "r2_score": round(float(r2), 4),
        "mae_linear_vel": round(mae_linear, 6),
        "mae_angular_vel": round(mae_angular, 6),
        "correlation_linear": round(corr_linear, 4),
        "correlation_angular": round(corr_angular, 4),
        "pred_linear_range": [round(x, 3) for x in pred_lin_range],
        "pred_angular_range": [round(x, 3) for x in pred_ang_range],
        "total_samples": n,
        "validation_samples": val_n,
        "demo_hours": round(demo_hours, 2),
        "sessions_used": sessions_used,
    }

    return metrics


def print_summary(metrics: dict) -> None:
    """Print a formatted evaluation report to stdout."""
    print("=" * 60)
    print("  BEHAVIORAL CLONING POLICY — EVALUATION REPORT")
    print("=" * 60)
    print()
    print("  DATA")
    print(f"    Training samples:    {metrics['total_samples']:,}")
    print(f"    Validation samples:  {metrics['validation_samples']:,}")
    print(f"    Demo hours:          {metrics['demo_hours']:.1f}h")
    print(f"    Sessions used:       {metrics['sessions_used']}")
    print()
    print("  ACCURACY")
    print(f"    Validation MSE:      {metrics['validation_mse']:.5f}")
    print(f"    R² score:            {metrics['r2_score']:.3f}")
    print(f"    MAE (linear vel):    {metrics['mae_linear_vel']:.4f}")
    print(f"    MAE (angular vel):   {metrics['mae_angular_vel']:.4f}")
    print()
    print("  CORRELATION")
    print(f"    Linear velocity:     {metrics['correlation_linear']:.3f}")
    print(f"    Angular velocity:    {metrics['correlation_angular']:.3f}")
    print()
    print("  PREDICTION RANGE")
    lr = metrics["pred_linear_range"]
    ar = metrics["pred_angular_range"]
    print(f"    Linear:  [{lr[0]:.3f}, {lr[1]:.3f}]")
    print(f"    Angular: [{ar[0]:.3f}, {ar[1]:.3f}]")
    print()
    print("=" * 60)
    print("  Model trained on real operator teleop data.")
    print("  Classical nav stack provides safety backbone at runtime.")
    print("=" * 60)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Evaluate behavioral cloning policy"
    )
    parser.add_argument(
        "--policy",
        type=Path,
        required=True,
        help="Path to policy JSON file",
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
        default=None,
        help="Optional path to save metrics JSON",
    )
    args = parser.parse_args()

    metrics = evaluate(args.policy, args.data)
    print_summary(metrics)

    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        with open(args.output, "w") as f:
            json.dump(metrics, f, indent=2)
        print(f"\nMetrics saved to {args.output}")


if __name__ == "__main__":
    main()
