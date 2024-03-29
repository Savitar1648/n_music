use bitcode::{Decode, Encode};
use n_audio::music_track::MusicTrack;
use n_audio::queue::QueuePlayer;
use n_audio::TrackTime;
use std::cmp::Ordering;
use std::ffi::OsStr;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

pub mod player;

pub fn loader_thread(tx: Sender<LoaderMessage>, tracks: Vec<PathBuf>) {
    for (i, track) in tracks.iter().enumerate() {
        if let Ok(music_track) = MusicTrack::new(track) {
            let mut format = music_track.get_format();

            tx.send(LoaderMessage::Duration(
                i,
                MusicTrack::get_duration_from_format(format.as_ref()).dur_secs,
            ))
            .expect("can't send back loaded times");

            tx.send(LoaderMessage::Artist(
                i,
                MusicTrack::get_artist_from_format(format.as_mut()),
            ))
            .expect("can't send back artist");

            tx.send(LoaderMessage::Image(
                i,
                MusicTrack::get_cover_from_format(format.as_mut()),
            ))
            .expect("can't send back cover art")
        }
    }
}

#[derive(Debug)]
pub enum LoaderMessage {
    Duration(usize, u64),
    Artist(usize, String),
    Image(usize, Vec<u8>),
}

pub enum PlayerMessage {
    InitTracks(FileTracks),
    Loaded(Vec<LoaderMessage>),
    Clicked(usize),
    Seek(f32),
    SeekByTime(u64),
    Volume(f32),
    TimeUpdate(TrackTime),
    CurrentUpdated(usize),
    Next,
    Previous,
    Pause,
    Play,
    TogglePause,
}

#[derive(Debug, Clone, Decode, Encode)]
pub struct FileTrack {
    name: String,
    artist: String,
    duration: u64,
    cover: Vec<u8>,
    current: bool,
}

impl Default for FileTrack {
    fn default() -> Self {
        Self::new(
            String::from("NAME"),
            String::from("ARTIST"),
            0,
            vec![],
            false,
        )
    }
}

impl FileTrack {
    pub fn new(name: String, artist: String, duration: u64, cover: Vec<u8>, current: bool) -> Self {
        Self {
            name,
            artist,
            duration,
            cover,
            current,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn artist(&self) -> &str {
        &self.artist
    }
    pub fn duration(&self) -> u64 {
        self.duration
    }
    pub fn cover(&self) -> &Vec<u8> {
        &self.cover
    }
}

impl From<String> for FileTrack {
    fn from(value: String) -> Self {
        Self {
            name: value,
            ..Default::default()
        }
    }
}

impl From<&String> for FileTrack {
    fn from(value: &String) -> Self {
        value.clone().into()
    }
}

impl Into<PathBuf> for FileTrack {
    fn into(self) -> PathBuf {
        PathBuf::from(self.name)
    }
}

impl Into<PathBuf> for &FileTrack {
    fn into(self) -> PathBuf {
        self.clone().into()
    }
}

impl PartialEq<Self> for FileTrack {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl PartialOrd<Self> for FileTrack {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for FileTrack {}

impl Ord for FileTrack {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Clone, Debug, Decode, Encode)]
pub struct FileTracks {
    pub tracks: Vec<FileTrack>,
}

impl From<Vec<String>> for FileTracks {
    fn from(value: Vec<String>) -> Self {
        let tracks = value.iter().map(|s| s.into()).collect();

        Self { tracks }
    }
}

impl Into<Vec<PathBuf>> for FileTracks {
    fn into(self) -> Vec<PathBuf> {
        self.tracks.iter().map(|track| track.into()).collect()
    }
}

impl Deref for FileTracks {
    type Target = [FileTrack];

    fn deref(&self) -> &Self::Target {
        &self.tracks
    }
}

impl DerefMut for FileTracks {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tracks
    }
}

fn vec_contains(tracks: &FileTracks, name: &String) -> (bool, usize) {
    for (i, track) in tracks.tracks.iter().enumerate() {
        if &track.name == name {
            return (true, i);
        }
    }

    (false, 0)
}

pub fn add_all_tracks_to_player<P: AsRef<Path>>(player: &mut QueuePlayer, path: P)
where
    P: AsRef<OsStr> + From<String>,
{
    let dir = fs::read_dir(path).expect("Can't read files in the chosen directory");
    dir.filter_map(|item| item.ok()).for_each(|file| {
        let mut p = file.path().to_str().unwrap().to_string();
        p.shrink_to_fit();
        player.add::<P>(p.into());
    });

    player.shuffle();
}
