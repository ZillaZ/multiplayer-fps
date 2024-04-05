use crossbeam::channel::{unbounded, Receiver, Sender};
use deku::prelude::*;
use game::GameManager;
use game::*;
use network::*;
use objects::{Shape as S, Sphere};
use player::*;
use rand::prelude::*;
use rapier3d::prelude::*;
use raylib::{math::Vector3, shaders::RaylibShader};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub mod game;
pub mod lights;
pub mod network;
pub mod objects;
pub mod player;
pub mod reader;

#[derive(Debug, DekuRead, DekuWrite)]
struct Test {
    number: i32,
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:9001").await.unwrap();
    let (mut handle, _thread) = raylib::init().build();
    handle.set_target_fps(60);
    let (state_sender, state_receiver) = unbounded();
    let (player_sender, player_receiver): (Sender<GameManager>, Receiver<GameManager>) =
        unbounded();

    let _ = tokio::spawn(async move {
        let mut manager = GameManager::new();
        let mut pipeline = PhysicsPipeline::new();
        setup_scenario(&mut manager);
        let sender = state_sender.clone();
        loop {
            sender.send(manager.clone()).unwrap();
            let player_state = player_receiver.recv();
            if let Ok(state) = player_state {
                manager = state;
            }
            manager.update(&mut pipeline);
        }
    });

    while let Ok((mut stream, _addr)) = listener.accept().await {
        let rec_clone = state_receiver.clone();
        let clone = player_sender.clone();
        let _ = tokio::spawn(async move {
            handle_connection(&mut stream, rec_clone, clone).await;
            stream.shutdown().await.unwrap();
        });
    }
}

pub fn setup_scenario(manager: &mut GameManager) {
    manager.add_object(
        Vector3::zero(),
        S::CONVEX,
        RigidBodyType::Fixed,
        1.0,
        "static/models/untitled.obj",
        objects::ObjectType::GROUND,
        0.0,
        0.0,
    );
    manager.add_object(
        Vector3::up() * 10.0,
        S::SPHERE(Sphere::new(2.0)),
        RigidBodyType::Dynamic,
        1.0,
        "static/models/ball.obj",
        objects::ObjectType::BALL,
        0.0,
        10.0,
    );
    manager.add_object(
        Vector3::new(0.0, 10.0, 20.0),
        S::MULTI,
        RigidBodyType::Fixed,
        0.0,
        "static/models/roscakk.obj",
        objects::ObjectType::RING,
        0.0,
        0.0,
    );
    println!("Scenario is ready to go!");
}
