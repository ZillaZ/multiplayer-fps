use crossbeam::channel::{unbounded, Receiver, Sender};
use deku::prelude::*;
use game::GameManager;
use network::*;
use objects::{Shape as S, Sphere};
use player::*;
use rand::prelude::*;
use rapier3d::prelude::*;
use raylib::{math::Vector3, shaders::RaylibShader};
use session::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub mod custom_events;
pub mod game;
pub mod lights;
pub mod network;
pub mod objects;
pub mod player;
pub mod reader;
pub mod session;

#[derive(Debug, DekuRead, DekuWrite)]
struct Test {
    number: i32,
}

#[tokio::main]
async fn main() {
    let mut network = GameNetwork::new("127.0.0.1:9001".into());
    network.start().await;
    loop {
        network.update().await;
    }
}
