//! Depth is metric: pixels that see the sidewalk back-project onto its plane.
//!
//! Needs the K1 assets (`MUNI_K1_DIR`) and a GPU with ray queries; skips
//! (passes with a note) when either is missing, so CI without them is green.

use muni_irys::street::SIDEWALK_Z;
use muni_irys::{Eye, HeadCamera, K1Model, SimCamera, Street};
use std::collections::HashMap;

#[test]
fn sidewalk_pixels_land_on_the_sidewalk() {
    let Ok(k1) = K1Model::load_default() else {
        eprintln!("skipping: K1 assets not found");
        return;
    };
    let pose = k1.standing(0.0, 0.1, 0.0, 0.0, 0.0, HashMap::from([("Head_pitch".into(), 0.5)]), SIDEWALK_Z);
    let cam = HeadCamera::K1.scaled(136, 112);
    let sim = match SimCamera::new(k1, &Street::snowy(80.0), cam, vec![pose]) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("skipping: {e}");
            return;
        }
    };
    let c = sim.camera(0, Eye::Left);
    let d = sim.depth(0, Eye::Left);
    let (w, h) = (d.width as usize, d.height as usize);
    let f = cam.f;
    let mut checked = 0;
    // Centre columns, lower third: the cleared strip of sidewalk.
    for v in (h * 2 / 3)..h {
        for u in (w / 2 - 4)..(w / 2 + 4) {
            let z = d.z[v * w + u] as f64;
            assert!(z > 0.0, "no return at ({u},{v})");
            let (x, y) = ((u as f64 + 0.5 - w as f64 / 2.0) * z / f, (v as f64 + 0.5 - h as f64 / 2.0) * z / f);
            let p: Vec<f64> = (0..3).map(|i| c.eye[i] + c.forward[i] * z + c.right[i] * x - c.up[i] * y).collect();
            assert!((p[2] - SIDEWALK_Z).abs() < 0.01, "({u},{v}) lands at z = {:.4}", p[2]);
            checked += 1;
        }
    }
    assert!(checked > 100);
}
