use crate::*;

pub struct JoinPlayer {
    pub player: Player,
    pub sender: Sender<Player>,
    pub receiver: Receiver<(Player, ResponseSignal)>,
    pub dt: f32,
    pub stream: TcpStream,
}

impl JoinPlayer {
    pub fn new(
        player: Player,
        sender: Sender<Player>,
        receiver: Receiver<(Player, ResponseSignal)>,
        dt: f32,
        stream: TcpStream,
    ) -> Self {
        Self {
            player,
            sender,
            receiver,
            dt,
            stream,
        }
    }
}

#[derive(Clone, Debug, DekuRead, DekuWrite)]
pub struct NewSessionRequest {
    #[deku(update = "self.id.len()")]
    id_count: usize,
    #[deku(count = "id_count")]
    pub id: Vec<u8>,
    #[deku(update = "self.password.len()")]
    count: usize,
    #[deku(count = "count")]
    pub password: Vec<u8>,
    pub player_limit: u8,
}

impl NewSessionRequest {
    pub fn new(id: &str, password: &str) -> Self {
        Self {
            id_count: id.len(),
            id: id.as_bytes().to_vec(),
            count: password.len(),
            password: password.as_bytes().to_vec(),
            player_limit: 8,
        }
    }
}

#[derive(Debug, DekuRead, DekuWrite)]
pub struct JoinSessionRequest {
    #[deku(update = "self.id.len()")]
    id_count: usize,
    #[deku(count = "id_count")]
    pub id: Vec<u8>,
    #[deku(update = "self.password.len()")]
    count: usize,
    #[deku(count = "count")]
    pub password: Vec<u8>,
}

impl JoinSessionRequest {
    pub fn new(id: &str, password: &str) -> Self {
        Self {
            id_count: id.len(),
            id: id.as_bytes().to_vec(),
            count: password.len(),
            password: password.as_bytes().to_vec(),
        }
    }
}

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum ServerRequest {
    #[deku(id = "0x1")]
    NewSession(NewSessionRequest),
    #[deku(id = "0x2")]
    JoinSession(JoinSessionRequest),
}

#[derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum JoinResponse {
    #[deku(id = "0x1")]
    Ok,
    #[deku(id = "0x2")]
    Err(Reason),
}

pub struct Session {
    pub id: String,
    pub game_manager: GameManager,
    password: String,
    player_limit: u8,
    sender: Sender<(JoinResponse, Option<JoinPlayer>)>,
    receiver: Receiver<(JoinSessionRequest, TcpStream)>,
    manager_receiver: Receiver<GameManager>,
    manager_sender: Sender<GameManager>,
}

impl Session {
    pub fn new(
        request: NewSessionRequest,
        receiver: Receiver<(JoinSessionRequest, TcpStream)>,
        mut game_manager: GameManager,
    ) -> Result<(Self, Receiver<(JoinResponse, Option<JoinPlayer>)>), Reason> {
        let (sender, response_receiver) = unbounded();
        let (manager_sender, manager_receiver) = unbounded();
        let (new_sender, new_receiver) = unbounded();
        let (nsender, nreceiver) = unbounded();
        game_manager.nsender = nsender;
        game_manager.nreceiver = nreceiver;
        game_manager.sender = new_sender;
        game_manager.receiver = new_receiver;
        if let Ok(password) = String::from_utf8(request.password) {
            return Ok((
                Self {
                    id: String::from_utf8(request.id).unwrap(),
                    player_limit: request.player_limit,
                    game_manager,
                    password,
                    receiver,
                    sender,
                    manager_receiver,
                    manager_sender,
                },
                response_receiver,
            ));
        } else {
            return Err(Reason::InvalidPassword);
        }
    }

    pub async fn update(
        &mut self,
        pipeline: &mut PhysicsPipeline,
        instant: &mut tokio::time::Instant,
    ) {
        if !self.manager_receiver.is_empty() {
            self.game_manager = self.manager_receiver.recv().unwrap();
        }
        while !self.receiver.is_empty() {
            self.join_player().await;
        }
        self.game_manager.update(pipeline, instant).await;
    }

    async fn join_player(&mut self) {
        println!("joining player");
        let (request, mut stream) = self.receiver.recv().unwrap();
        if request.password != self.password.as_bytes() {
            stream
                .write(&JoinResponse::Err(Reason::WrongPassword).to_bytes().unwrap())
                .await
                .unwrap();
            self.sender
                .send((JoinResponse::Err(Reason::WrongPassword), None))
                .unwrap();
            return;
        }
        let player = self.game_manager.new_player();
        stream
            .write(&JoinResponse::Ok.to_bytes().unwrap())
            .await
            .unwrap();
        stream.flush().await.unwrap();
        let (sender, receiver) = (
            self.game_manager.sender.clone(),
            self.game_manager.nreceiver.clone(),
        );
        let dt = self.game_manager.dt;
        self.sender
            .send((
                JoinResponse::Ok,
                Some(JoinPlayer::new(player, sender, receiver, dt, stream)),
            ))
            .unwrap();
        println!("player joined!");
    }
}
