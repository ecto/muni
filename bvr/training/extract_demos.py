#!/usr/bin/env python3
"""Extract (observation, action) pairs from .rrd teleop recordings.

Reads Rerun .rrd session files, filters to teleop-mode segments, computes
pseudo-goals from future trajectory, and outputs a normalized dataset
suitable for behavioral cloning.

Output: demos.npz with:
    observations: (N, 7) float32 — [x, y, theta, lin_vel, ang_vel, goal_dx, goal_dy]
    actions: (N, 2) float32 — [linear_cmd, angular_cmd]

All values normalized to approximately [-1, 1].
"""

from __future__ import annotations

import argparse
import math
from pathlib import Path

import numpy as np

try:
    import rerun as rr
except ImportError:
    raise SystemExit("rerun-sdk is required: pip install rerun-sdk")

# --- Normalization constants (must match policy crate NormalizationConfig) ---
MAX_POSITION = 50.0
MAX_LINEAR_VEL = 2.0
MAX_ANGULAR_VEL = 2.0

# --- Extraction parameters ---
GOAL_HORIZON_S = 2.0  # seconds ahead for pseudo-goal
SOURCE_HZ = 100  # assumed recording rate
TARGET_HZ = 10  # output rate
SUBSAMPLE_STEP = SOURCE_HZ // TARGET_HZ
MIN_SPEED = 0.05  # filter stationary samples


def transform_to_robot_frame(
    dx: float, dy: float, theta: float
) -> tuple[float, float]:
    """Transform a world-frame delta into robot-frame coordinates."""
    cos_t = math.cos(theta)
    sin_t = math.sin(theta)
    rx = dx * cos_t + dy * sin_t
    ry = -dx * sin_t + dy * cos_t
    return rx, ry


def extract_session(rrd_path: Path) -> tuple[np.ndarray, np.ndarray] | None:
    """Extract demo pairs from a single .rrd session file.

    Returns (observations, actions) arrays or None if no valid data.
    """
    recording = rr.dataframe.load_recording(str(rrd_path))
    view = recording.view(
        index="log_time",
        contents={
            "robot/x": [],
            "robot/y": [],
            "robot/heading": [],
            "velocity/linear/commanded": [],
            "velocity/angular/commanded": [],
            "state/mode": [],
        },
    )
    table = view.select().read_all()

    # Convert to numpy arrays
    def col(name: str) -> np.ndarray:
        return table.column(name).to_numpy()

    try:
        x = col("robot/x").astype(np.float64)
        y = col("robot/y").astype(np.float64)
        heading = col("robot/heading").astype(np.float64)
        lin_cmd = col("velocity/linear/commanded").astype(np.float32)
        ang_cmd = col("velocity/angular/commanded").astype(np.float32)
        mode = col("state/mode")
    except (KeyError, TypeError) as e:
        print(f"  Skipping {rrd_path.name}: missing column {e}")
        return None

    n = len(x)
    if n < SOURCE_HZ * (GOAL_HORIZON_S + 1):
        print(f"  Skipping {rrd_path.name}: too short ({n} samples)")
        return None

    # Goal lookahead in samples
    goal_offset = int(GOAL_HORIZON_S * SOURCE_HZ)

    observations = []
    actions = []

    for i in range(0, n - goal_offset, SUBSAMPLE_STEP):
        # Filter: teleop mode only
        m = mode[i]
        if hasattr(m, "item"):
            m = m.item()
        if isinstance(m, (str, bytes)):
            m_str = m if isinstance(m, str) else m.decode()
            if "teleop" not in m_str.lower():
                continue
        elif isinstance(m, (int, float, np.integer, np.floating)):
            # Mode enum: assume 1 = Teleop
            if int(m) != 1:
                continue
        else:
            continue

        # Filter: moving
        speed = abs(float(lin_cmd[i]))
        if speed < MIN_SPEED:
            continue

        # Pseudo-goal: future position in robot frame
        future_idx = i + goal_offset
        dx_world = float(x[future_idx] - x[i])
        dy_world = float(y[future_idx] - y[i])
        goal_rx, goal_ry = transform_to_robot_frame(
            dx_world, dy_world, float(heading[i])
        )

        # Build normalized observation
        obs = np.array(
            [
                np.clip(x[i] / MAX_POSITION, -1, 1),
                np.clip(y[i] / MAX_POSITION, -1, 1),
                np.clip(heading[i] / math.pi, -1, 1),
                np.clip(lin_cmd[i] / MAX_LINEAR_VEL, -1, 1),
                np.clip(ang_cmd[i] / MAX_ANGULAR_VEL, -1, 1),
                np.clip(goal_rx / MAX_POSITION, -1, 1),
                np.clip(goal_ry / MAX_POSITION, -1, 1),
            ],
            dtype=np.float32,
        )

        # Action: commanded velocities (already normalized above for obs,
        # but actions are separate — normalize independently)
        act = np.array(
            [
                np.clip(lin_cmd[i] / MAX_LINEAR_VEL, -1, 1),
                np.clip(ang_cmd[i] / MAX_ANGULAR_VEL, -1, 1),
            ],
            dtype=np.float32,
        )

        observations.append(obs)
        actions.append(act)

    if not observations:
        print(f"  Skipping {rrd_path.name}: no valid teleop samples")
        return None

    return np.array(observations), np.array(actions)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Extract behavioral cloning demos from .rrd files"
    )
    parser.add_argument(
        "--sessions-dir",
        type=Path,
        required=True,
        help="Directory containing .rrd session files",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("demos.npz"),
        help="Output .npz file (default: demos.npz)",
    )
    args = parser.parse_args()

    rrd_files = sorted(args.sessions_dir.glob("*.rrd"))
    if not rrd_files:
        raise SystemExit(f"No .rrd files found in {args.sessions_dir}")

    print(f"Found {len(rrd_files)} session files")

    all_obs = []
    all_act = []
    sessions_used = 0

    for rrd_path in rrd_files:
        print(f"Processing {rrd_path.name}...")
        result = extract_session(rrd_path)
        if result is not None:
            obs, act = result
            all_obs.append(obs)
            all_act.append(act)
            sessions_used += 1
            print(f"  Extracted {len(obs)} samples")

    if not all_obs:
        raise SystemExit("No valid samples extracted from any session")

    observations = np.concatenate(all_obs)
    actions = np.concatenate(all_act)

    print(f"\nTotal: {len(observations)} samples from {sessions_used} sessions")
    print(f"Observations shape: {observations.shape}")
    print(f"Actions shape: {actions.shape}")
    print(f"Obs range: [{observations.min():.3f}, {observations.max():.3f}]")
    print(f"Act range: [{actions.min():.3f}, {actions.max():.3f}]")

    np.savez(
        args.output,
        observations=observations,
        actions=actions,
        sessions_used=sessions_used,
    )
    print(f"\nSaved to {args.output}")


if __name__ == "__main__":
    main()
