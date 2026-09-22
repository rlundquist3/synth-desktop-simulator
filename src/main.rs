mod audio;
mod controls;
mod display;
mod log;
mod midi;
mod utils;

use crate::{
    audio::audio_handler,
    controls::control_handler,
    display::display_handler,
    midi::{midi_input_task, tasks::midi_buffer_handler},
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
    tokio::spawn(control_handler());

    display_handler(engine).await;
}
