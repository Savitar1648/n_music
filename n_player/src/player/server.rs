use crate::player::mpris::PlayerBridge;
use crate::PlayerMessage;
use std::sync::mpsc::Sender;

#[cfg(target_os = "linux")]
pub type Server = mpris_server::Server<PlayerBridge>;

pub struct ServerManager {
    server: Server,
}

impl ServerManager {
    pub fn new(tx: Sender<PlayerMessage>) -> Self {
        if cfg!(target_os = "linux") {
            Self::new_linux(tx)
        } else {
            todo!()
        }
    }

    #[cfg(target_os = "linux")]
    pub fn new_linux(tx: Sender<PlayerMessage>) -> Self {
        let bridge = PlayerBridge::new(tx);
        let server = smol::block_on(Server::new("NMusic", bridge)).expect("can't create server");

        Self { server }
    }
}
