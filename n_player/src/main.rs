#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] //Hide console window in release builds on Windows, this blocks stdout.

use n_audio::from_path_to_name_without_ext;
use n_audio::queue::QueuePlayer;
use n_player::app::run;
use n_player::{add_all_tracks_to_player, loader_thread, FileTrack, FileTracks, PlayerMessage};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

fn player_thread(rx: Receiver<PlayerMessage>, tx: Sender<PlayerMessage>) {
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
                    )
                })
                .collect(),
        };
        tx.send(PlayerMessage::InitTracks(tracks.clone()))
            .expect("can't init tracks");
    }

    loop {
        while let Ok(message) = rx.try_recv() {
            match message {
                PlayerMessage::Clicked(i) => {
                    player.end_current().expect("can't stop current track");
                    player.play_index(i);
                    tx.send(PlayerMessage::CurrentUpdated(player.index()))
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
                _ => {}
            }
        }

        let mut messages = vec![];
        while let Ok(message) = rx_l.try_recv() {
            messages.push(message);
        }
        if !messages.is_empty() {
            tx.send(PlayerMessage::Loaded(messages))
                .expect("can't send loaded message");
        }

        if player.has_ended() {
            player.play_next();
            tx.send(PlayerMessage::CurrentUpdated(player.index()))
                .expect("can't send updated track");
        }

        if let Some(time) = player.get_time() {
            tx.send(PlayerMessage::TimeUpdate(time))
                .expect("can't send updated time");
        }

        thread::sleep(Duration::from_millis(500));
    }
}

fn main() {
    let (tx_a, rx_a) = mpsc::channel();
    let (tx_b, rx_b) = mpsc::channel();
    thread::spawn(|| {
        player_thread(rx_a, tx_b);
    });
    run(rx_b, tx_a);
}
