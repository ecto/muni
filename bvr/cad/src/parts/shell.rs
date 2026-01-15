//! 2-Panel Clam Shell Design for BVR1 Rover
//!
//! Simplified shell design with 2 bent panels for SendCutSend order:
//! - **Wall Wrap**: Front + sides + rear as single bent piece (3 bends)
//! - **Top Lid**: Removable top panel with sensor mast and e-stop holes
//!
//! Material: 5052-H32 Aluminum, 2mm thickness
//! Finish: Orange powder coat (RAL 2004)
//!
//! Features:
//! - LED channel parallel to blower nozzle (replaces headlights)
//! - Louver vents (angled slots) for airflow
//! - Drain holes at bottom corners
//! - Security Torx fasteners + keyed quarter-turns for lid

use crate::export::DxfDocument;
use crate::{centered_cube, centered_cylinder, Part};
use std::path::Path;

// =============================================================================
// Configuration
// =============================================================================

/// Shell panel configuration
#[derive(Debug, Clone)]
pub struct ShellConfig {
    /// Panel thickness (mm)
    pub thickness: f64,
    /// Clearance around frame (mm per side)
    pub clearance: f64,
    /// Frame dimensions (from BVR1FrameConfig)
    pub frame_width: f64,
    pub frame_length: f64,
    pub frame_height: f64,
    /// Mounting hole diameter (M5 = 5.3mm)
    pub mount_hole_diameter: f64,
    /// Mounting hole inset from edge (mm)
    pub mount_hole_inset: f64,
    /// Corner radius for panel edges
    pub corner_radius: f64,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            thickness: 2.0,           // 2mm aluminum
            clearance: 20.0,          // 20mm clearance around frame
            frame_width: 380.0,       // BVR1 frame width
            frame_length: 500.0,      // BVR1 frame length
            frame_height: 180.0,      // BVR1 frame height
            mount_hole_diameter: 5.3, // M5 clearance
            mount_hole_inset: 10.0,   // 10mm from edge
            corner_radius: 5.0,       // 5mm corner radius
        }
    }
}

impl ShellConfig {
    /// Calculate shell outer width
    pub fn shell_width(&self) -> f64 {
        self.frame_width + self.clearance * 2.0
    }

    /// Calculate shell outer length
    pub fn shell_length(&self) -> f64 {
        self.frame_length + self.clearance * 2.0
    }

    /// Calculate shell outer height
    pub fn shell_height(&self) -> f64 {
        self.frame_height + self.clearance
    }
}

// =============================================================================
// Wall Wrap (Panel 1)
// =============================================================================

/// Wall Wrap configuration
#[derive(Debug, Clone)]
pub struct WallWrapConfig {
    pub shell: ShellConfig,
    /// Nozzle slot width (mm)
    pub nozzle_width: f64,
    /// Nozzle slot height (mm)
    pub nozzle_height: f64,
    /// Nozzle slot corner radius (mm)
    pub nozzle_corner_radius: f64,
    /// Nozzle slot vertical offset from bottom (mm)
    pub nozzle_offset_y: f64,
    /// LED channel slot width (mm)
    pub led_channel_width: f64,
    /// LED channel slot height (mm)
    pub led_channel_height: f64,
    /// LED channel offset above nozzle (mm)
    pub led_channel_gap: f64,
    /// Louver vent slot width (mm)
    pub louver_width: f64,
    /// Louver vent slot height (mm)
    pub louver_height: f64,
    /// Number of louver slots
    pub louver_count: usize,
    /// Louver slot spacing (mm)
    pub louver_spacing: f64,
    /// Drain hole diameter (mm)
    pub drain_hole_diameter: f64,
    /// Bend radius inside (mm)
    pub bend_radius: f64,
}

impl Default for WallWrapConfig {
    fn default() -> Self {
        Self {
            shell: ShellConfig::default(),
            // Nozzle slot (from blower-cad-spec)
            nozzle_width: 500.0,
            nozzle_height: 50.0,
            nozzle_corner_radius: 15.0,
            nozzle_offset_y: 60.0,  // 60mm from bottom
            // LED channel
            led_channel_width: 500.0,
            led_channel_height: 20.0,
            led_channel_gap: 10.0,  // 10mm above nozzle slot
            // Louver vents (8 slots, ~150mm² total area per artifact-plan)
            louver_width: 60.0,
            louver_height: 8.0,
            louver_count: 8,
            louver_spacing: 35.0,
            // Drain holes
            drain_hole_diameter: 6.0,
            // Bend radius (standard for 2mm aluminum)
            bend_radius: 2.0,
        }
    }
}

impl WallWrapConfig {
    /// Front section width (same as shell width)
    pub fn front_width(&self) -> f64 {
        self.shell.shell_width()
    }

    /// Side section length (same as shell length)
    pub fn side_length(&self) -> f64 {
        self.shell.shell_length()
    }

    /// Rear section width (same as shell width)
    pub fn rear_width(&self) -> f64 {
        self.shell.shell_width()
    }

    /// Panel height (same as shell height)
    pub fn panel_height(&self) -> f64 {
        self.shell.shell_height()
    }

    /// Total flat pattern width (before bending)
    /// Front + Left Side + Rear + Right Side + gap
    pub fn flat_width(&self) -> f64 {
        // Using neutral axis for bend allowance: arc = pi * (r + k*t) * angle/180
        // k-factor ~0.33 for aluminum, but for simplicity use inside dimension
        let bend_allowance = std::f64::consts::PI * self.bend_radius * 0.5; // 90° bend
        self.front_width()
            + self.side_length()
            + self.rear_width()
            + self.side_length()
            - 3.0 * bend_allowance // 3 bends consume material
            + 20.0 // gap at end
    }
}

/// Wall Wrap: Front + sides + rear as single bent piece
pub struct WallWrap {
    config: WallWrapConfig,
}

impl WallWrap {
    pub fn new(config: WallWrapConfig) -> Self {
        Self { config }
    }

    pub fn default_bvr1() -> Self {
        Self::new(WallWrapConfig::default())
    }

    /// Generate 3D representation (simplified - just front panel for visualization)
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let height = cfg.panel_height();
        let thickness = cfg.shell.thickness;
        let segments = 32;

        // For 3D visualization, create assembled shell (bent into shape)
        let width = cfg.shell.shell_width();
        let length = cfg.shell.shell_length();

        // Front panel
        let front = centered_cube("front_panel", width, thickness, height)
            .translate(0.0, length / 2.0, height / 2.0);

        // Rear panel
        let rear = centered_cube("rear_panel", width, thickness, height)
            .translate(0.0, -length / 2.0, height / 2.0);

        // Left side
        let left = centered_cube("left_panel", thickness, length, height)
            .translate(-width / 2.0, 0.0, height / 2.0);

        // Right side
        let right = centered_cube("right_panel", thickness, length, height)
            .translate(width / 2.0, 0.0, height / 2.0);

        // Nozzle cutout in front
        let nozzle_cutout = centered_cube(
            "nozzle",
            cfg.nozzle_width,
            thickness * 3.0,
            cfg.nozzle_height,
        )
        .translate(
            0.0,
            length / 2.0,
            cfg.nozzle_offset_y + cfg.nozzle_height / 2.0,
        );

        // LED channel cutout
        let led_offset_y = cfg.nozzle_offset_y + cfg.nozzle_height + cfg.led_channel_gap + cfg.led_channel_height / 2.0;
        let led_cutout = centered_cube(
            "led_channel",
            cfg.led_channel_width,
            thickness * 3.0,
            cfg.led_channel_height,
        )
        .translate(0.0, length / 2.0, led_offset_y);

        // Louver vents in rear
        let mut louvers = Part::empty("louvers");
        let total_louver_width = (cfg.louver_count - 1) as f64 * cfg.louver_spacing;
        let start_x = -total_louver_width / 2.0;
        let louver_y = height * 0.6; // Upper portion

        for i in 0..cfg.louver_count {
            let x = start_x + i as f64 * cfg.louver_spacing;
            let louver = centered_cube(
                "louver",
                cfg.louver_width,
                thickness * 3.0,
                cfg.louver_height,
            )
            .translate(x, -length / 2.0, louver_y);
            louvers = louvers.union(&louver);
        }

        // Drain holes
        let drain_inset = 20.0;
        let drain1 = centered_cylinder(
            "drain",
            cfg.drain_hole_diameter / 2.0,
            thickness * 3.0,
            segments,
        )
        .rotate(90.0, 0.0, 0.0)
        .translate(-width / 2.0 + drain_inset, -length / 2.0, drain_inset);

        let drain2 = centered_cylinder(
            "drain",
            cfg.drain_hole_diameter / 2.0,
            thickness * 3.0,
            segments,
        )
        .rotate(90.0, 0.0, 0.0)
        .translate(width / 2.0 - drain_inset, -length / 2.0, drain_inset);

        front
            .union(&rear)
            .union(&left)
            .union(&right)
            .difference(&nozzle_cutout)
            .difference(&led_cutout)
            .difference(&louvers)
            .difference(&drain1)
            .difference(&drain2)
    }

    /// Generate 2D DXF flat pattern for laser cutting
    pub fn to_dxf(&self) -> DxfDocument {
        let cfg = &self.config;
        let mut dxf = DxfDocument::new();

        let height = cfg.panel_height();
        let inset = cfg.shell.mount_hole_inset;
        let hole_r = cfg.shell.mount_hole_diameter / 2.0;

        // Calculate section positions (left to right in flat pattern)
        // Layout: [FRONT][LEFT SIDE][REAR][RIGHT SIDE][gap]
        let front_w = cfg.front_width();
        let side_l = cfg.side_length();
        let rear_w = cfg.rear_width();

        // Section centers (X position)
        let front_cx = -front_w / 2.0 - side_l - rear_w / 2.0;
        let left_cx = -side_l / 2.0 - rear_w / 2.0;
        let rear_cx = 0.0;
        let right_cx = side_l / 2.0 + rear_w / 2.0;

        // Total width for outer rectangle
        let total_width = front_w + side_l * 2.0 + rear_w + 20.0; // 20mm gap

        // Outer boundary (entire flat pattern)
        dxf.add_rectangle(total_width, height, (right_cx + front_cx) / 2.0, 0.0);

        // --- Bend lines (vertical, on BEND layer) ---
        let bend1_x = front_cx + front_w / 2.0;  // After front
        let bend2_x = left_cx + side_l / 2.0;    // After left side
        let bend3_x = rear_cx + rear_w / 2.0;    // After rear

        dxf.add_bend_line(bend1_x, -height / 2.0, bend1_x, height / 2.0);
        dxf.add_bend_line(bend2_x, -height / 2.0, bend2_x, height / 2.0);
        dxf.add_bend_line(bend3_x, -height / 2.0, bend3_x, height / 2.0);

        // --- FRONT SECTION ---
        // Nozzle slot (rounded rectangle)
        let nozzle_cy = -height / 2.0 + cfg.nozzle_offset_y + cfg.nozzle_height / 2.0;
        dxf.add_rounded_rectangle(
            cfg.nozzle_width,
            cfg.nozzle_height,
            front_cx,
            nozzle_cy,
            cfg.nozzle_corner_radius,
        );

        // LED channel slot (simple rectangle, narrower)
        let led_cy = nozzle_cy + cfg.nozzle_height / 2.0 + cfg.led_channel_gap + cfg.led_channel_height / 2.0;
        dxf.add_rounded_rectangle(
            cfg.led_channel_width,
            cfg.led_channel_height,
            front_cx,
            led_cy,
            cfg.led_channel_height / 4.0, // Slight rounding
        );

        // Front mounting holes (4 corners)
        let front_hx = front_w / 2.0 - inset;
        let hy = height / 2.0 - inset;
        for (dx, dy) in [(-front_hx, hy), (front_hx, hy), (-front_hx, -hy), (front_hx, -hy)] {
            dxf.add_circle(front_cx + dx, dy, hole_r);
        }

        // --- LEFT SIDE SECTION ---
        // Clean panel, just mounting holes (6 holes)
        let side_hx = side_l / 2.0 - inset;
        for (dx, dy) in [
            (-side_hx, hy), (side_hx, hy),
            (-side_hx, 0.0), (side_hx, 0.0),
            (-side_hx, -hy), (side_hx, -hy),
        ] {
            dxf.add_circle(left_cx + dx, dy, hole_r);
        }

        // --- REAR SECTION ---
        // Louver vent slots (8 slots arranged horizontally)
        let total_louver_span = (cfg.louver_count - 1) as f64 * cfg.louver_spacing;
        let louver_start_x = rear_cx - total_louver_span / 2.0;
        let louver_cy = height / 4.0; // Upper portion

        for i in 0..cfg.louver_count {
            let lx = louver_start_x + i as f64 * cfg.louver_spacing;
            dxf.add_slot(cfg.louver_width, cfg.louver_height, lx, louver_cy);
        }

        // Drain holes (bottom corners)
        let drain_inset = 20.0;
        let rear_hx = rear_w / 2.0 - drain_inset;
        let drain_y = -height / 2.0 + drain_inset;
        dxf.add_circle(rear_cx - rear_hx, drain_y, cfg.drain_hole_diameter / 2.0);
        dxf.add_circle(rear_cx + rear_hx, drain_y, cfg.drain_hole_diameter / 2.0);

        // Rear mounting holes (4 corners)
        let rear_mount_hx = rear_w / 2.0 - inset;
        for (dx, dy) in [(-rear_mount_hx, hy), (rear_mount_hx, hy), (-rear_mount_hx, -hy), (rear_mount_hx, -hy)] {
            dxf.add_circle(rear_cx + dx, dy, hole_r);
        }

        // --- RIGHT SIDE SECTION ---
        // Clean panel, just mounting holes (6 holes)
        for (dx, dy) in [
            (-side_hx, hy), (side_hx, hy),
            (-side_hx, 0.0), (side_hx, 0.0),
            (-side_hx, -hy), (side_hx, -hy),
        ] {
            dxf.add_circle(right_cx + dx, dy, hole_r);
        }

        dxf
    }
}

// =============================================================================
// Top Lid (Panel 2)
// =============================================================================

/// Top Lid configuration
#[derive(Debug, Clone)]
pub struct TopLidConfig {
    pub shell: ShellConfig,
    /// Sensor mast hole diameter (mm) - fits LiDAR base + camera clearance
    pub sensor_hole_diameter: f64,
    /// Sensor mast hole position offset from center (x, y)
    /// Rear-center position
    pub sensor_hole_offset: (f64, f64),
    /// E-stop hole diameter (mm)
    pub estop_hole_diameter: f64,
    /// E-stop hole position offset from center (x, y)
    /// Front-right position for accessibility
    pub estop_hole_offset: (f64, f64),
    /// GPS cable grommet hole diameter (mm)
    pub gps_grommet_diameter: f64,
    /// GPS grommet offset from sensor hole (x, y)
    pub gps_grommet_offset: (f64, f64),
}

impl Default for TopLidConfig {
    fn default() -> Self {
        let shell = ShellConfig::default();
        Self {
            sensor_hole_diameter: 150.0,  // Large hole for LiDAR + camera mast
            // Rear-center: Y is negative (toward rear), centered on X
            sensor_hole_offset: (0.0, -shell.shell_length() / 4.0),
            estop_hole_diameter: 30.0,
            // Front-right: positive X (right), positive Y (front)
            estop_hole_offset: (shell.shell_width() / 4.0, shell.shell_length() / 4.0),
            gps_grommet_diameter: 12.0,
            // Offset from sensor hole
            gps_grommet_offset: (80.0, 20.0),
            shell,
        }
    }
}

/// Top Lid: Removable panel with sensor mast and e-stop holes
pub struct TopLid {
    config: TopLidConfig,
}

impl TopLid {
    pub fn new(config: TopLidConfig) -> Self {
        Self { config }
    }

    pub fn default_bvr1() -> Self {
        Self::new(TopLidConfig::default())
    }

    /// Generate 3D representation
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let shell = &cfg.shell;
        let segments = 32;

        let width = shell.shell_width();
        let length = shell.shell_length();

        // Main panel body
        let panel = centered_cube("top_lid", width, length, shell.thickness);

        // Sensor mast hole
        let sensor_hole = centered_cylinder(
            "sensor_hole",
            cfg.sensor_hole_diameter / 2.0,
            shell.thickness * 3.0,
            segments,
        )
        .translate(cfg.sensor_hole_offset.0, cfg.sensor_hole_offset.1, 0.0);

        // E-stop hole
        let estop_hole = centered_cylinder(
            "estop_hole",
            cfg.estop_hole_diameter / 2.0,
            shell.thickness * 3.0,
            segments,
        )
        .translate(cfg.estop_hole_offset.0, cfg.estop_hole_offset.1, 0.0);

        // GPS grommet hole
        let gps_hole = centered_cylinder(
            "gps_grommet",
            cfg.gps_grommet_diameter / 2.0,
            shell.thickness * 3.0,
            segments,
        )
        .translate(
            cfg.sensor_hole_offset.0 + cfg.gps_grommet_offset.0,
            cfg.sensor_hole_offset.1 + cfg.gps_grommet_offset.1,
            0.0,
        );

        // Mounting holes
        let mounts = self.create_mount_holes(segments);

        panel
            .difference(&sensor_hole)
            .difference(&estop_hole)
            .difference(&gps_hole)
            .difference(&mounts)
    }

    fn create_mount_holes(&self, segments: u32) -> Part {
        let cfg = &self.config;
        let shell = &cfg.shell;

        let width = shell.shell_width();
        let length = shell.shell_length();
        let inset = shell.mount_hole_inset;

        let hole = |x: f64, y: f64| {
            centered_cylinder(
                "mount_hole",
                shell.mount_hole_diameter / 2.0,
                shell.thickness * 3.0,
                segments,
            )
            .translate(x, y, 0.0)
        };

        let hx = width / 2.0 - inset;
        let hy = length / 2.0 - inset;

        // 12 holes around perimeter (3 per side + corners)
        hole(-hx, hy)
            .union(&hole(0.0, hy))
            .union(&hole(hx, hy))
            .union(&hole(-hx, hy / 2.0))
            .union(&hole(hx, hy / 2.0))
            .union(&hole(-hx, 0.0))
            .union(&hole(hx, 0.0))
            .union(&hole(-hx, -hy / 2.0))
            .union(&hole(hx, -hy / 2.0))
            .union(&hole(-hx, -hy))
            .union(&hole(0.0, -hy))
            .union(&hole(hx, -hy))
    }

    /// Generate 2D DXF profile for laser cutting
    pub fn to_dxf(&self) -> DxfDocument {
        let cfg = &self.config;
        let shell = &cfg.shell;

        let mut dxf = DxfDocument::new();

        let width = shell.shell_width();
        let length = shell.shell_length();
        let inset = shell.mount_hole_inset;

        // Outer panel rectangle
        dxf.add_rectangle(width, length, 0.0, 0.0);

        // Sensor mast hole (large)
        dxf.add_circle(
            cfg.sensor_hole_offset.0,
            cfg.sensor_hole_offset.1,
            cfg.sensor_hole_diameter / 2.0,
        );

        // E-stop hole
        dxf.add_circle(
            cfg.estop_hole_offset.0,
            cfg.estop_hole_offset.1,
            cfg.estop_hole_diameter / 2.0,
        );

        // GPS grommet hole
        dxf.add_circle(
            cfg.sensor_hole_offset.0 + cfg.gps_grommet_offset.0,
            cfg.sensor_hole_offset.1 + cfg.gps_grommet_offset.1,
            cfg.gps_grommet_diameter / 2.0,
        );

        // Mounting holes (12 around perimeter)
        let hx = width / 2.0 - inset;
        let hy = length / 2.0 - inset;
        let r = shell.mount_hole_diameter / 2.0;

        for (x, y) in [
            (-hx, hy), (0.0, hy), (hx, hy),
            (-hx, hy / 2.0), (hx, hy / 2.0),
            (-hx, 0.0), (hx, 0.0),
            (-hx, -hy / 2.0), (hx, -hy / 2.0),
            (-hx, -hy), (0.0, -hy), (hx, -hy),
        ] {
            dxf.add_circle(x, y, r);
        }

        dxf
    }
}

// =============================================================================
// Skid Plate (Panel 3 - Bottom)
// =============================================================================

/// Skid Plate configuration (bottom panel)
#[derive(Debug, Clone)]
pub struct SkidPlateConfig {
    pub shell: ShellConfig,
    /// Drain hole diameter (mm)
    pub drain_hole_diameter: f64,
}

impl Default for SkidPlateConfig {
    fn default() -> Self {
        Self {
            shell: ShellConfig::default(),
            drain_hole_diameter: 6.0,
        }
    }
}

/// Skid Plate: Bottom panel for protection and closure
pub struct SkidPlate {
    config: SkidPlateConfig,
}

impl SkidPlate {
    pub fn new(config: SkidPlateConfig) -> Self {
        Self { config }
    }

    pub fn default_bvr1() -> Self {
        Self::new(SkidPlateConfig::default())
    }

    /// Generate 3D representation
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let shell = &cfg.shell;
        let segments = 32;

        let width = shell.shell_width();
        let length = shell.shell_length();

        // Main panel body
        let panel = centered_cube("skid_plate", width, length, shell.thickness);

        // Mounting holes
        let mounts = self.create_mount_holes(segments);

        // Drain holes at corners
        let drains = self.create_drain_holes(segments);

        panel.difference(&mounts).difference(&drains)
    }

    fn create_mount_holes(&self, segments: u32) -> Part {
        let cfg = &self.config;
        let shell = &cfg.shell;

        let width = shell.shell_width();
        let length = shell.shell_length();
        let inset = shell.mount_hole_inset;

        let hole = |x: f64, y: f64| {
            centered_cylinder(
                "mount_hole",
                shell.mount_hole_diameter / 2.0,
                shell.thickness * 3.0,
                segments,
            )
            .translate(x, y, 0.0)
        };

        let hx = width / 2.0 - inset;
        let hy = length / 2.0 - inset;

        // 12 holes around perimeter (matching top lid)
        hole(-hx, hy)
            .union(&hole(0.0, hy))
            .union(&hole(hx, hy))
            .union(&hole(-hx, hy / 2.0))
            .union(&hole(hx, hy / 2.0))
            .union(&hole(-hx, 0.0))
            .union(&hole(hx, 0.0))
            .union(&hole(-hx, -hy / 2.0))
            .union(&hole(hx, -hy / 2.0))
            .union(&hole(-hx, -hy))
            .union(&hole(0.0, -hy))
            .union(&hole(hx, -hy))
    }

    fn create_drain_holes(&self, segments: u32) -> Part {
        let cfg = &self.config;
        let shell = &cfg.shell;

        let width = shell.shell_width();
        let length = shell.shell_length();
        let inset = 30.0; // Corner inset for drains

        let hole = |x: f64, y: f64| {
            centered_cylinder(
                "drain_hole",
                cfg.drain_hole_diameter / 2.0,
                shell.thickness * 3.0,
                segments,
            )
            .translate(x, y, 0.0)
        };

        let hx = width / 2.0 - inset;
        let hy = length / 2.0 - inset;

        // 4 drain holes at corners
        hole(-hx, hy)
            .union(&hole(hx, hy))
            .union(&hole(-hx, -hy))
            .union(&hole(hx, -hy))
    }

    /// Generate 2D DXF profile for laser cutting
    pub fn to_dxf(&self) -> DxfDocument {
        let cfg = &self.config;
        let shell = &cfg.shell;

        let mut dxf = DxfDocument::new();

        let width = shell.shell_width();
        let length = shell.shell_length();
        let inset = shell.mount_hole_inset;

        // Outer panel rectangle
        dxf.add_rectangle(width, length, 0.0, 0.0);

        // Mounting holes (12 around perimeter)
        let hx = width / 2.0 - inset;
        let hy = length / 2.0 - inset;
        let r = shell.mount_hole_diameter / 2.0;

        for (x, y) in [
            (-hx, hy), (0.0, hy), (hx, hy),
            (-hx, hy / 2.0), (hx, hy / 2.0),
            (-hx, 0.0), (hx, 0.0),
            (-hx, -hy / 2.0), (hx, -hy / 2.0),
            (-hx, -hy), (0.0, -hy), (hx, -hy),
        ] {
            dxf.add_circle(x, y, r);
        }

        // Drain holes at corners
        let drain_inset = 30.0;
        let dhx = width / 2.0 - drain_inset;
        let dhy = length / 2.0 - drain_inset;
        let dr = cfg.drain_hole_diameter / 2.0;

        for (x, y) in [(-dhx, dhy), (dhx, dhy), (-dhx, -dhy), (dhx, -dhy)] {
            dxf.add_circle(x, y, dr);
        }

        dxf
    }
}

// =============================================================================
// Complete Shell Assembly
// =============================================================================

/// Complete 3-panel shell assembly (Wall Wrap + Top Lid + Skid Plate)
pub struct ShellAssembly {
    config: ShellConfig,
}

impl ShellAssembly {
    pub fn new(config: ShellConfig) -> Self {
        Self { config }
    }

    pub fn default_bvr1() -> Self {
        Self::new(ShellConfig::default())
    }

    /// Generate complete shell assembly (wall wrap + top lid + skid plate in position)
    pub fn generate(&self) -> Part {
        let shell = &self.config;
        let height = shell.shell_height();

        // Wall wrap (in assembled position)
        let wall_wrap = WallWrap::new(WallWrapConfig {
            shell: self.config.clone(),
            ..Default::default()
        })
        .generate();

        // Top lid (positioned on top)
        let top_lid = TopLid::new(TopLidConfig {
            shell: self.config.clone(),
            ..Default::default()
        })
        .generate()
        .translate(0.0, 0.0, height);

        // Skid plate (positioned at bottom)
        let skid_plate = SkidPlate::new(SkidPlateConfig {
            shell: self.config.clone(),
            ..Default::default()
        })
        .generate()
        .translate(0.0, 0.0, 0.0);

        wall_wrap.union(&top_lid).union(&skid_plate)
    }

    /// Get wall wrap for individual export
    pub fn wall_wrap(&self) -> WallWrap {
        WallWrap::new(WallWrapConfig {
            shell: self.config.clone(),
            ..Default::default()
        })
    }

    /// Get top lid for individual export
    pub fn top_lid(&self) -> TopLid {
        TopLid::new(TopLidConfig {
            shell: self.config.clone(),
            ..Default::default()
        })
    }

    /// Get skid plate for individual export
    pub fn skid_plate(&self) -> SkidPlate {
        SkidPlate::new(SkidPlateConfig {
            shell: self.config.clone(),
            ..Default::default()
        })
    }

    /// Export all panels as DXF files for laser cutting
    pub fn export_dxf_files(&self, dir: impl AsRef<Path>) -> std::io::Result<()> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;

        // Wall Wrap (flat pattern with bend lines)
        self.wall_wrap().to_dxf().export(dir.join("shell_wall_wrap.dxf"))?;

        // Top Lid
        self.top_lid().to_dxf().export(dir.join("shell_top_lid.dxf"))?;

        // Skid Plate
        self.skid_plate().to_dxf().export(dir.join("shell_skid_plate.dxf"))?;

        Ok(())
    }
}

// =============================================================================
// Legacy exports for compatibility
// =============================================================================

// Keep the old types as aliases for now
pub type FrontPanel = WallWrap;
pub type RearPanel = WallWrap;
pub type SidePanel = WallWrap;
pub type TopPanel = TopLid;

pub type FrontPanelConfig = WallWrapConfig;
pub type RearPanelConfig = WallWrapConfig;
pub type SidePanelConfig = WallWrapConfig;
pub type TopPanelConfig = TopLidConfig;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_config_dimensions() {
        let cfg = ShellConfig::default();
        assert_eq!(cfg.shell_width(), 420.0);  // 380 + 20*2
        assert_eq!(cfg.shell_length(), 540.0); // 500 + 20*2
        assert_eq!(cfg.shell_height(), 200.0); // 180 + 20
    }

    #[test]
    fn test_wall_wrap_config() {
        let cfg = WallWrapConfig::default();
        assert_eq!(cfg.front_width(), 420.0);
        assert_eq!(cfg.side_length(), 540.0);
        assert_eq!(cfg.panel_height(), 200.0);

        // Flat width should be roughly: 420 + 540 + 420 + 540 + 20 - bend allowances
        let flat_w = cfg.flat_width();
        assert!(flat_w > 1900.0 && flat_w < 1950.0, "Flat width: {}", flat_w);
    }

    #[test]
    fn test_wall_wrap_generation() {
        let wrap = WallWrap::default_bvr1();
        let part = wrap.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_top_lid_generation() {
        let lid = TopLid::default_bvr1();
        let part = lid.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_shell_assembly_generation() {
        let assembly = ShellAssembly::default_bvr1();
        let part = assembly.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_wall_wrap_dxf() {
        let wrap = WallWrap::default_bvr1();
        let dxf = wrap.to_dxf();

        let path = "/tmp/test_wall_wrap.dxf";
        dxf.export(path).unwrap();

        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("LWPOLYLINE"), "Should have polylines");
        assert!(content.contains("BEND"), "Should have bend lines");
        assert!(content.contains("LINE"), "Should have line entities");
    }

    #[test]
    fn test_top_lid_dxf() {
        let lid = TopLid::default_bvr1();
        let dxf = lid.to_dxf();

        let path = "/tmp/test_top_lid.dxf";
        dxf.export(path).unwrap();

        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("CIRCLE"), "Should have circles for holes");
        assert!(content.contains("LWPOLYLINE"), "Should have polyline for outline");
    }

    #[test]
    fn test_louver_vent_area() {
        let cfg = WallWrapConfig::default();
        // Each louver: 60mm × 8mm = 480mm²
        // 8 louvers = 3840mm² > 150mm² minimum from artifact-plan
        let area = cfg.louver_width * cfg.louver_height * cfg.louver_count as f64;
        assert!(area >= 150.0, "Louver area {} should be >= 150mm²", area);
    }

    #[test]
    fn test_skid_plate_generation() {
        let plate = SkidPlate::default_bvr1();
        let part = plate.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_skid_plate_dxf() {
        let plate = SkidPlate::default_bvr1();
        let dxf = plate.to_dxf();

        let path = "/tmp/test_skid_plate.dxf";
        dxf.export(path).unwrap();

        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("CIRCLE"), "Should have circles for holes");
        assert!(content.contains("LWPOLYLINE"), "Should have polyline for outline");
    }

    #[test]
    fn test_export_dxf_files() {
        let assembly = ShellAssembly::default_bvr1();
        let dir = "/tmp/shell_dxf_test";
        assembly.export_dxf_files(dir).unwrap();

        assert!(std::path::Path::new(dir).join("shell_wall_wrap.dxf").exists());
        assert!(std::path::Path::new(dir).join("shell_top_lid.dxf").exists());
        assert!(std::path::Path::new(dir).join("shell_skid_plate.dxf").exists());
    }
}
