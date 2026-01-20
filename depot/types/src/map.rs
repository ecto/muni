//! Map types for depot services.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::geo::GpsBounds;

/// Map metadata manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapManifest {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub bounds: GpsBounds,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub assets: MapAssets,
    pub sessions: Vec<MapSessionRef>,
    pub stats: MapStats,
}

impl MapManifest {
    /// Create a new map manifest.
    pub fn new(name: String, bounds: GpsBounds) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            bounds,
            version: 1,
            created_at: now,
            updated_at: now,
            assets: MapAssets::default(),
            sessions: Vec::new(),
            stats: MapStats::default(),
        }
    }

    /// Update the manifest version and timestamp.
    pub fn bump_version(&mut self) {
        self.version += 1;
        self.updated_at = Utc::now();
    }
}

/// Map asset paths (relative to map directory).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MapAssets {
    /// Gaussian splat file (.ply or .splat)
    pub splat: Option<String>,
    /// Point cloud file (.pcd or .las)
    pub pointcloud: Option<String>,
    /// Mesh file (.obj or .glb)
    pub mesh: Option<String>,
    /// Thumbnail image
    pub thumbnail: Option<String>,
}

impl MapAssets {
    /// Check if the map has any processed assets.
    pub fn has_any(&self) -> bool {
        self.splat.is_some()
            || self.pointcloud.is_some()
            || self.mesh.is_some()
            || self.thumbnail.is_some()
    }

    /// Check if the map has a splat (primary 3D representation).
    pub fn has_splat(&self) -> bool {
        self.splat.is_some()
    }
}

/// Reference to a session that contributed to a map.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapSessionRef {
    pub session_id: Uuid,
    pub rover_id: String,
    pub date: DateTime<Utc>,
}

/// Map statistics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MapStats {
    /// Total number of LiDAR points
    pub total_points: u64,
    /// Total number of Gaussian splats
    pub total_splats: u64,
    /// Estimated coverage percentage (0-100)
    pub coverage_pct: f32,
}

/// Map index (list of all maps).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MapIndex {
    pub maps: Vec<MapIndexEntry>,
    pub updated_at: DateTime<Utc>,
}

impl MapIndex {
    /// Create a new empty index.
    pub fn new() -> Self {
        Self {
            maps: Vec::new(),
            updated_at: Utc::now(),
        }
    }

    /// Add or update a map entry.
    pub fn upsert(&mut self, entry: MapIndexEntry) {
        if let Some(existing) = self.maps.iter_mut().find(|m| m.id == entry.id) {
            *existing = entry;
        } else {
            self.maps.push(entry);
        }
        self.updated_at = Utc::now();
    }

    /// Remove a map entry by ID.
    pub fn remove(&mut self, id: Uuid) -> bool {
        let len_before = self.maps.len();
        self.maps.retain(|m| m.id != id);
        if self.maps.len() != len_before {
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }
}

/// Entry in the map index (summary info).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapIndexEntry {
    pub id: Uuid,
    pub name: String,
    pub bounds: GpsBounds,
    pub version: u32,
    pub updated_at: DateTime<Utc>,
}

impl From<&MapManifest> for MapIndexEntry {
    fn from(manifest: &MapManifest) -> Self {
        Self {
            id: manifest.id,
            name: manifest.name.clone(),
            bounds: manifest.bounds.clone(),
            version: manifest.version,
            updated_at: manifest.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_manifest_new() {
        let bounds = GpsBounds::new(42.0, 43.0, -72.0, -71.0);
        let manifest = MapManifest::new("Test Map".to_string(), bounds.clone());

        assert_eq!(manifest.name, "Test Map");
        assert_eq!(manifest.version, 1);
        assert_eq!(manifest.bounds, bounds);
        assert!(manifest.sessions.is_empty());
    }

    #[test]
    fn test_map_manifest_bump_version() {
        let bounds = GpsBounds::new(42.0, 43.0, -72.0, -71.0);
        let mut manifest = MapManifest::new("Test Map".to_string(), bounds);
        let original_updated = manifest.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        manifest.bump_version();

        assert_eq!(manifest.version, 2);
        assert!(manifest.updated_at > original_updated);
    }

    #[test]
    fn test_map_assets_has_any() {
        let empty = MapAssets::default();
        assert!(!empty.has_any());

        let with_splat = MapAssets {
            splat: Some("map.splat".to_string()),
            ..Default::default()
        };
        assert!(with_splat.has_any());
        assert!(with_splat.has_splat());
    }

    #[test]
    fn test_map_index_upsert() {
        let mut index = MapIndex::new();
        let entry = MapIndexEntry {
            id: Uuid::new_v4(),
            name: "Test".to_string(),
            bounds: GpsBounds::default(),
            version: 1,
            updated_at: Utc::now(),
        };

        index.upsert(entry.clone());
        assert_eq!(index.maps.len(), 1);

        // Update existing
        let mut updated = entry.clone();
        updated.version = 2;
        index.upsert(updated);
        assert_eq!(index.maps.len(), 1);
        assert_eq!(index.maps[0].version, 2);
    }

    #[test]
    fn test_map_index_entry_from_manifest() {
        let bounds = GpsBounds::new(42.0, 43.0, -72.0, -71.0);
        let manifest = MapManifest::new("Test Map".to_string(), bounds);
        let entry = MapIndexEntry::from(&manifest);

        assert_eq!(entry.id, manifest.id);
        assert_eq!(entry.name, manifest.name);
        assert_eq!(entry.version, manifest.version);
    }
}
