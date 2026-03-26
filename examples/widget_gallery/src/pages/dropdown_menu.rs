use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_dropdown_menu() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(crate::page_header("Dropdown Menu", "A menu that appears on trigger click with a list of actions."))
        .child(crate::example_section("Default", "Click the button to open the menu."))
        .child(crate::example_card(
            dropdown_menu::DropdownMenu::new()
                .trigger(button!("Open Menu").background_color(colors::secondary()).hover_background_color(colors::secondary().opacity(0.8)).border_radius(Corners::all(6.0)).width(120).height(36))
                .item("Edit").item("Duplicate").separator().item("Archive").item("Delete")
        ))
}
