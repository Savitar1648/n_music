use crate::{
    add_all_tracks_to_player, loader_thread, FileTrack, FileTracks, LoaderMessage, PlayerMessage,
};
use mpris_server::zbus::fdo::Result;
use mpris_server::{
    zbus, LoopStatus, Metadata, PlaybackRate, PlaybackStatus, PlayerInterface, RootInterface, Time,
    TrackId, Volume,
};
use n_audio::from_path_to_name_without_ext;
use n_audio::queue::QueuePlayer;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

pub struct Player {
    player: Mutex<QueuePlayer>,
    rx_l: Mutex<Receiver<LoaderMessage>>,
    tx: Sender<PlayerMessage>,
    rx: Mutex<Receiver<PlayerMessage>>,
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

        Self {
            player: Mutex::new(player),
            rx_l: Mutex::new(rx_l),
            tx,
            rx: Mutex::new(rx),
        }
    }

    pub fn run(&mut self) {
        loop {
            self.run_once();
        }
    }

    fn next(&self, player: &mut MutexGuard<QueuePlayer>) {
        player.play_next();
        self.tx
            .send(PlayerMessage::CurrentUpdated(player.index()))
            .expect("can't send updated track");
    }

    pub fn run_once(&mut self) {
        {
            let mut player = self.player.lock().unwrap();
            let rx = self.rx.lock().unwrap();
            let rx_l = self.rx_l.lock().unwrap();
            while let Ok(message) = rx.try_recv() {
                self.parse_message(message, &mut player);
            }

            let mut messages = vec![];
            while let Ok(message) = rx_l.try_recv() {
                messages.push(message);
            }
            if !messages.is_empty() {
                self.tx
                    .send(PlayerMessage::Loaded(messages))
                    .expect("can't send loaded message");
            }

            if player.has_ended() {
                self.next(&mut player);
            }

            if let Some(time) = player.get_time() {
                self.tx
                    .send(PlayerMessage::TimeUpdate(time))
                    .expect("can't send updated time");
            }
        }

        thread::sleep(Duration::from_millis(500));
    }

    pub fn parse_message(&mut self, message: PlayerMessage, player: &mut MutexGuard<QueuePlayer>) {
        match message {
            PlayerMessage::Clicked(i) => {
                player.end_current().expect("can't stop current track");
                player.play_index(i);
                self.tx
                    .send(PlayerMessage::CurrentUpdated(player.index()))
                    .expect("can't send updated track");
            }
            PlayerMessage::Seek(seek) => {
                player
                    .seek_to(seek.floor() as u64, seek.fract() as f64)
                    .expect("can't seek");
            }
            PlayerMessage::Volume(volume) => {
                player.set_volume(volume).expect("can't set volume");
            }
            PlayerMessage::Next => {
                player.end_current().expect("can't end current song");
                self.next(player);
            }
            PlayerMessage::Previous => {
                player.end_current().expect("can't end current song");
                player.play_previous();
            }
            PlayerMessage::Pause => player.pause().expect("can't pause the player"),
            PlayerMessage::TogglePause => {
                if player.is_paused() {
                    player.unpause().expect("can't unpause the player");
                } else {
                    player.pause().expect("can't pause the player");
                }
            }
            PlayerMessage::Play => {
                player.unpause().expect("can't unpause the player");
            }
            PlayerMessage::SeekByTime(time) => {
                player.seek_to(time, 0.0).expect("can't seek by time");
            }
            _ => {}
        }
    }
}

#[cfg(target_os = "linux")]
impl RootInterface for Player {
    async fn raise(&self) -> Result<()> {
        Ok(())
    }

    async fn quit(&self) -> Result<()> {
        Ok(())
    }

    async fn can_quit(&self) -> Result<bool> {
        Ok(false)
    }

    async fn fullscreen(&self) -> Result<bool> {
        Ok(false)
    }

    async fn set_fullscreen(&self, _fullscreen: bool) -> zbus::Result<()> {
        Ok(())
    }

    async fn can_set_fullscreen(&self) -> Result<bool> {
        Ok(false)
    }

    async fn can_raise(&self) -> Result<bool> {
        Ok(false)
    }

    async fn has_track_list(&self) -> Result<bool> {
        Ok(false)
    }

    async fn identity(&self) -> Result<String> {
        Ok(String::from("N Music"))
    }

    async fn desktop_entry(&self) -> Result<String> {
        Err(zbus::fdo::Error::NotSupported(String::from(
            "Not yet supported",
        )))
    }

    async fn supported_uri_schemes(&self) -> Result<Vec<String>> {
        Err(zbus::fdo::Error::NotSupported(String::from(
            "Not yet supported",
        )))
    }

    async fn supported_mime_types(&self) -> Result<Vec<String>> {
        Err(zbus::fdo::Error::NotSupported(String::from(
            "Not yet supported",
        )))
    }
}

#[cfg(target_os = "linux")]
impl PlayerInterface for Player {
    async fn next(&self) -> Result<()> {
        self.next(&mut self.player.lock().unwrap());
        Ok(())
    }

    async fn previous(&self) -> Result<()> {
        todo!()
    }

    async fn pause(&self) -> Result<()> {
        todo!()
    }

    async fn play_pause(&self) -> Result<()> {
        todo!()
    }

    async fn stop(&self) -> Result<()> {
        todo!()
    }

    async fn play(&self) -> Result<()> {
        todo!()
    }

    async fn seek(&self, offset: Time) -> Result<()> {
        todo!()
    }

    async fn set_position(&self, track_id: TrackId, position: Time) -> Result<()> {
        todo!()
    }

    async fn open_uri(&self, uri: String) -> Result<()> {
        todo!()
    }

    async fn playback_status(&self) -> Result<PlaybackStatus> {
        todo!()
    }

    async fn loop_status(&self) -> Result<LoopStatus> {
        todo!()
    }

    async fn set_loop_status(&self, loop_status: LoopStatus) -> zbus::Result<()> {
        todo!()
    }

    async fn rate(&self) -> Result<PlaybackRate> {
        todo!()
    }

    async fn set_rate(&self, rate: PlaybackRate) -> zbus::Result<()> {
        todo!()
    }

    async fn shuffle(&self) -> Result<bool> {
        todo!()
    }

    async fn set_shuffle(&self, shuffle: bool) -> zbus::Result<()> {
        todo!()
    }

    async fn metadata(&self) -> Result<Metadata> {
        todo!()
    }

    async fn volume(&self) -> Result<Volume> {
        todo!()
    }

    async fn set_volume(&self, volume: Volume) -> zbus::Result<()> {
        todo!()
    }

    async fn position(&self) -> Result<Time> {
        todo!()
    }

    async fn minimum_rate(&self) -> Result<PlaybackRate> {
        todo!()
    }

    async fn maximum_rate(&self) -> Result<PlaybackRate> {
        todo!()
    }

    async fn can_go_next(&self) -> Result<bool> {
        todo!()
    }

    async fn can_go_previous(&self) -> Result<bool> {
        todo!()
    }

    async fn can_play(&self) -> Result<bool> {
        todo!()
    }

    async fn can_pause(&self) -> Result<bool> {
        todo!()
    }

    async fn can_seek(&self) -> Result<bool> {
        todo!()
    }

    async fn can_control(&self) -> Result<bool> {
        todo!()
    }
}
