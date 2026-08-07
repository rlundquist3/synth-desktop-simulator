use crate::{fm_synth_instrument::FMSynthInstrument, log, voices::Voice};
use midir::{ConnectError, Ignore, InitError, MidiInput, PortInfoError};
use std::{
    io::{Error, Write, stdin, stdout},
    num::ParseIntError,
    sync::atomic::Ordering,
    thread,
};

mod notes;

#[derive(Debug)]
struct MidiMessage(u8, u8, u8);

#[derive(Debug)]
pub enum MidiInputError {
    ErrorCreatingInput(InitError),
    NoPortFound,
    InvalidPortSelected,
    PortConnectionError,
    ParseIntError,
    PortInfoError,
    StdIOError,
}
impl From<InitError> for MidiInputError {
    fn from(err: InitError) -> Self {
        MidiInputError::ErrorCreatingInput(err)
    }
}
impl From<PortInfoError> for MidiInputError {
    fn from(_err: PortInfoError) -> Self {
        MidiInputError::PortInfoError
    }
}
impl From<ConnectError<MidiInput>> for MidiInputError {
    fn from(_err: ConnectError<MidiInput>) -> Self {
        MidiInputError::PortConnectionError
    }
}
impl From<ParseIntError> for MidiInputError {
    fn from(_err: ParseIntError) -> Self {
        MidiInputError::ParseIntError
    }
}
impl From<Error> for MidiInputError {
    fn from(_err: Error) -> Self {
        MidiInputError::StdIOError
    }
}

#[derive(Clone, Debug)]
pub struct Midi {
    instrument: FMSynthInstrument,
}

impl Midi {
    pub fn new(instrument: FMSynthInstrument) -> Self {
        Midi { instrument }
    }

    pub fn read_input(mut self) -> Result<(), MidiInputError> {
        log::push("Attempting to open MIDI connection");
        let mut input = String::new();
        let mut midi_in = MidiInput::new("reading MIDI input")?;

        midi_in.ignore(Ignore::None);

        let ports = midi_in.ports();
        let port = match ports.len() {
            0 => return Err(MidiInputError::NoPortFound),
            1 => {
                log::push(format!(
                    "1 midi input port found: {}",
                    midi_in.port_name(&ports[0]).unwrap()
                ));
                &ports[0]
            }
            _ => {
                log::push("Available MIDI input ports:");
                for (i, p) in ports.iter().enumerate() {
                    log::push(format!("{}: {}", i, midi_in.port_name(p).unwrap()));
                }
                log::push("Please select MIDI input port: ");
                stdout().flush()?;
                stdin().read_line(&mut input)?;
                ports
                    .get(input.trim().parse::<usize>()?)
                    .ok_or(MidiInputError::InvalidPortSelected)?
            }
        };

        log::push("Opening MIDI connection");
        let port_name = midi_in.port_name(port)?;
        let _connection = midi_in.connect(
            port,
            &port_name,
            move |_timestamp, message, _| {
                self.handle_event(MidiMessage(message[0], message[1], message[2]));
            },
            (),
        )?;

        log::push(format!(
            "Connection open, reading input from '{}'...",
            port_name
        ));

        // keep connection alive
        loop {
            thread::park();
        }
    }

    fn handle_event(&mut self, message: MidiMessage) {
        // Assumes single channel; need to match 129-143, 145 - 159 for channels 2-16
        match message.0 {
            128 => self.handle_note_off(message),
            144 => match message.2 {
                0 => self.handle_note_off(message),
                _ => self.handle_note_on(message),
            },
            176 => self.handle_control_change(message),
            _ => log::push(format!("unsupported status {message:?}")),
        };
    }

    fn handle_note_on(&mut self, message: MidiMessage) {
        log::push(format!("note on {message:?}"));

        let MidiMessage(_status, note, _vel) = message;
        let mut voice = self.instrument.voices.voice_on(note).lock().unwrap();
        voice.set_freq(notes::MIDI_NOTE_FREQS[note as usize]);
        voice.on.store(true, Ordering::Relaxed);
    }

    fn handle_note_off(&mut self, message: MidiMessage) {
        log::push(format!("note off {message:?}"));

        let MidiMessage(_status, note, _vel) = message;
        if let Some(voice) = self.instrument.voices.voice_off(note) {
            voice.lock().unwrap().on.store(false, Ordering::Relaxed);
        }
    }

    fn handle_control_change(&mut self, message: MidiMessage) {
        log::push(format!("control change {message:?}"));
    }
}
