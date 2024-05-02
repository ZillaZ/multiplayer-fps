use crate::*;

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
    WrongPassword,
}

pub struct Session {
    pub id: String,
    pub game_manager: GameManager,
    password: String,
    player_limit: u8,
    sender: Sender<(JoinResponse, Option<Player>)>,
    receiver: Receiver<(JoinSessionRequest, TcpStream)>,
    manager_receiver: Receiver<GameManager>,
    manager_sender: Sender<GameManager>,
}

impl Session {
    pub fn new(
        request: NewSessionRequest,
        receiver: Receiver<(JoinSessionRequest, TcpStream)>,
        game_manager: GameManager
    ) -> Result<(Self, Receiver<(JoinResponse, Option<Player>)>), Reason> {
        let (sender, response_receiver) = unbounded();
        let (manager_sender, manager_receiver) = unbounded();
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

    pub async fn update(&mut self, pipeline: &mut PhysicsPipeline) {
        if !self.manager_receiver.is_empty() {
            self.game_manager = self.manager_receiver.recv().unwrap();
        }
        while !self.receiver.is_empty() {
            self.join_player().await;
        }
        self.game_manager.update(pipeline).await;
    }

    async fn join_player(&mut self) {
        println!("joining player");
        let (request, mut stream) = self.receiver.recv().unwrap();
        if request.password != self.password.as_bytes() {
            stream
                .write(&JoinResponse::WrongPassword.to_bytes().unwrap())
                .await
                .unwrap();
            self.sender
                .send((JoinResponse::WrongPassword, None))
                .unwrap();
            return;
        }
        let mut player = self.game_manager.new_player();
        stream
            .write(&JoinResponse::Ok.to_bytes().unwrap())
            .await
            .unwrap();
        stream.flush().await.unwrap();
        let (mut sender, mut receiver) = (
            self.game_manager.sender.clone(),
            self.game_manager.nreceiver.clone(),
        );
        let dt = self.game_manager.dt;
        tokio::spawn(async move {
            println!("i'm inside the spawned thread!");
            player
                .update(
                    &mut sender,
                    &mut receiver,
                    dt,
                    Vector3::up() * -9.81,
                    &mut stream,
                ).await;
        });
        println!("player joined!");
    }
}
