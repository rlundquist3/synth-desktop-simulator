use std::{
    io::{Error, Write, stdin, stdout},
    num::ParseIntError,
    thread,
};

use midir::{ConnectError, Ignore, InitError, MidiInput, PortInfoError};

use crate::log;

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

pub fn read_midi_input() -> Result<(), MidiInputError> {
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
            log::push(format!("MIDI Event: {:?}", message));
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
