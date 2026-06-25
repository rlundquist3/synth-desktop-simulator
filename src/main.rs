mod oscillator;

use crate::oscillator::{Oscillator, Waveform::Sine};
use rodio::{DeviceSinkBuilder, Source};
use std::time::Duration;

// Hardcoded for now; TODO: make this configurable
pub static SAMPLE_RATE: u32 = 44100;

fn main() {
    let audio_device =
        DeviceSinkBuilder::open_default_sink().expect("Should open default audio device");

    let mut sine = Oscillator::new(Sine);
    sine.set_freq(440.0);

    audio_device
        .mixer()
        .add(sine.take_duration(Duration::from_millis(3000)));
    std::thread::sleep(Duration::from_millis(3000));
}
