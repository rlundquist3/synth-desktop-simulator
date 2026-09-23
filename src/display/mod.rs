use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use std::{
    cell::RefCell,
    sync::{LazyLock, Mutex},
};
use synth_core::engines::fm::FMSynth;
use tokio::sync::mpsc::{self, Receiver, Sender, error::TrySendError};

use crate::{
    controls::{
        CONTROL_BUFFER, MODE,
        Mode::{
            EffectsDetail, EffectsMain, EngineEnvelope, EngineLFO, EngineMain, FilterDetail,
            FiltersMain,
        },
        navigation_event_for_key,
    },
    display::fm::{render_engine_envelope, render_engine_lfo, render_engine_main},
    log,
};

mod fm;

pub type Display = SimulatorDisplay<BinaryColor>;
pub type DisplayError = <Display as DrawTarget>::Error;

const DISPLAY_BUFFER_SIZE: usize = 16;
const DISPLAY_SIZE: Size = Size::new(128, 64);

pub struct DisplayContent {
    // pub text: String,
}

pub struct DisplayBuffer {
    sender: Sender<DisplayContent>,
    receiver: Mutex<Option<Receiver<DisplayContent>>>,
}

impl DisplayBuffer {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel(DISPLAY_BUFFER_SIZE);
        DisplayBuffer {
            sender,
            receiver: Mutex::new(Some(receiver)),
        }
    }

    pub fn try_send(&self, content: DisplayContent) -> Result<(), TrySendError<DisplayContent>> {
        self.sender.try_send(content)
    }

    pub fn receiver(&self) -> Receiver<DisplayContent> {
        self.receiver
            .lock()
            .unwrap()
            .take()
            .expect("DISPLAY_BUFFER receiver should only be taken once")
    }
}

pub static DISPLAY_BUFFER: LazyLock<DisplayBuffer> = LazyLock::new(DisplayBuffer::new);

pub async fn display_handler(engine: &'static Mutex<RefCell<FMSynth>>) {
    let mut display = SimulatorDisplay::<BinaryColor>::new(DISPLAY_SIZE);
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledWhite)
        .build();
    let mut window = Window::new("Synth", &output_settings);

    let mut buffer_rx = DISPLAY_BUFFER.receiver();
    let mode_rx = MODE.receiver();

    log::push("Display initialized");

    loop {
        while buffer_rx.try_recv().is_ok() {}

        display.clear(BinaryColor::Off).unwrap();

        let mode = mode_rx.borrow().clone();
        match mode {
            EngineMain => render_engine_main(&mut display, engine).unwrap(),
            EngineLFO => render_engine_lfo(&mut display, engine).unwrap(),
            EngineEnvelope => render_engine_envelope(&mut display, engine).unwrap(),
            FiltersMain => {}
            FilterDetail => {}
            EffectsMain => {}
            EffectsDetail => {}
        }

        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => return,
                SimulatorEvent::KeyDown {
                    keycode, keymod, ..
                } => {
                    if let Some(navigation_event) = navigation_event_for_key(keycode, keymod) {
                        // Stands in for the hardware's encoder interrupts
                        if CONTROL_BUFFER.try_send(navigation_event).is_err() {
                            log::push("Control buffer full");
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
