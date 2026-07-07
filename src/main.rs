mod effects;
mod fm_synth;
mod instrument;
mod note;
mod oscillator;
mod ui;
mod utils;
mod voices;

use crate::ui::UI;
use rodio::DeviceSinkBuilder;
use std::io::Result;

// Hardcoded for now; TODO: make this configurable
pub static SAMPLE_RATE: u32 = 44100;

fn main() -> Result<()> {
    let audio_device =
        DeviceSinkBuilder::open_default_sink().expect("Should open default audio device");

    let mut ui = UI::new(audio_device);

    ratatui::run(|terminal| ui.run(terminal))
}
