mod keyboard_input;
mod oscillator;
mod utils;

use crate::{
    keyboard_input::KeyboardInput,
    oscillator::{Oscillator, Waveform::Sine},
};
use rodio::DeviceSinkBuilder;
use std::io::Result;

// Hardcoded for now; TODO: make this configurable
pub static SAMPLE_RATE: u32 = 44100;

fn main() -> Result<()> {
    let audio_device =
        DeviceSinkBuilder::open_default_sink().expect("Should open default audio device");

    let sine = Oscillator::new(Sine);

    let mut keyboard_input = KeyboardInput::new(audio_device);
    keyboard_input.set_oscillator(sine);
    keyboard_input.listen()
}
