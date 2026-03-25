use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_breadcrumb() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(crate::page_header("Breadcrumb", "Displays a breadcrumb navigation trail."))
        .child(crate::example_section("Default", "A simple breadcrumb trail."))
        .child(crate::example_card(
            breadcrumb::Breadcrumb::new().item("Home").item("Components").item("Breadcrumb")
        ))
}
