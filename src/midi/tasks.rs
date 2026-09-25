use std::{cell::RefCell, sync::Mutex, sync::atomic::Ordering};
use synth_core::{
    engines::fm::FMSynth,
    midi::{MIDI_NOTE_FREQS, MidiMessage, get_pitch_bend_value},
    voices::Voice,
};

use crate::{SharedChain, log, midi::MIDI_BUFFER};

pub async fn midi_buffer_handler(chain: &'static SharedChain) {
    let mut receiver = MIDI_BUFFER.receiver();

    loop {
        let Some(message) = receiver.recv().await else {
            log::push("MIDI buffer closed");
            return;
        };

        match message.0 {
            128 => handle_note_off(chain, message).await,
            144 => match message.2 {
                0 => handle_note_off(chain, message).await,
                _ => handle_note_on(chain, message).await,
            },
            224 => handle_pitch_bend(chain, get_pitch_bend_value(message)).await,
            _ => log::push(format!("unsupported status {message:?}")),
        };
    }
}

async fn handle_note_on(chain: &'static SharedChain, message: MidiMessage) {
    log::push(format!("note on {message:?}"));

    let MidiMessage(_status, note, _vel) = message;

    let c = chain.lock().unwrap();
    let mut chain = c.borrow_mut();

    chain.get_engine().note_on(note);
}

async fn handle_note_off(chain: &'static SharedChain, message: MidiMessage) {
    log::push(format!("note off {message:?}"));

    let MidiMessage(_status, note, _vel) = message;

    let c = chain.lock().unwrap();
    let mut chain = c.borrow_mut();

    chain.get_engine().note_off(note);
}

async fn handle_pitch_bend(chain: &'static SharedChain, bend: u16) {
    let c = chain.lock().unwrap();
    let mut chain = c.borrow_mut();

    chain.get_engine().set_pitch_bend(bend);
}
