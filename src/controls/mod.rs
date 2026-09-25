use embedded_graphics_simulator::sdl2::{Keycode, Mod};
use std::{
    cell::RefCell,
    sync::{LazyLock, Mutex},
};
use synth_core::{
    engines::fm::FMSynth,
    parameter::{
        self,
        ParameterChange::{Decrement, Increment},
        UserParameters,
    },
};
use tokio::sync::{
    mpsc::{self, Receiver, Sender, error::TrySendError},
    watch,
};

use crate::{
    SharedChain,
    controls::ControlEvent::{
        Encoder1Click, Encoder1Clockwise, Encoder1Counterclockwise, Encoder2Click,
        Encoder2Clockwise, Encoder2Counterclockwise, Encoder3Click, Encoder3Clockwise,
        Encoder3Counterclockwise, Encoder4Click, Encoder4Clockwise, Encoder4Counterclockwise,
        NavigationDown, NavigationEnter, NavigationLeft, NavigationRight, NavigationUp,
    },
    log,
};

const CONTROL_BUFFER_SIZE: usize = 16;

#[derive(Clone, Debug)]
pub enum Mode {
    EngineMain,
    EngineLFO,
    EngineEnvelope,
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

pub struct NavigationWatch {
    sender: watch::Sender<usize>,
}

impl NavigationWatch {
    fn new() -> Self {
        let (sender, _) = watch::channel(0);
        NavigationWatch { sender }
    }

    pub fn receiver(&self) -> watch::Receiver<usize> {
        self.sender.subscribe()
    }

    pub fn sender(&self) -> &watch::Sender<usize> {
        &self.sender
    }
}

pub static NAVIGATION_LOCATION: LazyLock<NavigationWatch> = LazyLock::new(NavigationWatch::new);

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

pub async fn control_handler(chain: &'static SharedChain) {
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
            Mode::EngineMain => engine_main_handler(chain, event).await,
            Mode::EngineLFO => engine_lfo_handler(chain, event).await,
            Mode::EngineEnvelope => engine_envelope_handler(chain, event).await,
            Mode::FiltersMain => filters_main_handler(chain, event).await,
            Mode::FilterDetail => {}
            Mode::EffectsMain => {}
            Mode::EffectsDetail => {}
        }
    }
}

async fn engine_main_handler(chain: &'static SharedChain, control_event: ControlEvent) {
    let mode_tx = MODE.sender();
    match control_event {
        NavigationRight => {
            mode_tx.send(Mode::EngineLFO);
            return;
        }
        _ => {}
    }

    if let Some((param_index, change)) = match control_event {
        Encoder1Clockwise => Some((0, Increment)),
        Encoder1Counterclockwise => Some((0, Decrement)),
        Encoder2Clockwise => Some((1, Increment)),
        Encoder2Counterclockwise => Some((1, Decrement)),
        Encoder3Clockwise => Some((2, Increment)),
        Encoder3Counterclockwise => Some((2, Decrement)),
        _ => None,
    } {
        let c = chain.lock().unwrap();
        let mut chain = c.borrow_mut();

        chain.get_engine().update_parameter(param_index, change);
    };
}

async fn engine_lfo_handler(chain: &'static SharedChain, control_event: ControlEvent) {
    let mode_tx = MODE.sender();
    match control_event {
        NavigationLeft => {
            mode_tx.send(Mode::EngineMain);
            return;
        }
        NavigationRight => {
            mode_tx.send(Mode::EngineEnvelope);
            return;
        }
        _ => {}
    }

    if let Some((param_index, change)) = match control_event {
        Encoder1Clockwise => Some((3, Increment)),
        Encoder1Counterclockwise => Some((3, Decrement)),
        Encoder2Clockwise => Some((4, Increment)),
        Encoder2Counterclockwise => Some((4, Decrement)),
        _ => None,
    } {
        let c = chain.lock().unwrap();
        let mut chain = c.borrow_mut();

        chain.get_engine().update_parameter(param_index, change);
    };
}

async fn engine_envelope_handler(chain: &'static SharedChain, control_event: ControlEvent) {
    let mode_tx = MODE.sender();
    match control_event {
        NavigationLeft => {
            mode_tx.send(Mode::EngineLFO);
            return;
        }
        NavigationRight => {
            mode_tx.send(Mode::FiltersMain);
            return;
        }
        _ => {}
    }

    if let Some((param_index, change)) = match control_event {
        Encoder1Clockwise => Some((5, Increment)),
        Encoder1Counterclockwise => Some((5, Decrement)),
        Encoder2Clockwise => Some((6, Increment)),
        Encoder2Counterclockwise => Some((6, Decrement)),
        Encoder3Clockwise => Some((7, Increment)),
        Encoder3Counterclockwise => Some((7, Decrement)),
        Encoder4Clockwise => Some((8, Decrement)),
        Encoder4Counterclockwise => Some((8, Increment)),
        _ => None,
    } {
        let c = chain.lock().unwrap();
        let mut chain = c.borrow_mut();

        chain.get_engine().update_parameter(param_index, change);
    };
}

async fn filters_main_handler(chain: &'static SharedChain, control_event: ControlEvent) {
    let mode_tx = MODE.sender();
    let navigation_tx = NAVIGATION_LOCATION.sender();
    let navigation_rx = NAVIGATION_LOCATION.receiver();

    let navigation_location = navigation_rx.borrow().clone();
    match navigation_location {
        0 => match control_event {
            NavigationLeft => {
                mode_tx.send(Mode::EngineEnvelope);
                return;
            }
            NavigationRight => {
                mode_tx.send(Mode::EffectsMain);
                return;
            }
            NavigationUp => {
                navigation_tx.send(1);
                return;
            }
            _ => {}
        },
        _ => match control_event {
            NavigationLeft => {
                match navigation_location {
                    1 => {}
                    _ => {
                        navigation_tx.send(navigation_location - 1);
                    }
                };
                return;
            }
            NavigationRight => {
                navigation_tx.send(navigation_location + 1);
                return;
            }
            NavigationDown => {
                navigation_tx.send(0);
                return;
            }
            NavigationEnter => {
                mode_tx.send(Mode::FilterDetail);
                return;
            }
            _ => {}
        },
    }
}
