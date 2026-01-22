//! RRD File Extraction
//!
//! Extracts pose, LiDAR, camera, and GPS data from Rerun `.rrd` session files
//! and writes them in the format expected by the splat-worker.
//!
//! # Data Flow
//!
//! ```text
//! .rrd file (Rerun recording)
//!     │
//!     ├── robot/x, robot/y, robot/heading → poses.csv
//!     ├── lidar/points                    → lidar/*.pcd
//!     ├── camera/image                    → camera/*.jpg
//!     └── gps/latitude, gps/longitude     → metadata.json (bounds)
//! ```

use arrow::array::{Array, RecordBatch};
use re_log_encoding::decoder::Decoder;
use re_log_encoding::VersionPolicy;
use re_log_types::LogMsg;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Errors that can occur during RRD extraction.
#[derive(Error, Debug)]
pub enum ExtractionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to load RRD file: {0}")]
    RrdLoad(String),
    #[error("Decode error: {0}")]
    Decode(String),
    #[error("No valid store found in RRD file")]
    NoStore,
    #[error("No LiDAR data found (required for mapping)")]
    NoLidarData,
}

/// A single pose sample with timestamp.
#[derive(Debug, Clone)]
pub struct Pose {
    /// Timestamp in seconds since session start
    pub time: f64,
    /// X position in meters (local frame)
    pub x: f64,
    /// Y position in meters (local frame)
    pub y: f64,
    /// Heading in radians
    pub theta: f64,
}

/// A LiDAR point cloud frame.
#[derive(Debug)]
pub struct PointCloud {
    /// Timestamp in seconds
    pub time: f64,
    /// Points as [x, y, z] in rover frame
    pub points: Vec<[f32; 3]>,
}

/// A camera image frame.
#[derive(Debug)]
pub struct ImageFrame {
    /// Timestamp in seconds
    pub time: f64,
    /// JPEG-encoded image data
    pub jpeg_data: Vec<u8>,
}

/// GPS bounding box.
#[derive(Debug, Clone, Default)]
pub struct GpsBounds {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

impl GpsBounds {
    pub fn expand(&mut self, lat: f64, lon: f64) {
        if self.min_lat == 0.0 && self.max_lat == 0.0 {
            self.min_lat = lat;
            self.max_lat = lat;
            self.min_lon = lon;
            self.max_lon = lon;
        } else {
            self.min_lat = self.min_lat.min(lat);
            self.max_lat = self.max_lat.max(lat);
            self.min_lon = self.min_lon.min(lon);
            self.max_lon = self.max_lon.max(lon);
        }
    }

    pub fn is_valid(&self) -> bool {
        self.min_lat != 0.0 || self.max_lat != 0.0
    }
}

/// Intermediate storage for time-synchronized data.
struct DataCollector {
    /// X values by timestamp
    x_values: BTreeMap<i64, f64>,
    /// Y values by timestamp
    y_values: BTreeMap<i64, f64>,
    /// Heading values by timestamp
    heading_values: BTreeMap<i64, f64>,
    /// LiDAR frames by timestamp
    lidar_frames: BTreeMap<i64, Vec<[f32; 3]>>,
    /// Camera frames by timestamp
    camera_frames: BTreeMap<i64, Vec<u8>>,
    /// GPS coordinates
    gps_points: Vec<(f64, f64)>,
    /// Session ID
    session_id: Option<String>,
    /// Rover ID
    rover_id: Option<String>,
}

impl DataCollector {
    fn new() -> Self {
        Self {
            x_values: BTreeMap::new(),
            y_values: BTreeMap::new(),
            heading_values: BTreeMap::new(),
            lidar_frames: BTreeMap::new(),
            camera_frames: BTreeMap::new(),
            gps_points: Vec::new(),
            session_id: None,
            rover_id: None,
        }
    }
}

/// Result of extracting data from an RRD file.
#[derive(Debug)]
pub struct ExtractionResult {
    pub poses: Vec<Pose>,
    pub lidar_frames: Vec<PointCloud>,
    pub camera_frames: Vec<ImageFrame>,
    pub gps_bounds: Option<GpsBounds>,
    pub session_id: Option<String>,
    pub rover_id: Option<String>,
}

/// Extract all relevant data from an RRD file.
pub fn extract_from_rrd(rrd_path: &Path) -> Result<ExtractionResult, ExtractionError> {
    info!(path = %rrd_path.display(), "Loading RRD file");

    // Open and decode the RRD file
    let file = File::open(rrd_path)?;
    let reader = BufReader::new(file);
    let decoder = Decoder::new(VersionPolicy::Warn, reader)
        .map_err(|e| ExtractionError::RrdLoad(format!("{:?}", e)))?;

    let mut collector = DataCollector::new();
    let mut msg_count = 0;
    let mut entity_paths = std::collections::HashSet::new();

    // Process each log message
    for msg_result in decoder {
        let msg = match msg_result {
            Ok(m) => m,
            Err(e) => {
                // Log decode errors but continue processing
                debug!(error = ?e, "Decode error, skipping message");
                continue;
            }
        };
        msg_count += 1;

        match msg {
            LogMsg::ArrowMsg(_store_id, arrow_msg) => {
                // The batch is a RecordBatch
                let batch: &RecordBatch = &arrow_msg.batch;

                // Get entity path from the timepoint_max metadata or schema
                // The entity path is stored in the chunk_id or we need to derive from data
                let entity_path = extract_entity_path_from_batch(batch);

                if let Some(ref path) = entity_path {
                    entity_paths.insert(path.clone());

                    // Get timestamp from timepoint_max
                    // TimePoint is a map from Timeline to TimeInt
                    let time = arrow_msg
                        .timepoint_max
                        .iter()
                        .find(|(tl, _)| tl.name().as_str() == "time")
                        .map(|(_, t)| t.as_i64());

                    process_batch(&mut collector, path, batch, time);
                }
            }
            LogMsg::SetStoreInfo(_) => {
                // Could extract rover_id from store info
            }
            LogMsg::BlueprintActivationCommand(_) => {
                // Ignore blueprint commands
            }
        }
    }

    info!(
        messages = msg_count,
        entities = entity_paths.len(),
        "Processed RRD file"
    );

    for path in &entity_paths {
        debug!(entity = %path, "Found entity");
    }

    // Build result from collected data
    let result = build_result(collector);

    info!(
        poses = result.poses.len(),
        lidar_frames = result.lidar_frames.len(),
        camera_frames = result.camera_frames.len(),
        has_gps = result.gps_bounds.is_some(),
        "Extraction complete"
    );

    Ok(result)
}

/// Extract entity path from RecordBatch metadata.
fn extract_entity_path_from_batch(batch: &RecordBatch) -> Option<String> {
    // The entity path may be stored in schema metadata
    batch
        .schema()
        .metadata()
        .get("rerun.entity_path")
        .cloned()
        .or_else(|| {
            // Try alternative metadata keys
            batch.schema().metadata().get("entity_path").cloned()
        })
}

/// Process a RecordBatch and extract relevant data.
fn process_batch(
    collector: &mut DataCollector,
    entity_path: &str,
    batch: &RecordBatch,
    time: Option<i64>,
) {
    // Normalize entity path (remove leading /)
    let path = entity_path.strip_prefix('/').unwrap_or(entity_path);

    match path {
        "robot/x" => {
            if let Some(value) = extract_scalar_from_batch(batch) {
                if let Some(t) = time {
                    collector.x_values.insert(t, value);
                }
            }
        }
        "robot/y" => {
            if let Some(value) = extract_scalar_from_batch(batch) {
                if let Some(t) = time {
                    collector.y_values.insert(t, value);
                }
            }
        }
        "robot/heading" => {
            if let Some(value) = extract_scalar_from_batch(batch) {
                if let Some(t) = time {
                    collector.heading_values.insert(t, value);
                }
            }
        }
        "robot/pose" => {
            // Extract pose from Transform3D (also handles heading)
            if let Some((x, y, theta)) = extract_transform3d_from_batch(batch) {
                if let Some(t) = time {
                    collector.x_values.insert(t, x);
                    collector.y_values.insert(t, y);
                    collector.heading_values.insert(t, theta);
                }
            }
        }
        "lidar/points" => {
            if let Some(points) = extract_points3d_from_batch(batch) {
                if let Some(t) = time {
                    collector.lidar_frames.insert(t, points);
                }
            }
        }
        "camera/image" => {
            if let Some(data) = extract_image_from_batch(batch) {
                if let Some(t) = time {
                    collector.camera_frames.insert(t, data);
                }
            }
        }
        "gps/latitude" => {
            if let Some(lat) = extract_scalar_from_batch(batch) {
                collector.gps_points.push((lat, 0.0));
            }
        }
        "gps/longitude" => {
            if let Some(lon) = extract_scalar_from_batch(batch) {
                if let Some(last) = collector.gps_points.last_mut() {
                    last.1 = lon;
                }
            }
        }
        "session/id" => {
            if let Some(id) = extract_text_from_batch(batch) {
                collector.session_id = Some(id);
            }
        }
        "session/rover_id" => {
            if let Some(id) = extract_text_from_batch(batch) {
                collector.rover_id = Some(id);
            }
        }
        _ => {
            // Ignore other entities
        }
    }
}

/// Extract a scalar value from a RecordBatch.
/// Handles Rerun's Scalar component: List(Float64) or List(Float32)
fn extract_scalar_from_batch(batch: &RecordBatch) -> Option<f64> {
    for col_idx in 0..batch.num_columns() {
        let col = batch.column(col_idx);

        // Try direct f64 array
        if let Some(arr) = col.as_any().downcast_ref::<arrow::array::Float64Array>() {
            if arr.len() > 0 {
                return Some(arr.value(0));
            }
        }

        // Try direct f32 array
        if let Some(arr) = col.as_any().downcast_ref::<arrow::array::Float32Array>() {
            if arr.len() > 0 {
                return Some(arr.value(0) as f64);
            }
        }

        // Try list array containing f64/f32 (Rerun's Scalar format)
        if let Some(list_arr) = col.as_any().downcast_ref::<arrow::array::ListArray>() {
            if list_arr.len() > 0 {
                let values = list_arr.values();
                if let Some(f64_arr) = values.as_any().downcast_ref::<arrow::array::Float64Array>()
                {
                    if f64_arr.len() > 0 {
                        return Some(f64_arr.value(0));
                    }
                }
                if let Some(f32_arr) = values.as_any().downcast_ref::<arrow::array::Float32Array>()
                {
                    if f32_arr.len() > 0 {
                        return Some(f32_arr.value(0) as f64);
                    }
                }

                // Try struct array (Scalar component)
                if let Some(struct_arr) =
                    values.as_any().downcast_ref::<arrow::array::StructArray>()
                {
                    for i in 0..struct_arr.num_columns() {
                        let field = struct_arr.column(i);
                        if let Some(f64_arr) =
                            field.as_any().downcast_ref::<arrow::array::Float64Array>()
                        {
                            if f64_arr.len() > 0 {
                                return Some(f64_arr.value(0));
                            }
                        }
                    }
                }
            }
        }

        // Try struct array directly
        if let Some(struct_arr) = col.as_any().downcast_ref::<arrow::array::StructArray>() {
            for i in 0..struct_arr.num_columns() {
                let field = struct_arr.column(i);
                if let Some(f64_arr) = field.as_any().downcast_ref::<arrow::array::Float64Array>() {
                    if f64_arr.len() > 0 {
                        return Some(f64_arr.value(0));
                    }
                }
                if let Some(f32_arr) = field.as_any().downcast_ref::<arrow::array::Float32Array>() {
                    if f32_arr.len() > 0 {
                        return Some(f32_arr.value(0) as f64);
                    }
                }
            }
        }
    }

    None
}

/// Extract Transform3D (x, y, theta) from a RecordBatch.
/// Returns (x, y, theta) where theta is extracted from rotation.
fn extract_transform3d_from_batch(batch: &RecordBatch) -> Option<(f64, f64, f64)> {
    // Transform3D in Rerun is stored with these column names:
    // - rerun.components.Translation3D: List(FixedSizeList(Float32, 3))
    // - rerun.components.RotationAxisAngle: List(Struct(axis, angle))

    let mut translation_x: Option<f64> = None;
    let mut translation_y: Option<f64> = None;
    let mut theta: Option<f64> = None;

    for col_idx in 0..batch.num_columns() {
        let col = batch.column(col_idx);
        let schema = batch.schema();
        let col_name = schema.field(col_idx).name();

        // Look for Translation3D column
        if col_name.contains("Translation3D") {
            if let Some((x, y, _z)) = extract_vec3_from_column(col) {
                translation_x = Some(x);
                translation_y = Some(y);
            }
        }

        // Look for RotationAxisAngle column
        if col_name.contains("RotationAxisAngle") {
            if let Some(angle) = extract_rotation_angle_from_column(col) {
                theta = Some(angle);
            }
        }
    }

    // If we found translation, return with theta (default 0 if not found)
    if let (Some(x), Some(y)) = (translation_x, translation_y) {
        Some((x, y, theta.unwrap_or(0.0)))
    } else {
        None
    }
}

/// Extract Vec3 (x, y, z) from a column.
/// Handles: List(FixedSizeList(Float32, 3))
fn extract_vec3_from_column(col: &dyn Array) -> Option<(f64, f64, f64)> {
    // Try FixedSizeList directly (3 floats)
    if let Some(fsl) = col.as_any().downcast_ref::<arrow::array::FixedSizeListArray>() {
        let values = fsl.values();
        if let Some(f32_arr) = values.as_any().downcast_ref::<arrow::array::Float32Array>() {
            if f32_arr.len() >= 3 {
                return Some((f32_arr.value(0) as f64, f32_arr.value(1) as f64, f32_arr.value(2) as f64));
            }
        }
    }

    // Try List containing FixedSizeList
    if let Some(list_arr) = col.as_any().downcast_ref::<arrow::array::ListArray>() {
        if list_arr.len() > 0 && !list_arr.is_null(0) {
            let values = list_arr.values();
            // values is FixedSizeList
            if let Some(fsl) = values.as_any().downcast_ref::<arrow::array::FixedSizeListArray>() {
                let inner_values = fsl.values();
                if let Some(f32_arr) = inner_values.as_any().downcast_ref::<arrow::array::Float32Array>() {
                    if f32_arr.len() >= 3 {
                        return Some((f32_arr.value(0) as f64, f32_arr.value(1) as f64, f32_arr.value(2) as f64));
                    }
                }
            }
        }
    }

    // Try struct array with x, y, z fields (fallback)
    if let Some(struct_arr) = col.as_any().downcast_ref::<arrow::array::StructArray>() {
        let x_col = struct_arr.column_by_name("x");
        let y_col = struct_arr.column_by_name("y");
        let z_col = struct_arr.column_by_name("z");

        if let (Some(x), Some(y), Some(z)) = (x_col, y_col, z_col) {
            if let (Some(x_arr), Some(y_arr), Some(z_arr)) = (
                x.as_any().downcast_ref::<arrow::array::Float32Array>(),
                y.as_any().downcast_ref::<arrow::array::Float32Array>(),
                z.as_any().downcast_ref::<arrow::array::Float32Array>(),
            ) {
                if x_arr.len() > 0 {
                    return Some((x_arr.value(0) as f64, y_arr.value(0) as f64, z_arr.value(0) as f64));
                }
            }
        }
    }

    None
}

/// Extract rotation angle (around Z axis) from a column.
/// Handles: List(Struct(axis: FixedSizeList, angle: Float32))
fn extract_rotation_angle_from_column(col: &dyn Array) -> Option<f64> {
    // Try List containing Struct
    if let Some(list_arr) = col.as_any().downcast_ref::<arrow::array::ListArray>() {
        if list_arr.len() > 0 && !list_arr.is_null(0) {
            let values = list_arr.values();
            // values is Struct with axis and angle
            if let Some(struct_arr) = values.as_any().downcast_ref::<arrow::array::StructArray>() {
                // Look for angle field (should be Float32)
                if let Some(angle_col) = struct_arr.column_by_name("angle") {
                    if let Some(f32_arr) = angle_col.as_any().downcast_ref::<arrow::array::Float32Array>() {
                        if f32_arr.len() > 0 {
                            return Some(f32_arr.value(0) as f64);
                        }
                    }
                }
            }
        }
    }

    // Try Struct directly (RotationAxisAngle has axis (Vec3) and angle (float))
    if let Some(struct_arr) = col.as_any().downcast_ref::<arrow::array::StructArray>() {
        // Look for angle field
        if let Some(angle_col) = struct_arr.column_by_name("angle") {
            if let Some(f32_arr) = angle_col.as_any().downcast_ref::<arrow::array::Float32Array>() {
                if f32_arr.len() > 0 {
                    return Some(f32_arr.value(0) as f64);
                }
            }
        }
    }

    None
}

/// Extract 3D points from a RecordBatch.
fn extract_points3d_from_batch(batch: &RecordBatch) -> Option<Vec<[f32; 3]>> {
    let mut points = Vec::new();

    for col_idx in 0..batch.num_columns() {
        let col = batch.column(col_idx);

        // Try list array containing struct with x, y, z
        if let Some(list_arr) = col.as_any().downcast_ref::<arrow::array::ListArray>() {
            let values = list_arr.values();

            if let Some(struct_arr) = values.as_any().downcast_ref::<arrow::array::StructArray>() {
                let x_col = struct_arr.column_by_name("x");
                let y_col = struct_arr.column_by_name("y");
                let z_col = struct_arr.column_by_name("z");

                if let (Some(x), Some(y), Some(z)) = (x_col, y_col, z_col) {
                    if let (Some(x_arr), Some(y_arr), Some(z_arr)) = (
                        x.as_any().downcast_ref::<arrow::array::Float32Array>(),
                        y.as_any().downcast_ref::<arrow::array::Float32Array>(),
                        z.as_any().downcast_ref::<arrow::array::Float32Array>(),
                    ) {
                        for i in 0..x_arr.len() {
                            points.push([x_arr.value(i), y_arr.value(i), z_arr.value(i)]);
                        }
                    }
                }
            }

            // Try nested list
            if let Some(inner_list) = values.as_any().downcast_ref::<arrow::array::ListArray>() {
                let inner_values = inner_list.values();
                if let Some(struct_arr) =
                    inner_values.as_any().downcast_ref::<arrow::array::StructArray>()
                {
                    let x_col = struct_arr.column_by_name("x");
                    let y_col = struct_arr.column_by_name("y");
                    let z_col = struct_arr.column_by_name("z");

                    if let (Some(x), Some(y), Some(z)) = (x_col, y_col, z_col) {
                        if let (Some(x_arr), Some(y_arr), Some(z_arr)) = (
                            x.as_any().downcast_ref::<arrow::array::Float32Array>(),
                            y.as_any().downcast_ref::<arrow::array::Float32Array>(),
                            z.as_any().downcast_ref::<arrow::array::Float32Array>(),
                        ) {
                            for i in 0..x_arr.len() {
                                points.push([x_arr.value(i), y_arr.value(i), z_arr.value(i)]);
                            }
                        }
                    }
                }
            }
        }

        // Try struct array directly
        if let Some(struct_arr) = col.as_any().downcast_ref::<arrow::array::StructArray>() {
            let x_col = struct_arr.column_by_name("x");
            let y_col = struct_arr.column_by_name("y");
            let z_col = struct_arr.column_by_name("z");

            if let (Some(x), Some(y), Some(z)) = (x_col, y_col, z_col) {
                if let (Some(x_arr), Some(y_arr), Some(z_arr)) = (
                    x.as_any().downcast_ref::<arrow::array::Float32Array>(),
                    y.as_any().downcast_ref::<arrow::array::Float32Array>(),
                    z.as_any().downcast_ref::<arrow::array::Float32Array>(),
                ) {
                    for i in 0..x_arr.len() {
                        points.push([x_arr.value(i), y_arr.value(i), z_arr.value(i)]);
                    }
                }
            }
        }
    }

    if points.is_empty() {
        None
    } else {
        Some(points)
    }
}

/// Extract image data from a RecordBatch.
fn extract_image_from_batch(batch: &RecordBatch) -> Option<Vec<u8>> {
    for col_idx in 0..batch.num_columns() {
        let col = batch.column(col_idx);

        // Try binary array directly
        if let Some(arr) = col.as_any().downcast_ref::<arrow::array::BinaryArray>() {
            if arr.len() > 0 {
                return Some(arr.value(0).to_vec());
            }
        }

        if let Some(arr) = col.as_any().downcast_ref::<arrow::array::LargeBinaryArray>() {
            if arr.len() > 0 {
                return Some(arr.value(0).to_vec());
            }
        }

        // Try list array containing binary
        if let Some(list_arr) = col.as_any().downcast_ref::<arrow::array::ListArray>() {
            let values = list_arr.values();

            if let Some(arr) = values.as_any().downcast_ref::<arrow::array::BinaryArray>() {
                if arr.len() > 0 {
                    return Some(arr.value(0).to_vec());
                }
            }

            if let Some(arr) = values.as_any().downcast_ref::<arrow::array::LargeBinaryArray>() {
                if arr.len() > 0 {
                    return Some(arr.value(0).to_vec());
                }
            }

            // Try struct with data field
            if let Some(struct_arr) = values.as_any().downcast_ref::<arrow::array::StructArray>() {
                if let Some(data_col) = struct_arr.column_by_name("data") {
                    if let Some(arr) = data_col.as_any().downcast_ref::<arrow::array::BinaryArray>()
                    {
                        if arr.len() > 0 {
                            return Some(arr.value(0).to_vec());
                        }
                    }
                    if let Some(arr) = data_col
                        .as_any()
                        .downcast_ref::<arrow::array::LargeBinaryArray>()
                    {
                        if arr.len() > 0 {
                            return Some(arr.value(0).to_vec());
                        }
                    }
                }
            }
        }

        // Try struct array with data field
        if let Some(struct_arr) = col.as_any().downcast_ref::<arrow::array::StructArray>() {
            if let Some(data_col) = struct_arr.column_by_name("data") {
                if let Some(arr) = data_col.as_any().downcast_ref::<arrow::array::BinaryArray>() {
                    if arr.len() > 0 {
                        return Some(arr.value(0).to_vec());
                    }
                }
            }
        }
    }

    None
}

/// Extract text from a RecordBatch.
fn extract_text_from_batch(batch: &RecordBatch) -> Option<String> {
    for col_idx in 0..batch.num_columns() {
        let col = batch.column(col_idx);

        if let Some(arr) = col.as_any().downcast_ref::<arrow::array::StringArray>() {
            if arr.len() > 0 {
                return Some(arr.value(0).to_string());
            }
        }

        if let Some(arr) = col.as_any().downcast_ref::<arrow::array::LargeStringArray>() {
            if arr.len() > 0 {
                return Some(arr.value(0).to_string());
            }
        }

        // Try list array
        if let Some(list_arr) = col.as_any().downcast_ref::<arrow::array::ListArray>() {
            let values = list_arr.values();

            if let Some(arr) = values.as_any().downcast_ref::<arrow::array::StringArray>() {
                if arr.len() > 0 {
                    return Some(arr.value(0).to_string());
                }
            }
        }
    }

    None
}

/// Build the final extraction result from collected data.
fn build_result(collector: DataCollector) -> ExtractionResult {
    // Build poses by matching x, y, heading at same timestamps
    let mut poses = Vec::new();
    for (time, x) in &collector.x_values {
        if let (Some(y), Some(theta)) = (
            collector.y_values.get(time),
            collector.heading_values.get(time),
        ) {
            poses.push(Pose {
                time: *time as f64,
                x: *x,
                y: *y,
                theta: *theta,
            });
        }
    }

    // Build LiDAR frames
    let lidar_frames: Vec<PointCloud> = collector
        .lidar_frames
        .into_iter()
        .map(|(time, points)| PointCloud {
            time: time as f64,
            points,
        })
        .collect();

    // Build camera frames
    let camera_frames: Vec<ImageFrame> = collector
        .camera_frames
        .into_iter()
        .map(|(time, jpeg_data)| ImageFrame {
            time: time as f64,
            jpeg_data,
        })
        .collect();

    // Build GPS bounds
    let gps_bounds = if !collector.gps_points.is_empty() {
        let mut bounds = GpsBounds::default();
        for (lat, lon) in &collector.gps_points {
            if lat.abs() > 0.001 && lon.abs() > 0.001 {
                bounds.expand(*lat, *lon);
            }
        }
        if bounds.is_valid() {
            Some(bounds)
        } else {
            None
        }
    } else {
        None
    };

    ExtractionResult {
        poses,
        lidar_frames,
        camera_frames,
        gps_bounds,
        session_id: collector.session_id,
        rover_id: collector.rover_id,
    }
}

/// Write extracted data to output directory in splat-worker format.
pub fn write_extracted_data(
    result: &ExtractionResult,
    output_dir: &Path,
) -> Result<(), ExtractionError> {
    info!(path = %output_dir.display(), "Writing extracted data");

    fs::create_dir_all(output_dir)?;

    // Write poses.csv
    write_poses_csv(&result.poses, output_dir)?;

    // Write lidar/*.pcd
    write_lidar_pcd(&result.lidar_frames, output_dir)?;

    // Write camera/*.jpg (if any)
    write_camera_jpg(&result.camera_frames, output_dir)?;

    // Write metadata.json
    write_metadata(result, output_dir)?;

    Ok(())
}

/// Write poses to CSV file.
fn write_poses_csv(poses: &[Pose], output_dir: &Path) -> Result<(), ExtractionError> {
    if poses.is_empty() {
        warn!("No poses to write, creating empty poses.csv");
    }

    let poses_path = output_dir.join("poses.csv");
    let file = File::create(&poses_path)?;
    let mut writer = BufWriter::new(file);

    // Header: timestamp, x, y, z, qx, qy, qz, qw
    writeln!(writer, "timestamp,x,y,z,qx,qy,qz,qw")?;

    for pose in poses {
        // Convert 2D heading to 3D quaternion (rotation around Z axis)
        let (qx, qy, qz, qw) = heading_to_quaternion(pose.theta);

        writeln!(
            writer,
            "{},{},{},{},{},{},{},{}",
            pose.time, pose.x, pose.y, 0.0, qx, qy, qz, qw
        )?;
    }

    writer.flush()?;
    debug!(path = %poses_path.display(), count = poses.len(), "Wrote poses.csv");
    Ok(())
}

/// Convert heading angle to quaternion (rotation around Z axis).
fn heading_to_quaternion(theta: f64) -> (f64, f64, f64, f64) {
    let half_angle = theta / 2.0;
    let qx = 0.0;
    let qy = 0.0;
    let qz = half_angle.sin();
    let qw = half_angle.cos();
    (qx, qy, qz, qw)
}

/// Write LiDAR frames as PCD files.
fn write_lidar_pcd(frames: &[PointCloud], output_dir: &Path) -> Result<(), ExtractionError> {
    if frames.is_empty() {
        return Ok(());
    }

    let lidar_dir = output_dir.join("lidar");
    fs::create_dir_all(&lidar_dir)?;

    for (i, frame) in frames.iter().enumerate() {
        let pcd_path = lidar_dir.join(format!("{:06}.pcd", i));
        write_pcd(&frame.points, &pcd_path)?;
    }

    debug!(path = %lidar_dir.display(), count = frames.len(), "Wrote LiDAR PCD files");
    Ok(())
}

/// Write a single PCD file in ASCII format.
fn write_pcd(points: &[[f32; 3]], path: &Path) -> Result<(), ExtractionError> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // PCD header
    writeln!(writer, "# .PCD v0.7 - Point Cloud Data")?;
    writeln!(writer, "VERSION 0.7")?;
    writeln!(writer, "FIELDS x y z")?;
    writeln!(writer, "SIZE 4 4 4")?;
    writeln!(writer, "TYPE F F F")?;
    writeln!(writer, "COUNT 1 1 1")?;
    writeln!(writer, "WIDTH {}", points.len())?;
    writeln!(writer, "HEIGHT 1")?;
    writeln!(writer, "VIEWPOINT 0 0 0 1 0 0 0")?;
    writeln!(writer, "POINTS {}", points.len())?;
    writeln!(writer, "DATA ascii")?;

    // Point data
    for p in points {
        writeln!(writer, "{} {} {}", p[0], p[1], p[2])?;
    }

    writer.flush()?;
    Ok(())
}

/// Write camera frames as JPEG files.
fn write_camera_jpg(frames: &[ImageFrame], output_dir: &Path) -> Result<(), ExtractionError> {
    if frames.is_empty() {
        return Ok(());
    }

    let camera_dir = output_dir.join("camera");
    fs::create_dir_all(&camera_dir)?;

    for (i, frame) in frames.iter().enumerate() {
        let jpg_path = camera_dir.join(format!("{:06}.jpg", i));
        fs::write(&jpg_path, &frame.jpeg_data)?;
    }

    debug!(path = %camera_dir.display(), count = frames.len(), "Wrote camera JPEG files");
    Ok(())
}

/// Write extraction metadata.
fn write_metadata(result: &ExtractionResult, output_dir: &Path) -> Result<(), ExtractionError> {
    let metadata = serde_json::json!({
        "extraction_version": 1,
        "pose_count": result.poses.len(),
        "lidar_frame_count": result.lidar_frames.len(),
        "camera_frame_count": result.camera_frames.len(),
        "gps_bounds": result.gps_bounds.as_ref().map(|b| serde_json::json!({
            "min_lat": b.min_lat,
            "max_lat": b.max_lat,
            "min_lon": b.min_lon,
            "max_lon": b.max_lon,
        })),
        "session_id": result.session_id,
        "rover_id": result.rover_id,
    });

    let metadata_path = output_dir.join("extraction_metadata.json");
    let json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| ExtractionError::Io(std::io::Error::other(e)))?;
    fs::write(&metadata_path, json)?;

    debug!(path = %metadata_path.display(), "Wrote extraction metadata");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_to_quaternion() {
        // 0 radians -> identity rotation
        let (qx, qy, qz, qw) = heading_to_quaternion(0.0);
        assert!((qx).abs() < 1e-10);
        assert!((qy).abs() < 1e-10);
        assert!((qz).abs() < 1e-10);
        assert!((qw - 1.0).abs() < 1e-10);

        // 90 degrees -> 45 degree half-angle
        let (qx, qy, qz, qw) = heading_to_quaternion(std::f64::consts::FRAC_PI_2);
        assert!((qx).abs() < 1e-10);
        assert!((qy).abs() < 1e-10);
        assert!((qz - 0.7071067811865476).abs() < 1e-10);
        assert!((qw - 0.7071067811865476).abs() < 1e-10);
    }

    #[test]
    fn test_gps_bounds() {
        let mut bounds = GpsBounds::default();
        assert!(!bounds.is_valid());

        bounds.expand(44.9778, -93.2650);
        assert!(bounds.is_valid());
        assert_eq!(bounds.min_lat, 44.9778);
        assert_eq!(bounds.max_lat, 44.9778);

        bounds.expand(44.9812, -93.2580);
        assert_eq!(bounds.min_lat, 44.9778);
        assert_eq!(bounds.max_lat, 44.9812);
        assert_eq!(bounds.min_lon, -93.2650);
        assert_eq!(bounds.max_lon, -93.2580);
    }
}
