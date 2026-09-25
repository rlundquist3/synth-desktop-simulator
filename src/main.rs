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
use synth_core::{chain::Chain, engines::fm::FMSynth};

pub type SharedChain = Mutex<RefCell<Chain>>;

pub static CHAIN: StaticCell<SharedChain> = StaticCell::new();

#[tokio::main]
async fn main() {
    let chain: &'static SharedChain = CHAIN.init(Mutex::new(RefCell::new(Chain::new(Box::new(
        FMSynth::new(),
    )))));

    tokio::spawn(audio_handler(chain));
    tokio::spawn(midi_input_task());
    tokio::spawn(midi_buffer_handler(chain));
    tokio::spawn(control_handler(chain));

    display_handler(chain).await;
}
