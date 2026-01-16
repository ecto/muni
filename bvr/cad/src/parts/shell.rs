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
            clearance: 20.0,          // 20mm clearance around frame (assembly + tolerance)
            frame_width: 540.0,       // BVR1 frame width (500mm extrusion + 2×20mm)
            frame_length: 540.0,      // BVR1 frame length (500mm extrusion + 2×20mm)
            frame_height: 180.0,      // BVR1 frame height
            mount_hole_diameter: 5.3, // M5 clearance
            mount_hole_inset: 15.0,   // 15mm from edge (clear of bends)
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
    // --- Integrated bottom panel ---
    /// Include bottom panel in flat pattern (L-shaped)
    pub include_bottom: bool,
    // --- Knuckle hinge for lid attachment ---
    /// Number of hinge tabs on shell (lid gets N-1 mating tabs)
    pub hinge_tab_count: usize,
    /// Hinge tab width (mm)
    pub hinge_tab_width: f64,
    /// Hinge tab height - how far tabs extend (mm)
    pub hinge_tab_height: f64,
    /// Hinge pin hole diameter (mm) - for 6mm rod
    pub hinge_pin_diameter: f64,
    // --- Gas strut mounting ---
    /// Gas strut ball stud hole diameter (mm) - typically 8mm for M8 stud
    pub gas_strut_hole_diameter: f64,
    /// Gas strut hole offset from rear edge (mm)
    pub gas_strut_offset_from_rear: f64,
    /// Gas strut hole offset down from top edge (mm)
    pub gas_strut_offset_from_top: f64,
    // --- Quarter-turn latch mounting ---
    /// Quarter-turn receptacle hole diameter (mm) - typically 19mm
    pub quarter_turn_hole_diameter: f64,
    /// Quarter-turn hole inset from front corners (mm)
    pub quarter_turn_inset: f64,
}

impl Default for WallWrapConfig {
    fn default() -> Self {
        Self {
            shell: ShellConfig::default(),
            // Nozzle slot (500mm blower nozzle + 2mm tolerance)
            nozzle_width: 502.0,
            nozzle_height: 52.0,
            nozzle_corner_radius: 15.0,
            nozzle_offset_y: 60.0, // 60mm from bottom
            // LED channel (500mm LED strip + 2mm tolerance)
            led_channel_width: 502.0,
            led_channel_height: 22.0,
            led_channel_gap: 10.0, // 10mm above nozzle slot
            // Louver vents (8 slots, ~150mm² total area per artifact-plan)
            louver_width: 60.0,
            louver_height: 8.0,
            louver_count: 8,
            louver_spacing: 40.0, // Spread across 580mm rear panel
            // Drain holes
            drain_hole_diameter: 6.0,
            // Bend radius (standard for 2mm aluminum)
            bend_radius: 2.0,
            // Integrated bottom (L-shaped flat pattern)
            include_bottom: true,
            // Knuckle hinge (5 tabs on shell, 4 mating tabs on lid)
            hinge_tab_count: 5,
            hinge_tab_width: 40.0,   // 40mm wide tabs
            hinge_tab_height: 20.0,  // 20mm tall tabs
            hinge_pin_diameter: 6.5, // 6.5mm hole for 6mm rod with clearance
            // Gas strut mounting (M8 ball studs)
            gas_strut_hole_diameter: 8.5, // 8.5mm for M8 clearance
            gas_strut_offset_from_rear: 100.0, // 100mm from rear
            gas_strut_offset_from_top: 50.0,   // 50mm from top
            // Quarter-turn latches
            quarter_turn_hole_diameter: 19.0, // Standard quarter-turn size
            quarter_turn_inset: 60.0,         // 60mm from front corners
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
    ///
    /// Layout (with include_bottom=true):
    /// ```text
    ///                    ┌─────────────────────┐
    ///                    │       BOTTOM        │
    ///                    │      (580x580)      │
    ///                    └──────────┬──────────┘
    ///                               │ bend 4
    /// ┌────────┬─────────┬──────────┴──────────┬─────────┬───┐
    /// │ FRONT  │  LEFT   │ REAR (with tabs)    │  RIGHT  │gap│
    /// │  580   │   580   │        580          │   580   │20 │
    /// └────────┴─────────┴─────────────────────┴─────────┴───┘
    ///          ↑         ↑                     ↑
    ///        bend 1    bend 2               bend 3
    /// ```
    pub fn to_dxf(&self) -> DxfDocument {
        let cfg = &self.config;
        let mut dxf = DxfDocument::new();

        let wall_height = cfg.panel_height();
        let inset = cfg.shell.mount_hole_inset;
        let hole_r = cfg.shell.mount_hole_diameter / 2.0;

        // Calculate section dimensions
        let front_w = cfg.front_width();
        let side_l = cfg.side_length();
        let rear_w = cfg.rear_width();
        let bottom_w = cfg.shell.shell_width();
        let bottom_l = cfg.shell.shell_length();

        // Section centers (X position) - walls are centered at y=0
        // Layout: [FRONT][LEFT][REAR][RIGHT][gap]
        let front_cx = -front_w / 2.0 - side_l - rear_w / 2.0;
        let left_cx = -side_l / 2.0 - rear_w / 2.0;
        let rear_cx = 0.0;
        let right_cx = side_l / 2.0 + rear_w / 2.0;

        // Wall strip boundaries
        let wall_left = front_cx - front_w / 2.0;
        let wall_right = right_cx + side_l / 2.0 + 20.0; // +20mm gap
        let wall_bottom = -wall_height / 2.0;
        let wall_top = wall_height / 2.0;

        // Bottom panel (above rear section)
        let bottom_cx = rear_cx;
        let bottom_cy = wall_top + bottom_l / 2.0;

        // --- OUTER BOUNDARY ---
        if cfg.include_bottom {
            // L-shaped profile with knuckle tabs on rear top edge
            let tab_h = cfg.hinge_tab_height;
            let num_tabs = cfg.hinge_tab_count;
            let rear_left = rear_cx - rear_w / 2.0;
            let rear_right = rear_cx + rear_w / 2.0;

            // Build L-shaped outline with tabs
            let mut outline: Vec<(f64, f64)> = Vec::new();

            // Start at bottom-left of front panel, go clockwise
            outline.push((wall_left, wall_bottom));
            outline.push((wall_right, wall_bottom));
            outline.push((wall_right, wall_top));

            // Right side up to bottom panel connection
            outline.push((rear_right, wall_top));

            // Bottom panel right edge (going up)
            outline.push((rear_right, wall_top + bottom_l));

            // Bottom panel top edge
            outline.push((rear_left, wall_top + bottom_l));

            // Bottom panel left edge (going down)
            outline.push((rear_left, wall_top));

            // Back along wall top to rear section with tabs
            // We'll add tabs between rear_left and rear_right at wall_top
            // For now, simplified - tabs will be added separately
            outline.push((wall_left, wall_top));

            dxf.add_polyline(outline, true);

            // Knuckle tabs on rear section top edge
            // Shell has tabs at positions 0, 2, 4 (if 5 tabs total)
            // These extend upward from wall_top
            let tab_spacing = rear_w / (num_tabs as f64);
            for i in 0..num_tabs {
                if i % 2 == 0 {
                    // Tab on shell (extends up)
                    let tab_x = rear_left + tab_spacing * (i as f64 + 0.5);
                    let tab_left = tab_x - cfg.hinge_tab_width / 2.0;
                    let tab_right = tab_x + cfg.hinge_tab_width / 2.0;

                    // Tab outline (rectangular)
                    dxf.add_polyline(
                        vec![
                            (tab_left, wall_top),
                            (tab_left, wall_top + tab_h),
                            (tab_right, wall_top + tab_h),
                            (tab_right, wall_top),
                        ],
                        false, // Open polyline - connects to main outline
                    );

                    // Hinge pin hole in tab
                    let pin_y = wall_top + tab_h / 2.0;
                    dxf.add_circle(tab_x, pin_y, cfg.hinge_pin_diameter / 2.0);
                }
            }
        } else {
            // Simple rectangle (original behavior)
            let total_width = front_w + side_l * 2.0 + rear_w + 20.0;
            dxf.add_rectangle(total_width, wall_height, (right_cx + front_cx) / 2.0, 0.0);
        }

        // --- BEND LINES ---
        let bend1_x = front_cx + front_w / 2.0;
        let bend2_x = left_cx + side_l / 2.0;
        let bend3_x = rear_cx + rear_w / 2.0;

        dxf.add_bend_line(bend1_x, wall_bottom, bend1_x, wall_top);
        dxf.add_bend_line(bend2_x, wall_bottom, bend2_x, wall_top);
        dxf.add_bend_line(bend3_x, wall_bottom, bend3_x, wall_top);

        if cfg.include_bottom {
            // Bend 4: between rear wall top and bottom panel
            let rear_left = rear_cx - rear_w / 2.0;
            let rear_right = rear_cx + rear_w / 2.0;
            dxf.add_bend_line(rear_left, wall_top, rear_right, wall_top);
        }

        // --- FRONT SECTION ---
        let nozzle_cy = wall_bottom + cfg.nozzle_offset_y + cfg.nozzle_height / 2.0;
        dxf.add_rounded_rectangle(
            cfg.nozzle_width,
            cfg.nozzle_height,
            front_cx,
            nozzle_cy,
            cfg.nozzle_corner_radius,
        );

        let led_cy = nozzle_cy + cfg.nozzle_height / 2.0 + cfg.led_channel_gap + cfg.led_channel_height / 2.0;
        dxf.add_rounded_rectangle(
            cfg.led_channel_width,
            cfg.led_channel_height,
            front_cx,
            led_cy,
            cfg.led_channel_height / 4.0,
        );

        // Front mounting holes
        let front_hx = front_w / 2.0 - inset;
        let hy = wall_height / 2.0 - inset;
        for (dx, dy) in [(-front_hx, hy), (front_hx, hy), (-front_hx, -hy), (front_hx, -hy)] {
            dxf.add_circle(front_cx + dx, dy, hole_r);
        }

        // Quarter-turn latch holes on front (near top edge)
        let qt_y = wall_top - cfg.quarter_turn_inset;
        let qt_x_left = front_cx - front_w / 2.0 + cfg.quarter_turn_inset;
        let qt_x_right = front_cx + front_w / 2.0 - cfg.quarter_turn_inset;
        dxf.add_circle(qt_x_left, qt_y, cfg.quarter_turn_hole_diameter / 2.0);
        dxf.add_circle(qt_x_right, qt_y, cfg.quarter_turn_hole_diameter / 2.0);

        // --- LEFT SIDE SECTION ---
        let side_hx = side_l / 2.0 - inset;
        for (dx, dy) in [
            (-side_hx, hy), (side_hx, hy),
            (-side_hx, 0.0), (side_hx, 0.0),
            (-side_hx, -hy), (side_hx, -hy),
        ] {
            dxf.add_circle(left_cx + dx, dy, hole_r);
        }

        // Gas strut hole on left side
        let gas_y = wall_top - cfg.gas_strut_offset_from_top;
        let gas_x_left = left_cx + side_l / 2.0 - cfg.gas_strut_offset_from_rear;
        dxf.add_circle(gas_x_left, gas_y, cfg.gas_strut_hole_diameter / 2.0);

        // --- REAR SECTION ---
        let total_louver_span = (cfg.louver_count - 1) as f64 * cfg.louver_spacing;
        let louver_start_x = rear_cx - total_louver_span / 2.0;
        let louver_cy = wall_height / 4.0;

        for i in 0..cfg.louver_count {
            let lx = louver_start_x + i as f64 * cfg.louver_spacing;
            dxf.add_slot(cfg.louver_width, cfg.louver_height, lx, louver_cy);
        }

        let drain_inset = 20.0;
        let rear_hx = rear_w / 2.0 - drain_inset;
        let drain_y = wall_bottom + drain_inset;
        dxf.add_circle(rear_cx - rear_hx, drain_y, cfg.drain_hole_diameter / 2.0);
        dxf.add_circle(rear_cx + rear_hx, drain_y, cfg.drain_hole_diameter / 2.0);

        let rear_mount_hx = rear_w / 2.0 - inset;
        for (dx, dy) in [(-rear_mount_hx, hy), (rear_mount_hx, hy), (-rear_mount_hx, -hy), (rear_mount_hx, -hy)] {
            dxf.add_circle(rear_cx + dx, dy, hole_r);
        }

        // --- RIGHT SIDE SECTION ---
        for (dx, dy) in [
            (-side_hx, hy), (side_hx, hy),
            (-side_hx, 0.0), (side_hx, 0.0),
            (-side_hx, -hy), (side_hx, -hy),
        ] {
            dxf.add_circle(right_cx + dx, dy, hole_r);
        }

        // Gas strut hole on right side
        let gas_x_right = right_cx - side_l / 2.0 + cfg.gas_strut_offset_from_rear;
        dxf.add_circle(gas_x_right, gas_y, cfg.gas_strut_hole_diameter / 2.0);

        // --- BOTTOM PANEL (if included) ---
        if cfg.include_bottom {
            // Mounting holes around bottom perimeter
            let bx = bottom_w / 2.0 - inset;
            let by = bottom_l / 2.0 - inset;
            for (dx, dy) in [
                (-bx, by), (0.0, by), (bx, by),
                (-bx, 0.0), (bx, 0.0),
                (-bx, -by), (0.0, -by), (bx, -by),
            ] {
                dxf.add_circle(bottom_cx + dx, bottom_cy + dy, hole_r);
            }

            // Drain holes in bottom corners
            let bd = 30.0; // Drain inset
            for (dx, dy) in [(-bx + bd, -by + bd), (bx - bd, -by + bd), (-bx + bd, by - bd), (bx - bd, by - bd)] {
                dxf.add_circle(bottom_cx + dx, bottom_cy + dy, cfg.drain_hole_diameter / 2.0);
            }
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
    // --- Knuckle hinge (mates with shell tabs) ---
    /// Number of hinge tabs on lid (shell has N+1 mating tabs)
    pub hinge_tab_count: usize,
    /// Hinge tab width (mm)
    pub hinge_tab_width: f64,
    /// Hinge tab height - how far tabs extend (mm)
    pub hinge_tab_height: f64,
    /// Hinge pin hole diameter (mm)
    pub hinge_pin_diameter: f64,
    // --- Gas strut mounting ---
    /// Gas strut ball stud hole diameter (mm)
    pub gas_strut_hole_diameter: f64,
    /// Gas strut hole offset from rear edge (mm)
    pub gas_strut_offset_from_rear: f64,
    /// Gas strut hole offset from side edges (mm)
    pub gas_strut_offset_from_side: f64,
    // --- Quarter-turn latch mounting ---
    /// Quarter-turn receptacle hole diameter (mm)
    pub quarter_turn_hole_diameter: f64,
    /// Quarter-turn hole inset from front corners (mm)
    pub quarter_turn_inset: f64,
}

impl Default for TopLidConfig {
    fn default() -> Self {
        let shell = ShellConfig::default();
        Self {
            sensor_hole_diameter: 150.0,
            sensor_hole_offset: (0.0, -shell.shell_length() / 4.0),
            estop_hole_diameter: 30.0,
            estop_hole_offset: (shell.shell_width() / 4.0, shell.shell_length() / 4.0),
            gps_grommet_diameter: 12.0,
            gps_grommet_offset: (80.0, 20.0),
            // Knuckle hinge (4 tabs on lid mate with 5 tabs on shell)
            hinge_tab_count: 4,
            hinge_tab_width: 40.0,
            hinge_tab_height: 20.0,
            hinge_pin_diameter: 6.5,
            // Gas strut mounting
            gas_strut_hole_diameter: 8.5,
            gas_strut_offset_from_rear: 80.0,
            gas_strut_offset_from_side: 30.0,
            // Quarter-turn latches
            quarter_turn_hole_diameter: 19.0,
            quarter_turn_inset: 60.0,
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
    ///
    /// Includes:
    /// - Knuckle hinge tabs on rear edge (mate with shell)
    /// - Gas strut mounting holes on sides
    /// - Quarter-turn latch holes on front edge
    /// - Sensor mast, e-stop, and GPS grommet holes
    pub fn to_dxf(&self) -> DxfDocument {
        let cfg = &self.config;
        let shell = &cfg.shell;

        let mut dxf = DxfDocument::new();

        let width = shell.shell_width();
        let length = shell.shell_length();
        let inset = shell.mount_hole_inset;

        // --- OUTER BOUNDARY WITH KNUCKLE TABS ---
        // Tabs extend from rear edge (-Y direction)
        let num_tabs = cfg.hinge_tab_count;
        let tab_h = cfg.hinge_tab_height;
        let tab_w = cfg.hinge_tab_width;

        // Main panel corners
        let left = -width / 2.0;
        let right = width / 2.0;
        let front = length / 2.0;   // +Y is front
        let rear = -length / 2.0;   // -Y is rear

        // Build outline with tabs on rear edge
        let mut outline: Vec<(f64, f64)> = Vec::new();

        // Front edge (left to right)
        outline.push((left, front));
        outline.push((right, front));

        // Right edge (front to rear)
        outline.push((right, rear));

        // Rear edge with tabs (right to left)
        // Tabs extend downward (-Y) from rear edge
        // Lid tabs at positions 1, 3 (offset from shell tabs at 0, 2, 4)
        let tab_spacing = width / ((num_tabs + 1) as f64);
        let mut x = right;

        for i in 0..=num_tabs {
            let next_x = right - tab_spacing * ((i + 1) as f64);

            if i < num_tabs && i % 2 == 0 {
                // Gap (no tab) - shell's tab goes here
                outline.push((x, rear));
                outline.push((next_x.max(left), rear));
            } else if i < num_tabs {
                // Lid tab extends downward
                let tab_left = (x - tab_w / 2.0).max(left);
                let tab_right = (x - tab_spacing + tab_w / 2.0).min(right);
                let tab_center = (tab_left + tab_right) / 2.0;

                outline.push((tab_right + (tab_spacing - tab_w) / 2.0, rear));
                outline.push((tab_right, rear));
                outline.push((tab_right, rear - tab_h));
                outline.push((tab_left, rear - tab_h));
                outline.push((tab_left, rear));
                outline.push((tab_left - (tab_spacing - tab_w) / 2.0, rear));

                // Add hinge pin hole for this tab
                dxf.add_circle(tab_center, rear - tab_h / 2.0, cfg.hinge_pin_diameter / 2.0);
            }
            x = next_x;
        }

        // Left edge (rear to front)
        outline.push((left, rear));
        outline.push((left, front));

        dxf.add_polyline(outline, true);

        // --- FUNCTIONAL HOLES ---
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

        // --- GAS STRUT MOUNTING HOLES ---
        let gas_y = rear + cfg.gas_strut_offset_from_rear;
        let gas_x_left = left + cfg.gas_strut_offset_from_side;
        let gas_x_right = right - cfg.gas_strut_offset_from_side;
        dxf.add_circle(gas_x_left, gas_y, cfg.gas_strut_hole_diameter / 2.0);
        dxf.add_circle(gas_x_right, gas_y, cfg.gas_strut_hole_diameter / 2.0);

        // --- QUARTER-TURN LATCH HOLES ---
        let qt_y = front - cfg.quarter_turn_inset;
        let qt_x_left = left + cfg.quarter_turn_inset;
        let qt_x_right = right - cfg.quarter_turn_inset;
        dxf.add_circle(qt_x_left, qt_y, cfg.quarter_turn_hole_diameter / 2.0);
        dxf.add_circle(qt_x_right, qt_y, cfg.quarter_turn_hole_diameter / 2.0);

        // --- MOUNTING HOLES (reduced - tabs replace some) ---
        let hx = width / 2.0 - inset;
        let hy = length / 2.0 - inset;
        let r = shell.mount_hole_diameter / 2.0;

        // Front edge holes (3)
        for x in [-hx, 0.0, hx] {
            dxf.add_circle(x, hy, r);
        }

        // Side holes (4 per side, excluding rear corners which have tabs)
        for x in [-hx, hx] {
            for y in [hy / 3.0, 0.0, -hy / 3.0] {
                dxf.add_circle(x, y, r);
            }
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

/// Complete 2-panel shell assembly (Big Shell with integrated bottom + Hinged Top Lid)
///
/// New design features:
/// - Big Shell: Wall wrap + bottom as single L-shaped piece (4 bends)
/// - Top Lid: Hinged with knuckle tabs, opens for maintenance
/// - Gas struts hold lid open
/// - Quarter-turn latches secure lid when closed
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

    /// Generate complete shell assembly (big shell + hinged top lid)
    pub fn generate(&self) -> Part {
        let shell = &self.config;
        let height = shell.shell_height();

        // Big shell with integrated bottom (wall wrap with include_bottom=true)
        let big_shell = WallWrap::new(WallWrapConfig {
            shell: self.config.clone(),
            include_bottom: true,
            ..Default::default()
        })
        .generate();

        // Top lid (positioned on top, hinged at rear)
        let top_lid = TopLid::new(TopLidConfig {
            shell: self.config.clone(),
            ..Default::default()
        })
        .generate()
        .translate(0.0, 0.0, height);

        // Gas struts (simplified representation)
        let gas_struts = self.generate_gas_struts();

        big_shell.union(&top_lid).union(&gas_struts)
    }

    /// Generate simplified gas strut representation for visualization
    fn generate_gas_struts(&self) -> Part {
        use crate::centered_cylinder;

        let shell = &self.config;
        let height = shell.shell_height();
        let length = shell.shell_length();
        let width = shell.shell_width();

        // Gas strut parameters
        let strut_diameter = 15.0;
        let strut_length = 200.0; // Extended length
        let segments = 16;

        // Strut positions (on side walls, near rear, angling up to lid)
        let strut_x = width / 2.0 - 30.0;
        let strut_y_bottom = -length / 2.0 + 100.0; // Wall attachment point
        let strut_z_bottom = height - 50.0;
        let strut_z_top = height + 20.0; // Lid attachment point

        // Simple cylinder representation (angled strut)
        let strut_right = centered_cylinder("gas_strut", strut_diameter / 2.0, strut_length, segments)
            .rotate(15.0, 0.0, 0.0) // Angled
            .translate(strut_x, strut_y_bottom, (strut_z_bottom + strut_z_top) / 2.0);

        let strut_left = centered_cylinder("gas_strut", strut_diameter / 2.0, strut_length, segments)
            .rotate(15.0, 0.0, 0.0)
            .translate(-strut_x, strut_y_bottom, (strut_z_bottom + strut_z_top) / 2.0);

        strut_right.union(&strut_left)
    }

    /// Get big shell (wall wrap + integrated bottom) for individual export
    pub fn wall_wrap(&self) -> WallWrap {
        WallWrap::new(WallWrapConfig {
            shell: self.config.clone(),
            include_bottom: true, // New design has integrated bottom
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

    /// Get skid plate for individual export (legacy, now integrated into wall_wrap)
    #[deprecated(note = "Use wall_wrap() which now includes integrated bottom")]
    pub fn skid_plate(&self) -> SkidPlate {
        SkidPlate::new(SkidPlateConfig {
            shell: self.config.clone(),
            ..Default::default()
        })
    }

    /// Export all panels as DXF files for laser cutting
    ///
    /// New 2-panel design:
    /// - shell_big_shell.dxf: L-shaped walls + bottom (4 bends)
    /// - shell_top_lid.dxf: Hinged lid with knuckle tabs
    pub fn export_dxf_files(&self, dir: impl AsRef<Path>) -> std::io::Result<()> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;

        // Big Shell (L-shaped flat pattern with bend lines)
        self.wall_wrap().to_dxf().export(dir.join("shell_wall_wrap.dxf"))?;

        // Top Lid (with knuckle hinge tabs)
        self.top_lid().to_dxf().export(dir.join("shell_top_lid.dxf"))?;

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
        // With 540mm frame + 20mm clearance each side = 580mm
        assert_eq!(cfg.shell_width(), 580.0);  // 540 + 20*2
        assert_eq!(cfg.shell_length(), 580.0); // 540 + 20*2
        assert_eq!(cfg.shell_height(), 200.0); // 180 + 20
    }

    #[test]
    fn test_wall_wrap_config() {
        let cfg = WallWrapConfig::default();
        // With 540mm frame + 20mm clearance each side = 580mm
        assert_eq!(cfg.front_width(), 580.0);
        assert_eq!(cfg.side_length(), 580.0);
        assert_eq!(cfg.panel_height(), 200.0);

        // Flat width: 580 + 580 + 580 + 580 + 20 - bend allowances
        let flat_w = cfg.flat_width();
        assert!(flat_w > 2300.0 && flat_w < 2400.0, "Flat width: {}", flat_w);
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

        // 2-panel design: wall wrap + top lid (no separate skid plate)
        assert!(std::path::Path::new(dir).join("shell_wall_wrap.dxf").exists());
        assert!(std::path::Path::new(dir).join("shell_top_lid.dxf").exists());
    }
}
