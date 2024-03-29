use crate::player::server::ServerManager;
use crate::{
    add_all_tracks_to_player, loader_thread, FileTrack, FileTracks, LoaderMessage, PlayerMessage,
};
use n_audio::from_path_to_name_without_ext;
use n_audio::queue::QueuePlayer;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

pub struct Player {
    player: QueuePlayer,
    rx_l: Receiver<LoaderMessage>,
    tx: Sender<PlayerMessage>,
    rx: Receiver<PlayerMessage>,
    rx_s: Receiver<PlayerMessage>,
    server_manager: ServerManager,
}

impl Player {
    pub fn new(tx: Sender<PlayerMessage>, rx: Receiver<PlayerMessage>) -> Self {
        let path = String::from("/home/enn3/Music/");
        let mut player = QueuePlayer::new(path.clone());
        // TODO: get path somehow
        add_all_tracks_to_player(&mut player, path.clone());
        let (tx_l, rx_l) = mpsc::channel();
        {
            let tracks: FileTracks = player.queue().clone().into();
            if !tracks.is_empty() {
                let tracks = tracks
                    .iter()
                    .map(|track| PathBuf::from(&path).join(track.name()))
                    .collect();
                thread::spawn(|| loader_thread(tx_l, tracks));
            }

            let tracks: FileTracks = FileTracks {
                tracks: tracks
                    .iter()
                    .map(|track| {
                        FileTrack::new(
                            from_path_to_name_without_ext(track.name()),
                            track.artist().to_string(),
                            track.duration(),
                            track.cover().clone(),
                            false,
                        )
                    })
                    .collect(),
            };
            tx.send(PlayerMessage::InitTracks(tracks.clone()))
                .expect("can't init tracks");
        }

        let (tx_s, rx_s) = mpsc::channel();

        let server_manager = ServerManager::new(tx_s);

        Self {
            player,
            rx_l,
            tx,
            rx,
            rx_s,
            server_manager,
        }
    }

    pub fn run(&mut self) {
        loop {
            self.run_once();
        }
    }

    pub fn run_once(&mut self) {
        while let Ok(message) = self.rx.try_recv() {
            self.parse_message(message);
        }

        while let Ok(message) = self.rx_s.try_recv() {
            self.parse_message(message);
        }

        let mut messages = vec![];
        while let Ok(message) = self.rx_l.try_recv() {
            messages.push(message);
        }
        if !messages.is_empty() {
            self.tx
                .send(PlayerMessage::Loaded(messages))
                .expect("can't send loaded message");
        }

        if self.player.has_ended() {
            self.player.play_next();
            self.tx
                .send(PlayerMessage::CurrentUpdated(self.player.index()))
                .expect("can't send updated track");
        }

        if let Some(time) = self.player.get_time() {
            self.tx
                .send(PlayerMessage::TimeUpdate(time))
                .expect("can't send updated time");
        }

        thread::sleep(Duration::from_millis(500));
    }

    pub fn parse_message(&mut self, message: PlayerMessage) {
        match message {
            PlayerMessage::Clicked(i) => {
                self.player.end_current().expect("can't stop current track");
                self.player.play_index(i);
                self.tx
                    .send(PlayerMessage::CurrentUpdated(self.player.index()))
                    .expect("can't send updated track");
            }
            PlayerMessage::Seek(seek) => {
                self.player
                    .seek_to(seek.floor() as u64, seek.fract() as f64)
                    .expect("can't seek");
            }
            PlayerMessage::Volume(volume) => {
                self.player.set_volume(volume).expect("can't set volume");
            }
            PlayerMessage::Next => {
                self.player.end_current().expect("can't end current song");
                self.player.play_next();
            }
            PlayerMessage::Previous => {
                self.player.end_current().expect("can't end current song");
                self.player.play_previous();
            }
            PlayerMessage::Pause => self.player.pause().expect("can't pause the player"),
            PlayerMessage::TogglePause => {
                if self.player.is_paused() {
                    self.player.unpause().expect("can't unpause the player");
                } else {
                    self.player.pause().expect("can't pause the player");
                }
            }
            PlayerMessage::Play => {
                self.player.unpause().expect("can't unpause the player");
            }
            PlayerMessage::SeekByTime(time) => {
                self.player.seek_to(time, 0.0).expect("can't seek by time");
            }
            _ => {}
        }
    }
}
