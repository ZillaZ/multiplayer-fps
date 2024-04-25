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

pub mod custom_events;
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
    let (collision_send, collision_recv) = crossbeam::channel::unbounded();
    let (contact_force_send, contact_force_recv) = crossbeam::channel::unbounded();
    let event_handler = ChannelEventCollector::new(collision_send, contact_force_send);

    let _ = tokio::spawn(async move {
        let mut manager = GameManager::new();
        let mut pipeline = PhysicsPipeline::new();
        manager.init_scene("static/models/scene.obj");
        let sender = state_sender.clone();
        loop {
            sender.send(manager.clone()).unwrap();
            let player_state = player_receiver.recv();
            if let Ok(state) = player_state {
                manager = state;
            }
            manager.update(
                &mut pipeline,
                &event_handler,
                &collision_recv,
                &contact_force_recv,
            );
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
