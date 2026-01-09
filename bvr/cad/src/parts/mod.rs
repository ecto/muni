//! BVR1 Parts Library
//!
//! Custom fabricated parts and reference geometry for the BVR1 rover.
//!
//! ## Part Categories
//!
//! **Custom Fabricated** (for manufacturing):
//! - [`motor_mount`] - L-bracket motor mounts
//! - [`electronics_plate`] - Electronics mounting plate
//! - [`sensor_mount`] - Sensor mast mounting bracket
//! - [`wheel_spacer`] - Wheel spacers
//!
//! **Reference Parts** (for visualization):
//! - [`hub_motor`] - Hub motors and wheels
//! - [`sensors`] - LiDAR, cameras, GPS antennas
//! - [`electronics`] - VESCs, Jetson, DC-DC converters
//! - [`battery`] - Downtube and custom battery packs
//!
//! **Frame Components**:
//! - [`frame`] - 2020 extrusions, brackets, and full frame assembly

// Custom fabricated parts
pub mod access_panel;
pub mod base_tray;
pub mod electronics_plate;
pub mod frame;
pub mod motor_mount;
pub mod sensor_mount;
pub mod uumotor;
pub mod wheel_spacer;

// Reference parts (for visualization and assembly)
pub mod battery;
pub mod electronics;
pub mod hub_motor;
pub mod scale_refs;
pub mod sensors;

// Complete assemblies
pub mod assembly;

// Custom fabricated exports
pub use access_panel::AccessPanel;
pub use base_tray::BaseTray;
pub use electronics_plate::ElectronicsPlate;
pub use frame::{BVR1Frame, CornerBracket, Extrusion2020, TNut};
pub use motor_mount::MotorMount;
pub use sensor_mount::SensorMount;
pub use uumotor::{LBracketMount, UUMotor, UUMotorMount};
pub use wheel_spacer::WheelSpacer;

// Reference part exports
pub use battery::{BatteryTray, CustomBattery, DowntubeBattery};
pub use electronics::{DcDc, EStopButton, Jetson, Vesc};
pub use hub_motor::HubMotor;
pub use scale_refs::{Banana, Human};
pub use sensors::{Camera, GpsAntenna, Lidar};

// Assembly exports
pub use assembly::{BVR0Assembly, BVR1Assembly};
