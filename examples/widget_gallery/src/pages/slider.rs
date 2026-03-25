use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_slider() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(crate::page_header("Slider", "An input where the user selects a value from within a given range."))
        .child(crate::example_section("Default", "Drag the thumb to set a value."))
        .child(crate::example_card(
            col!().spacing(16.0).child(slider::Slider::new().value(0.3).width(400.0)).child(slider::Slider::new().value(0.7).width(400.0))
        ))
}
