#!/usr/bin/env python3
"""Extract (observation, action) pairs from .rrd teleop recordings.

This is the first step in the behavioral cloning pipeline. It reads Rerun
.rrd session files recorded during teleop, filters to teleop-mode segments,
computes pseudo-goals from the future trajectory, and outputs a normalized
dataset ready for supervised learning.

Pipeline position:
    teleop .rrd files  →  [THIS SCRIPT]  →  demos.npz  →  train_bc.py

Output format (demos.npz):
    observations : ndarray, shape (N, 7), dtype float32
        Each row is a normalized observation vector:
            [0] x          — robot x position / MAX_POSITION
            [1] y          — robot y position / MAX_POSITION
            [2] theta      — robot heading / pi
            [3] lin_vel    — commanded linear velocity / MAX_LINEAR_VEL
            [4] ang_vel    — commanded angular velocity / MAX_ANGULAR_VEL
            [5] goal_dx    — pseudo-goal delta-x in robot frame / MAX_POSITION
            [6] goal_dy    — pseudo-goal delta-y in robot frame / MAX_POSITION

    actions : ndarray, shape (N, 2), dtype float32
        Each row is a normalized action (the operator's commanded velocity):
            [0] linear_cmd  — commanded linear velocity / MAX_LINEAR_VEL
            [1] angular_cmd — commanded angular velocity / MAX_ANGULAR_VEL

    sessions_used : int
        Number of .rrd files that contributed at least one sample.

    All values are clipped to [-1, 1].

Pseudo-goal strategy:
    Teleop recordings contain no explicit navigation goals. To create the
    goal input that the policy expects at deploy time, we use the robot's
    actual position GOAL_HORIZON_S seconds in the future as a "pseudo-goal."
    This position is transformed into the robot's local frame at the current
    timestep. At deploy time, the classical planner provides real goals in
    the same format.

Filtering:
    - Only samples in Teleop mode are included (autonomous/idle excluded).
    - Stationary samples (|linear_vel| < MIN_SPEED) are discarded to avoid
      training on "do nothing" data that would bias the policy toward zero.
    - The raw 100Hz recording is subsampled to 10Hz (SUBSAMPLE_STEP=10) to
      reduce temporal correlation between adjacent samples.
    - The last GOAL_HORIZON_S seconds of each session are discarded because
      there is no future trajectory data to construct a pseudo-goal.

Normalization constants:
    MAX_POSITION, MAX_LINEAR_VEL, MAX_ANGULAR_VEL must match the Rust
    policy crate's NormalizationConfig::default(). If those constants change,
    update them here too — otherwise the Python-trained policy will produce
    incorrect outputs when loaded in Rust.

Usage:
    python extract_demos.py --sessions-dir /path/to/sessions --output demos.npz

Requirements:
    pip install rerun-sdk numpy   (see requirements.txt)
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

# --- Normalization constants ---
# These MUST match the Rust policy crate's NormalizationConfig::default().
# See: bvr/firmware/crates/policy/src/lib.rs → NormalizationConfig
MAX_POSITION = 50.0      # meters — normalizes x, y, goal_dx, goal_dy
MAX_LINEAR_VEL = 2.0     # m/s — normalizes linear velocity & commands
MAX_ANGULAR_VEL = 2.0    # rad/s — normalizes angular velocity & commands

# --- Extraction parameters ---
GOAL_HORIZON_S = 2.0     # seconds ahead for pseudo-goal lookahead
SOURCE_HZ = 100          # assumed .rrd recording rate (bvrd default)
TARGET_HZ = 10           # output dataset rate (reduce temporal correlation)
SUBSAMPLE_STEP = SOURCE_HZ // TARGET_HZ  # take every 10th sample
MIN_SPEED = 0.05         # m/s — discard stationary samples below this


def transform_to_robot_frame(
    dx: float, dy: float, theta: float
) -> tuple[float, float]:
    """Transform a world-frame displacement into robot-frame coordinates.

    Given a displacement (dx, dy) in the world frame and the robot's heading
    theta, returns the same displacement expressed in the robot's local frame
    where +x is forward and +y is left.

    This is a 2D rotation by -theta (inverse of the robot's heading):
        rx =  dx * cos(theta) + dy * sin(theta)
        ry = -dx * sin(theta) + dy * cos(theta)

    This matches the transform in the Rust policy crate's
    PolicyObservation::from_raw() method.

    Args:
        dx: World-frame x displacement (meters).
        dy: World-frame y displacement (meters).
        theta: Robot heading in radians (0 = east, pi/2 = north).

    Returns:
        (rx, ry): Displacement in robot frame (forward, left).
    """
    cos_t = math.cos(theta)
    sin_t = math.sin(theta)
    rx = dx * cos_t + dy * sin_t
    ry = -dx * sin_t + dy * cos_t
    return rx, ry


def extract_session(rrd_path: Path) -> tuple[np.ndarray, np.ndarray] | None:
    """Extract (observation, action) pairs from a single .rrd session file.

    Opens the .rrd file using Rerun's dataframe API, queries the required
    channels, and iterates through timesteps to build normalized
    observation-action pairs.

    The required .rrd channels are:
        - robot/x, robot/y, robot/heading — pose from localization
        - velocity/linear/commanded — operator's linear velocity command
        - velocity/angular/commanded — operator's angular velocity command
        - state/mode — rover operating mode (filters to Teleop only)

    For each valid timestep (teleop mode, moving, not in last 2s), the
    pseudo-goal is computed as the robot's position GOAL_HORIZON_S seconds
    in the future, transformed into the robot's local frame.

    Args:
        rrd_path: Path to a .rrd session recording file.

    Returns:
        Tuple of (observations, actions) numpy arrays, or None if the
        session has no valid teleop samples. observations has shape
        (N, 7) and actions has shape (N, 2), both float32.
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
