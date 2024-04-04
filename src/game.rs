use std::collections::HashMap;

use rapier3d::control::KinematicCharacterController;
use rapier3d::prelude::*;
use raylib::prelude::*;

use crate::player::{self, Player};
use crate::reader::get_vertices;
use crate::{lights, objects::*, S};

#[derive(Clone)]
pub struct GameManager {
    pub colliders: ColliderSet,
    pub bodies: RigidBodySet,
    pub query_pipeline: QueryPipeline,
    pub dt: f32,
    integration_parameters: IntegrationParameters,
    pub island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    _physics_hooks: Option<String>,
    _event_handler: Option<String>,
    pub objects: Vec<(Object, ColliderHandle, RigidBodyHandle)>,
    pub network_objects: Vec<Object>,
    pub players: HashMap<u64, Player>,
}

impl GameManager {
    pub fn update(&mut self) {
        let rapier_gravity = vector![0.0, -90.81, 0.0];
        PhysicsPipeline::new().step(
            &rapier_gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
    }

    pub fn update_player(&mut self, player: &mut Player) {
        let this = self.players.get_mut(&player.id);
        if let Some(data) = this {
            *data = player.clone();
        }
    }

    pub fn new() -> Self {
        Self {
            colliders: ColliderSet::new(),
            bodies: RigidBodySet::new(),
            ccd_solver: CCDSolver::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            island_manager: IslandManager::new(),
            narrow_phase: NarrowPhase::new(),
            broad_phase: BroadPhase::new(),
            integration_parameters: IntegrationParameters::default(),
            query_pipeline: QueryPipeline::new(),
            _physics_hooks: None,
            _event_handler: None,
            objects: Vec::new(),
            players: HashMap::new(),
            dt: 0.0,
            network_objects: Vec::new(),
        }
    }

    pub fn add_object(
        &mut self,
        position: Vector3,
        shape: S,
        body_type: RigidBodyType,
        restitution: f32,
        model_path: &str,
        model_type: ObjectType,
        linear_damping: f32,
        additional_mass: f32,
    ) {
        let vertices = get_vertices(model_path);
        let object = Object::new(model_type, position, Vector4::identity());
        let collider = create_collider(&shape, restitution, Some(vertices));
        let body = create_body(body_type, position, linear_damping, additional_mass);
        let body_handle = self.bodies.insert(body);
        let collider_handle =
            self.colliders
                .insert_with_parent(collider, body_handle, &mut self.bodies);
        self.objects
            .push((object.clone(), collider_handle, body_handle));
        self.network_objects.push(object);
    }

    pub fn create_light(&mut self, light_shader: &mut Shader) {
        lights::Light::new(
            1,
            Vector3::up() * 10.0,
            Vector3::zero(),
            Color::RED,
            light_shader,
        );
    }

    pub fn remove_player(&mut self, player_id: &u64) {
        println!("REMOVING ID {}", player_id);
        let player = self.players.remove(player_id).unwrap();
        self.colliders.remove(
            player.collider,
            &mut self.island_manager,
            &mut self.bodies,
            false,
        );
    }
}

pub fn create_player(
    manager: &mut GameManager,
    id: u64,
    speed: f32,
    position: Vector3,
    restitution: f32,
    mass: f32,
    model_path: &str,
    shape: S,
) -> Player {
    let character_controller = KinematicCharacterController::default();
    let vertices = get_vertices(model_path);
    let cam_controller = KinematicCharacterController::default();
    let collider = create_collider(&shape, restitution, Some(vertices));
    let col = manager.colliders.insert(collider);
    let player = Player::new(
        id,
        speed,
        character_controller,
        col,
        position,
        mass,
        cam_controller,
    );
    manager.players.insert(id, player.clone());
    player
}
