use crate::*;
use objects::*;
use rapier3d::control::{CharacterCollision, KinematicCharacterController};
use raylib::math::Vector2;

#[derive(Clone, DekuRead, DekuWrite)]
pub struct PlayerSignal {
    desired_mov: [f32; 3],
    desired_rot: [f32; 2],
    pub dt: f32,
}

#[derive(Clone, Debug, DekuRead, DekuWrite)]
pub struct ResponseSignal {
    #[deku(update = "self.players.len()")]
    player_count: u8,
    #[deku(update = "self.objects.len()")]
    object_count: u8,
    translation: [f32; 3],
    camera_pos: [f32; 3],
    camera_target: [f32; 3],
    fwd: [f32; 3],
    right: [f32; 3],
    #[deku(count = "player_count")]
    players: Vec<ResponseSignal>,
    #[deku(count = "object_count")]
    objects: Vec<Object>,
}

impl ResponseSignal {
    pub fn new(
        translation: Vector3,
        camera_pos: Vector3,
        camera_target: Vector3,
        fwd: Vector3,
        right: Vector3,
    ) -> Self {
        Self {
            player_count: 0,
            object_count: 0,
            translation: translation.to_array(),
            camera_pos: camera_pos.to_array(),
            camera_target: camera_target.to_array(),
            fwd: fwd.to_array(),
            right: right.to_array(),
            players: Vec::new(),
            objects: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    pub id: u64,
    pub obj: KinematicCharacterController,
    pub collider: ColliderHandle,
    pub position: Vector3,
    pub fwd: Vector3,
    pub camera_controller: KinematicCharacterController,
    camera_position: Vector3,
    camera_target: Vector3,
    pub speed: f32,
    pitch: f32,
    yaw: f32,
    right: Vector3,
    pub mass: f32,
    pub dt: f32,
}

impl Player {
    pub fn new(
        id: u64,
        speed: f32,
        handle: KinematicCharacterController,
        collider: ColliderHandle,
        position: Vector3,
        mass: f32,
        cam_controller: KinematicCharacterController,
    ) -> Self {
        Self {
            id,
            pitch: 0.0,
            yaw: 0.0,
            speed,
            obj: handle,
            collider,
            position,
            fwd: Vector3::forward(),
            right: Vector3::right(),
            mass,
            camera_controller: cam_controller,
            camera_position: Vector3::zero(),
            camera_target: Vector3::forward(),
            dt: 0.0,
        }
    }

    pub fn update(
        &mut self,
        manager: &mut GameManager,
        state: PlayerSignal,
        dt: f32,
        gravity: Vector3,
    ) -> ResponseSignal {
        let mut collisions = vec![];
        let camera_pos =
            Vector3::new(self.position.x, self.position.y, self.position.z) - self.fwd * 5.0;
        let cam_mov = self.camera_controller.move_shape(
            dt,
            &mut manager.bodies,
            &mut manager.colliders,
            &manager.query_pipeline,
            &rapier3d::parry::shape::Ball::new(2.0),
            &Isometry::translation(
                self.camera_position.x,
                self.camera_position.y,
                self.camera_position.z,
            ),
            vector![camera_pos.x, camera_pos.y, camera_pos.z],
            QueryFilter::default().exclude_collider(self.collider),
            |_| {},
        );
        self.camera_position = Vector3::new(
            cam_mov.translation.x,
            cam_mov.translation.y,
            cam_mov.translation.z,
        );
        let player_mov = Vector3::new(
            state.desired_mov[0],
            state.desired_mov[1],
            state.desired_mov[2],
        ) + gravity * dt;
        self.update_camera(dt, Vector2::new(state.desired_rot[0], state.desired_rot[1]));
        let mov = self.obj.move_shape(
            dt,
            &mut manager.bodies,
            &mut manager.colliders,
            &manager.query_pipeline,
            &rapier3d::parry::shape::Ball::new(2.0),
            &Isometry::translation(self.position.x, self.position.y, self.position.z),
            vector![player_mov.x, player_mov.y, player_mov.z],
            QueryFilter::default().exclude_collider(self.collider),
            |collision| collisions.push(collision),
        );
        self.position += Vector3::new(mov.translation.x, mov.translation.y, mov.translation.z);

        self.solve_collisions(manager.clone(), collisions, dt);
        self.update_collider(manager);

        let mut response = ResponseSignal::new(
            self.position,
            self.camera_position,
            self.camera_target,
            self.fwd,
            self.right,
        );
        for player in manager.players.values() {
            response.players.push(ResponseSignal::new(
                player.position,
                player.camera_position,
                player.camera_target,
                player.fwd,
                player.right,
            ));
        }
        for object in manager.network_objects.iter() {
            response.objects.push(object.clone());
        }
        response.update().unwrap();
        manager.update_player(self);
        response
    }

    fn solve_collisions(
        &mut self,
        mut manager: GameManager,
        collisions: Vec<CharacterCollision>,
        dt: f32,
    ) {
        for collision in collisions {
            self.obj.solve_character_collision_impulses(
                dt,
                &mut manager.bodies,
                &manager.colliders,
                &manager.query_pipeline,
                &rapier3d::parry::shape::Ball::new(2.0),
                self.mass,
                &collision,
                QueryFilter::new(),
            );
        }
    }

    fn update_collider(&mut self, manager: &mut GameManager) {
        let access = manager.colliders.get_mut(self.collider);
        if let Some(data) = access {
            data.set_translation(vector![self.position.x, self.position.y, self.position.z]);
        }
    }

    pub fn update_camera(&mut self, dt: f32, delta: Vector2) {
        self.pitch += delta.x / 500.0;
        self.yaw += delta.y / 500.0;
        self.yaw = self.yaw.clamp(-1.5, 1.5);
        let r_matrix = raylib::prelude::Matrix::rotate_xyz(Vector3::new(self.yaw, self.pitch, 0.0));
        let target = Vector3::forward().transform_with(r_matrix);
        self.fwd = target;
        self.right = Vector3::new(-target.z, 0.0, target.x);
        self.camera_target = self.camera_position + target * dt;
    }
}
