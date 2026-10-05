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
use synth_core::{buffer_pool::BufferPool, chain::Chain, engines::fm::FMSynth};

const SDRAM_SIZE: usize = 64 * 1024 * 1024;

pub type SharedChain = Mutex<RefCell<Chain>>;

pub static CHAIN: StaticCell<SharedChain> = StaticCell::new();

#[tokio::main]
async fn main() {
    let backing: &'static mut [f32] = Box::leak(vec![0.0; SDRAM_SIZE].into_boxed_slice());
    let mut buffer_pool = BufferPool::from_slice(backing);

    let chain: &'static SharedChain = CHAIN.init(Mutex::new(RefCell::new(Chain::new(
        Box::new(FMSynth::new()),
        &mut buffer_pool,
    ))));

    tokio::spawn(audio_handler(chain));
    tokio::spawn(midi_input_task());
    tokio::spawn(midi_buffer_handler(chain));
    tokio::spawn(control_handler(chain));

    display_handler(chain).await;
}
