mod amp_envelope;
mod effects;
mod fm_synth;
mod fm_synth_instrument;
mod log;
mod midi;
mod oscillator;
mod parameter;
mod ui;
mod utils;
mod voices;

use crate::{fm_synth_instrument::FMSynthInstrument, midi::Midi, ui::UI};
use rodio::DeviceSinkBuilder;
use std::{io::Result, thread};

// Hardcoded for now; TODO: make this configurable
pub static SAMPLE_RATE: u32 = 44100;

fn main() -> Result<()> {
    let audio_device =
        DeviceSinkBuilder::open_default_sink().expect("Should open default audio device");
    let instrument = FMSynthInstrument::new();
    audio_device.mixer().add(instrument.clone());

    let mut ui = UI::new(instrument.clone());
    let midi = Midi::new(instrument.clone());

    thread::spawn(|| match midi.read_input() {
        Ok(_) => (),
        Err(err) => log::push(format!("Error: {:?}", err)),
    });

    ratatui::run(|terminal| ui.run(terminal))
}
