//! The K1 walking a snowy sidewalk, seen from its own head camera and from
//! a chase camera. Writes per-frame depth + RGB PNGs and a side-by-side.
//!
//! ```text
//! cargo run --release --example demo -- out/demo [frames] [spp] [trace] [odom]
//! ```
//!
//! With a trace (an ipse `*.trace`), joints and IMU tilt come from it and
//! the body stays put unless the fifth argument is `odom` (a seated trace's
//! odometer drifts metres); without a trace, a synthetic gait walks it.

use muni_irys::street::SIDEWALK_Z;
use muni_irys::trace;
use muni_irys::{Eye, HeadCamera, K1Model, SimCamera, Street};
use std::collections::HashMap;
use std::path::PathBuf;

fn gait(t: f64) -> HashMap<String, f64> {
    let w = std::f64::consts::TAU * 1.2;
    let s = (w * t).sin();
    let q = |n: &str, v: f64| (n.to_string(), v);
    HashMap::from([
        q("Left_Hip_Pitch", -0.35 + 0.3 * s),
        q("Right_Hip_Pitch", -0.35 - 0.3 * s),
        q("Left_Knee_Pitch", 0.7 + 0.25 * (w * t).cos().max(0.0)),
        q("Right_Knee_Pitch", 0.7 + 0.25 * (-(w * t).cos()).max(0.0)),
        q("Left_Ankle_Pitch", -0.35 - 0.1 * s),
        q("Right_Ankle_Pitch", -0.35 + 0.1 * s),
        q("ALeft_Shoulder_Pitch", 0.25 * s),
        q("ARight_Shoulder_Pitch", -0.25 * s),
        q("Left_Shoulder_Roll", -1.3),
        q("Right_Shoulder_Roll", 1.3),
        q("Head_pitch", 0.25),
    ])
}

fn save_rgb(path: &std::path::Path, w: u32, h: u32, rgb: &[u8]) -> anyhow::Result<()> {
    image::RgbImage::from_raw(w, h, rgb.to_vec()).unwrap().save(path)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let out = PathBuf::from(args.get(1).map(String::as_str).unwrap_or("out/demo"));
    let frames: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(12);
    let spp: u32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(32);
    // A trace plays in real time from its start, odometry made relative to
    // its first sample so it starts where the synthetic gait does.
    let samples = match args.get(4) {
        Some(p) => {
            let mut s = trace::read(std::path::Path::new(p))?;
            anyhow::ensure!(!s.is_empty(), "trace has no low_state records");
            let (t0, o0) = (s[0].t, s[0].odom);
            let odom = args.get(5).map(String::as_str) == Some("odom");
            for x in &mut s {
                x.t -= t0;
                for k in 0..3 {
                    x.odom[k] = if odom { x.odom[k] - o0[k] } else { 0.0 };
                }
            }
            println!("trace: {} samples over {:.1} s", s.len(), s.last().unwrap().t);
            Some(s)
        }
        None => None,
    };
    std::fs::create_dir_all(&out)?;

    let k1 = K1Model::load_default()?;
    println!("K1: {} meshes, {} triangles", k1.meshes.len(), k1.triangles());
    let street = Street::snowy(80.0);
    let cam = HeadCamera::K1;
    let pose_at = |k1: &K1Model, f: usize| {
        let t = f as f64 / cam.rgb_hz;
        match &samples {
            Some(s) => {
                let i = s.partition_point(|x| x.t < t).min(s.len() - 1);
                k1.from_sample(&s[i], [-3.0, 0.1, 0.0], SIDEWALK_Z)
            }
            None => k1.standing(-3.0 + 0.4 * t, 0.1, 0.0, 0.0, 0.0, gait(t), SIDEWALK_Z),
        }
    };
    let mut sim = SimCamera::new(k1.clone(), &street, cam, vec![pose_at(&k1, 0)])?;
    // A chase camera at the same resolution, for the side-by-side.
    let chase = HeadCamera { mount: [-1.6, 0.9, 0.25], ..cam };
    let mut strip = Vec::new();
    for f in 0..frames {
        sim.set_poses(vec![pose_at(&k1, f)])?;
        let d = sim.depth(0, Eye::Left);
        let rgb = sim.rgb(0, Eye::Left, spp, 1.0);
        let gray = d.to_gray8(12.0);
        image::GrayImage::from_raw(d.width, d.height, gray.clone()).unwrap().save(out.join(format!("depth_{f:03}.png")))?;
        let mm: Vec<u16> = d.z.iter().map(|&z| (z * 1000.0).min(65535.0) as u16).collect();
        image::ImageBuffer::<image::Luma<u16>, _>::from_raw(d.width, d.height, mm)
            .unwrap()
            .save(out.join(format!("depth_mm_{f:03}.png")))?;
        save_rgb(&out.join(format!("rgb_{f:03}.png")), rgb.width, rgb.height, &rgb.rgb)?;
        let valid = d.z.iter().filter(|&&z| z > 0.0).count() as f64 / d.z.len() as f64;
        println!("frame {f}: coverage {:.2}, upload {:.1} ms", valid, sim.upload_seconds * 1e3);
        if f == frames / 2 {
            // Chase view: look back at the robot from behind and to its left.
            let head = sim.k1().head(&sim.poses()[0]);
            let mut c = chase.pose(&head, Eye::Centre);
            let target = head.point([0.0, 0.0, -0.4]);
            let fw: Vec<f64> = (0..3).map(|i| target[i] - c.eye[i]).collect();
            let n = (fw[0] * fw[0] + fw[1] * fw[1] + fw[2] * fw[2]).sqrt();
            c.forward = [fw[0] / n, fw[1] / n, fw[2] / n];
            let r = [c.forward[1], -c.forward[0], 0.0];
            let rn = (r[0] * r[0] + r[1] * r[1]).sqrt();
            c.right = [r[0] / rn, r[1] / rn, 0.0];
            c.up = [
                c.right[1] * c.forward[2] - c.right[2] * c.forward[1],
                c.right[2] * c.forward[0] - c.right[0] * c.forward[2],
                c.right[0] * c.forward[1] - c.right[1] * c.forward[0],
            ];
            c.tan_half = 0.55;
            let chase_rgb = sim.render_rgb_from(&c, spp, 1.0);
            strip = vec![chase_rgb.rgb, rgb.rgb.clone(), gray.iter().flat_map(|&g| [g, g, g]).collect()];
        }
    }
    // Side-by-side: chase | head RGB | head depth.
    let (w, h) = (cam.width, cam.height);
    let mut side = image::RgbImage::new(w * 3 + 16, h);
    for (k, img) in strip.iter().enumerate() {
        let src = image::RgbImage::from_raw(w, h, img.clone()).unwrap();
        image::imageops::overlay(&mut side, &src, (k as u32 * (w + 8)) as i64, 0);
    }
    side.save(out.join("side_by_side.png"))?;
    println!("wrote {}", out.display());
    Ok(())
}
