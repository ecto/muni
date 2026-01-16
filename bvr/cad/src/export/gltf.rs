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

// =============================================================================
// Multi-material Scene export
// =============================================================================

use crate::Scene;
use super::Materials;

/// Export a scene with multiple parts and materials to GLB
pub fn export_scene_glb(
    scene: &Scene,
    materials_db: &Materials,
    path: impl AsRef<Path>,
) -> Result<(), CadError> {
    let glb_data = scene_to_glb_bytes(scene, materials_db)?;
    std::fs::write(path, glb_data)?;
    Ok(())
}

/// Convert a scene to GLB bytes with multiple meshes and materials
pub fn scene_to_glb_bytes(scene: &Scene, materials_db: &Materials) -> Result<Vec<u8>, CadError> {
    if scene.is_empty() {
        return Err(CadError::EmptyGeometry);
    }

    // Collect all unique materials used in the scene
    let mut material_indices: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut materials_list: Vec<Material> = Vec::new();

    for node in &scene.nodes {
        if !material_indices.contains_key(&node.material_key) {
            let mat = materials_db.get_for_part_or_default(&node.material_key);
            material_indices.insert(node.material_key.clone(), materials_list.len());
            materials_list.push(mat);
        }
    }

    // Build binary buffer for all meshes
    let mut bin_buffer: Vec<u8> = Vec::new();
    let mut buffer_views: Vec<String> = Vec::new();
    let mut accessors: Vec<String> = Vec::new();
    let mut meshes: Vec<String> = Vec::new();
    let mut nodes: Vec<String> = Vec::new();

    let mut accessor_idx = 0;
    let mut buffer_view_idx = 0;

    for (mesh_idx, node) in scene.nodes.iter().enumerate() {
        let mesh = node.part.to_mesh();
        let vertices = mesh.vertices();
        let indices = mesh.indices();

        if vertices.is_empty() || indices.is_empty() {
            continue;
        }

        let vertex_count = vertices.len() / 3;
        let index_count = indices.len();

        // Calculate bounds
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

        // Record byte offsets
        let indices_byte_offset = bin_buffer.len();
        let indices_byte_length = index_count * 4;

        // Write indices
        for &idx in &indices {
            bin_buffer.extend_from_slice(&idx.to_le_bytes());
        }

        // Pad to 4-byte alignment
        while bin_buffer.len() % 4 != 0 {
            bin_buffer.push(0);
        }

        let vertices_byte_offset = bin_buffer.len();
        let vertices_byte_length = vertex_count * 12;

        // Write vertices
        for &v in &vertices {
            bin_buffer.extend_from_slice(&v.to_le_bytes());
        }

        // Pad to 4-byte alignment
        while bin_buffer.len() % 4 != 0 {
            bin_buffer.push(0);
        }

        // Buffer views for this mesh
        buffer_views.push(format!(
            r#"{{ "buffer": 0, "byteOffset": {}, "byteLength": {}, "target": 34963 }}"#,
            indices_byte_offset, indices_byte_length
        ));
        let indices_bv = buffer_view_idx;
        buffer_view_idx += 1;

        buffer_views.push(format!(
            r#"{{ "buffer": 0, "byteOffset": {}, "byteLength": {}, "target": 34962 }}"#,
            vertices_byte_offset, vertices_byte_length
        ));
        let vertices_bv = buffer_view_idx;
        buffer_view_idx += 1;

        // Accessors for this mesh
        accessors.push(format!(
            r#"{{ "bufferView": {}, "componentType": 5125, "count": {}, "type": "SCALAR" }}"#,
            indices_bv, index_count
        ));
        let indices_acc = accessor_idx;
        accessor_idx += 1;

        accessors.push(format!(
            r#"{{ "bufferView": {}, "componentType": 5126, "count": {}, "type": "VEC3", "min": [{}, {}, {}], "max": [{}, {}, {}] }}"#,
            vertices_bv, vertex_count,
            min[0], min[1], min[2],
            max[0], max[1], max[2]
        ));
        let vertices_acc = accessor_idx;
        accessor_idx += 1;

        // Get material index for this node
        let mat_idx = material_indices.get(&node.material_key).copied().unwrap_or(0);

        // Mesh
        meshes.push(format!(
            r#"{{ "name": "{}", "primitives": [{{ "attributes": {{ "POSITION": {} }}, "indices": {}, "material": {} }}] }}"#,
            node.part.name, vertices_acc, indices_acc, mat_idx
        ));

        // Node
        nodes.push(format!(
            r#"{{ "mesh": {}, "name": "{}" }}"#,
            mesh_idx, node.part.name
        ));
    }

    if meshes.is_empty() {
        return Err(CadError::EmptyGeometry);
    }

    // Build materials JSON
    let materials_json: Vec<String> = materials_list.iter().map(|m| {
        format!(
            r#"{{ "name": "{}", "pbrMetallicRoughness": {{ "baseColorFactor": [{}, {}, {}, 1.0], "metallicFactor": {}, "roughnessFactor": {} }} }}"#,
            m.name, m.color[0], m.color[1], m.color[2], m.metallic, m.roughness
        )
    }).collect();

    // Build node indices for scene
    let node_indices: Vec<String> = (0..nodes.len()).map(|i| i.to_string()).collect();

    // Build JSON
    let json = format!(
        r#"{{
  "asset": {{ "version": "2.0", "generator": "bvr-cad" }},
  "scene": 0,
  "scenes": [{{ "name": "{}", "nodes": [{}] }}],
  "nodes": [{}],
  "meshes": [{}],
  "materials": [{}],
  "accessors": [{}],
  "bufferViews": [{}],
  "buffers": [{{ "byteLength": {} }}]
}}"#,
        scene.name,
        node_indices.join(", "),
        nodes.join(",\n    "),
        meshes.join(",\n    "),
        materials_json.join(",\n    "),
        accessors.join(",\n    "),
        buffer_views.join(",\n    "),
        bin_buffer.len()
    );

    let json_bytes = json.as_bytes();
    let json_padded_length = (json_bytes.len() + 3) & !3;
    let mut json_chunk = json_bytes.to_vec();
    while json_chunk.len() < json_padded_length {
        json_chunk.push(b' ');
    }

    let bin_padded_length = (bin_buffer.len() + 3) & !3;
    while bin_buffer.len() < bin_padded_length {
        bin_buffer.push(0);
    }

    // Build GLB
    let total_length = 12 + 8 + json_padded_length + 8 + bin_padded_length;
    let mut glb = Vec::with_capacity(total_length);

    // GLB header
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes());
    glb.extend_from_slice(&(total_length as u32).to_le_bytes());

    // JSON chunk
    glb.extend_from_slice(&(json_padded_length as u32).to_le_bytes());
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes());
    glb.extend_from_slice(&json_chunk);

    // BIN chunk
    glb.extend_from_slice(&(bin_padded_length as u32).to_le_bytes());
    glb.extend_from_slice(&0x004E4942u32.to_le_bytes());
    glb.extend_from_slice(&bin_buffer);

    Ok(glb)
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

    #[test]
    fn test_scene_glb_export() {
        let mut scene = Scene::new("test_scene");
        scene.add(Part::cube("cube1", 10.0, 10.0, 10.0), "aluminum_6061");
        scene.add(
            Part::cube("cube2", 5.0, 5.0, 5.0).translate(20.0, 0.0, 0.0),
            "aluminum_powder_orange"
        );

        let materials = Materials::parse(r#"
            [materials.aluminum_6061]
            color = [0.85, 0.85, 0.88]
            metallic = 0.95
            roughness = 0.35
            [materials.aluminum_powder_orange]
            color = [1.0, 0.4, 0.0]
            metallic = 0.3
            roughness = 0.6
        "#).unwrap();

        let glb_data = scene_to_glb_bytes(&scene, &materials).unwrap();

        // Check GLB magic
        assert_eq!(&glb_data[0..4], b"glTF");
    }
}
