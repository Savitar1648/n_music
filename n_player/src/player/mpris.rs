use crate::player::player::Player;
use mpris_server::{
    zbus, LoopStatus, Metadata, PlaybackRate, PlaybackStatus, PlayerInterface, RootInterface, Time,
    TrackId, Volume,
};

#[cfg(target_os = "linux")]
impl RootInterface for Player {
    async fn raise(&self) -> zbus::fdo::Result<()> {
        Ok(())
    }

    async fn quit(&self) -> zbus::fdo::Result<()> {
        Ok(())
    }

    async fn can_quit(&self) -> zbus::fdo::Result<bool> {
        Ok(false)
    }

    async fn fullscreen(&self) -> zbus::fdo::Result<bool> {
        Ok(false)
    }

    async fn set_fullscreen(&self, _fullscreen: bool) -> zbus::Result<()> {
        Ok(())
    }

    async fn can_set_fullscreen(&self) -> zbus::fdo::Result<bool> {
        Ok(false)
    }

    async fn can_raise(&self) -> zbus::fdo::Result<bool> {
        Ok(false)
    }

    async fn has_track_list(&self) -> zbus::fdo::Result<bool> {
        Ok(false)
    }

    async fn identity(&self) -> zbus::fdo::Result<String> {
        Ok(String::from("N Music"))
    }

    async fn desktop_entry(&self) -> zbus::fdo::Result<String> {
        Err(zbus::fdo::Error::NotSupported(String::from(
            "Not yet supported",
        )))
    }

    async fn supported_uri_schemes(&self) -> zbus::fdo::Result<Vec<String>> {
        Err(zbus::fdo::Error::NotSupported(String::from(
            "Not yet supported",
        )))
    }

    async fn supported_mime_types(&self) -> zbus::fdo::Result<Vec<String>> {
        Err(zbus::fdo::Error::NotSupported(String::from(
            "Not yet supported",
        )))
    }
}

#[cfg(target_os = "linux")]
impl PlayerInterface for Player {
    async fn next(&self) -> zbus::fdo::Result<()> {
        self.play_next(&mut self.player.lock().unwrap());
        Ok(())
    }

    async fn previous(&self) -> zbus::fdo::Result<()> {
        todo!()
    }

    async fn pause(&self) -> zbus::fdo::Result<()> {
        todo!()
    }

    async fn play_pause(&self) -> zbus::fdo::Result<()> {
        todo!()
    }

    async fn stop(&self) -> zbus::fdo::Result<()> {
        todo!()
    }

    async fn play(&self) -> zbus::fdo::Result<()> {
        todo!()
    }

    async fn seek(&self, offset: Time) -> zbus::fdo::Result<()> {
        todo!()
    }

    async fn set_position(&self, track_id: TrackId, position: Time) -> zbus::fdo::Result<()> {
        todo!()
    }

    async fn open_uri(&self, uri: String) -> zbus::fdo::Result<()> {
        todo!()
    }

    async fn playback_status(&self) -> zbus::fdo::Result<PlaybackStatus> {
        todo!()
    }

    async fn loop_status(&self) -> zbus::fdo::Result<LoopStatus> {
        Ok(LoopStatus::Playlist)
    }

    async fn set_loop_status(&self, _loop_status: LoopStatus) -> zbus::Result<()> {
        Ok(())
    }

    async fn rate(&self) -> zbus::fdo::Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn set_rate(&self, _rate: PlaybackRate) -> zbus::Result<()> {
        Ok(())
    }

    async fn shuffle(&self) -> zbus::fdo::Result<bool> {
        todo!()
    }

    async fn set_shuffle(&self, shuffle: bool) -> zbus::Result<()> {
        todo!()
    }

    async fn metadata(&self) -> zbus::fdo::Result<Metadata> {
        todo!()
    }

    async fn volume(&self) -> zbus::fdo::Result<Volume> {
        todo!()
    }

    async fn set_volume(&self, volume: Volume) -> zbus::Result<()> {
        todo!()
    }

    async fn position(&self) -> zbus::fdo::Result<Time> {
        todo!()
    }

    async fn minimum_rate(&self) -> zbus::fdo::Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn maximum_rate(&self) -> zbus::fdo::Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn can_go_next(&self) -> zbus::fdo::Result<bool> {
        Ok(true)
    }

    async fn can_go_previous(&self) -> zbus::fdo::Result<bool> {
        Ok(true)
    }

    async fn can_play(&self) -> zbus::fdo::Result<bool> {
        Ok(true)
    }

    async fn can_pause(&self) -> zbus::fdo::Result<bool> {
        Ok(true)
    }

    async fn can_seek(&self) -> zbus::fdo::Result<bool> {
        Ok(true)
    }

    async fn can_control(&self) -> zbus::fdo::Result<bool> {
        Ok(true)
    }
}
