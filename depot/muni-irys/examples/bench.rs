//! Throughput of the K1 head camera at 544×448: depth-only and RGB, batch
//! 1 / 8 / 64, plus the per-step cost of moving every K1 (scene re-upload).
//!
//! ```text
//! cargo run --release --example bench -- [batches=1,8,64] [iters=10]
//! ```

use muni_irys::street::SIDEWALK_Z;
use muni_irys::{Eye, HeadCamera, K1Model, SimCamera, Street};
use std::collections::HashMap;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let batches: Vec<usize> = args
        .get(1)
        .map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect())
        .unwrap_or(vec![1, 8, 64]);
    let iters: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10);
    let k1 = K1Model::load_default()?;
    let street = Street::snowy(80.0);
    let cam = HeadCamera::K1;
    println!(
        "K1 head camera {}x{}, K1 {} tris, street {} boxes, iters {iters}",
        cam.width,
        cam.height,
        k1.triangles(),
        street.meshes.len()
    );
    println!("{:>5} {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}", "batch", "depth fps", "depth ms/b", "rgb1 fps", "rgb16 fps", "upload ms", "step ms");
    for &n in &batches {
        let poses = |phase: f64| -> Vec<_> {
            (0..n)
                .map(|i| {
                    let q = HashMap::from([("Head_pitch".to_string(), 0.2 + 0.05 * (i as f64 + phase).sin())]);
                    k1.standing(-3.0 + 0.1 * i as f64 % 6.0, 0.1, 0.0, 0.0, 0.0, q, SIDEWALK_Z)
                })
                .collect()
        };
        let mut sim = SimCamera::new(k1.clone(), &street, cam, poses(0.0))?;
        // Warm up pipelines.
        let _ = sim.depth(0, Eye::Left);
        let _ = sim.rgb(0, Eye::Left, 1, 1.0);

        let t = Instant::now();
        for _ in 0..iters {
            std::hint::black_box(sim.depth_batch(Eye::Left));
        }
        let depth_s = t.elapsed().as_secs_f64() / iters as f64;
        let mut gpu = 0.0;
        for e in 0..n {
            let _ = sim.depth(e, Eye::Left);
            gpu += sim.gpu_seconds.get();
        }
        let depth_gpu_ms = gpu / n as f64 * 1e3;
        let t = Instant::now();
        let d = sim.depth(0, Eye::Left);
        let _ = std::hint::black_box(d.points(&cam));
        let one_ms = t.elapsed().as_secs_f64() * 1e3;

        let t = Instant::now();
        for _ in 0..iters {
            std::hint::black_box(sim.rgb_batch(Eye::Left, 1, 1.0));
        }
        let rgb1_s = t.elapsed().as_secs_f64() / iters as f64;

        let it16 = (iters / 4).max(1);
        let t = Instant::now();
        for _ in 0..it16 {
            std::hint::black_box(sim.rgb_batch(Eye::Left, 16, 1.0));
        }
        let rgb16_s = t.elapsed().as_secs_f64() / it16 as f64;

        // A full sim step: every K1 moves, then depth for every env.
        let mut upload = 0.0;
        let t = Instant::now();
        for k in 0..iters {
            sim.set_poses(poses(k as f64 + 1.0))?;
            upload += sim.upload_seconds;
            std::hint::black_box(sim.depth_batch(Eye::Left));
        }
        let step_s = t.elapsed().as_secs_f64() / iters as f64;
        println!(
            "{:>5} {:>12.1} {:>12.2} {:>12.1} {:>12.1} {:>12.1} {:>12.1}",
            n,
            n as f64 / depth_s,
            depth_s * 1e3,
            n as f64 / rgb1_s,
            n as f64 / rgb16_s,
            upload / iters as f64 * 1e3,
            step_s * 1e3
        );
        println!("      depth per camera: dispatch {depth_gpu_ms:.2} ms of {:.2} ms total (one call + unprojection {one_ms:.2} ms)", depth_s / n as f64 * 1e3);
    }
    Ok(())
}
