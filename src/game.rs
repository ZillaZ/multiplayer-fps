use crate::*;
use std::collections::HashMap;

use rapier3d::control::CharacterCollision;
use rapier3d::control::KinematicCharacterController;
use rapier3d::na::{Matrix, Quaternion, Unit, UnitQuaternion};
use rapier3d::{
    dynamics::RigidBodyType,
    na::{Const, OPoint},
};
use raylib::prelude::*;

use crate::player::Player;
use crate::reader::load_scene;
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
    pub objects: Vec<(NetworkObject, ColliderHandle, RigidBodyHandle)>,
    pub network_objects: Vec<NetworkObject>,
    pub players: HashMap<u64, Player>,
    pub default_player: Option<Player>,
    pub sender: Sender<Player>,
    pub receiver: Receiver<Player>,
    pub nsender: Sender<(Player, ResponseSignal)>,
    pub nreceiver: Receiver<(Player, ResponseSignal)>,
}

impl GameManager {
    fn move_player(&mut self, player: &mut Player) {
        let mut collisions = vec![];
        if let Some(player_now) = self.players.get(&player.id) {
            let player_mov = player.position - player_now.position;
            let mov = player.obj.move_shape(
                self.dt,
                &mut self.bodies,
                &mut self.colliders,
                &self.query_pipeline,
                &rapier3d::parry::shape::Ball::new(2.0),
                &Isometry::translation(player.position.x, player.position.y, player.position.z),
                vector![player_mov.x, player_mov.y, player_mov.z],
                QueryFilter::default().exclude_collider(player.collider),
                |collision| collisions.push(collision),
            );
            player.position += Vector3::new(mov.translation.x, mov.translation.y, mov.translation.z);

            self.solve_collisions(player, collisions, self.dt);
            self.update_collider(player);
        }
    }
    fn solve_collisions(
        &mut self,
        player: &mut Player,
        collisions: Vec<CharacterCollision>,
        dt: f32,
    ) {
        for collision in collisions {
            player.obj.solve_character_collision_impulses(
                dt,
                &mut self.bodies,
                &self.colliders,
                &self.query_pipeline,
                &rapier3d::parry::shape::Ball::new(2.0),
                player.mass,
                &collision,
                QueryFilter::new(),
            );
        }
    }

    fn update_collider(&mut self, player: &mut Player) {
        let access = self.colliders.get_mut(player.collider);
        if let Some(data) = access {
            data.set_translation(vector![
                player.position.x,
                player.position.y,
                player.position.z
            ]);
        }
    }
    pub async fn update(&mut self, pipeline: &mut PhysicsPipeline) {
        println!("prewait");
        let mut interval = tokio::time::interval(std::time::Duration::from_secs_f32(self.dt));
        interval.tick().await;
        println!("postwait");
        let rapier_gravity = vector![0.0, -90.81, 0.0];
        pipeline.step(
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

        for object in self.network_objects.iter_mut() {
            for physics in self.objects.iter() {
                if object.id == physics.0.id {
                    let access = self.colliders.get(physics.1).unwrap();
                    let position = access.translation();
                    let rotation = access.rotation().as_vector();
                    object.position = [position.x, position.y, position.z];
                    object.rotation = [rotation.z, rotation.y, rotation.x, -rotation.w];
                }
            }
        }
        while !self.receiver.is_empty() {
            let mut player = self.receiver.recv().unwrap();
            self.update_player(&mut player);
        }
    }

    pub fn update_player(&mut self, player: &mut Player) {
        self.move_player(player);
        self.players.insert(player.id, player.clone());
        let signal = self.build_signal(player);
        self.nsender.send((player.to_owned(), signal)).unwrap();
    }

    fn build_signal(&mut self, player: &mut Player) -> ResponseSignal {
        let mut signal = ResponseSignal::new(
            player.position,
            player.camera_position,
            player.camera_target,
            player.fwd,
            player.right,
        );
        signal.objects = self.network_objects.clone();
        signal.players = self
            .players
            .values()
            .map(|x| {
                ResponseSignal::new(
                    x.position,
                    x.camera_position,
                    x.camera_target,
                    x.fwd,
                    x.right,
                )
            })
            .collect::<Vec<ResponseSignal>>();
        signal.update().unwrap();
        signal
    }

    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        let (nsender, nreceiver) = unbounded();
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
            dt: 0.016,
            network_objects: Vec::new(),
            default_player: None,
            sender,
            receiver,
            nsender,
            nreceiver,
        }
    }

    pub fn add_object(
        &mut self,
        position: Vector3,
        rotation: Vector3,
        shape: S,
        body_type: RigidBodyType,
        restitution: f32,
        vertices: (Vec<OPoint<f32, Const<3>>>, Vec<[u32; 3]>),
        name: String,
        linear_damping: f32,
        additional_mass: f32,
        density: f32,
    ) {
        let object = NetworkObject::new(
            name,
            position,
            Vector4::new(rotation.x, rotation.y, rotation.z, 1.0),
        );
        let mut collider = create_collider(&shape, restitution, density, Some(vertices));
        collider.set_rotation(UnitQuaternion::new(vector![
            rotation.x, rotation.y, rotation.z
        ]));
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

    pub fn init_scene(&mut self, scene_path: &str) {
        let objects = load_scene(scene_path);
        for object in objects {
            match &object.name[2..] {
                "Player" => {
                    let player = self.create_player(
                        1.0,
                        object.position,
                        0.0,
                        1.0,
                        100.0,
                        (object.vertices, object.indices),
                        object.shape,
                    );
                    self.default_player = Some(player);
                }
                "Ball" => self.add_object(
                    object.position,
                    object.rotation,
                    S::SPHERE(Sphere::new(object.radius)),
                    object.body_type,
                    1.0,
                    (object.vertices, object.indices),
                    object.name,
                    1.0,
                    1.0,
                    1.0,
                ),
                _ => self.add_object(
                    object.position,
                    object.rotation,
                    object.shape,
                    object.body_type,
                    1.0,
                    (object.vertices, object.indices),
                    object.name,
                    1.0,
                    1.0,
                    1.0,
                ),
            }
        }
    }
    fn create_player(
        &mut self,
        speed: f32,
        position: Vector3,
        restitution: f32,
        density: f32,
        mass: f32,
        vertices: (Vec<OPoint<f32, Const<3>>>, Vec<[u32; 3]>),
        shape: S,
    ) -> Player {
        let mut rng = rand::thread_rng();
        let id = rng.gen_range(0..std::u64::MAX);
        let character_controller = KinematicCharacterController::default();
        let cam_controller = KinematicCharacterController::default();
        let collider = create_collider(&shape, restitution, density, Some(vertices.clone()));
        let col = self.colliders.insert(collider);
        let mut player = Player::new(
            id,
            speed,
            character_controller,
            col,
            position,
            mass,
            cam_controller,
        );
        player.vertices = Some(vertices);
        self.colliders
            .remove(col, &mut self.island_manager, &mut self.bodies, false);
        player
    }
    pub fn new_player(&mut self) -> Player {
        let mut rng = rand::thread_rng();
        let id = rng.gen_range(0..std::u64::MAX);
        let mut player = self.default_player.as_mut().unwrap().clone();
        player.camera_controller = KinematicCharacterController::default();
        player.obj = KinematicCharacterController::default();
        let collider = create_collider(&S::CONVEX, 0.0, 1.0, player.clone().vertices);
        let collider_handle = self.colliders.insert(collider);
        player.collider = collider_handle;
        player.id = id;
        self.players.insert(id, player.clone());
        player
    }
}
