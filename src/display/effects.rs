use embedded_graphics::{Drawable, geometry::Dimensions};
use synth_gui::effects::{effects_detail::EffectsDetailLayout, effects_main::EffectsMainLayout};

use crate::{
    SharedChain,
    controls::NAVIGATION_LOCATION,
    display::{self, Display, DisplayError},
};

pub fn render_effects_main(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let navigation_rx = NAVIGATION_LOCATION.receiver();
    let navigation_location = navigation_rx.borrow().clone();

    let display_area = display.bounding_box();

    let c = chain.lock().unwrap();
    let mut chain = c.borrow_mut();

    let effects = chain.get_effects();

    EffectsMainLayout::new(effects, display_area, navigation_location).draw(display);

    Ok(())
}

pub fn render_effects_detail(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let navigation_rx = NAVIGATION_LOCATION.receiver();
    let navigation_location = navigation_rx.borrow().clone();

    let display_area = display.bounding_box();

    let c = chain.lock().unwrap();
    let mut chain = c.borrow_mut();

    let effects = chain.get_effects();

    EffectsDetailLayout::new(effects, display_area, navigation_location).draw(display);

    Ok(())
}
