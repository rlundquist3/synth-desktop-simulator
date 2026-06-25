use crate::oscillator::{Oscillator, Waveform::Sine};
use crate::utils::{A440, get_freq_for_note};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rodio::{MixerDeviceSink, Source};
use std::{
    io::{Result, stdout},
    time::Duration,
};

pub struct KeyboardInput {
    audio_device: MixerDeviceSink,
    oscillator: Oscillator,
}

impl KeyboardInput {
    pub fn new(audio_device: MixerDeviceSink) -> Self {
        KeyboardInput {
            audio_device: audio_device,
            oscillator: Oscillator::new(Sine),
        }
    }

    pub fn set_oscillator(&mut self, oscillator: Oscillator) {
        self.oscillator = oscillator;
    }

    pub fn listen(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
        println!("Command Line Synth\nCtrl+C to quit");

        loop {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }

                let freq = match key.code {
                    KeyCode::Char('q') => get_freq_for_note(-4), // F
                    KeyCode::Char('2') => get_freq_for_note(-3),
                    KeyCode::Char('w') => get_freq_for_note(-2), // G
                    KeyCode::Char('3') => get_freq_for_note(-1),
                    KeyCode::Char('e') => A440, // A
                    KeyCode::Char('4') => get_freq_for_note(1),
                    KeyCode::Char('r') => get_freq_for_note(2), // B
                    KeyCode::Char('t') => get_freq_for_note(4), // C
                    KeyCode::Char('6') => get_freq_for_note(5),
                    KeyCode::Char('y') => get_freq_for_note(6), // D
                    KeyCode::Char('7') => get_freq_for_note(7),
                    KeyCode::Char('u') => get_freq_for_note(8), // E
                    KeyCode::Char('i') => get_freq_for_note(9), // F
                    KeyCode::Char('9') => get_freq_for_note(10),
                    KeyCode::Char('o') => get_freq_for_note(11), // G
                    KeyCode::Char('0') => get_freq_for_note(12),
                    KeyCode::Char('p') => get_freq_for_note(13), // A
                    KeyCode::Char('-') => get_freq_for_note(14),
                    KeyCode::Char('[') => get_freq_for_note(15), // B
                    KeyCode::Char(']') => get_freq_for_note(16), // C
                    _ => 0.0,
                };

                if freq > 0.0 {
                    println!("Freq: {freq}");
                    self.oscillator.set_freq(freq);
                    self.audio_device.mixer().add(
                        self.oscillator
                            .clone()
                            .take_duration(Duration::from_millis(1000)),
                    );
                }
            }
        }

        disable_raw_mode()?;
        execute!(stdout(), LeaveAlternateScreen)?;
        Ok(())
    }
}
