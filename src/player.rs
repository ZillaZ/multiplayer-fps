use crate::*;
use objects::*;
use rapier3d::control::{CharacterCollision, KinematicCharacterController};
use rapier3d::na::{Const, OPoint};
use raylib::math::Vector2;

#[derive(Clone, Debug, DekuRead, DekuWrite)]
pub struct PlayerSignal {
    desired_mov: [f32; 3],
    desired_rot: [f32; 2],
    pub dt: f32,
}

#[derive(Clone, Debug, DekuRead, DekuWrite)]
pub struct ResponseSignal {
    #[deku(update = "self.players.len()")]
    pub player_count: usize,
    #[deku(update = "self.objects.len()")]
    pub object_count: usize,
    pub translation: [f32; 3],
    pub camera_pos: [f32; 3],
    pub camera_target: [f32; 3],
    pub fwd: [f32; 3],
    pub right: [f32; 3],
    #[deku(count = "player_count")]
    pub players: Vec<ResponseSignal>,
    #[deku(count = "object_count")]
    pub objects: Vec<NetworkObject>,
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

impl Default for ResponseSignal {
    fn default() -> Self {
        Self::new(
            Vector3::zero(),
            Vector3::zero(),
            Vector3::zero(),
            Vector3::forward(),
            Vector3::forward(),
        )
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
    pub camera_position: Vector3,
    pub camera_target: Vector3,
    pub speed: f32,
    pub right: Vector3,
    pitch: f32,
    yaw: f32,
    pub mass: f32,
    pub dt: f32,
    pub vertices: Option<(Vec<OPoint<f32, Const<3>>>, Vec<[u32; 3]>)>,
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
            vertices: None,
        }
    }

    pub async fn update(
        &mut self,
        sender: &mut Sender<Player>,
        receiver: &mut Receiver<(Player, ResponseSignal)>,
        dt: f32,
        gravity: Vector3,
        stream: &mut TcpStream,
    ) {
        while let Some(state) = self.get_state(stream).await {
            self.position += Vector3::new(
                state.desired_mov[0],
                state.desired_mov[1],
                state.desired_mov[2],
            ) + gravity * dt;

            self.camera_position = Vector3::new(self.position.x, self.position.y, self.position.z)
                + Vector3::up() * 5.0
                - self.fwd * 5.0;

            self.update_camera(dt, Vector2::new(state.desired_rot[0], state.desired_rot[1]));
            sender.send(self.clone()).unwrap();
            let (player, signal) = receiver.recv().unwrap();
            *self = player;
            println!("{:?}", signal);
            let _ = stream.write_all(&signal.to_bytes().unwrap()).await;
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

    async fn get_state(&mut self, stream: &mut TcpStream) -> Option<PlayerSignal> {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).await.unwrap();
        let result = PlayerSignal::from_bytes((&buffer, 0));
        if let Ok(signal) = result {
            return Some(signal.1);
        }
        None
    }
}
