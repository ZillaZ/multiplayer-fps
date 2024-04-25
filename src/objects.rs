use crate::*;
use rapier3d::na::{Const, OPoint};
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
    CONVEX,
    MULTI,
    SensorMulti,
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
pub struct NetworkObject {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    #[deku(update = "self.id.len()")]
    id_len: usize,
    #[deku(count = "id_len")]
    pub id: Vec<u8>,
}

impl NetworkObject {
    pub fn new(id: String, position: Vector3, rotation: Vector4) -> Self {
        Self {
            id: id.as_bytes().to_vec(),
            id_len: id.len(),
            position: position.to_array(),
            rotation: [rotation.x, rotation.y, rotation.z, rotation.w],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Object {
    pub shape: S,
    pub body_type: RigidBodyType,
    pub vertices: Vec<OPoint<f32, Const<3>>>,
    pub indices: Vec<[u32; 3]>,
    pub rotation: Vector3,
    pub position: Vector3,
    pub radius: f32,
    pub name: String,
}

impl Object {
    pub fn new(
        shape: S,
        body_type: RigidBodyType,
        vertices: Vec<OPoint<f32, Const<3>>>,
        indices: Vec<[u32; 3]>,
        rotation: Vector3,
        position: Vector3,
        radius: f32,
        name: String,
    ) -> Self {
        Self {
            name,
            shape,
            body_type,
            vertices,
            indices,
            rotation,
            position,
            radius,
        }
    }
}

pub fn create_collider(
    shape: &Shape,
    restitution: f32,
    density: f32,
    vertices: Option<(Vec<OPoint<f32, Const<3>>>, Vec<[u32; 3]>)>,
) -> Collider {
    match shape {
        Shape::CUBOID(val) => ColliderBuilder::cuboid(val.hx, val.hy, val.hz)
            .restitution(restitution)
            .density(density)
            .build(),
        Shape::SPHERE(val) => ColliderBuilder::ball(val.radius)
            .restitution(restitution)
            .density(density)
            .build(),
        Shape::CONVEX => ColliderBuilder::convex_hull(vertices.unwrap().0.as_slice())
            .unwrap()
            .restitution(restitution)
            .density(density)
            .build(),
        Shape::MULTI => ColliderBuilder::convex_decomposition(
            vertices.as_ref().unwrap().0.as_slice(),
            vertices.as_ref().unwrap().1.as_slice(),
        )
        .restitution(restitution)
        .density(density)
        .build(),
        Shape::SensorMulti => ColliderBuilder::convex_decomposition(
            vertices.as_ref().unwrap().0.as_slice(),
            vertices.as_ref().unwrap().1.as_slice(),
        )
        .sensor(true)
        .density(density)
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
