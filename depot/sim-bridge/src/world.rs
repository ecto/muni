//! Simulated world with obstacles for collision detection and LiDAR simulation.
//!
//! Copied from bvr/firmware/crates/sim/src/world.rs (standalone, no firmware deps).

use nalgebra::{Isometry3, Point3, Vector3};
use parry3d::query::Ray;
use parry3d::shape::{Cuboid, SharedShape};

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl Aabb {
    pub fn new(min: Point3<f32>, max: Point3<f32>) -> Self {
        Self { min, max }
    }

    pub fn contains(&self, point: &Point3<f32>) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }
}

/// An obstacle in the world.
#[derive(Debug, Clone)]
pub struct Obstacle {
    pub position: Isometry3<f32>,
    pub shape: SharedShape,
    pub aabb: Aabb,
    pub name: String,
}

impl Obstacle {
    pub fn box_obstacle(
        name: impl Into<String>,
        center: Point3<f32>,
        half_extents: Vector3<f32>,
    ) -> Self {
        let position = Isometry3::translation(center.x, center.y, center.z);
        let shape = SharedShape::new(Cuboid::new(half_extents));
        let aabb = Aabb::new(
            Point3::new(
                center.x - half_extents.x,
                center.y - half_extents.y,
                center.z - half_extents.z,
            ),
            Point3::new(
                center.x + half_extents.x,
                center.y + half_extents.y,
                center.z + half_extents.z,
            ),
        );

        Self {
            position,
            shape,
            aabb,
            name: name.into(),
        }
    }

    pub fn wall(
        name: impl Into<String>,
        start: Point3<f32>,
        end: Point3<f32>,
        height: f32,
        thickness: f32,
    ) -> Self {
        let center = Point3::new(
            (start.x + end.x) / 2.0,
            (start.y + end.y) / 2.0,
            height / 2.0,
        );
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let length = (dx * dx + dy * dy).sqrt();

        let half_extents = Vector3::new(length / 2.0, thickness / 2.0, height / 2.0);

        let angle = dy.atan2(dx);
        let rotation = nalgebra::UnitQuaternion::from_axis_angle(&Vector3::z_axis(), angle);
        let position = Isometry3::from_parts(
            nalgebra::Translation3::new(center.x, center.y, center.z),
            rotation,
        );

        let shape = SharedShape::new(Cuboid::new(half_extents));

        let corners = [
            Point3::new(-half_extents.x, -half_extents.y, -half_extents.z),
            Point3::new(half_extents.x, -half_extents.y, -half_extents.z),
            Point3::new(-half_extents.x, half_extents.y, -half_extents.z),
            Point3::new(half_extents.x, half_extents.y, -half_extents.z),
            Point3::new(-half_extents.x, -half_extents.y, half_extents.z),
            Point3::new(half_extents.x, -half_extents.y, half_extents.z),
            Point3::new(-half_extents.x, half_extents.y, half_extents.z),
            Point3::new(half_extents.x, half_extents.y, half_extents.z),
        ];

        let mut min = Point3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Point3::new(f32::MIN, f32::MIN, f32::MIN);

        for corner in corners {
            let world_corner = position * corner;
            min.x = min.x.min(world_corner.x);
            min.y = min.y.min(world_corner.y);
            min.z = min.z.min(world_corner.z);
            max.x = max.x.max(world_corner.x);
            max.y = max.y.max(world_corner.y);
            max.z = max.z.max(world_corner.z);
        }

        Self {
            position,
            shape,
            aabb: Aabb::new(min, max),
            name: name.into(),
        }
    }

    pub fn ray_cast(&self, ray: &Ray, max_toi: f32) -> Option<f32> {
        self.shape.cast_ray(&self.position, ray, max_toi, true)
    }
}

/// The simulated world containing all obstacles.
#[derive(Debug, Clone, Default)]
pub struct World {
    pub obstacles: Vec<Obstacle>,
    pub ground_z: f32,
    pub bounds: Option<Aabb>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn empty_room(size: f32, wall_height: f32) -> Self {
        let half = size / 2.0;
        let thickness = 0.2;

        let mut world = Self::new();

        world.obstacles.push(Obstacle::wall(
            "north_wall",
            Point3::new(-half, half, 0.0),
            Point3::new(half, half, 0.0),
            wall_height,
            thickness,
        ));
        world.obstacles.push(Obstacle::wall(
            "south_wall",
            Point3::new(-half, -half, 0.0),
            Point3::new(half, -half, 0.0),
            wall_height,
            thickness,
        ));
        world.obstacles.push(Obstacle::wall(
            "east_wall",
            Point3::new(half, -half, 0.0),
            Point3::new(half, half, 0.0),
            wall_height,
            thickness,
        ));
        world.obstacles.push(Obstacle::wall(
            "west_wall",
            Point3::new(-half, -half, 0.0),
            Point3::new(-half, half, 0.0),
            wall_height,
            thickness,
        ));

        world.bounds = Some(Aabb::new(
            Point3::new(-half, -half, 0.0),
            Point3::new(half, half, wall_height),
        ));

        world
    }

    pub fn random_obstacles(size: f32, wall_height: f32, num_obstacles: usize, seed: u64) -> Self {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};

        let mut world = Self::empty_room(size, wall_height);
        let mut rng = StdRng::seed_from_u64(seed);

        let half = size / 2.0;
        let margin = 2.0;

        for i in 0..num_obstacles {
            let mut x: f32;
            let mut y: f32;
            loop {
                x = rng.gen_range(-half + 1.0..half - 1.0);
                y = rng.gen_range(-half + 1.0..half - 1.0);
                if x.abs() > margin || y.abs() > margin {
                    break;
                }
            }

            let w = rng.gen_range(0.3..1.5);
            let d = rng.gen_range(0.3..1.5);
            let h = rng.gen_range(0.5..2.0);

            world.obstacles.push(Obstacle::box_obstacle(
                format!("obstacle_{}", i),
                Point3::new(x, y, h / 2.0),
                Vector3::new(w / 2.0, d / 2.0, h / 2.0),
            ));
        }

        world
    }

    pub fn circle_collides(&self, center: Point3<f32>, radius: f32) -> bool {
        let rover_aabb = Aabb::new(
            Point3::new(center.x - radius, center.y - radius, 0.0),
            Point3::new(center.x + radius, center.y + radius, 0.5),
        );

        for obs in &self.obstacles {
            if rover_aabb.max.x >= obs.aabb.min.x
                && rover_aabb.min.x <= obs.aabb.max.x
                && rover_aabb.max.y >= obs.aabb.min.y
                && rover_aabb.min.y <= obs.aabb.max.y
                && rover_aabb.max.z >= obs.aabb.min.z
                && rover_aabb.min.z <= obs.aabb.max.z
            {
                return true;
            }
        }
        false
    }

    pub fn ray_cast(
        &self,
        origin: Point3<f32>,
        direction: Vector3<f32>,
        max_range: f32,
    ) -> Option<f32> {
        let ray = Ray::new(origin, direction);
        let mut closest: Option<f32> = None;

        if direction.z < -0.001 {
            let t = (self.ground_z - origin.z) / direction.z;
            if t > 0.0 && t < max_range {
                closest = Some(t);
            }
        }

        for obs in &self.obstacles {
            if let Some(t) = obs.ray_cast(&ray, max_range) {
                if t > 0.0 {
                    closest = Some(closest.map_or(t, |c| c.min(t)));
                }
            }
        }

        closest
    }

    pub fn in_bounds(&self, x: f64, y: f64) -> bool {
        if let Some(bounds) = &self.bounds {
            x as f32 >= bounds.min.x
                && x as f32 <= bounds.max.x
                && y as f32 >= bounds.min.y
                && y as f32 <= bounds.max.y
        } else {
            true
        }
    }
}
