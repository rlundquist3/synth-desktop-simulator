use std::{cell::RefCell, sync::Mutex, sync::atomic::Ordering};
use synth_core::{
    engines::fm::FMSynth,
    midi::{MIDI_NOTE_FREQS, MidiMessage, get_pitch_bend_value},
    voices::Voice,
};

use crate::{log, midi::MIDI_BUFFER};

pub async fn midi_buffer_handler(engine: &'static Mutex<RefCell<FMSynth>>) {
    let mut receiver = MIDI_BUFFER.receiver();

    loop {
        let Some(message) = receiver.recv().await else {
            log::push("MIDI buffer closed");
            return;
        };

        match message.0 {
            128 => handle_note_off(engine, message).await,
            144 => match message.2 {
                0 => handle_note_off(engine, message).await,
                _ => handle_note_on(engine, message).await,
            },
            224 => handle_pitch_bend(engine, get_pitch_bend_value(message)).await,
            _ => log::push(format!("unsupported status {message:?}")),
        };
    }
}

async fn handle_note_on(engine: &'static Mutex<RefCell<FMSynth>>, message: MidiMessage) {
    log::push(format!("note on {message:?}"));

    let MidiMessage(_status, note, _vel) = message;

    let e = engine.lock().unwrap();
    let mut engine = e.borrow_mut();

    let voice = engine.voices.voice_on(note);
    voice.set_freq(MIDI_NOTE_FREQS[note as usize], note as usize);
    voice.on.store(true, Ordering::Relaxed);
}

async fn handle_note_off(engine: &'static Mutex<RefCell<FMSynth>>, message: MidiMessage) {
    log::push(format!("note off {message:?}"));

    let MidiMessage(_status, note, _vel) = message;

    let e = engine.lock().unwrap();
    let mut engine = e.borrow_mut();
    if let Some(voice) = engine.voices.voice_off(note) {
        voice.on.store(false, Ordering::Relaxed);
    }
}

async fn handle_pitch_bend(engine: &'static Mutex<RefCell<FMSynth>>, bend: u16) {
    let e = engine.lock().unwrap();
    let mut engine = e.borrow_mut();
    engine.set_pitch_bend(bend);
}
