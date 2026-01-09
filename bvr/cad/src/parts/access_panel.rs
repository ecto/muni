//! Top access panel for BVR1
//!
//! Removable lid that covers the top of the frame.
//! Sensor mast mounts to this panel.

use crate::{centered_cube, centered_cylinder, Part};

/// Access panel configuration
#[derive(Debug, Clone)]
pub struct AccessPanelConfig {
    /// Panel width (X dimension, mm)
    pub width: f64,
    /// Panel length (Y dimension, mm)
    pub length: f64,
    /// Panel thickness (mm)
    pub thickness: f64,
    /// Lip/flange width for seating on frame (mm)
    pub lip_width: f64,
    /// Sensor mast hole diameter (mm)
    pub mast_hole_diameter: f64,
    /// Mast hole Y offset from center (mm)
    pub mast_offset_y: f64,
}

impl Default for AccessPanelConfig {
    fn default() -> Self {
        // Sized for compact 380x500mm ADA-compliant frame
        Self {
            width: 380.0,   // Full frame width
            length: 500.0,  // Full frame length
            thickness: 4.0, // Thinner than base tray
            lip_width: 10.0,
            mast_hole_diameter: 26.0, // 1" tube = 25.4mm + clearance
            mast_offset_y: 150.0, // Mast toward front (scaled down)
        }
    }
}

/// Top access panel with sensor mast mount
pub struct AccessPanel {
    config: AccessPanelConfig,
}

impl AccessPanel {
    pub fn new(config: AccessPanelConfig) -> Self {
        Self { config }
    }

    pub fn default_bvr1() -> Self {
        Self::new(AccessPanelConfig::default())
    }

    /// Generate the access panel
    ///
    /// Features:
    /// - Flat panel that sits on top of frame
    /// - Center hole for sensor mast
    /// - Mounting holes at corners
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let segments = 48;

        // Main panel
        let panel = centered_cube("panel", cfg.width, cfg.length, cfg.thickness);

        // Sensor mast hole (toward front)
        let mast_hole = centered_cylinder(
            "mast_hole",
            cfg.mast_hole_diameter / 2.0,
            cfg.thickness * 2.0,
            segments,
        )
        .translate(0.0, cfg.mast_offset_y, 0.0);

        // Reinforcement ring around mast hole
        let reinforcement_outer = centered_cylinder(
            "reinforce_outer",
            cfg.mast_hole_diameter / 2.0 + 15.0,
            cfg.thickness,
            segments,
        )
        .translate(0.0, cfg.mast_offset_y, 0.0);

        let reinforcement_inner = centered_cylinder(
            "reinforce_inner",
            cfg.mast_hole_diameter / 2.0,
            cfg.thickness * 2.0,
            segments,
        )
        .translate(0.0, cfg.mast_offset_y, 0.0);

        let reinforcement = reinforcement_outer.difference(&reinforcement_inner);

        // Mounting holes at corners (to bolt to frame)
        let mount_hole = centered_cylinder("mount", 5.5 / 2.0, cfg.thickness * 2.0, 24);
        let corner_inset = 15.0;
        let hx = cfg.width / 2.0 - corner_inset;
        let hy = cfg.length / 2.0 - corner_inset;

        let holes = mount_hole.translate(-hx, hy, 0.0)
            .union(&mount_hole.translate(hx, hy, 0.0))
            .union(&mount_hole.translate(-hx, -hy, 0.0))
            .union(&mount_hole.translate(hx, -hy, 0.0));

        // Additional mounting holes along edges
        let edge_holes_x = mount_hole.translate(-hx, 0.0, 0.0)
            .union(&mount_hole.translate(hx, 0.0, 0.0));
        let edge_holes_y = mount_hole.translate(0.0, hy, 0.0)
            .union(&mount_hole.translate(0.0, -hy, 0.0));

        let all_holes = holes
            .union(&edge_holes_x)
            .union(&edge_holes_y)
            .union(&mast_hole);

        panel.union(&reinforcement).difference(&all_holes)
    }

    /// Generate simplified panel
    pub fn generate_simple(&self) -> Part {
        let cfg = &self.config;
        centered_cube("panel", cfg.width, cfg.length, cfg.thickness)
    }

    /// Get mast hole position (for placing sensor mast)
    pub fn mast_position(&self) -> (f64, f64) {
        (0.0, self.config.mast_offset_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_panel() {
        let panel = AccessPanel::default_bvr1();
        let part = panel.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_access_panel_simple() {
        let panel = AccessPanel::default_bvr1();
        let part = panel.generate_simple();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_mast_position() {
        let panel = AccessPanel::default_bvr1();
        let (x, y) = panel.mast_position();
        assert_eq!(x, 0.0);
        assert_eq!(y, 150.0);  // Scaled down for compact frame
    }
}
