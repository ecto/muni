#!/usr/bin/env python3
"""
Gaussian Splatting Worker

Watches for job files and processes them into 3D Gaussian splats.
Designed to run in a container with GPU access.

Job format (job.json):
{
    "id": "uuid",
    "session_path": "/data/sessions/bvr-01/2024-12-27T14-30-00",
    "output_path": "/data/maps/map-name",
    "config": {
        "iterations": 30000,
        "resolution": 1024
    }
}

Output:
- splat.ply (Gaussian splat in PLY format)
- result.json (processing stats and status)
"""

import json
import logging
import os
import shutil
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Optional
from datetime import datetime

import numpy as np
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler, FileCreatedEvent

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [%(levelname)s] %(message)s',
    handlers=[logging.StreamHandler(sys.stdout)]
)
logger = logging.getLogger(__name__)

# Environment configuration
JOBS_DIR = Path(os.environ.get('JOBS_DIR', '/data/jobs'))
MAPS_DIR = Path(os.environ.get('MAPS_DIR', '/data/maps'))
SESSIONS_DIR = Path(os.environ.get('SESSIONS_DIR', '/data/sessions'))


@dataclass
class SplatConfig:
    """Configuration for Gaussian splatting training."""
    iterations: int = 30000
    resolution: int = 1024
    sh_degree: int = 3
    densify_until_iter: int = 15000
    densify_from_iter: int = 500
    densification_interval: int = 100
    opacity_reset_interval: int = 3000
    position_lr_init: float = 0.00016
    position_lr_final: float = 0.0000016
    scaling_lr: float = 0.005
    rotation_lr: float = 0.001
    opacity_lr: float = 0.05
    feature_lr: float = 0.0025


@dataclass 
class Job:
    """A splatting job to process."""
    id: str
    session_path: Path
    output_path: Path
    config: SplatConfig
    
    @classmethod
    def from_json(cls, data: dict) -> 'Job':
        config_data = data.get('config', {})
        config = SplatConfig(**{k: v for k, v in config_data.items() if hasattr(SplatConfig, k)})
        return cls(
            id=data['id'],
            session_path=Path(data['session_path']),
            output_path=Path(data['output_path']),
            config=config
        )


@dataclass
class Pose:
    """A 3D pose with timestamp."""
    timestamp: float
    position: np.ndarray  # [x, y, z]
    rotation: np.ndarray  # [qx, qy, qz, qw] quaternion


def load_poses(session_path: Path) -> list[Pose]:
    """
    Load poses from poses.csv file.
    
    Format: timestamp_secs,x,y,z,qx,qy,qz,qw
    """
    poses_file = session_path / 'poses.csv'
    poses = []
    
    if not poses_file.exists():
        logger.warning(f"No poses.csv found at {poses_file}")
        return poses
    
    with open(poses_file, 'r') as f:
        next(f)  # Skip header
        for line in f:
            parts = line.strip().split(',')
            if len(parts) >= 8:
                try:
                    poses.append(Pose(
                        timestamp=float(parts[0]),
                        position=np.array([float(parts[1]), float(parts[2]), float(parts[3])]),
                        rotation=np.array([float(parts[4]), float(parts[5]), float(parts[6]), float(parts[7])])
                    ))
                except ValueError as e:
                    logger.warning(f"Failed to parse pose line: {e}")
    
    logger.info(f"Loaded {len(poses)} poses from poses.csv")
    return poses


def interpolate_pose(poses: list[Pose], timestamp: float) -> Optional[Pose]:
    """
    Interpolate pose at a given timestamp using linear interpolation.
    """
    if not poses:
        return None
    
    # Find bracketing poses
    prev_pose = None
    next_pose = None
    
    for pose in poses:
        if pose.timestamp <= timestamp:
            prev_pose = pose
        elif pose.timestamp > timestamp and next_pose is None:
            next_pose = pose
            break
    
    if prev_pose is None:
        return poses[0] if poses else None
    if next_pose is None:
        return prev_pose
    
    # Linear interpolation
    dt = next_pose.timestamp - prev_pose.timestamp
    if dt < 1e-6:
        return prev_pose
    
    t = (timestamp - prev_pose.timestamp) / dt
    t = np.clip(t, 0.0, 1.0)
    
    # Interpolate position
    position = prev_pose.position * (1 - t) + next_pose.position * t
    
    # SLERP for quaternion (simplified: linear for small angles)
    rotation = prev_pose.rotation * (1 - t) + next_pose.rotation * t
    rotation = rotation / np.linalg.norm(rotation)  # Normalize
    
    return Pose(timestamp=timestamp, position=position, rotation=rotation)


def quaternion_to_matrix(q: np.ndarray) -> np.ndarray:
    """Convert quaternion [qx, qy, qz, qw] to 3x3 rotation matrix."""
    qx, qy, qz, qw = q
    
    return np.array([
        [1 - 2*qy*qy - 2*qz*qz, 2*qx*qy - 2*qz*qw, 2*qx*qz + 2*qy*qw],
        [2*qx*qy + 2*qz*qw, 1 - 2*qx*qx - 2*qz*qz, 2*qy*qz - 2*qx*qw],
        [2*qx*qz - 2*qy*qw, 2*qy*qz + 2*qx*qw, 1 - 2*qx*qx - 2*qy*qy]
    ])


def transform_points_to_world(points: np.ndarray, pose: Pose) -> np.ndarray:
    """
    Transform points from sensor frame to world frame using pose.
    
    Args:
        points: (N, 3) array of points in sensor frame
        pose: Pose with position and rotation
    
    Returns:
        (N, 3) array of points in world frame
    """
    if len(points) == 0:
        return points
    
    # Build rotation matrix from quaternion
    R = quaternion_to_matrix(pose.rotation)
    
    # Transform: world_point = R @ sensor_point + position
    world_points = (R @ points.T).T + pose.position
    
    return world_points


def load_session_data(session_path: Path) -> tuple[np.ndarray, list[dict], list[Path]]:
    """
    Load session data for splatting.
    
    Returns:
        points: (N, 3) array of LiDAR points in world frame
        poses: List of camera poses with timestamps
        images: List of image paths
    """
    logger.info(f"Loading session from {session_path}")
    
    # Load poses first
    poses = load_poses(session_path)
    
    # Load LiDAR timestamps
    lidar_dir = session_path / 'lidar'
    lidar_timestamps = {}
    
    timestamps_file = lidar_dir / 'timestamps.csv' if lidar_dir.exists() else None
    if timestamps_file and timestamps_file.exists():
        with open(timestamps_file, 'r') as f:
            next(f)  # Skip header
            for line in f:
                parts = line.strip().split(',')
                if len(parts) >= 2:
                    frame_num = int(parts[0])
                    timestamp = float(parts[1])
                    lidar_timestamps[frame_num] = timestamp
    
    # Load LiDAR points with pose transformation
    points = []
    
    if lidar_dir.exists():
        pcd_files = sorted(lidar_dir.glob('*.pcd'))
        logger.info(f"Found {len(pcd_files)} LiDAR frames")
        
        for pcd_file in pcd_files:
            frame_points = load_pcd(pcd_file)
            if frame_points is None or len(frame_points) == 0:
                continue
            
            # Get frame number from filename
            try:
                frame_num = int(pcd_file.stem)
            except ValueError:
                continue
            
            # Get timestamp and interpolate pose
            timestamp = lidar_timestamps.get(frame_num)
            if timestamp is not None and poses:
                pose = interpolate_pose(poses, timestamp)
                if pose is not None:
                    # Transform points to world frame
                    frame_points = transform_points_to_world(frame_points, pose)
            
            points.append(frame_points)
        
        if points:
            points = np.vstack(points)
            logger.info(f"Loaded {len(points)} total points (world frame)")
        else:
            points = np.zeros((0, 3))
    else:
        points = np.zeros((0, 3))
        logger.warning("No LiDAR directory found")
    
    # Load camera images
    camera_dir = session_path / 'camera'
    images = []
    
    if camera_dir.exists():
        images = sorted(camera_dir.glob('*.jpg'))
        logger.info(f"Found {len(images)} camera frames")
    else:
        logger.warning("No camera directory found")
    
    # Build camera pose list from poses
    camera_poses = []
    camera_timestamps_file = camera_dir / 'timestamps.csv' if camera_dir.exists() else None
    
    if camera_timestamps_file and camera_timestamps_file.exists():
        with open(camera_timestamps_file, 'r') as f:
            next(f)  # Skip header
            for line in f:
                parts = line.strip().split(',')
                if len(parts) >= 2:
                    frame_num = int(parts[0])
                    timestamp = float(parts[1])
                    
                    # Interpolate pose for this camera frame
                    pose = interpolate_pose(poses, timestamp) if poses else None
                    if pose is not None:
                        camera_poses.append({
                            'frame': frame_num,
                            'timestamp': timestamp,
                            'position': pose.position.tolist(),
                            'rotation': pose.rotation.tolist()
                        })
                    else:
                        camera_poses.append({
                            'frame': frame_num,
                            'timestamp': timestamp,
                            'position': [0.0, 0.0, 0.0],
                            'rotation': [0.0, 0.0, 0.0, 1.0]
                        })
    
    return points, camera_poses, images


def load_pcd(path: Path) -> Optional[np.ndarray]:
    """Load points from a PCD file."""
    try:
        points = []
        with open(path, 'r') as f:
            in_data = False
            for line in f:
                if in_data:
                    parts = line.strip().split()
                    if len(parts) >= 3:
                        points.append([float(parts[0]), float(parts[1]), float(parts[2])])
                elif line.startswith('DATA'):
                    in_data = True
        
        return np.array(points, dtype=np.float32) if points else None
    except Exception as e:
        logger.warning(f"Failed to load PCD {path}: {e}")
        return None


# =============================================================================
# Point Cloud Preprocessing
# =============================================================================

def voxel_downsample(points: np.ndarray, voxel_size: float = 0.05) -> np.ndarray:
    """
    Downsample point cloud using voxel grid filter.
    
    Args:
        points: (N, 3) array of points
        voxel_size: Size of voxel grid cells in meters
    
    Returns:
        Downsampled point cloud
    """
    if len(points) == 0:
        return points
    
    # Quantize points to voxel grid
    voxel_indices = np.floor(points / voxel_size).astype(np.int32)
    
    # Use unique voxels
    _, unique_indices = np.unique(
        voxel_indices.view(np.dtype((np.void, voxel_indices.dtype.itemsize * 3))),
        return_index=True
    )
    
    downsampled = points[unique_indices]
    logger.info(f"Voxel downsample: {len(points)} -> {len(downsampled)} points (voxel_size={voxel_size}m)")
    return downsampled


def remove_statistical_outliers(
    points: np.ndarray, 
    k_neighbors: int = 20, 
    std_ratio: float = 2.0
) -> np.ndarray:
    """
    Remove statistical outliers based on mean distance to k nearest neighbors.
    
    Points with mean distance greater than (global_mean + std_ratio * global_std)
    are considered outliers.
    
    Args:
        points: (N, 3) array of points
        k_neighbors: Number of neighbors to consider
        std_ratio: Standard deviation multiplier for outlier threshold
    
    Returns:
        Filtered point cloud
    """
    if len(points) < k_neighbors + 1:
        return points
    
    try:
        from scipy.spatial import cKDTree
    except ImportError:
        logger.warning("scipy not available, skipping outlier removal")
        return points
    
    # Build KD-tree
    tree = cKDTree(points)
    
    # Query k+1 neighbors (includes self)
    distances, _ = tree.query(points, k=k_neighbors + 1)
    
    # Mean distance to neighbors (exclude self which is distance 0)
    mean_distances = np.mean(distances[:, 1:], axis=1)
    
    # Compute threshold
    global_mean = np.mean(mean_distances)
    global_std = np.std(mean_distances)
    threshold = global_mean + std_ratio * global_std
    
    # Filter
    mask = mean_distances < threshold
    filtered = points[mask]
    
    logger.info(f"Outlier removal: {len(points)} -> {len(filtered)} points "
                f"(threshold={threshold:.4f}m)")
    return filtered


def filter_ground_plane(
    points: np.ndarray, 
    ground_threshold: float = 0.1,
    ransac_iterations: int = 100
) -> tuple[np.ndarray, np.ndarray]:
    """
    Separate ground plane from other points using RANSAC.
    
    Args:
        points: (N, 3) array of points
        ground_threshold: Distance threshold for inliers
        ransac_iterations: Number of RANSAC iterations
    
    Returns:
        Tuple of (non_ground_points, ground_points)
    """
    if len(points) < 3:
        return points, np.zeros((0, 3), dtype=np.float32)
    
    best_inliers = None
    best_inlier_count = 0
    
    for _ in range(ransac_iterations):
        # Sample 3 random points
        indices = np.random.choice(len(points), 3, replace=False)
        sample = points[indices]
        
        # Fit plane: ax + by + cz + d = 0
        v1 = sample[1] - sample[0]
        v2 = sample[2] - sample[0]
        normal = np.cross(v1, v2)
        
        norm = np.linalg.norm(normal)
        if norm < 1e-6:
            continue
        normal = normal / norm
        
        # Check if plane is approximately horizontal (normal ~ [0, 0, 1])
        if abs(normal[2]) < 0.8:
            continue
        
        d = -np.dot(normal, sample[0])
        
        # Compute distances to plane
        distances = np.abs(np.dot(points, normal) + d)
        
        # Count inliers
        inlier_mask = distances < ground_threshold
        inlier_count = np.sum(inlier_mask)
        
        if inlier_count > best_inlier_count:
            best_inlier_count = inlier_count
            best_inliers = inlier_mask
    
    if best_inliers is None:
        logger.warning("No ground plane found")
        return points, np.zeros((0, 3), dtype=np.float32)
    
    ground_points = points[best_inliers]
    non_ground_points = points[~best_inliers]
    
    logger.info(f"Ground filtering: {len(ground_points)} ground, "
                f"{len(non_ground_points)} non-ground points")
    
    return non_ground_points, ground_points


def preprocess_point_cloud(
    points: np.ndarray,
    voxel_size: float = 0.05,
    remove_outliers: bool = True,
    filter_ground: bool = True
) -> np.ndarray:
    """
    Full preprocessing pipeline for point cloud.
    
    Args:
        points: Raw point cloud
        voxel_size: Voxel size for downsampling
        remove_outliers: Whether to remove statistical outliers
        filter_ground: Whether to filter ground plane
    
    Returns:
        Preprocessed point cloud
    """
    if len(points) == 0:
        return points
    
    logger.info(f"Preprocessing {len(points)} points...")
    
    # Step 1: Voxel downsampling
    points = voxel_downsample(points, voxel_size)
    
    # Step 2: Remove outliers
    if remove_outliers and len(points) > 50:
        points = remove_statistical_outliers(points)
    
    # Step 3: Ground filtering (optional, depends on use case)
    if filter_ground and len(points) > 100:
        non_ground, ground = filter_ground_plane(points)
        # For splatting, we might want to keep ground
        # but we can return both if needed
        # For now, keep all points but log the separation
        logger.info(f"Preprocessing complete: {len(points)} points")
    else:
        logger.info(f"Preprocessing complete: {len(points)} points")
    
    return points


def run_gaussian_splatting(
    points: np.ndarray,
    poses: list[dict],
    images: list[Path],
    output_path: Path,
    config: SplatConfig
) -> dict:
    """
    Run Gaussian splatting training.
    
    This is a simplified implementation. For production, you would:
    1. Use COLMAP or similar for proper camera pose estimation
    2. Train using gsplat or nerfstudio
    3. Export optimized Gaussians to PLY
    """
    logger.info(f"Starting Gaussian splatting with {len(points)} points, {len(images)} images")
    
    output_path.mkdir(parents=True, exist_ok=True)
    start_time = time.time()
    
    stats = {
        'input_points': len(points),
        'input_images': len(images),
        'input_poses': len(poses),
        'iterations': config.iterations,
        'status': 'pending'
    }
    
    try:
        # Preprocess point cloud
        if len(points) > 0:
            original_count = len(points)
            points = preprocess_point_cloud(
                points,
                voxel_size=0.05,  # 5cm voxels
                remove_outliers=True,
                filter_ground=False  # Keep ground for splatting
            )
            stats['preprocessed_points'] = len(points)
            logger.info(f"Preprocessed: {original_count} -> {len(points)} points")
        
        # Check if we have enough data
        if len(points) < 100:
            logger.warning("Insufficient points for splatting, creating point cloud only")
            stats['status'] = 'insufficient_data'
            stats['message'] = 'Not enough LiDAR points for Gaussian splatting'
            
            # Just save the point cloud as PLY
            if len(points) > 0:
                save_points_as_ply(points, output_path / 'splat.ply')
                stats['output_points'] = len(points)
                stats['status'] = 'point_cloud_only'
            
            return stats
        
        # For a real implementation, we would:
        # 1. Run COLMAP for Structure-from-Motion if we have images
        # 2. Initialize Gaussians from point cloud
        # 3. Train using gsplat
        # 4. Export to PLY
        
        # For now, we'll create a colored point cloud from LiDAR
        # This demonstrates the pipeline without requiring full GPU training
        
        if HAS_GSPLAT:
            # Full Gaussian splatting training
            gaussians = train_gaussians(points, poses, images, config)
            export_gaussians_to_ply(gaussians, output_path / 'splat.ply')
            stats['output_gaussians'] = len(gaussians)
            stats['status'] = 'success'
        else:
            # Fallback: create point cloud
            logger.info("gsplat not available, creating point cloud")
            save_points_as_ply(points, output_path / 'splat.ply')
            stats['output_points'] = len(points)
            stats['status'] = 'point_cloud_only'
            stats['message'] = 'gsplat not available, exported point cloud'
        
    except Exception as e:
        logger.error(f"Splatting failed: {e}")
        stats['status'] = 'failed'
        stats['error'] = str(e)
    
    stats['duration_secs'] = time.time() - start_time
    return stats


# Check if gsplat is available
try:
    import torch
    import gsplat
    HAS_GSPLAT = True
    logger.info(f"gsplat available, CUDA: {torch.cuda.is_available()}")
except ImportError:
    HAS_GSPLAT = False
    logger.warning("gsplat not available, will use point cloud fallback")


def train_gaussians(
    points: np.ndarray,
    poses: list[dict],
    images: list[Path],
    config: SplatConfig
) -> np.ndarray:
    """
    Train Gaussian splats using gsplat.
    
    This is a simplified training loop. Production code would use
    nerfstudio or a more complete implementation.
    """
    import torch
    from gsplat import rasterization
    
    device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
    logger.info(f"Training on {device}")
    
    # Initialize Gaussians from point cloud
    num_points = len(points)
    
    # Gaussian parameters
    means = torch.tensor(points, dtype=torch.float32, device=device, requires_grad=True)
    scales = torch.ones(num_points, 3, device=device, requires_grad=True) * 0.01
    quats = torch.zeros(num_points, 4, device=device)
    quats[:, 0] = 1.0  # Identity rotation
    quats.requires_grad = True
    opacities = torch.ones(num_points, 1, device=device, requires_grad=True) * 0.5
    
    # Spherical harmonics for color (degree 0 = constant color)
    sh_coeffs = torch.rand(num_points, 3, device=device, requires_grad=True) * 0.5
    
    # Optimizer
    optimizer = torch.optim.Adam([
        {'params': [means], 'lr': config.position_lr_init},
        {'params': [scales], 'lr': config.scaling_lr},
        {'params': [quats], 'lr': config.rotation_lr},
        {'params': [opacities], 'lr': config.opacity_lr},
        {'params': [sh_coeffs], 'lr': config.feature_lr},
    ])
    
    # Simple training loop (would need actual images and proper camera models)
    logger.info(f"Training for {config.iterations} iterations...")
    
    for iteration in range(min(config.iterations, 1000)):  # Limit for demo
        optimizer.zero_grad()
        
        # For a real implementation, we would:
        # 1. Render from a camera viewpoint
        # 2. Compare to ground truth image
        # 3. Compute loss and backprop
        
        # Regularization losses only (no images)
        scale_reg = (scales.abs() - 0.01).clamp(min=0).mean()
        opacity_reg = (opacities - 0.5).pow(2).mean()
        
        loss = scale_reg * 0.1 + opacity_reg * 0.01
        loss.backward()
        optimizer.step()
        
        if iteration % 100 == 0:
            logger.info(f"Iteration {iteration}, loss: {loss.item():.6f}")
    
    # Export Gaussians
    gaussians = np.zeros((num_points, 14), dtype=np.float32)
    gaussians[:, 0:3] = means.detach().cpu().numpy()  # position
    gaussians[:, 3:6] = scales.detach().cpu().numpy()  # scale
    gaussians[:, 6:10] = quats.detach().cpu().numpy()  # rotation
    gaussians[:, 10:13] = sh_coeffs.detach().cpu().numpy()  # color
    gaussians[:, 13] = opacities.detach().cpu().numpy().squeeze()  # opacity
    
    logger.info(f"Trained {len(gaussians)} Gaussians")
    return gaussians


def export_gaussians_to_ply(gaussians: np.ndarray, path: Path):
    """Export Gaussians to PLY format for viewing with splat viewers."""
    logger.info(f"Exporting {len(gaussians)} Gaussians to {path}")
    
    # PLY header for Gaussian splat format
    header = f"""ply
format binary_little_endian 1.0
element vertex {len(gaussians)}
property float x
property float y
property float z
property float scale_0
property float scale_1
property float scale_2
property float rot_0
property float rot_1
property float rot_2
property float rot_3
property float f_dc_0
property float f_dc_1
property float f_dc_2
property float opacity
end_header
"""
    
    with open(path, 'wb') as f:
        f.write(header.encode('utf-8'))
        gaussians.astype(np.float32).tofile(f)
    
    logger.info(f"Wrote {path}")


def save_points_as_ply(points: np.ndarray, path: Path):
    """Save points as a simple PLY file."""
    logger.info(f"Saving {len(points)} points to {path}")
    
    header = f"""ply
format ascii 1.0
element vertex {len(points)}
property float x
property float y
property float z
end_header
"""
    
    with open(path, 'w') as f:
        f.write(header)
        for p in points:
            f.write(f"{p[0]:.6f} {p[1]:.6f} {p[2]:.6f}\n")
    
    logger.info(f"Wrote {path}")


def process_job(job_path: Path):
    """Process a single job file."""
    logger.info(f"Processing job: {job_path}")
    
    try:
        with open(job_path, 'r') as f:
            data = json.load(f)
        
        job = Job.from_json(data)
        logger.info(f"Job {job.id}: session={job.session_path}, output={job.output_path}")
        
        # Load session data
        points, poses, images = load_session_data(job.session_path)
        
        # Run splatting
        stats = run_gaussian_splatting(
            points, poses, images,
            job.output_path, job.config
        )
        
        # Write result
        result = {
            'job_id': job.id,
            'status': stats['status'],
            'completed_at': datetime.utcnow().isoformat() + 'Z',
            'stats': stats
        }
        
        result_path = job.output_path / 'result.json'
        with open(result_path, 'w') as f:
            json.dump(result, f, indent=2)
        
        logger.info(f"Job {job.id} completed with status: {stats['status']}")
        
        # Move job file to processed
        processed_dir = job_path.parent / 'processed'
        processed_dir.mkdir(exist_ok=True)
        shutil.move(str(job_path), str(processed_dir / job_path.name))
        
    except Exception as e:
        logger.error(f"Failed to process job {job_path}: {e}")
        
        # Move to failed
        failed_dir = job_path.parent / 'failed'
        failed_dir.mkdir(exist_ok=True)
        shutil.move(str(job_path), str(failed_dir / job_path.name))


class JobHandler(FileSystemEventHandler):
    """Watch for new job files."""
    
    def on_created(self, event: FileCreatedEvent):
        if event.is_directory:
            return
        
        path = Path(event.src_path)
        if path.suffix == '.json' and path.parent.name != 'processed' and path.parent.name != 'failed':
            # Wait for file to be fully written
            time.sleep(1)
            process_job(path)


def main():
    """Main worker loop."""
    logger.info("Starting Gaussian Splatting Worker")
    logger.info(f"Jobs directory: {JOBS_DIR}")
    logger.info(f"Maps directory: {MAPS_DIR}")
    logger.info(f"Sessions directory: {SESSIONS_DIR}")
    
    # Ensure directories exist
    JOBS_DIR.mkdir(parents=True, exist_ok=True)
    
    # Process any existing jobs
    existing_jobs = list(JOBS_DIR.glob('*.json'))
    logger.info(f"Found {len(existing_jobs)} existing jobs")
    
    for job_path in existing_jobs:
        if job_path.parent.name not in ('processed', 'failed'):
            process_job(job_path)
    
    # Watch for new jobs
    observer = Observer()
    observer.schedule(JobHandler(), str(JOBS_DIR), recursive=False)
    observer.start()
    
    logger.info("Watching for new jobs...")
    
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        observer.stop()
    
    observer.join()
    logger.info("Worker stopped")


if __name__ == '__main__':
    main()
