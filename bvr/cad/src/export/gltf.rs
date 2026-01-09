//! glTF/GLB export for visualization.
//!
//! Generates binary GLB files with PBR materials for web and app rendering.

use crate::{CadError, Part};
use super::materials::Material;
use std::path::Path;

/// Export a part to binary GLB format with PBR material.
pub fn export_glb(
    part: &Part,
    material: &Material,
    path: impl AsRef<Path>,
) -> Result<(), CadError> {
    let glb_data = to_glb_bytes(part, material)?;
    std::fs::write(path, glb_data)?;
    Ok(())
}

/// Convert a part to binary GLB bytes.
pub fn to_glb_bytes(part: &Part, material: &Material) -> Result<Vec<u8>, CadError> {
    let mesh = part.to_mesh();
    let vertices = mesh.vertices();
    let indices = mesh.indices();

    if vertices.is_empty() || indices.is_empty() {
        return Err(CadError::EmptyGeometry);
    }

    // Build the GLB using the gltf crate's JSON structures
    // GLB format: 12-byte header + JSON chunk + BIN chunk

    // Prepare vertex data (positions only for now)
    let vertex_count = vertices.len() / 3;
    let index_count = indices.len();

    // Calculate bounds for accessor
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for i in 0..vertex_count {
        let x = vertices[i * 3];
        let y = vertices[i * 3 + 1];
        let z = vertices[i * 3 + 2];
        min[0] = min[0].min(x);
        min[1] = min[1].min(y);
        min[2] = min[2].min(z);
        max[0] = max[0].max(x);
        max[1] = max[1].max(y);
        max[2] = max[2].max(z);
    }

    // Build binary buffer: indices (u32) + vertices (f32 * 3)
    let indices_byte_length = index_count * 4;
    let vertices_byte_length = vertex_count * 12;
    let total_buffer_length = indices_byte_length + vertices_byte_length;

    // Pad to 4-byte alignment
    let padded_buffer_length = (total_buffer_length + 3) & !3;

    let mut bin_buffer = Vec::with_capacity(padded_buffer_length);

    // Write indices as u32
    for &idx in &indices {
        bin_buffer.extend_from_slice(&idx.to_le_bytes());
    }

    // Write vertices as f32
    for &v in &vertices {
        bin_buffer.extend_from_slice(&v.to_le_bytes());
    }

    // Pad buffer
    while bin_buffer.len() < padded_buffer_length {
        bin_buffer.push(0);
    }

    // Build JSON
    let json = build_gltf_json(
        &part.name,
        material,
        vertex_count,
        index_count,
        indices_byte_length,
        vertices_byte_length,
        padded_buffer_length,
        &min,
        &max,
    );

    let json_bytes = json.as_bytes();
    let json_padded_length = (json_bytes.len() + 3) & !3;
    let mut json_chunk = json_bytes.to_vec();
    while json_chunk.len() < json_padded_length {
        json_chunk.push(b' '); // Pad with spaces
    }

    // Build GLB
    let total_length = 12 + 8 + json_padded_length + 8 + padded_buffer_length;
    let mut glb = Vec::with_capacity(total_length);

    // GLB header
    glb.extend_from_slice(b"glTF"); // magic
    glb.extend_from_slice(&2u32.to_le_bytes()); // version
    glb.extend_from_slice(&(total_length as u32).to_le_bytes()); // length

    // JSON chunk
    glb.extend_from_slice(&(json_padded_length as u32).to_le_bytes()); // chunk length
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // chunk type "JSON"
    glb.extend_from_slice(&json_chunk);

    // BIN chunk
    glb.extend_from_slice(&(padded_buffer_length as u32).to_le_bytes()); // chunk length
    glb.extend_from_slice(&0x004E4942u32.to_le_bytes()); // chunk type "BIN\0"
    glb.extend_from_slice(&bin_buffer);

    Ok(glb)
}

fn build_gltf_json(
    name: &str,
    material: &Material,
    vertex_count: usize,
    index_count: usize,
    indices_byte_length: usize,
    vertices_byte_length: usize,
    buffer_length: usize,
    min: &[f32; 3],
    max: &[f32; 3],
) -> String {
    // Build JSON manually for control over output
    format!(
        r#"{{
  "asset": {{ "version": "2.0", "generator": "bvr-cad" }},
  "scene": 0,
  "scenes": [{{ "nodes": [0] }}],
  "nodes": [{{ "mesh": 0, "name": "{name}" }}],
  "meshes": [{{
    "name": "{name}",
    "primitives": [{{
      "attributes": {{ "POSITION": 1 }},
      "indices": 0,
      "material": 0
    }}]
  }}],
  "materials": [{{
    "name": "{mat_name}",
    "pbrMetallicRoughness": {{
      "baseColorFactor": [{r}, {g}, {b}, 1.0],
      "metallicFactor": {metallic},
      "roughnessFactor": {roughness}
    }}
  }}],
  "accessors": [
    {{
      "bufferView": 0,
      "componentType": 5125,
      "count": {index_count},
      "type": "SCALAR"
    }},
    {{
      "bufferView": 1,
      "componentType": 5126,
      "count": {vertex_count},
      "type": "VEC3",
      "min": [{min0}, {min1}, {min2}],
      "max": [{max0}, {max1}, {max2}]
    }}
  ],
  "bufferViews": [
    {{
      "buffer": 0,
      "byteOffset": 0,
      "byteLength": {indices_byte_length},
      "target": 34963
    }},
    {{
      "buffer": 0,
      "byteOffset": {indices_byte_length},
      "byteLength": {vertices_byte_length},
      "target": 34962
    }}
  ],
  "buffers": [{{ "byteLength": {buffer_length} }}]
}}"#,
        name = name,
        mat_name = material.name,
        r = material.color[0],
        g = material.color[1],
        b = material.color[2],
        metallic = material.metallic,
        roughness = material.roughness,
        index_count = index_count,
        vertex_count = vertex_count,
        min0 = min[0],
        min1 = min[1],
        min2 = min[2],
        max0 = max[0],
        max1 = max[1],
        max2 = max[2],
        indices_byte_length = indices_byte_length,
        vertices_byte_length = vertices_byte_length,
        buffer_length = buffer_length,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glb_export() {
        let cube = Part::cube("test_cube", 10.0, 10.0, 10.0);
        let material = Material::default();
        let glb_data = to_glb_bytes(&cube, &material).unwrap();

        // Check GLB magic
        assert_eq!(&glb_data[0..4], b"glTF");

        // Check version
        let version = u32::from_le_bytes([glb_data[4], glb_data[5], glb_data[6], glb_data[7]]);
        assert_eq!(version, 2);
    }
}
