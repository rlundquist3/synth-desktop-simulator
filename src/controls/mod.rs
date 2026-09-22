use embedded_graphics_simulator::sdl2::{Keycode, Mod};
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
        ControlEvent::{
            Encoder1Click, Encoder1Clockwise, Encoder1Counterclockwise, Encoder2Click,
            Encoder2Clockwise, Encoder2Counterclockwise, Encoder3Click, Encoder3Clockwise,
            Encoder3Counterclockwise, Encoder4Click, Encoder4Clockwise, Encoder4Counterclockwise,
            NavigationDown, NavigationEnter, NavigationLeft, NavigationRight, NavigationUp,
        },
        EncoderEvent::{Click, Clockwise, Counterclockwise},
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
pub enum ControlEvent {
    NavigationUp,
    NavigationDown,
    NavigationLeft,
    NavigationRight,
    NavigationEnter,
    Encoder1Clockwise,
    Encoder1Counterclockwise,
    Encoder1Click,
    Encoder2Clockwise,
    Encoder2Counterclockwise,
    Encoder2Click,
    Encoder3Clockwise,
    Encoder3Counterclockwise,
    Encoder3Click,
    Encoder4Clockwise,
    Encoder4Counterclockwise,
    Encoder4Click,
}

#[derive(Debug)]
pub enum EncoderEvent {
    Clockwise,
    Counterclockwise,
    Click,
}

pub struct ControlBuffer {
    sender: Sender<ControlEvent>,
    receiver: Mutex<Option<Receiver<ControlEvent>>>,
}

impl ControlBuffer {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel(CONTROL_BUFFER_SIZE);
        ControlBuffer {
            sender,
            receiver: Mutex::new(Some(receiver)),
        }
    }

    pub fn try_send(&self, event: ControlEvent) -> Result<(), TrySendError<ControlEvent>> {
        self.sender.try_send(event)
    }

    pub fn receiver(&self) -> Receiver<ControlEvent> {
        self.receiver
            .lock()
            .unwrap()
            .take()
            .expect("CONTROL_BUFFER receiver should only be taken once")
    }
}

pub static CONTROL_BUFFER: LazyLock<ControlBuffer> = LazyLock::new(ControlBuffer::new);

pub fn navigation_event_for_key(keycode: Keycode, keymod: Mod) -> Option<ControlEvent> {
    match keycode {
        Keycode::Up => Some(NavigationUp),
        Keycode::Down => Some(NavigationDown),
        Keycode::Left => Some(NavigationLeft),
        Keycode::Right => Some(NavigationRight),
        Keycode::Return => Some(NavigationEnter),
        Keycode::NUM_1 => {
            if keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD) {
                return Some(Encoder1Counterclockwise);
            }
            if keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::RCTRLMOD) {
                return Some(Encoder1Click);
            }
            Some(Encoder1Clockwise)
        }
        Keycode::NUM_2 => {
            if keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD) {
                return Some(Encoder2Counterclockwise);
            }
            if keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::RCTRLMOD) {
                return Some(Encoder2Click);
            }
            Some(Encoder2Clockwise)
        }
        Keycode::NUM_3 => {
            if keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD) {
                return Some(Encoder3Counterclockwise);
            }
            if keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::RCTRLMOD) {
                return Some(Encoder3Click);
            }
            Some(Encoder3Clockwise)
        }
        Keycode::NUM_4 => {
            if keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD) {
                return Some(Encoder4Counterclockwise);
            }
            if keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::RCTRLMOD) {
                return Some(Encoder4Click);
            }
            Some(Encoder4Clockwise)
        }
        _ => None,
    }
}

pub async fn control_handler(engine: &'static Mutex<RefCell<FMSynth>>) {
    let mut control_rx = CONTROL_BUFFER.receiver();
    let mode_rx = MODE.receiver();

    loop {
        let Some(event) = control_rx.recv().await else {
            log::push("Control buffer closed");
            return;
        };

        log::push(&format!("Control event: {:?}", event));

        let mode = mode_rx.borrow().clone();
        match mode {
            Mode::EngineMain => engine_main_handler(engine, event).await,
            Mode::EngineEnvelope => {}
            Mode::EngineLFO => {}
            Mode::FiltersMain => {}
            Mode::FilterDetail => {}
            Mode::EffectsMain => {}
            Mode::EffectsDetail => {}
        }
    }
}

pub async fn engine_main_handler(
    engine: &'static Mutex<RefCell<FMSynth>>,
    control_event: ControlEvent,
) {
    if let Some((encoder_index, change)) = match control_event {
        Encoder1Clockwise => Some((0, Increment)),
        Encoder1Counterclockwise => Some((0, Decrement)),
        Encoder2Clockwise => Some((1, Increment)),
        Encoder2Counterclockwise => Some((1, Decrement)),
        Encoder3Clockwise => Some((2, Increment)),
        Encoder3Counterclockwise => Some((2, Decrement)),
        _ => None,
    } {
        let e = engine.lock().unwrap();
        let mut engine = e.borrow_mut();

        engine.update_parameter(encoder_index, change);
    };
}
