//! Scale reference models for visualization
//!
//! Provides common reference objects to help users understand model scale:
//! - Banana (~180mm long) - classic "banana for scale"
//! - Human figure (~1750mm tall) - average adult mannequin

use crate::{centered_cube, centered_cylinder, Part};

/// A banana for scale reference
///
/// Approximate dimensions: 180mm long x 30mm diameter
/// Simple elongated spheroid with stem
pub struct Banana;

impl Banana {
    /// Generate a simple banana shape
    ///
    /// Single elongated spheroid (yellow body) with cylindrical stem.
    pub fn generate() -> Part {
        let segments = 64;

        // Main body: single elongated spheroid
        // 180mm long, 30mm diameter, slightly flattened
        let body = Part::sphere("body", 15.0, segments)
            .scale(6.0, 1.0, 0.9) // 180mm long (15*2*6), 30mm tall, 27mm wide
            .translate(0.0, 17.0, 0.0); // Sit on ground

        // Stem: small cylinder pointing outward from the end
        let stem = Part::cylinder("stem", 3.0, 15.0, segments)
            .rotate(0.0, 90.0, 0.0) // Point along X axis (outward)
            .translate(-90.0, 20.0, 0.0); // At the end of the banana, slightly up

        body.union(&stem)
    }
}

/// A simplified human figure for scale reference
///
/// Approximate dimensions: 1750mm tall (average adult)
/// Standing at origin, facing +X direction
pub struct Human;

impl Human {
    /// Generate a simplified mannequin-style human figure
    ///
    /// Proportions based on average adult male:
    /// - Total height: 1750mm
    /// - Shoulder width: 450mm
    /// - Head diameter: 220mm
    pub fn generate() -> Part {
        let segments = 24;

        // Key proportions (mm)
        let total_height = 1750.0;
        let head_diameter = 220.0;
        let neck_height = 80.0;
        let torso_height = 550.0;
        let torso_width = 380.0;
        let torso_depth = 220.0;
        let shoulder_width = 450.0;
        let arm_length = 600.0;
        let arm_diameter = 80.0;
        let leg_length = 850.0;
        let leg_diameter = 120.0;
        let hip_width = 350.0;

        // Head
        let head = Part::sphere("head", head_diameter / 2.0, segments)
            .translate(0.0, total_height - head_diameter / 2.0, 0.0);

        // Neck
        let neck = centered_cylinder("neck", 60.0, neck_height, segments)
            .rotate(90.0, 0.0, 0.0)
            .translate(0.0, total_height - head_diameter - neck_height / 2.0, 0.0);

        // Torso (tapered box approximation using cylinders)
        let torso_bottom = total_height - head_diameter - neck_height - torso_height;
        let torso = centered_cube("torso", torso_depth, torso_height, torso_width)
            .translate(0.0, torso_bottom + torso_height / 2.0, 0.0);

        // Shoulders (rounded)
        let shoulder_y = total_height - head_diameter - neck_height - 50.0;
        let left_shoulder = Part::sphere("lshoulder", 70.0, segments)
            .translate(0.0, shoulder_y, shoulder_width / 2.0);
        let right_shoulder = Part::sphere("rshoulder", 70.0, segments)
            .translate(0.0, shoulder_y, -shoulder_width / 2.0);

        // Arms (simplified as tapered cylinders)
        let arm_y = shoulder_y - arm_length / 2.0;
        let left_arm = centered_cylinder("larm", arm_diameter / 2.0, arm_length, segments)
            .rotate(90.0, 0.0, 0.0)
            .translate(0.0, arm_y, shoulder_width / 2.0 + 20.0);
        let right_arm = centered_cylinder("rarm", arm_diameter / 2.0, arm_length, segments)
            .rotate(90.0, 0.0, 0.0)
            .translate(0.0, arm_y, -shoulder_width / 2.0 - 20.0);

        // Hands (spheres)
        let hand_y = arm_y - arm_length / 2.0;
        let left_hand = Part::sphere("lhand", 50.0, segments)
            .translate(0.0, hand_y, shoulder_width / 2.0 + 20.0);
        let right_hand = Part::sphere("rhand", 50.0, segments)
            .translate(0.0, hand_y, -shoulder_width / 2.0 - 20.0);

        // Pelvis
        let pelvis = centered_cube("pelvis", torso_depth * 0.9, 100.0, hip_width)
            .translate(0.0, torso_bottom - 50.0, 0.0);

        // Legs
        let leg_top = torso_bottom - 100.0;
        let left_leg = centered_cylinder("lleg", leg_diameter / 2.0, leg_length, segments)
            .rotate(90.0, 0.0, 0.0)
            .translate(0.0, leg_top - leg_length / 2.0, hip_width / 4.0);
        let right_leg = centered_cylinder("rleg", leg_diameter / 2.0, leg_length, segments)
            .rotate(90.0, 0.0, 0.0)
            .translate(0.0, leg_top - leg_length / 2.0, -hip_width / 4.0);

        // Feet (boxes)
        let foot_length = 260.0;
        let foot_width = 100.0;
        let foot_height = 50.0;
        let left_foot = centered_cube("lfoot", foot_length, foot_height, foot_width)
            .translate(foot_length / 4.0, foot_height / 2.0, hip_width / 4.0);
        let right_foot = centered_cube("rfoot", foot_length, foot_height, foot_width)
            .translate(foot_length / 4.0, foot_height / 2.0, -hip_width / 4.0);

        // Combine all parts
        head.union(&neck)
            .union(&torso)
            .union(&left_shoulder)
            .union(&right_shoulder)
            .union(&left_arm)
            .union(&right_arm)
            .union(&left_hand)
            .union(&right_hand)
            .union(&pelvis)
            .union(&left_leg)
            .union(&right_leg)
            .union(&left_foot)
            .union(&right_foot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_banana_generation() {
        let banana = Banana::generate();
        assert!(!banana.is_empty());
    }

    #[test]
    fn test_human_generation() {
        let human = Human::generate();
        assert!(!human.is_empty());
    }
}
