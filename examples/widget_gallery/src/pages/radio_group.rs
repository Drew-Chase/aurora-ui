use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_radio_group() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(crate::page_header("Radio Group", "A set of checkable buttons where only one can be checked at a time."))
        .child(crate::example_section("Default", "Select one option from the group."))
        .child(crate::example_card(radio_group::RadioGroup::new().item("Default").item("Comfortable").item("Compact").selected(0)))
}
