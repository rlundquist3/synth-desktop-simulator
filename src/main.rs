mod log;
mod midi;
mod ui;
mod utils;

use crate::{midi::Midi, ui::UI};
use rodio::{DeviceSinkBuilder, Source};
use std::{
    io::Result,
    num::NonZero,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
    vec,
};
use synth_core::{SAMPLE_RATE, engines::fm::FMSynth};

struct FMSynthSource {
    synth: FMSynth,
}
impl FMSynthSource {
    pub fn new(synth: FMSynth) -> Self {
        FMSynthSource { synth }
    }
}

impl Iterator for FMSynthSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.synth.next()
    }
}

impl Source for FMSynthSource {
    fn channels(&self) -> NonZero<u16> {
        NonZero::new(1).unwrap()
    }

    fn sample_rate(&self) -> NonZero<u32> {
        NonZero::new(SAMPLE_RATE).unwrap()
    }

    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

fn main() {
    let audio_device =
        DeviceSinkBuilder::open_default_sink().expect("Should open default audio device");
    let instrument = Arc::new(Mutex::new(FMSynth::new()));

    let audio_instrument = Arc::clone(&instrument);
    let h1 = thread::spawn(move || {
        let i = audio_instrument.lock().unwrap();
        let source = FMSynthSource::new(*i);
        audio_device.mixer().add(source);
    });

    let midi_instrument = Arc::clone(&instrument);
    let h2 = thread::spawn(move || {
        let i = midi_instrument.lock().unwrap();
        let midi = Midi::new(*i);

        match midi.read_input() {
            Ok(_) => (),
            Err(err) => log::push(format!("Error: {:?}", err)),
        }
    });

    let ui_instrument = Arc::clone(&instrument);
    let h3 = thread::spawn(move || {
        let i = ui_instrument.lock().unwrap();
        let mut ui = UI::new(*i);
        ratatui::run(|terminal| ui.run(terminal))
    });

    h1.join().unwrap();
    h2.join().unwrap();
    h3.join().unwrap();
}
