use crate::*;

pub async fn handle_connection(
    stream: &mut TcpStream,
    rec_clone: Receiver<GameManager>,
    channel: Sender<GameManager>,
) {
    let mut control = false;
    let mut player: Option<Player> = None;
    let mut buf = [0; 1024];
    let mut manager = None;
    let _ = manager;
    while let Ok(_data) = stream.read(&mut buf).await {
        manager = Some(rec_clone.recv().unwrap());
        if !control {
            player = Some(new_player(&mut manager.as_mut().unwrap()));
            control = true;
        }
        let state = PlayerSignal::from_bytes((&buf, 0)).unwrap().1;
        let signal = player.as_mut().unwrap().update(
            &mut manager.as_mut().unwrap(),
            state.clone(),
            state.dt,
            Vector3::new(0.0, -9.81, 0.0),
        );
        manager.as_mut().unwrap().dt = state.dt;
        channel.send(manager.as_ref().unwrap().clone()).unwrap();
        let _count = stream.write(&signal.to_bytes().unwrap()).await.unwrap();
        stream.flush().await.unwrap();
    }
    manager = Some(rec_clone.recv().unwrap());
    manager.as_mut().unwrap().remove_player(&player.unwrap().id);
    channel.send(manager.unwrap()).unwrap();
}
