use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_select() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(crate::page_header("Select", "Displays a list of options for the user to pick from."))
        .child(crate::example_section("Default", "Click to open the dropdown."))
        .child(crate::example_card(
            select::Select::new().placeholder("Select a fruit...").option("Apple").option("Banana").option("Cherry").option("Date").option("Elderberry").width(250.0)
        ))
}
