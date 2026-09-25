use embedded_graphics::prelude::*;
use std::{cell::RefCell, sync::Mutex};
use synth_core::{engines::fm::FMSynth, parameter::UserParameters};
use synth_gui::{engines::fm::EngineMainLayout, envelope::EnvelopeLayout, lfo::LfoLayout};

use crate::{
    SharedChain,
    display::{Display, DisplayError},
};

pub fn render_engine_main(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let display_area = display.bounding_box();

    let c = chain.lock().unwrap();
    let mut chain = c.borrow_mut();

    let parameters = chain.get_engine().get_parameters();

    EngineMainLayout::new(parameters, display_area).draw(display)?;

    Ok(())
}

pub fn render_engine_lfo(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let display_area = display.bounding_box();

    let c = chain.lock().unwrap();
    let mut chain = c.borrow_mut();

    let parameters = chain.get_engine().get_parameters().clone().split_off(3);

    LfoLayout::new(parameters, display_area).draw(display)?;

    Ok(())
}

pub fn render_engine_envelope(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let display_area = display.bounding_box();

    let c = chain.lock().unwrap();
    let mut chain = c.borrow_mut();

    let parameters = chain.get_engine().get_envelope_parameters();

    EnvelopeLayout::new(parameters, display_area).draw(display)?;

    Ok(())
}
