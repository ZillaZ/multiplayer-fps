use deku::prelude::*;
use rand::Rng;
use rapier3d::na::{Const, OPoint};
use rapier3d::prelude::*;
use raylib::math::*;

#[derive(Clone, Debug)]
pub struct Cuboid {
    pub hx: f32,
    pub hy: f32,
    pub hz: f32,
}

impl Cuboid {
    pub fn new(hx: f32, hy: f32, hz: f32) -> Self {
        Self { hx, hy, hz }
    }
}

#[derive(Clone, Debug)]
pub struct Sphere {
    pub radius: f32,
}

impl Sphere {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

#[derive(Clone, Debug)]
pub enum Shape {
    CUBOID(Cuboid),
    SPHERE(Sphere),
    DYNAMIC,
}

impl Shape {
    pub fn cuboid(&self) -> Cuboid {
        match self {
            Shape::CUBOID(cuboid) => cuboid.clone(),
            _ => panic!("Muito sexo meokk"),
        }
    }
    pub fn sphere(&self) -> Sphere {
        match self {
            Shape::SPHERE(sphere) => sphere.clone(),
            _ => panic!("Muito sexo meokk"),
        }
    }
}

#[derive(Debug, DekuRead, DekuWrite, Clone)]
#[deku(type = "u8")]
pub enum ObjectType {
    #[deku(id = "0x1")]
    GROUND,
    #[deku(id = "0x2")]
    PLAYER,
    #[deku(id = "0x3")]
    BALL,
}

#[derive(Debug, DekuRead, DekuWrite, Clone)]
pub struct Object {
    pub model_type: ObjectType,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub id: u64,
}

impl Object {
    pub fn new(model_type: ObjectType, position: Vector3, rotation: Vector4) -> Self {
        let mut rng = rand::thread_rng();
        let id = rng.gen_range(0..std::u64::MAX);
        Self {
            id,
            position: position.to_array(),
            rotation: [rotation.x, rotation.y, rotation.z, rotation.w],
            model_type,
        }
    }
}

pub fn create_collider(
    shape: &Shape,
    restitution: f32,
    vertices: Option<Vec<OPoint<f32, Const<3>>>>,
) -> Collider {
    match shape {
        Shape::CUBOID(val) => ColliderBuilder::cuboid(val.hx, val.hy, val.hz)
            .restitution(restitution)
            .build(),
        Shape::SPHERE(val) => ColliderBuilder::ball(val.radius)
            .restitution(restitution)
            .build(),
        Shape::DYNAMIC => ColliderBuilder::convex_hull(vertices.unwrap().as_slice())
            .unwrap()
            .restitution(restitution)
            .build(),
    }
}

pub fn create_body(
    body_type: RigidBodyType,
    position: Vector3,
    linear_damping: f32,
    additional_mass: f32,
) -> RigidBody {
    use RigidBodyType::*;

    let pos = vector![position.x, position.y, position.z];
    match body_type {
        Fixed => RigidBodyBuilder::fixed()
            .translation(pos)
            .linear_damping(linear_damping)
            .additional_mass(additional_mass)
            .build(),
        Dynamic => RigidBodyBuilder::dynamic()
            .translation(pos)
            .linear_damping(linear_damping)
            .additional_mass(additional_mass)
            .build(),
        KinematicPositionBased => RigidBodyBuilder::kinematic_position_based()
            .linear_damping(linear_damping)
            .translation(pos)
            .additional_mass(additional_mass)
            .build(),
        KinematicVelocityBased => RigidBodyBuilder::kinematic_velocity_based()
            .linear_damping(linear_damping)
            .translation(pos)
            .additional_mass(additional_mass)
            .build(),
    }
}
