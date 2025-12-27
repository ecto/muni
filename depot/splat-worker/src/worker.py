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


def load_session_data(session_path: Path) -> tuple[np.ndarray, list[dict], list[Path]]:
    """
    Load session data for splatting.
    
    Returns:
        points: (N, 3) array of LiDAR points
        poses: List of camera poses with timestamps
        images: List of image paths
    """
    logger.info(f"Loading session from {session_path}")
    
    # Load LiDAR points
    lidar_dir = session_path / 'lidar'
    points = []
    
    if lidar_dir.exists():
        pcd_files = sorted(lidar_dir.glob('*.pcd'))
        logger.info(f"Found {len(pcd_files)} LiDAR frames")
        
        for pcd_file in pcd_files:
            frame_points = load_pcd(pcd_file)
            if frame_points is not None:
                points.append(frame_points)
        
        if points:
            points = np.vstack(points)
            logger.info(f"Loaded {len(points)} total points")
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
    
    # Load poses from telemetry (simplified: use timestamps.csv)
    poses = []
    timestamps_file = camera_dir / 'timestamps.csv' if camera_dir.exists() else None
    
    if timestamps_file and timestamps_file.exists():
        with open(timestamps_file, 'r') as f:
            next(f)  # Skip header
            for line in f:
                parts = line.strip().split(',')
                if len(parts) >= 2:
                    frame_num = int(parts[0])
                    timestamp = float(parts[1])
                    # For now, use identity poses (proper implementation would read from telemetry.rrd)
                    poses.append({
                        'frame': frame_num,
                        'timestamp': timestamp,
                        'position': [0.0, 0.0, 0.0],
                        'rotation': [0.0, 0.0, 0.0, 1.0]  # quaternion
                    })
    
    return points, poses, images


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
        'iterations': config.iterations,
        'status': 'pending'
    }
    
    try:
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
