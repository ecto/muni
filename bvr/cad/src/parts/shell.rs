//! Friendly Industrial Shell Design for BVR1 Rover
//!
//! "Low to the ground. Wider than it is tall. Planted."
//!
//! Design language:
//! - **Dignified, not performing** - not cute, not aggressive, just present
//! - **Matte orange** - municipal, visible, warm (RAL 2004)
//! - **Gentle forehead slope** - 15° rake on front, welcoming not threatening
//! - **Chamfered corners** - 45° transitions, softens the box without curves
//! - **The Face** - stereo cameras as "eyes", LED bar as "gaze" (pareidolia-friendly)
//!
//! Shell components:
//! - **Wall Wrap**: Front + sides + rear as single bent piece (vertical walls only)
//! - **Top Lid**: Hinged lid with front visor (the "forehead" with camera eyes)
//! - **Sensor Dome**: 3D printed cover for LiDAR only (cameras are in the visor)
//!
//! Material: 5052-H32 Aluminum, 2mm thickness
//! Finish: Matte orange powder coat (RAL 2004)
//!
//! Manufacturing: SendCutSend laser cut + bend + powder coat

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
///
/// The "friendly industrial" wall wrap has:
/// - Simple 90° corners (4 vertical bends)
/// - Vertical front panel (the "forehead" visor is now part of the lid)
/// - LED channel positioned as the robot's "mouth" - steady, aware
#[derive(Debug, Clone)]
pub struct WallWrapConfig {
    pub shell: ShellConfig,
    // --- Friendly Industrial Design ---
    // NOTE: Front rake (forehead with camera eyes) moved to TopLid as visor
    // The wall wrap front panel is now fully vertical
    // --- Blower Integration ---
    /// Nozzle slot width (mm)
    pub nozzle_width: f64,
    /// Nozzle slot height (mm)
    pub nozzle_height: f64,
    /// Nozzle slot corner radius (mm)
    pub nozzle_corner_radius: f64,
    /// Nozzle slot vertical offset from bottom (mm)
    pub nozzle_offset_y: f64,
    // --- LED "Face" ---
    /// LED channel slot width (mm) - the robot's "gaze"
    pub led_channel_width: f64,
    /// LED channel slot height (mm) - sized for diffuser strip
    pub led_channel_height: f64,
    /// LED channel offset above nozzle (mm)
    pub led_channel_gap: f64,
    /// LED diffuser mounting hole diameter (mm)
    pub led_mount_hole_diameter: f64,
    /// LED diffuser mounting hole spacing from channel ends (mm)
    pub led_mount_hole_inset: f64,
    // NOTE: Stereo camera "eyes" moved to TopLid visor configuration
    // --- Rear Vents ---
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
            // NOTE: Forehead with camera eyes is now on the lid visor
            // --- Blower Integration ---
            // Nozzle slot (500mm blower nozzle + 2mm tolerance)
            nozzle_width: 502.0,
            nozzle_height: 52.0,
            nozzle_corner_radius: 15.0,
            nozzle_offset_y: 40.0, // 40mm from bottom (in vertical section)
            // --- LED "Face" ---
            // LED channel spans most of front width - the robot's steady gaze
            led_channel_width: 480.0, // Slightly narrower than nozzle for visual balance
            led_channel_height: 25.0, // Sized for standard diffuser strip
            led_channel_gap: 15.0,    // 15mm above nozzle slot
            led_mount_hole_diameter: 3.5, // M3 mounting holes for diffuser clips
            led_mount_hole_inset: 20.0,   // Holes 20mm from channel ends
            // NOTE: Stereo camera "eyes" are now on the lid visor
            // --- Rear Vents ---
            // Louver vents (8 slots, ~150mm² total area per artifact-plan)
            louver_width: 60.0,
            louver_height: 8.0,
            louver_count: 8,
            louver_spacing: 40.0, // Spread across rear panel
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
    /// Front section width (full shell width)
    pub fn front_width(&self) -> f64 {
        self.shell.shell_width()
    }

    /// Side section length (full shell length)
    pub fn side_length(&self) -> f64 {
        self.shell.shell_length()
    }

    /// Rear section width (full shell width)
    pub fn rear_width(&self) -> f64 {
        self.shell.shell_width()
    }

    /// Panel height (same as shell height)
    /// Note: This is the vertical wall height only - the visor adds height on the lid
    pub fn panel_height(&self) -> f64 {
        self.shell.shell_height()
    }

    /// Total flat pattern width (before bending)
    ///
    /// Simple 4-bend layout: [FRONT][LEFT][REAR][RIGHT][gap]
    ///
    /// - Front/Rear: 580mm each
    /// - Sides: 580mm each
    /// - Gap: 20mm
    /// - Total: 580×4 + 20 = 2340mm
    ///
    /// Bend lines (4 total with bottom):
    /// 1-3. Three 90° corner bends
    /// 4. Bottom panel bend (if include_bottom)
    ///
    /// Note: Front rake removed - the forehead visor is now part of the lid
    pub fn flat_width(&self) -> f64 {
        let front = self.front_width();
        let sides = self.side_length() * 2.0;
        let rear = self.rear_width();

        // Total (bend allowances are negligible for 2mm radius)
        front + sides + rear + 20.0 // +20mm gap at end
    }

    /// Number of bend lines in the wall wrap
    pub fn bend_count(&self) -> usize {
        // 3 corner bends
        // With integrated bottom: + 1 = 4
        if self.include_bottom { 4 } else { 3 }
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

    /// Generate 3D representation with vertical walls
    ///
    /// The friendly industrial look:
    /// - Simple 90° corners (clean box shape)
    /// - Flat front panel with stereo camera "eyes"
    /// - LED channel is the robot's "mouth"
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let height = cfg.panel_height();
        let thickness = cfg.shell.thickness;

        // Overall shell dimensions
        let width = cfg.shell.shell_width();
        let length = cfg.shell.shell_length();

        // === FRONT PANEL (flat, full height) ===
        let front_w = cfg.front_width();
        let front = centered_cube("front_panel", front_w, thickness, height)
            .translate(0.0, length / 2.0, height / 2.0);

        // === SIDE PANELS ===
        let side_l = cfg.side_length();

        // Left side
        let left = centered_cube("left_panel", thickness, side_l, height)
            .translate(-width / 2.0, 0.0, height / 2.0);

        // Right side
        let right = centered_cube("right_panel", thickness, side_l, height)
            .translate(width / 2.0, 0.0, height / 2.0);

        // === REAR PANEL ===
        let rear_w = cfg.rear_width();
        let rear = centered_cube("rear_panel", rear_w, thickness, height)
            .translate(0.0, -length / 2.0, height / 2.0);

        // === CUTOUTS ===
        let segments = 32; // for circular cutouts

        // Nozzle cutout in front panel
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

        // LED channel cutout - the robot's "mouth"
        let led_z = cfg.nozzle_offset_y + cfg.nozzle_height + cfg.led_channel_gap + cfg.led_channel_height / 2.0;
        let led_cutout = centered_cube(
            "led_channel",
            cfg.led_channel_width,
            thickness * 3.0,
            cfg.led_channel_height,
        )
        .translate(0.0, length / 2.0, led_z);

        // Stereo camera "eyes" - positioned near top of front panel
        let lid_cfg = super::shell::TopLidConfig::default();
        let cam_z = height - lid_cfg.visor_camera_offset_from_top;
        let cam_spacing = lid_cfg.visor_camera_spacing;
        let cam_w = lid_cfg.visor_camera_window_width;
        let cam_h = lid_cfg.visor_camera_window_height;

        let left_eye = centered_cube("camera_left", cam_w, thickness * 3.0, cam_h)
            .translate(-cam_spacing / 2.0, length / 2.0, cam_z);
        let right_eye = centered_cube("camera_right", cam_w, thickness * 3.0, cam_h)
            .translate(cam_spacing / 2.0, length / 2.0, cam_z);
        let camera_cutouts = left_eye.union(&right_eye);

        // Louver vents in rear
        let mut louvers = Part::empty("louvers");
        let total_louver_width = (cfg.louver_count - 1) as f64 * cfg.louver_spacing;
        let start_x = -total_louver_width / 2.0;
        let louver_z = height * 0.6;

        for i in 0..cfg.louver_count {
            let x = start_x + i as f64 * cfg.louver_spacing;
            let louver = centered_cube(
                "louver",
                cfg.louver_width,
                thickness * 3.0,
                cfg.louver_height,
            )
            .translate(x, -length / 2.0, louver_z);
            louvers = louvers.union(&louver);
        }

        // Drain holes in rear corners
        let drain_inset = 20.0;
        let drain1 = centered_cylinder(
            "drain",
            cfg.drain_hole_diameter / 2.0,
            thickness * 3.0,
            segments,
        )
        .rotate(90.0, 0.0, 0.0)
        .translate(-rear_w / 2.0 + drain_inset, -length / 2.0, drain_inset);

        let drain2 = centered_cylinder(
            "drain",
            cfg.drain_hole_diameter / 2.0,
            thickness * 3.0,
            segments,
        )
        .rotate(90.0, 0.0, 0.0)
        .translate(rear_w / 2.0 - drain_inset, -length / 2.0, drain_inset);

        // Assemble the shell
        front
            .union(&left)
            .union(&right)
            .union(&rear)
            .difference(&nozzle_cutout)
            .difference(&led_cutout)
            .difference(&camera_cutouts)
            .difference(&louvers)
            .difference(&drain1)
            .difference(&drain2)
    }

    /// Generate 2D DXF flat pattern for laser cutting
    ///
    /// Simple 4-bend layout (vertical walls only):
    ///
    /// ```text
    ///                         ┌─────────────────────┐
    ///                         │       BOTTOM        │
    ///                         │      (580x580)      │
    ///                         └──────────┬──────────┘
    ///                                    │ bend 4
    /// ┌─────────┬─────────┬──────────────┴──────────┬─────────┬───┐
    /// │  FRONT  │  LEFT   │ REAR (with hinge tabs)  │  RIGHT  │gap│
    /// │   580   │   580   │          580            │   580   │20 │
    /// │         │         │                         │         │   │
    /// │         │         │                         │         │   │
    /// └─────────┴─────────┴─────────────────────────┴─────────┴───┘
    ///           ↑         ↑                         ↑
    ///          90°       90°                       90°
    ///          bend      bend                      bend
    /// ```
    ///
    /// Bend lines (4 total):
    /// 1-3. Three 90° corner bends
    /// 4. Bottom panel bend
    ///
    /// NOTE: Front rake (forehead with camera eyes) is now on the lid visor
    pub fn to_dxf(&self) -> DxfDocument {
        let cfg = &self.config;
        let mut dxf = DxfDocument::new();

        let wall_height = cfg.panel_height();
        let inset = cfg.shell.mount_hole_inset;
        let hole_r = cfg.shell.mount_hole_diameter / 2.0;

        // Panel dimensions (full shell dimensions, no chamfers)
        let front_w = cfg.front_width();      // 580mm
        let side_l = cfg.side_length();       // 580mm
        let rear_w = cfg.rear_width();        // 580mm
        let bottom_w = cfg.shell.shell_width();   // 580mm
        let bottom_l = cfg.shell.shell_length();  // 580mm

        // NOTE: Front is now fully vertical - forehead visor is on the lid

        // === FLAT PATTERN LAYOUT ===
        // Simple layout: FRONT | LEFT | REAR | RIGHT | gap

        // Calculate section X positions (cumulative)
        let mut x = 0.0;
        let front_start = x;
        x += front_w;
        let left_start = x;
        x += side_l;
        let rear_start = x;
        x += rear_w;
        let right_start = x;
        x += side_l;
        x += 20.0; // End gap
        let total_width = x;

        // Center the pattern horizontally
        let offset_x = -total_width / 2.0;

        // Section centers (for placing features)
        let front_cx = offset_x + front_start + front_w / 2.0;
        let left_cx = offset_x + left_start + side_l / 2.0;
        let rear_cx = offset_x + rear_start + rear_w / 2.0;
        let right_cx = offset_x + right_start + side_l / 2.0;

        // Wall strip boundaries
        let wall_left = offset_x;
        let wall_right = offset_x + total_width;
        let wall_bottom = -wall_height / 2.0;
        let wall_top = wall_height / 2.0;

        // Bottom panel (attached above rear section)
        let bottom_cx = rear_cx;
        let bottom_cy = wall_top + bottom_l / 2.0;

        // === OUTER BOUNDARY ===
        if cfg.include_bottom {
            // L-shaped profile with hinge tabs on rear top edge
            let tab_h = cfg.hinge_tab_height;
            let num_tabs = cfg.hinge_tab_count;
            let rear_left_edge = offset_x + rear_start;
            let rear_right_edge = offset_x + rear_start + rear_w;

            let mut outline: Vec<(f64, f64)> = Vec::new();

            // Start at bottom-left, go clockwise
            outline.push((wall_left, wall_bottom));
            outline.push((wall_right, wall_bottom));
            outline.push((wall_right, wall_top));

            // Connect to bottom panel at rear section
            outline.push((rear_right_edge, wall_top));
            outline.push((rear_right_edge, wall_top + bottom_l));
            outline.push((rear_left_edge, wall_top + bottom_l));
            outline.push((rear_left_edge, wall_top));

            // Complete the loop
            outline.push((wall_left, wall_top));

            dxf.add_polyline(outline, true);

            // Hinge tabs on rear section (extend upward from wall_top)
            let tab_spacing = rear_w / (num_tabs as f64);
            for i in 0..num_tabs {
                if i % 2 == 0 {
                    let tab_x = rear_left_edge + tab_spacing * (i as f64 + 0.5);
                    let tab_left = tab_x - cfg.hinge_tab_width / 2.0;
                    let tab_right = tab_x + cfg.hinge_tab_width / 2.0;

                    dxf.add_polyline(
                        vec![
                            (tab_left, wall_top),
                            (tab_left, wall_top + tab_h),
                            (tab_right, wall_top + tab_h),
                            (tab_right, wall_top),
                        ],
                        false,
                    );

                    // Hinge pin hole
                    dxf.add_circle(tab_x, wall_top + tab_h / 2.0, cfg.hinge_pin_diameter / 2.0);
                }
            }
        } else {
            dxf.add_rectangle(total_width, wall_height, 0.0, 0.0);
        }

        // === BEND LINES ===
        // Bends 1-3: Three 90° corner bends (vertical)
        // NOTE: Front rake bend removed - forehead visor is on the lid
        let corner_bend_positions = [
            left_start,   // Front -> Left
            rear_start,   // Left -> Rear
            right_start,  // Rear -> Right
        ];

        for bend_x in corner_bend_positions {
            let x = offset_x + bend_x;
            dxf.add_bend_line(x, wall_bottom, x, wall_top);
        }

        // Bend 4: Bottom panel bend (between rear wall top and bottom panel)
        if cfg.include_bottom {
            let rear_left_edge = offset_x + rear_start;
            let rear_right_edge = offset_x + rear_start + rear_w;
            dxf.add_bend_line(rear_left_edge, wall_top, rear_right_edge, wall_top);
        }

        // === FRONT SECTION (with nozzle and LED channel) ===
        // Nozzle slot (in lower vertical section - for the blower "breath")
        let nozzle_cy = wall_bottom + cfg.nozzle_offset_y + cfg.nozzle_height / 2.0;
        dxf.add_rounded_rectangle(
            cfg.nozzle_width.min(front_w - 20.0), // Ensure fits in panel
            cfg.nozzle_height,
            front_cx,
            nozzle_cy,
            cfg.nozzle_corner_radius,
        );

        // LED channel (the robot's "face" - steady gaze)
        let led_cy = nozzle_cy + cfg.nozzle_height / 2.0 + cfg.led_channel_gap + cfg.led_channel_height / 2.0;
        dxf.add_rounded_rectangle(
            cfg.led_channel_width.min(front_w - 20.0),
            cfg.led_channel_height,
            front_cx,
            led_cy,
            cfg.led_channel_height / 4.0,
        );

        // LED diffuser mounting holes (for the steady amber glow)
        let led_mount_y = led_cy;
        let led_mount_left = front_cx - cfg.led_channel_width / 2.0 + cfg.led_mount_hole_inset;
        let led_mount_right = front_cx + cfg.led_channel_width / 2.0 - cfg.led_mount_hole_inset;
        dxf.add_circle(led_mount_left, led_mount_y - cfg.led_channel_height, cfg.led_mount_hole_diameter / 2.0);
        dxf.add_circle(led_mount_right, led_mount_y - cfg.led_channel_height, cfg.led_mount_hole_diameter / 2.0);
        dxf.add_circle(led_mount_left, led_mount_y + cfg.led_channel_height, cfg.led_mount_hole_diameter / 2.0);
        dxf.add_circle(led_mount_right, led_mount_y + cfg.led_channel_height, cfg.led_mount_hole_diameter / 2.0);

        // NOTE: Stereo camera "eyes" are now on the lid visor

        // Front mounting holes (now full height, no rake section to avoid)
        let front_hx = front_w / 2.0 - inset;
        let hy_bottom = wall_bottom + inset;
        let hy_top = wall_top - inset;
        for (dx, dy) in [
            (-front_hx, hy_bottom),
            (front_hx, hy_bottom),
            (-front_hx, hy_top),
            (front_hx, hy_top),
        ] {
            dxf.add_circle(front_cx + dx, dy, hole_r);
        }

        // Quarter-turn latch holes on front (near top)
        let qt_y = wall_top - cfg.quarter_turn_inset;
        let qt_x_left = front_cx - front_w / 2.0 + cfg.quarter_turn_inset;
        let qt_x_right = front_cx + front_w / 2.0 - cfg.quarter_turn_inset;
        dxf.add_circle(qt_x_left, qt_y, cfg.quarter_turn_hole_diameter / 2.0);
        dxf.add_circle(qt_x_right, qt_y, cfg.quarter_turn_hole_diameter / 2.0);

        // === SIDE SECTIONS (left and right) ===
        let side_hx = side_l / 2.0 - inset;
        let hy = wall_height / 2.0 - inset;

        // Left side mounting holes
        for (dx, dy) in [
            (-side_hx, hy), (side_hx, hy),
            (-side_hx, 0.0), (side_hx, 0.0),
            (-side_hx, -hy), (side_hx, -hy),
        ] {
            dxf.add_circle(left_cx + dx, dy, hole_r);
        }

        // Right side mounting holes
        for (dx, dy) in [
            (-side_hx, hy), (side_hx, hy),
            (-side_hx, 0.0), (side_hx, 0.0),
            (-side_hx, -hy), (side_hx, -hy),
        ] {
            dxf.add_circle(right_cx + dx, dy, hole_r);
        }

        // Gas strut holes on sides (near rear)
        let gas_y = wall_top - cfg.gas_strut_offset_from_top;
        let gas_x_offset = side_l / 2.0 - cfg.gas_strut_offset_from_rear;
        dxf.add_circle(left_cx + gas_x_offset, gas_y, cfg.gas_strut_hole_diameter / 2.0);
        dxf.add_circle(right_cx - gas_x_offset, gas_y, cfg.gas_strut_hole_diameter / 2.0);

        // === REAR SECTION (squared off, honest about being the working end) ===
        // Louver vents
        let total_louver_span = (cfg.louver_count - 1) as f64 * cfg.louver_spacing;
        let louver_start_x = rear_cx - total_louver_span / 2.0;
        let louver_cy = wall_height / 4.0;

        for i in 0..cfg.louver_count {
            let lx = louver_start_x + i as f64 * cfg.louver_spacing;
            dxf.add_slot(cfg.louver_width.min(rear_w - 40.0), cfg.louver_height, lx, louver_cy);
        }

        // Drain holes
        let drain_inset = 20.0;
        let rear_hx = rear_w / 2.0 - drain_inset;
        let drain_y = wall_bottom + drain_inset;
        dxf.add_circle(rear_cx - rear_hx, drain_y, cfg.drain_hole_diameter / 2.0);
        dxf.add_circle(rear_cx + rear_hx, drain_y, cfg.drain_hole_diameter / 2.0);

        // Rear mounting holes
        let rear_mount_hx = rear_w / 2.0 - inset;
        for (dx, dy) in [
            (-rear_mount_hx, hy),
            (rear_mount_hx, hy),
            (-rear_mount_hx, -hy),
            (rear_mount_hx, -hy),
        ] {
            dxf.add_circle(rear_cx + dx, dy, hole_r);
        }

        // === BOTTOM PANEL ===
        if cfg.include_bottom {
            let bx = bottom_w / 2.0 - inset;
            let by = bottom_l / 2.0 - inset;

            // Mounting holes around perimeter
            for (dx, dy) in [
                (-bx, by), (0.0, by), (bx, by),
                (-bx, 0.0), (bx, 0.0),
                (-bx, -by), (0.0, -by), (bx, -by),
            ] {
                dxf.add_circle(bottom_cx + dx, bottom_cy + dy, hole_r);
            }

            // Drain holes in corners
            let bd = 30.0;
            for (dx, dy) in [
                (-bx + bd, -by + bd),
                (bx - bd, -by + bd),
                (-bx + bd, by - bd),
                (bx - bd, by - bd),
            ] {
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
///
/// The lid now includes a front "visor" - the raked forehead with camera eyes.
/// This allows the entire forehead to lift with the lid, providing access to
/// electronics while keeping the cameras mounted to the moving lid.
#[derive(Debug, Clone)]
pub struct TopLidConfig {
    pub shell: ShellConfig,
    // --- Front Visor (forehead with camera eyes) ---
    /// Visor rake angle (degrees) - gentle slope for friendly look
    /// 15° is welcoming, tilts backward when lid is closed
    pub visor_rake_angle: f64,
    /// Visor height (mm) - how far it hangs down from lid front edge
    pub visor_height: f64,
    /// Enable stereo camera cutouts in visor
    pub visor_cameras_enabled: bool,
    /// Camera spacing (mm) - matches human IPD for natural stereo
    pub visor_camera_spacing: f64,
    /// Camera window width (mm)
    pub visor_camera_window_width: f64,
    /// Camera window height (mm)
    pub visor_camera_window_height: f64,
    /// Camera window corner radius (mm)
    pub visor_camera_window_radius: f64,
    /// Camera vertical offset from top of visor (mm)
    pub visor_camera_offset_from_top: f64,
    // --- Sensor and utility holes ---
    /// LiDAR dome hole diameter (mm) - fits sensor dome base
    pub sensor_hole_diameter: f64,
    /// LiDAR dome hole position offset from center (x, y)
    /// Centered for the "head" effect
    pub sensor_hole_offset: (f64, f64),
    /// E-stop hole diameter (mm) - prominent for safety
    pub estop_hole_diameter: f64,
    /// E-stop hole position offset from center (x, y)
    /// Front-left for accessibility, visible
    pub estop_hole_offset: (f64, f64),
    /// Proxicast 5-in-1 antenna mount hole diameter (mm)
    /// 32mm (1.25") per Proxicast ANT-500-221 specs
    pub antenna_hole_diameter: f64,
    /// Antenna hole offset from center (x, y)
    /// Rear-right, needs clear sky view
    pub antenna_hole_offset: (f64, f64),
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
            // --- Front Visor (the "forehead" with camera eyes) ---
            // 15° is gentle, welcoming - tilts backward when closed
            visor_rake_angle: 15.0,
            // 60mm height ensures setback stays within frame clearance
            // tan(15°) × 60mm ≈ 16mm setback < 20mm frame clearance
            visor_height: 60.0,
            // Stereo cameras for pareidolia + Vision Pro telepresence
            visor_cameras_enabled: true,
            visor_camera_spacing: 63.0,         // Human IPD average
            visor_camera_window_width: 30.0,    // Wide enough for CSI camera
            visor_camera_window_height: 25.0,   // Slightly shorter for horizontal look
            visor_camera_window_radius: 5.0,    // Rounded corners
            visor_camera_offset_from_top: 25.0, // Centered in visor
            // LiDAR dome hole - centered, forms the "head"
            sensor_hole_diameter: 160.0, // Slightly larger for dome lip
            sensor_hole_offset: (0.0, 0.0), // Centered on lid
            // E-stop - front-left, prominent and accessible
            estop_hole_diameter: 30.0,
            estop_hole_offset: (-shell.shell_width() / 3.0, shell.shell_length() / 3.0),
            // Proxicast 5-in-1 antenna - rear-right, needs sky view
            // 32mm (1.25") mounting hole per specs
            antenna_hole_diameter: 32.0,
            antenna_hole_offset: (shell.shell_width() / 3.0, -shell.shell_length() / 3.0),
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

/// Top Lid: Hinged panel with front visor (the "forehead" with camera eyes)
///
/// The visor hangs down from the front edge of the lid at a 15° rake,
/// creating a welcoming forehead appearance. When the lid opens,
/// the visor lifts with it, providing access to the electronics.
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

    /// Calculate visor setback (how far the bottom of visor moves back)
    fn visor_setback(&self) -> f64 {
        let cfg = &self.config;
        cfg.visor_height * (cfg.visor_rake_angle * std::f64::consts::PI / 180.0).tan()
    }

    /// Generate 3D representation (flat panel)
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let shell = &cfg.shell;
        let segments = 32;

        let width = shell.shell_width();
        let length = shell.shell_length();

        // Main panel body (flat horizontal lid)
        let panel = centered_cube("top_lid", width, length, shell.thickness);

        // Sensor mast hole
        let sensor_hole = centered_cylinder(
            "sensor_hole",
            cfg.sensor_hole_diameter / 2.0,
            shell.thickness * 3.0,
            segments,
        )
        .translate(cfg.sensor_hole_offset.0, cfg.sensor_hole_offset.1, 0.0);

        // E-stop hole (front-left, prominent for safety)
        let estop_hole = centered_cylinder(
            "estop_hole",
            cfg.estop_hole_diameter / 2.0,
            shell.thickness * 3.0,
            segments,
        )
        .translate(cfg.estop_hole_offset.0, cfg.estop_hole_offset.1, 0.0);

        // Proxicast 5-in-1 antenna hole (rear-right, needs sky view)
        let antenna_hole = centered_cylinder(
            "antenna_hole",
            cfg.antenna_hole_diameter / 2.0,
            shell.thickness * 3.0,
            segments,
        )
        .translate(cfg.antenna_hole_offset.0, cfg.antenna_hole_offset.1, 0.0);

        // Mounting holes
        let mounts = self.create_mount_holes(segments);

        panel
            .difference(&sensor_hole)
            .difference(&estop_hole)
            .difference(&antenna_hole)
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

        // Proxicast 5-in-1 antenna hole (LTE + WiFi + GPS)
        dxf.add_circle(
            cfg.antenna_hole_offset.0,
            cfg.antenna_hole_offset.1,
            cfg.antenna_hole_diameter / 2.0,
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
// Sensor Dome (3D Printed Cover)
// =============================================================================

/// Sensor Dome configuration
///
/// LiDAR-only dome - cameras are now in the front face ("eyes").
/// "Subtle, not towering" - just tall enough for Livox Mid-360.
///
/// The sensor dome covers only the LiDAR, providing weather protection
/// while maintaining the low-profile "friendly industrial" aesthetic.
/// 3D printed to allow organic curves that would be expensive in sheet metal.
#[derive(Debug, Clone)]
pub struct SensorDomeConfig {
    /// Base diameter (matches top lid sensor hole)
    pub base_diameter: f64,
    /// Overall height - sized for Livox Mid-360 (77mm body + 10mm base)
    /// Kept minimal for "planted" look
    pub height: f64,
    /// Wall thickness (for 3D printing)
    pub wall_thickness: f64,
    /// Lip height (overlaps top lid edge)
    pub lip_height: f64,
    /// Lip inset (how far lip extends inward)
    pub lip_inset: f64,
    /// Number of mounting tabs around base
    pub mount_tab_count: usize,
    /// Mounting tab dimensions
    pub mount_tab_width: f64,
    pub mount_tab_length: f64,
    /// Mounting hole diameter (M4)
    pub mount_hole_diameter: f64,
}

impl Default for SensorDomeConfig {
    fn default() -> Self {
        Self {
            // Match top lid sensor hole (150mm) with slight overlap
            base_diameter: 155.0,
            // Low profile - just tall enough for Livox Mid-360
            // LiDAR is 77mm body + 10mm base = 87mm, add clearance = 95mm
            height: 95.0,
            // 3mm walls for strength without excessive material
            wall_thickness: 3.0,
            // 10mm lip overlaps top lid for weather seal
            lip_height: 10.0,
            lip_inset: 5.0,
            // 4 mounting tabs at 90° intervals
            mount_tab_count: 4,
            mount_tab_width: 20.0,
            mount_tab_length: 15.0,
            mount_hole_diameter: 4.5, // M4 clearance
        }
    }
}

/// Sensor Dome: 3D printed cover for LiDAR
///
/// Design characteristics:
/// - Organic dome shape (not a harsh cylinder)
/// - Low profile to maintain "planted" aesthetic
/// - Sized for Livox Mid-360 only (cameras are in front face)
/// - Mounting tabs with M4 holes
/// - Lip for weather sealing against top lid
pub struct SensorDome {
    config: SensorDomeConfig,
}

impl SensorDome {
    pub fn new(config: SensorDomeConfig) -> Self {
        Self { config }
    }

    pub fn default_bvr1() -> Self {
        Self::new(SensorDomeConfig::default())
    }

    /// Generate 3D representation of the sensor dome
    ///
    /// The dome uses a gentle curve - more "forehead" than "tower"
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let segments = 48; // Smooth curves for 3D printing

        let outer_r = cfg.base_diameter / 2.0;
        let inner_r = outer_r - cfg.wall_thickness;
        let height = cfg.height;

        // Outer dome shell (cylinder with rounded top approximation)
        // For now, simplified as cylinder + hemisphere
        let outer_cylinder = centered_cylinder(
            "dome_outer",
            outer_r,
            height * 0.7, // Lower cylinder portion
            segments,
        )
        .translate(0.0, 0.0, height * 0.35);

        // Dome top (hemisphere, approximated as squashed sphere)
        let dome_top = centered_cylinder(
            "dome_top",
            outer_r,
            height * 0.4,
            segments,
        )
        .translate(0.0, 0.0, height * 0.7);

        // Inner cavity
        let inner_cavity = centered_cylinder(
            "dome_inner",
            inner_r,
            height - cfg.wall_thickness,
            segments,
        )
        .translate(0.0, 0.0, (height - cfg.wall_thickness) / 2.0);

        // Lip (for weather seal against top lid)
        let lip_outer = centered_cylinder(
            "lip_outer",
            outer_r + cfg.lip_inset,
            cfg.lip_height,
            segments,
        )
        .translate(0.0, 0.0, -cfg.lip_height / 2.0);

        let lip_inner = centered_cylinder(
            "lip_inner",
            inner_r,
            cfg.lip_height + 1.0,
            segments,
        )
        .translate(0.0, 0.0, -cfg.lip_height / 2.0);

        let lip = lip_outer.difference(&lip_inner);

        // Note: Camera window removed - cameras are now in the front face ("eyes")
        // The dome is LiDAR-only for simplicity

        // Mounting tabs
        let mut tabs = Part::empty("mount_tabs");
        let tab_angle_step = 360.0 / cfg.mount_tab_count as f64;

        for i in 0..cfg.mount_tab_count {
            let angle = i as f64 * tab_angle_step + 45.0; // Offset from cardinal directions
            let rad = angle * std::f64::consts::PI / 180.0;
            let tab_x = (outer_r + cfg.mount_tab_length / 2.0) * rad.cos();
            let tab_y = (outer_r + cfg.mount_tab_length / 2.0) * rad.sin();

            let tab = centered_cube(
                "mount_tab",
                cfg.mount_tab_width,
                cfg.mount_tab_length,
                cfg.wall_thickness,
            )
            .rotate(0.0, 0.0, angle)
            .translate(tab_x, tab_y, -cfg.lip_height + cfg.wall_thickness / 2.0);

            // Mounting hole in tab
            let hole = centered_cylinder(
                "mount_hole",
                cfg.mount_hole_diameter / 2.0,
                cfg.wall_thickness * 3.0,
                16,
            )
            .translate(tab_x, tab_y, -cfg.lip_height);

            tabs = tabs.union(&tab.difference(&hole));
        }

        // Assemble the dome
        outer_cylinder
            .union(&dome_top)
            .difference(&inner_cavity)
            .union(&lip)
            .union(&tabs)
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

    /// Generate complete shell assembly (big shell + hinged top lid + sensor dome)
    ///
    /// "Friendly Industrial" shell with:
    /// - Chamfered corners (softened, not boxy)
    /// - Gentle front rake (welcoming forehead)
    /// - LED bar face (steady amber gaze)
    /// - Sensor dome (subtle, not towering)
    pub fn generate(&self) -> Part {
        let shell = &self.config;
        let height = shell.shell_height();
        let lid_cfg = TopLidConfig {
            shell: self.config.clone(),
            ..Default::default()
        };

        // Big shell with integrated bottom (wall wrap with include_bottom=true)
        let big_shell = WallWrap::new(WallWrapConfig {
            shell: self.config.clone(),
            include_bottom: true,
            ..Default::default()
        })
        .generate();

        // Top lid (positioned on top, hinged at rear)
        let top_lid = TopLid::new(lid_cfg.clone())
            .generate()
            .translate(0.0, 0.0, height);

        // Sensor dome (positioned over sensor hole in lid)
        // "Sits like a head without trying to be a head"
        let sensor_dome = SensorDome::default_bvr1()
            .generate()
            .translate(
                lid_cfg.sensor_hole_offset.0,
                lid_cfg.sensor_hole_offset.1,
                height + shell.thickness,
            );

        // Gas struts (inside shell, connecting walls to lid underside)
        let gas_struts = self.generate_gas_struts();

        big_shell
            .union(&top_lid)
            .union(&sensor_dome)
            .union(&gas_struts)
    }

    /// Generate simplified gas strut representation for visualization
    ///
    /// Gas struts connect the side walls to the underside of the lid,
    /// positioned near the rear to hold the lid open.
    fn generate_gas_struts(&self) -> Part {
        use crate::centered_cylinder;

        let shell = &self.config;
        let height = shell.shell_height();
        let length = shell.shell_length();
        let wall_cfg = WallWrapConfig::default();
        let lid_cfg = TopLidConfig::default();

        // Gas strut parameters - sized to fit inside shell
        let strut_diameter = 10.0;
        let strut_length = 100.0; // Short enough to stay inside
        let segments = 16;

        // Position inside shell, near rear corners
        let strut_x = shell.shell_width() / 2.0 - 50.0; // Inset from walls
        let strut_y = -length / 2.0 + 80.0; // Near rear
        let strut_z = height * 0.6; // Mid-height inside shell

        // Struts angled from wall toward lid (fully inside the shell)
        let strut_right = centered_cylinder("gas_strut", strut_diameter / 2.0, strut_length, segments)
            .rotate(-45.0, 0.0, 0.0) // Angled up toward front
            .translate(strut_x, strut_y, strut_z);

        let strut_left = centered_cylinder("gas_strut", strut_diameter / 2.0, strut_length, segments)
            .rotate(-45.0, 0.0, 0.0)
            .translate(-strut_x, strut_y, strut_z);

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

    /// Get sensor dome (3D printed cover for sensor mast)
    ///
    /// "Subtle, not towering. Sits like a head without trying to be a head."
    pub fn sensor_dome(&self) -> SensorDome {
        SensorDome::default_bvr1()
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

        // Simple 4-panel design (vertical walls only):
        // All panels = shell_width/length = 580mm
        assert_eq!(cfg.front_width(), 580.0);
        assert_eq!(cfg.side_length(), 580.0);
        assert_eq!(cfg.rear_width(), 580.0);
        assert_eq!(cfg.panel_height(), 200.0);

        // NOTE: Front rake (forehead) is now on the lid visor

        // Flat width: 580*4 + 20 = 2340mm
        let flat_w = cfg.flat_width();
        assert_eq!(flat_w, 2340.0, "Flat width should be 2340mm");

        // Bend count: 3 corners + 1 bottom = 4 (no front rake bend)
        assert_eq!(cfg.bend_count(), 4);
    }

    #[test]
    fn test_top_lid_visor() {
        let cfg = TopLidConfig::default();

        // Visor (forehead) configuration
        assert_eq!(cfg.visor_rake_angle, 15.0);
        assert_eq!(cfg.visor_height, 60.0);
        assert!(cfg.visor_cameras_enabled);
        assert_eq!(cfg.visor_camera_spacing, 63.0); // Human IPD

        // Visor setback should be within frame clearance (20mm)
        let setback = cfg.visor_height * (cfg.visor_rake_angle * std::f64::consts::PI / 180.0).tan();
        assert!(setback < 20.0, "Visor setback {} should be < 20mm frame clearance", setback);
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
    fn test_sensor_dome_generation() {
        // "Subtle, not towering. Sits like a head without trying to be a head."
        let dome = SensorDome::default_bvr1();
        let part = dome.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_sensor_dome_config() {
        let cfg = SensorDomeConfig::default();

        // Low profile - just tall enough for Livox Mid-360 (87mm + clearance)
        assert_eq!(cfg.height, 95.0);
        assert!(cfg.height < 120.0, "Dome should be low profile");

        // Matches top lid sensor hole (150mm) with slight overlap
        assert_eq!(cfg.base_diameter, 155.0);

        // 4 mounting tabs for secure attachment
        assert_eq!(cfg.mount_tab_count, 4);

        // Note: Camera window removed - cameras are now in front face ("eyes")
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
