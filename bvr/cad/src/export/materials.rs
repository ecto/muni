//! Material database for PBR rendering and physics simulation.

use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MaterialError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Material not found: {0}")]
    NotFound(String),
}

/// PBR material properties.
#[derive(Debug, Clone)]
pub struct Material {
    /// Material identifier
    pub name: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Base color [R, G, B] normalized 0.0-1.0
    pub color: [f32; 3],
    /// Metallic factor 0.0-1.0
    pub metallic: f32,
    /// Roughness factor 0.0-1.0
    pub roughness: f32,
    /// Density in kg/m³ (for mass calculation)
    pub density: f32,
    /// Coefficient of friction (for physics)
    pub friction: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: None,
            color: [0.5, 0.5, 0.5],
            metallic: 0.0,
            roughness: 0.5,
            density: 1000.0,
            friction: 0.5,
        }
    }
}

/// Material database loaded from TOML.
#[derive(Debug)]
pub struct Materials {
    materials: HashMap<String, Material>,
    part_materials: HashMap<String, String>,
}

impl Materials {
    /// Load materials from a TOML file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, MaterialError> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse materials from TOML string.
    pub fn parse(content: &str) -> Result<Self, MaterialError> {
        let value: toml::Value = toml::from_str(content)?;

        let mut materials = HashMap::new();
        let mut part_materials = HashMap::new();

        // Parse materials section
        if let Some(mats) = value.get("materials").and_then(|v| v.as_table()) {
            for (name, props) in mats {
                let mat = Self::parse_material(name, props)?;
                materials.insert(name.clone(), mat);
            }
        }

        // Parse part_materials section
        if let Some(parts) = value.get("part_materials").and_then(|v| v.as_table()) {
            for (part, mat_name) in parts {
                if let Some(mat_str) = mat_name.as_str() {
                    part_materials.insert(part.clone(), mat_str.to_string());
                }
            }
        }

        Ok(Self {
            materials,
            part_materials,
        })
    }

    fn parse_material(name: &str, props: &toml::Value) -> Result<Material, MaterialError> {
        let color = props
            .get("color")
            .and_then(|v| v.as_array())
            .map(|arr| {
                let mut c = [0.5f32; 3];
                for (i, val) in arr.iter().take(3).enumerate() {
                    c[i] = val.as_float().unwrap_or(0.5) as f32;
                }
                c
            })
            .unwrap_or([0.5, 0.5, 0.5]);

        let metallic = props
            .get("metallic")
            .and_then(|v| v.as_float())
            .unwrap_or(0.0) as f32;

        let roughness = props
            .get("roughness")
            .and_then(|v| v.as_float())
            .unwrap_or(0.5) as f32;

        let density = props
            .get("density")
            .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
            .unwrap_or(1000.0) as f32;

        let friction = props
            .get("friction")
            .and_then(|v| v.as_float())
            .unwrap_or(0.5) as f32;

        let description = props
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(Material {
            name: name.to_string(),
            description,
            color,
            metallic,
            roughness,
            density,
            friction,
        })
    }

    /// Get a material by name.
    pub fn get(&self, name: &str) -> Option<&Material> {
        self.materials.get(name)
    }

    /// Get the material assigned to a part.
    pub fn get_for_part(&self, part_name: &str) -> Option<&Material> {
        self.part_materials
            .get(part_name)
            .and_then(|mat_name| self.materials.get(mat_name))
    }

    /// Get material for a part, falling back to default.
    pub fn get_for_part_or_default(&self, part_name: &str) -> Material {
        self.get_for_part(part_name)
            .cloned()
            .unwrap_or_default()
    }

    /// List all material names.
    pub fn material_names(&self) -> impl Iterator<Item = &str> {
        self.materials.keys().map(|s| s.as_str())
    }

    /// List all part assignments.
    pub fn part_assignments(&self) -> impl Iterator<Item = (&str, &str)> {
        self.part_materials
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_materials() {
        let toml = r#"
[materials.aluminum]
color = [0.8, 0.8, 0.85]
metallic = 0.9
roughness = 0.3
density = 2700

[materials.rubber]
color = [0.1, 0.1, 0.1]
metallic = 0.0
roughness = 0.9
density = 1100
friction = 0.8

[part_materials]
frame = "aluminum"
tire = "rubber"
"#;

        let mats = Materials::parse(toml).unwrap();

        let aluminum = mats.get("aluminum").unwrap();
        assert!((aluminum.metallic - 0.9).abs() < 0.01);
        assert!((aluminum.density - 2700.0).abs() < 1.0);

        let rubber = mats.get("rubber").unwrap();
        assert!((rubber.friction - 0.8).abs() < 0.01);

        let frame_mat = mats.get_for_part("frame").unwrap();
        assert_eq!(frame_mat.name, "aluminum");
    }
}
