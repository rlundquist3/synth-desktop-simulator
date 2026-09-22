pub mod tasks;

use crate::log;
use midir::{Ignore, MidiInput};
use std::sync::{LazyLock, Mutex};
use synth_core::midi::MidiMessage;
use tokio::sync::mpsc::{self, Receiver, Sender, error::TrySendError};

const MIDI_BUFFER_SIZE: usize = 16;

pub static MIDI_BUFFER: LazyLock<MidiBuffer> = LazyLock::new(MidiBuffer::new);

pub struct MidiBuffer {
    sender: Sender<MidiMessage>,
    receiver: Mutex<Option<Receiver<MidiMessage>>>,
}

impl MidiBuffer {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel(MIDI_BUFFER_SIZE);
        MidiBuffer {
            sender,
            receiver: Mutex::new(Some(receiver)),
        }
    }

    pub fn try_send(&self, message: MidiMessage) -> Result<(), TrySendError<MidiMessage>> {
        self.sender.try_send(message)
    }

    pub fn receiver(&self) -> Receiver<MidiMessage> {
        self.receiver
            .lock()
            .unwrap()
            .take()
            .expect("MIDI_BUFFER receiver should only be taken once")
    }
}

pub async fn midi_input_task() {
    log::push("Attempting to open MIDI connection");
    let mut midi_in = match MidiInput::new("terminal-synth") {
        Ok(midi_in) => midi_in,
        Err(err) => {
            log::push(format!("Error creating MIDI input: {err}"));
            return;
        }
    };
    midi_in.ignore(Ignore::None);

    let ports = midi_in.ports();
    let Some(port) = ports.first() else {
        log::push("MIDI device not found");
        return;
    };

    let port_name = midi_in.port_name(port).unwrap_or_default();
    log::push(format!("MIDI input port found: {port_name}"));

    let connection = midi_in.connect(port, &port_name, |_timestamp, bytes, _| rx_cb(bytes), ());
    let _connection = match connection {
        Ok(connection) => connection,
        Err(err) => {
            log::push(format!("Error connecting to MIDI port: {err}"));
            return;
        }
    };

    log::push(format!(
        "MIDI onnection open, reading input from '{port_name}'..."
    ));

    // Hold the connection open for the life of the task
    std::future::pending::<()>().await;
}

/// Sends received events to MIDI buffer
fn rx_cb(bytes: &[u8]) {
    // TODO: handle additional bytes?
    if bytes.len() < 3 {
        return;
    }

    match MIDI_BUFFER.try_send(MidiMessage(bytes[0], bytes[1], bytes[2])) {
        Ok(()) => {}
        Err(_) => log::push("MIDI buffer full"),
    };
}
