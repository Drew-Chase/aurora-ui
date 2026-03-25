use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_button() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(crate::page_header("Button", "Displays a button or a component that looks like a button."))
        .child(crate::example_section("Default", "A standard button with hover animation."))
        .child(crate::example_card(
            row!()
                .spacing(12.0)
                .align(Align::Center)
                .child(button!("Primary").background_color(colors::primary()).hover_background_color(Color::new(220, 220, 255, 255)).border_radius(Corners::all(6.0)).width(100).height(36))
                .child(button!("Secondary").background_color(colors::secondary()).hover_background_color(Color::new(50, 50, 54, 255)).border_radius(Corners::all(6.0)).width(110).height(36))
                .child(button!("Destructive").background_color(colors::destructive()).hover_background_color(Color::new(220, 50, 50, 255)).border_radius(Corners::all(6.0)).width(120).height(36))
        ))
}
