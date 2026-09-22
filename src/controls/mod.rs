use embedded_graphics_simulator::sdl2::Keycode;
use std::{
    cell::RefCell,
    sync::{LazyLock, Mutex},
};
use synth_core::{
    engines::fm::FMSynth,
    parameter::{
        ParameterChange::{Decrement, Increment},
        UserParameters,
    },
};
use tokio::sync::{
    mpsc::{self, Receiver, Sender, error::TrySendError},
    watch,
};

use crate::{
    controls::{
        EncoderEvent::{Click, Clockwise, Counterclockwise},
        NavigationEvent::{Down, Enter, Left, Right, Up},
    },
    log,
};

const CONTROL_BUFFER_SIZE: usize = 16;

#[derive(Clone, Debug)]
pub enum Mode {
    EngineMain,
    EngineEnvelope,
    EngineLFO,
    FiltersMain,
    FilterDetail,
    EffectsMain,
    EffectsDetail,
}

pub struct ModeWatch {
    sender: watch::Sender<Mode>,
}

impl ModeWatch {
    fn new() -> Self {
        let (sender, _) = watch::channel(Mode::EngineMain);
        ModeWatch { sender }
    }

    pub fn receiver(&self) -> watch::Receiver<Mode> {
        self.sender.subscribe()
    }

    pub fn sender(&self) -> &watch::Sender<Mode> {
        &self.sender
    }
}

pub static MODE: LazyLock<ModeWatch> = LazyLock::new(ModeWatch::new);

#[derive(Debug)]
pub enum NavigationEvent {
    Up,
    Down,
    Left,
    Right,
    Enter,
}

#[derive(Debug)]
pub enum EncoderEvent {
    Clockwise,
    Counterclockwise,
    Click,
}

pub struct ControlBuffer {
    sender: Sender<NavigationEvent>,
    receiver: Mutex<Option<Receiver<NavigationEvent>>>,
}

impl ControlBuffer {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel(CONTROL_BUFFER_SIZE);
        ControlBuffer {
            sender,
            receiver: Mutex::new(Some(receiver)),
        }
    }

    pub fn try_send(&self, event: NavigationEvent) -> Result<(), TrySendError<NavigationEvent>> {
        self.sender.try_send(event)
    }

    pub fn receiver(&self) -> Receiver<NavigationEvent> {
        self.receiver
            .lock()
            .unwrap()
            .take()
            .expect("CONTROL_BUFFER receiver should only be taken once")
    }
}

pub static CONTROL_BUFFER: LazyLock<ControlBuffer> = LazyLock::new(ControlBuffer::new);

pub fn navigation_event_for_key(keycode: Keycode) -> Option<NavigationEvent> {
    match keycode {
        Keycode::Up => Some(Up),
        Keycode::Down => Some(Down),
        Keycode::Left => Some(Left),
        Keycode::Right => Some(Right),
        Keycode::Return => Some(Enter),
        _ => None,
    }
}

pub async fn control_handler() {
    let mut control_rx = CONTROL_BUFFER.receiver();

    loop {
        let Some(event) = control_rx.recv().await else {
            log::push("Control buffer closed");
            return;
        };

        match event {
            Up => log::push("Navigation: Up"),
            Down => log::push("Navigation: Down"),
            Left => log::push("Navigation: Left"),
            Right => log::push("Navigation: Right"),
            Enter => log::push("Navigation: Enter"),
        }
    }
}

pub async fn engine_main_handler(
    engine: &'static Mutex<RefCell<FMSynth>>,
    encoder_index: usize,
    encoder_event: EncoderEvent,
) {
    log::push(format!(
        "EngineMain: encoder {:?} {:?}",
        encoder_index, encoder_event
    ));

    let e = engine.lock().unwrap();
    let mut engine = e.borrow_mut();

    let change = match encoder_event {
        Clockwise => Some(Increment),
        Counterclockwise => Some(Decrement),
        Click => None,
    };

    if let Some(change) = change {
        engine.update_parameter(encoder_index, change);
    }
}
