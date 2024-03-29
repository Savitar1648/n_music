#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] //Hide console window in release builds on Windows, this blocks stdout.

use n_player::player::player::Player;
use n_player::PlayerMessage;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

fn player_thread(rx: Receiver<PlayerMessage>, tx: Sender<PlayerMessage>) {
    let mut player = Player::new(tx, rx);
    player.run();
}

fn main() {
    let (tx_a, rx_a) = mpsc::channel();
    let (tx_b, rx_b) = mpsc::channel();
    thread::spawn(|| {
        player_thread(rx_a, tx_b);
    });
    // run(rx_b, tx_a);
}
