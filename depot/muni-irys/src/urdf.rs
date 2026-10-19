//! A URDF reader and forward kinematics, enough for the Booster K1.
//!
//! Only what a camera needs: links, their `<visual>` meshes and colours, and
//! revolute/continuous/prismatic/fixed joints. Inertia and collision are
//! ignored. The robot is a tree rooted at the one link that is nobody's child.

use crate::xf::Xf;
use crate::Error;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// One `<visual>` of a link.
#[derive(Debug, Clone)]
pub struct Visual {
    /// Mesh file, resolved against the URDF's directory.
    pub mesh: PathBuf,
    /// Mesh scale.
    pub scale: [f64; 3],
    /// Link → mesh.
    pub origin: Xf,
    /// `<material><color rgba>`, linear-ish RGB as authored.
    pub rgb: [f64; 3],
}

/// One link.
#[derive(Debug, Clone)]
pub struct Link {
    /// Name.
    pub name: String,
    /// Visuals.
    pub visuals: Vec<Visual>,
}

/// Joint kinds this reader moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JointKind {
    /// Rotates about `axis`.
    Revolute,
    /// Slides along `axis`.
    Prismatic,
    /// Does not move.
    Fixed,
}

/// One joint.
#[derive(Debug, Clone)]
pub struct Joint {
    /// Name, as in the URDF (and in ipse's `K1_SERIAL_JOINTS`).
    pub name: String,
    /// Kind.
    pub kind: JointKind,
    /// Parent link index.
    pub parent: usize,
    /// Child link index.
    pub child: usize,
    /// Parent link → joint frame at zero.
    pub origin: Xf,
    /// Axis in the joint frame.
    pub axis: [f64; 3],
}

/// A parsed robot.
#[derive(Debug, Clone)]
pub struct Robot {
    /// Robot name.
    pub name: String,
    /// Links, in file order.
    pub links: Vec<Link>,
    /// Joints, topologically sorted (a parent's joint before its child's).
    pub joints: Vec<Joint>,
    /// The root link.
    pub root: usize,
}

fn floats<const N: usize>(s: Option<&str>, default: [f64; N]) -> [f64; N] {
    let Some(s) = s else { return default };
    let mut out = default;
    for (o, v) in out.iter_mut().zip(s.split_whitespace()) {
        *o = v.parse().unwrap_or(0.0);
    }
    out
}

fn origin_of(node: roxmltree::Node) -> Xf {
    node.children()
        .find(|c| c.has_tag_name("origin"))
        .map(|o| Xf::from_xyz_rpy(floats(o.attribute("xyz"), [0.0; 3]), floats(o.attribute("rpy"), [0.0; 3])))
        .unwrap_or(Xf::IDENTITY)
}

impl Robot {
    /// Parse `path`. Mesh filenames resolve against the URDF's directory;
    /// `package://` prefixes are stripped to their last two components.
    pub fn load(path: &Path) -> Result<Self, Error> {
        let text = std::fs::read_to_string(path).map_err(|e| Error::Io(path.into(), e))?;
        let dir = path.parent().unwrap_or(Path::new("."));
        Self::parse(&text, dir)
    }

    /// Parse URDF text.
    pub fn parse(text: &str, dir: &Path) -> Result<Self, Error> {
        let doc = roxmltree::Document::parse(text).map_err(|e| Error::Urdf(e.to_string()))?;
        let robot = doc.root_element();
        let mut links = Vec::new();
        let mut index = HashMap::new();
        for l in robot.children().filter(|c| c.has_tag_name("link")) {
            let name = l.attribute("name").unwrap_or("").to_string();
            let mut visuals = Vec::new();
            for v in l.children().filter(|c| c.has_tag_name("visual")) {
                let Some(mesh) = v
                    .descendants()
                    .find(|c| c.has_tag_name("mesh"))
                else {
                    continue;
                };
                let file = mesh.attribute("filename").unwrap_or("");
                let file = match file.strip_prefix("package://") {
                    Some(rest) => {
                        let parts: Vec<&str> = rest.rsplitn(3, '/').collect();
                        format!("{}/{}", parts.get(1).unwrap_or(&""), parts[0])
                    }
                    None => file.to_string(),
                };
                let rgb = v
                    .descendants()
                    .find(|c| c.has_tag_name("color"))
                    .map(|c| {
                        let f = floats(c.attribute("rgba"), [0.7, 0.7, 0.7, 1.0]);
                        [f[0], f[1], f[2]]
                    })
                    .unwrap_or([0.7; 3]);
                visuals.push(Visual {
                    mesh: dir.join(file),
                    scale: floats(mesh.attribute("scale"), [1.0; 3]),
                    origin: origin_of(v),
                    rgb,
                });
            }
            index.insert(name.clone(), links.len());
            links.push(Link { name, visuals });
        }
        let mut joints = Vec::new();
        for j in robot.children().filter(|c| c.has_tag_name("joint")) {
            let kind = match j.attribute("type").unwrap_or("fixed") {
                "revolute" | "continuous" => JointKind::Revolute,
                "prismatic" => JointKind::Prismatic,
                _ => JointKind::Fixed,
            };
            let link_of = |tag: &str| -> Result<usize, Error> {
                let n = j
                    .children()
                    .find(|c| c.has_tag_name(tag))
                    .and_then(|c| c.attribute("link"))
                    .ok_or_else(|| Error::Urdf(format!("joint without {tag}")))?;
                index.get(n).copied().ok_or_else(|| Error::Urdf(format!("unknown link {n}")))
            };
            joints.push(Joint {
                name: j.attribute("name").unwrap_or("").to_string(),
                kind,
                parent: link_of("parent")?,
                child: link_of("child")?,
                origin: origin_of(j),
                axis: j
                    .children()
                    .find(|c| c.has_tag_name("axis"))
                    .map(|a| floats(a.attribute("xyz"), [1.0, 0.0, 0.0]))
                    .unwrap_or([1.0, 0.0, 0.0]),
            });
        }
        let mut is_child = vec![false; links.len()];
        for j in &joints {
            is_child[j.child] = true;
        }
        let root = is_child
            .iter()
            .position(|c| !c)
            .ok_or_else(|| Error::Urdf("no root link".into()))?;
        // Topological order: breadth first from the root.
        let mut sorted = Vec::with_capacity(joints.len());
        let mut frontier = vec![root];
        while let Some(l) = frontier.pop() {
            for j in joints.iter().filter(|j| j.parent == l) {
                frontier.push(j.child);
                sorted.push(j.clone());
            }
        }
        Ok(Self {
            name: robot.attribute("name").unwrap_or("").to_string(),
            links,
            joints: sorted,
            root,
        })
    }

    /// Link index by name.
    pub fn link(&self, name: &str) -> Option<usize> {
        self.links.iter().position(|l| l.name == name)
    }

    /// Names of the moving joints, in topological order.
    pub fn joint_names(&self) -> Vec<&str> {
        self.joints
            .iter()
            .filter(|j| j.kind != JointKind::Fixed)
            .map(|j| j.name.as_str())
            .collect()
    }

    /// World pose of every link, given the root's pose and joint angles by
    /// name. A joint missing from `q` sits at zero.
    pub fn fk(&self, base: &Xf, q: &HashMap<String, f64>) -> Vec<Xf> {
        let mut out = vec![Xf::IDENTITY; self.links.len()];
        out[self.root] = *base;
        for j in &self.joints {
            let a = q.get(&j.name).copied().unwrap_or(0.0);
            let motion = match j.kind {
                JointKind::Revolute => Xf::axis_angle(j.axis, a),
                JointKind::Prismatic => Xf::translation([j.axis[0] * a, j.axis[1] * a, j.axis[2] * a]),
                JointKind::Fixed => Xf::IDENTITY,
            };
            out[j.child] = out[j.parent].then(&j.origin).then(&motion);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO: &str = r#"<robot name="r">
      <link name="a"/>
      <link name="b"><visual><origin xyz="0 0 1"/><geometry><mesh filename="m.stl"/></geometry></visual></link>
      <joint name="j" type="revolute"><origin xyz="1 0 0"/><parent link="a"/><child link="b"/><axis xyz="0 0 1"/></joint>
    </robot>"#;

    #[test]
    fn fk_rotates_child() {
        let r = Robot::parse(TWO, Path::new("/x")).unwrap();
        assert_eq!(r.root, 0);
        let q = HashMap::from([("j".to_string(), std::f64::consts::FRAC_PI_2)]);
        let x = r.fk(&Xf::IDENTITY, &q);
        let p = x[1].point([1.0, 0.0, 0.0]);
        assert!((p[0] - 1.0).abs() < 1e-12 && (p[1] - 1.0).abs() < 1e-12);
        assert_eq!(r.links[1].visuals[0].mesh, Path::new("/x/m.stl"));
    }
}
