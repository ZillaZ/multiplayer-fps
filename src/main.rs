use crossbeam::channel::{unbounded, Receiver, Sender};
use deku::prelude::*;
use game::GameManager;
use game::*;
use objects::Shape as S;
use player::*;
use rand::prelude::*;
use rapier3d::prelude::*;
use raylib::{math::Vector3, shaders::RaylibShader};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

pub mod game;
pub mod lights;
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
    let (player_sender, player_receiver): (
        Sender<(Mutex<Player>, GameManager)>,
        Receiver<(Mutex<Player>, GameManager)>,
    ) = unbounded();

    let _ = tokio::spawn(async move {
        let mut manager = Mutex::new(GameManager::new());
        manager.lock().await.add_object(
            Vector3::zero(),
            S::DYNAMIC,
            RigidBodyType::Fixed,
            1.0,
            "static/models/untitled.obj",
            objects::ObjectType::GROUND,
            1.0,
            0.0,
        );
        manager.lock().await.add_object(
            Vector3::up() * 10000.0,
            S::DYNAMIC,
            RigidBodyType::Dynamic,
            1.0,
            "static/models/untitled.obj",
            objects::ObjectType::BALL,
            0.0,
            10.0,
        );
        let sender = state_sender.clone();
        loop {
            sender.send(manager.lock().await.clone()).unwrap();
            let player_state = player_receiver.recv();
            if player_state.as_ref().is_ok() {
                manager = Mutex::new(player_state.as_ref().unwrap().1.clone());
                println!("PLAYERS :");
                for player in manager.lock().await.players.values() {
                    println!("{:?}", player);
                }
                println!("");
            }
            manager.lock().await.update();
        }
    });

    while let Ok((mut stream, _addr)) = listener.accept().await {
        let rec_clone = state_receiver.clone();
        let clone = player_sender.clone();
        let _ = tokio::spawn(async move {
            handle_connection(&mut stream, rec_clone, clone).await;

        });
    }
}

fn new_player(manager: &mut GameManager) -> Player {
    let mut rng = thread_rng();
    let id = rng.gen_range(0..std::u64::MAX);
    create_player(
        manager,
        id,
        1.0,
        Vector3::zero(),
        0.0,
        50.0,
        "static/models/ball.obj",
        S::DYNAMIC,
    )
}

async fn handle_connection(
    stream: &mut TcpStream,
    rec_clone: Receiver<GameManager>,
    channel: Sender<(Mutex<Player>, GameManager)>,
) {
    let mut control = false;
    let mut player : Option<Player> = None;
    let mut buf = [0; 1024];
    let mut manager = None;
    while let Ok(data) = stream.read(&mut buf).await {
        manager = Some(rec_clone.recv().unwrap());
        if !control {
            player = Some(new_player(&mut manager.as_mut().unwrap()));
            control = true;
        }
        if data > 0 {
            let state = PlayerSignal::from_bytes((&buf, 0)).unwrap().1;
            let signal = player.as_mut().unwrap().update(
                &mut manager.as_mut().unwrap(),
                state.clone(),
                state.dt,
                Vector3::new(0.0, -9.81, 0.0),
            );
            manager.as_mut().unwrap().dt = state.dt;
            channel
                .send((Mutex::new(player.clone().unwrap()), manager.as_ref().unwrap().clone()))
                .unwrap();
            let count = stream.write(&signal.to_bytes().unwrap()).await.unwrap();
            println!("{:?}", count);
        }
    }
}
