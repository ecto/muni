//! STL (binary and ASCII) into an indexed triangle list.

use crate::Error;
use std::collections::HashMap;
use std::path::Path;

/// Indexed triangles: welded positions and corner indices.
#[derive(Debug, Clone, Default)]
pub struct Tris {
    /// Positions.
    pub positions: Vec<[f32; 3]>,
    /// Corners, three per triangle.
    pub indices: Vec<u32>,
}

impl Tris {
    fn push(&mut self, weld: &mut HashMap<[u32; 3], u32>, p: [f32; 3]) {
        let key = p.map(f32::to_bits);
        let i = *weld.entry(key).or_insert_with(|| {
            self.positions.push(p);
            (self.positions.len() - 1) as u32
        });
        self.indices.push(i);
    }
}

/// Read an STL file, welding identical vertices.
pub fn load(path: &Path) -> Result<Tris, Error> {
    let bytes = std::fs::read(path).map_err(|e| Error::Io(path.into(), e))?;
    let bad = |m: &str| Error::Stl(path.into(), m.into());
    let mut out = Tris::default();
    let mut weld = HashMap::new();
    let binary_len = |n: usize| 84 + n * 50;
    let n = if bytes.len() >= 84 { u32::from_le_bytes(bytes[80..84].try_into().unwrap()) as usize } else { 0 };
    if bytes.len() >= 84 && binary_len(n) == bytes.len() {
        let f = |o: usize| f32::from_le_bytes(bytes[o..o + 4].try_into().unwrap());
        for t in 0..n {
            let o = 84 + t * 50 + 12;
            for c in 0..3 {
                let b = o + c * 12;
                out.push(&mut weld, [f(b), f(b + 4), f(b + 8)]);
            }
        }
        return Ok(out);
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| bad("neither binary nor ASCII"))?;
    for line in text.lines() {
        let mut w = line.split_whitespace();
        if w.next() == Some("vertex") {
            let v: Vec<f32> = w.filter_map(|x| x.parse().ok()).collect();
            if v.len() != 3 {
                return Err(bad("vertex without three floats"));
            }
            out.push(&mut weld, [v[0], v[1], v[2]]);
        }
    }
    if out.indices.len() % 3 != 0 || out.indices.is_empty() {
        return Err(bad("no triangles"));
    }
    Ok(out)
}
