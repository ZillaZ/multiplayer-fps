use std::collections::HashMap;

use crate::*;

#[derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum Reason {
    #[deku(id = "0x1")]
    IdInUse,
    #[deku(id = "0x2")]
    InvalidRequestFormat,
    #[deku(id = "0x3")]
    InvalidIdFormat,
    #[deku(id = "0x4")]
    InvalidPassword,
    #[deku(id = "0x5")]
    IdDoesntExist,
}

#[derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum ServerResponse {
    #[deku(id = "0x1")]
    Ok(ResponseSignal),
    #[deku(id = "0x2")]
    InvalidRequest(Reason),
}

pub struct GameNetwork {
    pub address: String,
    listener: Option<TcpListener>,
    active_sessions: HashMap<
        String,
        (
            Sender<(JoinSessionRequest, TcpStream)>,
            Receiver<(JoinResponse, Option<Player>)>,
        ),
    >,
    manager: GameManager,
}

impl GameNetwork {
    pub fn new(address: String) -> Self {
        let mut manager = GameManager::new();
        manager.init_scene("static/models/scene.obj");
        Self {
            address,
            listener: None,
            active_sessions: HashMap::new(),
            manager
        }
    }
    async fn open(&mut self) {
        if let Ok(listener) = TcpListener::bind(&self.address).await {
            println!("ready");
            self.listener = Some(listener);
        }
    }
    pub async fn start(&mut self) {
        self.open().await;
    }
    pub async fn update(&mut self) {
        self.accept_incoming().await;
    }
    async fn accept_incoming(&mut self) {
        use ServerRequest::*;

        while let Ok((mut stream, _addr)) = self.listener.as_ref().unwrap().accept().await {
            let mut buffer = [0; 4096];
            let _ = stream.read(&mut buffer).await;
            if let Ok((_, request)) = ServerRequest::from_bytes((&buffer, 0)) {
                match request {
                    NewSession(req) => self.create_session(req, stream).await,
                    JoinSession(req) => self.join_session(req, stream).await,
                };
            } else {
                let _ = stream
                    .write(
                        &ServerResponse::InvalidRequest(Reason::InvalidRequestFormat)
                            .to_bytes()
                            .unwrap(),
                    )
                    .await;
                let _ = stream.shutdown().await;
            }
        }
    }
    async fn create_session(&mut self, request: NewSessionRequest, mut stream: TcpStream) {
        let (sender, receiver) = unbounded();
        let session = Session::new(request.clone(), receiver, self.manager.clone());
        if let Ok((mut session, receiver)) = session {
            println!("here!");
            let _ = stream
                .write(
                    &ServerResponse::Ok(ResponseSignal::new(
                        Vector3::zero(),
                        Vector3::zero(),
                        Vector3::zero(),
                        Vector3::forward(),
                        Vector3::right(),
                    ))
                    .to_bytes()
                    .unwrap(),
                )
                .await;
            self.active_sessions
                .insert(session.id.clone(), (sender.clone(), receiver));
            let mut player = session.game_manager.new_player();
            let dt = session.game_manager.dt;
            let (mut sender, mut receiver) = (
                session.game_manager.sender.clone(),
                session.game_manager.nreceiver.clone(),
            );
            tokio::spawn(async move {
                let mut pipeline = PhysicsPipeline::new();
                loop {
                    session.update(&mut pipeline).await;
                }
            });

            tokio::spawn(async move {
                player.update(&mut sender, &mut receiver, dt, Vector3::up() * -9.81, &mut stream).await;
            });
        } else {
            println!("ta no else paekk");
            stream
                .write(
                    &ServerResponse::InvalidRequest(session.err().unwrap())
                        .to_bytes()
                        .unwrap(),
                )
                .await
                .unwrap();
        }
    }
    async fn join_session(&mut self, request: JoinSessionRequest, mut stream: TcpStream) {
        if let Some((sender, receiver)) = self
            .active_sessions
            .get(&String::from_utf8(request.id.clone()).unwrap())
        {
            sender.send((request, stream)).unwrap();
        } else {
            stream
                .write(
                    &ServerResponse::InvalidRequest(Reason::IdDoesntExist)
                        .to_bytes()
                        .unwrap(),
                )
                .await
                .unwrap();
        }
    }
}

async fn get_signal(stream: &mut TcpStream) -> Option<PlayerSignal> {
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer).await;
    if let Ok((_bytes, signal)) = PlayerSignal::from_bytes((&buffer, 0)) {
        return Some(signal);
    }
    None
}
