use embedded_graphics::{Drawable, geometry::Dimensions};
use synth_gui::effects::filters::FiltersMainLayout;

use crate::{
    SharedChain,
    controls::NAVIGATION_LOCATION,
    display::{self, Display, DisplayError},
};

pub fn render_filters_main(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let navigation_rx = NAVIGATION_LOCATION.receiver();
    let navigation_location = navigation_rx.borrow().clone();

    let display_area = display.bounding_box();

    let c = chain.lock().unwrap();
    let mut chain = c.borrow_mut();

    let filters = chain.get_filters();

    FiltersMainLayout::new(filters, display_area, navigation_location).draw(display);

    Ok(())
}
