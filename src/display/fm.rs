use embedded_graphics::prelude::*;
use std::{cell::RefCell, sync::Mutex};
use synth_core::{engines::fm::FMSynth, parameter::UserParameters};
use synth_gui::engines::fm::EngineMainLayout;

use crate::display::{Display, DisplayError};

pub async fn render_engine_main(
    display: &mut Display,
    engine: &'static Mutex<RefCell<FMSynth>>,
) -> Result<(), DisplayError> {
    let display_area = display.bounding_box();

    let e = engine.lock().unwrap();
    let engine = e.borrow();
    let parameters = engine.get_parameters();

    EngineMainLayout::new(parameters, display_area).draw(display)?;

    Ok(())
}
