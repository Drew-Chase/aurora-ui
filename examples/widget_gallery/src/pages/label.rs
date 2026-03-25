use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_label() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(crate::page_header("Label", "Renders an accessible label associated with controls."))
        .child(crate::example_section("Default", "Labels for form controls."))
        .child(crate::example_card(
            col!()
                .spacing(8.0)
                .child(label::Label::new("Username"))
                .child(label::Label::new("Email address").font_size(12.0))
                .child(label::Label::new("Bold label").font_weight(FontWeight::Bold))
        ))
}
