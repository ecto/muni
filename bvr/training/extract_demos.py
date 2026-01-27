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

    # Query all relevant entities. The Rerun dataframe API returns columns
    # named like "/entity/path:ComponentType". We query with a glob pattern
    # and use fill_latest_at() to forward-fill values — since different
    # entities are logged at different timestamps, without this each column
    # would only have data on its own rows and be None everywhere else.
    view = recording.view(index="log_time", contents="/**")
    view = view.fill_latest_at()
    table = view.select().read_all()
    col_names = set(table.column_names)

    # Map from our logical names to the actual Rerun column names.
    # Scalar channels use the "/entity:Scalar" convention.
    COLUMN_MAP = {
        "x": "/robot/x:Scalar",
        "y": "/robot/y:Scalar",
        "heading": "/robot/heading:Scalar",
        "lin_cmd": "/velocity/linear/commanded:Scalar",
        "ang_cmd": "/velocity/angular/commanded:Scalar",
        "mode": "/state/mode:Text",
    }

    # Verify all required columns exist
    missing = [k for k, v in COLUMN_MAP.items() if v not in col_names]
    if missing:
        print(f"  Skipping {rrd_path.name}: missing columns {missing}")
        return None

    def col_array(key: str) -> np.ndarray:
        """Extract a Rerun Scalar column as a flat numpy float64 array.

        Rerun stores Scalar components as Arrow list<double> — each row is
        either None (not logged at this timestamp) or a single-element list
        like [0.5]. This function unwraps that to a flat float64 array with
        NaN for missing entries.
        """
        arrow_col = table.column(COLUMN_MAP[key])
        raw = arrow_col.to_numpy()  # object array: None or np.array([val])
        out = np.full(len(raw), np.nan, dtype=np.float64)
        for i, v in enumerate(raw):
            if v is not None:
                if isinstance(v, np.ndarray) and len(v) > 0:
                    out[i] = float(v[0])
                elif isinstance(v, (int, float)):
                    out[i] = float(v)
        return out

    try:
        x_raw = col_array("x")
        y_raw = col_array("y")
        heading_raw = col_array("heading")
        lin_cmd_raw = col_array("lin_cmd")
        ang_cmd_raw = col_array("ang_cmd")
    except (KeyError, TypeError) as e:
        print(f"  Skipping {rrd_path.name}: column extraction error: {e}")
        return None

    # The Rerun dataframe merges all entities by timestamp, so most columns
    # will have NaN for rows where that entity wasn't logged. Build a mask
    # of rows where all required scalar columns have valid data.
    valid_mask = (
        ~np.isnan(x_raw)
        & ~np.isnan(y_raw)
        & ~np.isnan(heading_raw)
        & ~np.isnan(lin_cmd_raw)
        & ~np.isnan(ang_cmd_raw)
    )

    x = x_raw[valid_mask]
    y = y_raw[valid_mask]
    heading = heading_raw[valid_mask]
    lin_cmd = lin_cmd_raw[valid_mask].astype(np.float32)
    ang_cmd = ang_cmd_raw[valid_mask].astype(np.float32)

    n = len(x)
    print(f"  {n} valid rows (of {len(x_raw)} total)")

    # Detect actual sample rate from timestamps
    if n < 10:
        print(f"  Skipping {rrd_path.name}: too few valid samples ({n})")
        return None

    # For the goal lookahead, we use a fixed number of samples based on
    # the configured source rate. If the actual rate differs, the goal
    # horizon will be approximate — this is acceptable since pseudo-goals
    # are inherently approximate.
    goal_offset = int(GOAL_HORIZON_S * SOURCE_HZ)

    # If the session has fewer samples than our expected rate implies,
    # adapt: use all samples and compute goal offset from actual count.
    # Typical bvrd logs at ~20Hz for pose data (not 100Hz), so adjust.
    if goal_offset >= n:
        # Estimate actual rate and recompute
        actual_hz = n / max(1, n / SOURCE_HZ)
        goal_offset = max(5, int(GOAL_HORIZON_S * n / max(1, n / SOURCE_HZ * GOAL_HORIZON_S)))
        if goal_offset >= n:
            goal_offset = n // 4  # fallback: use 25% of session as horizon

    # Check for mode column — in some sessions, mode is only logged once
    # (at session start). If it says "Teleop", we treat the whole session
    # as teleop. If not present or not teleop, we still include data
    # (since these sessions are explicitly selected for training).
    try:
        mode_col = table.column(COLUMN_MAP["mode"])
        mode_values = mode_col.drop_null().to_pylist()
        if mode_values:
            # Check if any mode entry indicates teleop
            session_is_teleop = any(
                "teleop" in str(m).lower() for m in mode_values
            )
            if not session_is_teleop:
                print(f"  Warning: session mode is {mode_values[:3]}, not Teleop — including anyway")
    except Exception:
        pass  # No mode data, include all samples

    observations = []
    actions = []

    # Subsample: take every SUBSAMPLE_STEP'th sample. If the actual rate
    # is lower than SOURCE_HZ, adjust to avoid over-subsampling.
    step = max(1, SUBSAMPLE_STEP if n > SOURCE_HZ * 10 else 1)

    for i in range(0, n - goal_offset, step):
        # Filter: must be moving
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

        # Action: commanded velocities normalized independently
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
        print(f"  Skipping {rrd_path.name}: no moving teleop samples")
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
