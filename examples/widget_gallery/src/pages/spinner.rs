use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_spinner() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(crate::page_header("Spinner", "A loading indicator that rotates continuously."))
        .child(crate::example_section("Sizes", "Spinners at different sizes."))
        .child(crate::example_card(
            row!()
                .spacing(24.0)
                .align(Align::Center)
                .child(spinner::Spinner::new())
                .child(spinner::Spinner::new().size(32.0).color(Color::new(59, 130, 246, 255)))
                .child(spinner::Spinner::new().size(48.0).color(Color::new(76, 175, 80, 255)).thickness(4.0))
        ))
}
