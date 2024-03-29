use crate::PlayerMessage;
use mpris_server::zbus::fdo::Result;
use mpris_server::zbus::Error;
use mpris_server::{
    async_trait, zbus, LoopStatus, Metadata, PlaybackRate, PlaybackStatus, PlayerInterface,
    RootInterface, Time, TrackId, Volume,
};
use std::sync::mpsc::Sender;

pub struct PlayerBridge {
    tx: Sender<PlayerMessage>,
}

impl PlayerBridge {
    pub fn new(tx: Sender<PlayerMessage>) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl RootInterface for PlayerBridge {
    async fn raise(&self) -> Result<()> {
        Ok(())
    }

    async fn quit(&self) -> Result<()> {
        Ok(())
    }

    async fn can_quit(&self) -> Result<bool> {
        Ok(true)
    }

    async fn fullscreen(&self) -> Result<bool> {
        Ok(false)
    }

    async fn set_fullscreen(&self, _fullscreen: bool) -> zbus::Result<()> {
        Err(Error::Failure(String::from("Can't set fullscreen")))
    }

    async fn can_set_fullscreen(&self) -> Result<bool> {
        Ok(true)
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
        Ok(String::from("N Music"))
    }

    async fn supported_uri_schemes(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }

    async fn supported_mime_types(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }
}

#[async_trait]
impl PlayerInterface for PlayerBridge {
    async fn next(&self) -> Result<()> {
        Ok(self
            .tx
            .send(PlayerMessage::Next)
            .expect("can't send next packet"))
    }

    async fn previous(&self) -> Result<()> {
        Ok(self
            .tx
            .send(PlayerMessage::Previous)
            .expect("can't send previous packet"))
    }

    async fn pause(&self) -> Result<()> {
        Ok(self
            .tx
            .send(PlayerMessage::Pause)
            .expect("can't send pause packet"))
    }

    async fn play_pause(&self) -> Result<()> {
        Ok(self
            .tx
            .send(PlayerMessage::TogglePause)
            .expect("can't send toggle_pause packet"))
    }

    async fn stop(&self) -> Result<()> {
        Ok(())
    }

    async fn play(&self) -> Result<()> {
        Ok(self
            .tx
            .send(PlayerMessage::Play)
            .expect("can't send next packet"))
    }

    async fn seek(&self, offset: Time) -> Result<()> {
        Ok(self
            .tx
            .send(PlayerMessage::SeekByTime(offset.as_secs() as u64))
            .expect("can't send next packet"))
    }

    async fn set_position(&self, _track_id: TrackId, position: Time) -> Result<()> {
        Ok(self
            .tx
            .send(PlayerMessage::SeekByTime(position.as_secs() as u64))
            .expect("can't seek"))
    }

    async fn open_uri(&self, uri: String) -> Result<()> {
        Ok(())
    }

    async fn playback_status(&self) -> Result<PlaybackStatus> {
        Ok(PlaybackStatus::Playing)
    }

    async fn loop_status(&self) -> Result<LoopStatus> {
        Ok(LoopStatus::Playlist)
    }

    async fn set_loop_status(&self, _loop_status: LoopStatus) -> zbus::Result<()> {
        Err(Error::Failure(String::from("can't set loop status")))
    }

    async fn rate(&self) -> Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn set_rate(&self, rate: PlaybackRate) -> zbus::Result<()> {
        Err(Error::Failure(String::from("can't set rate")))
    }

    async fn shuffle(&self) -> Result<bool> {
        Ok(true)
    }

    async fn set_shuffle(&self, shuffle: bool) -> zbus::Result<()> {
        Err(Error::Failure(String::from("can't set shuffle")))
    }

    async fn metadata(&self) -> Result<Metadata> {
        Err(zbus::fdo::Error::Failed(String::from(
            "can't return metadata",
        )))
    }

    async fn volume(&self) -> Result<Volume> {
        Ok(1.0)
    }

    async fn set_volume(&self, volume: Volume) -> zbus::Result<()> {
        Ok(self
            .tx
            .send(PlayerMessage::Volume(volume as f32))
            .expect("can't send next packet"))
    }

    async fn position(&self) -> Result<Time> {
        Ok(Time::from_secs(0))
    }

    async fn minimum_rate(&self) -> Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn maximum_rate(&self) -> Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn can_go_next(&self) -> Result<bool> {
        Ok(true)
    }

    async fn can_go_previous(&self) -> Result<bool> {
        Ok(true)
    }

    async fn can_play(&self) -> Result<bool> {
        Ok(true)
    }

    async fn can_pause(&self) -> Result<bool> {
        Ok(true)
    }

    async fn can_seek(&self) -> Result<bool> {
        Ok(true)
    }

    async fn can_control(&self) -> Result<bool> {
        Ok(true)
    }
}
