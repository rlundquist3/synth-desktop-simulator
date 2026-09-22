mod audio;
mod log;
mod midi;
mod ui;
mod utils;

use crate::{
    audio::audio_handler,
    midi::{midi_input_task, tasks::midi_buffer_handler},
    ui::UI,
};
use static_cell::StaticCell;
use std::{cell::RefCell, sync::Mutex};
use synth_core::engines::fm::FMSynth;

pub static ENGINE: StaticCell<Mutex<RefCell<FMSynth>>> = StaticCell::new();

#[tokio::main]
async fn main() {
    let engine: &'static Mutex<RefCell<FMSynth>> =
        ENGINE.init(Mutex::new(RefCell::new(FMSynth::new())));

    tokio::spawn(audio_handler(engine));
    tokio::spawn(midi_input_task());
    tokio::spawn(midi_buffer_handler(engine));

    tokio::task::spawn_blocking(move || {
        let mut ui = UI::new(engine);
        ratatui::run(|terminal| ui.run(terminal))
    })
    .await
    .unwrap()
    .unwrap();
}
