use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;
use crate::theme;
use std::sync::Arc;

aurora_iconify::icon_sets!("lucide");

fn icon(svg_str: &str, size: f32, color: Color) -> Svg {
    let data = Arc::new(SvgData::from_bytes(svg_str.as_bytes()).unwrap());
    Svg::new(data).width(size).height(size).color(color)
}

pub fn page_button() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Button", "Displays a button or a component that looks like a button."))
        .child(crate::example_section("Default", "A standard button with hover animation."))
        .child(crate::example_card(
            row!()
                .spacing(12.0)
                .align(Align::Center)
                .child(button!("Primary").background_color(colors::primary()).hover_background_color(colors::primary().opacity(0.8)).foreground_color(colors::primary_foreground()).border_radius(theme::corners::button_radius()).width(100).height(36))
                .child(button!("Secondary").background_color(colors::secondary()).hover_background_color(colors::secondary().opacity(0.8)).foreground_color(colors::secondary_foreground()).border_radius(theme::corners::button_radius()).width(110).height(36))
                .child(button!("Destructive").background_color(colors::destructive()).hover_background_color(colors::destructive().opacity(0.8)).foreground_color(colors::destructive_foreground()).border_radius(theme::corners::button_radius()).width(120).height(36))
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"// Primary
button!("Primary")
    .background_color(colors::primary())
    .foreground_color(colors::primary_foreground())

// Secondary
button!("Secondary")
    .background_color(colors::secondary())
    .foreground_color(colors::secondary_foreground())

// Destructive
button!("Destructive")
    .background_color(colors::destructive())
    .foreground_color(colors::destructive_foreground())"#
        ).font_size(13.0))
        .child(crate::example_section("With Icons", "Buttons with start and end icon slots."))
        .child(crate::example_card(
            row!()
                .spacing(12.0)
                .align(Align::Center)
                .child(
                    button!("Send")
                        .start(icon(Icon::lucide().send(), 16.0, colors::primary_foreground()))
                        .background_color(colors::primary())
                        .hover_background_color(colors::primary().opacity(0.8))
                        .foreground_color(colors::primary_foreground())
                        .border_radius(theme::corners::button_radius())
                        .width(140)
                        .height(36),
                )
                .child(
                    button!("Next")
                        .end(icon(Icon::lucide().arrow_right(), 16.0, colors::secondary_foreground()))
                        .background_color(colors::secondary())
                        .hover_background_color(colors::secondary().opacity(0.8))
                        .foreground_color(colors::secondary_foreground())
                        .border_radius(theme::corners::button_radius())
                        .width(140)
                        .height(36),
                )
                .child(
                    button!("Save")
                        .start(icon(Icon::lucide().save(), 16.0, colors::destructive_foreground()))
                        .end(icon(Icon::lucide().check(), 16.0, colors::destructive_foreground()))
                        .background_color(colors::destructive())
                        .hover_background_color(colors::destructive().opacity(0.8))
                        .foreground_color(colors::destructive_foreground())
                        .border_radius(theme::corners::button_radius())
                        .width(140)
                        .height(36),
                )
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"// Start icon
button!("Send")
    .start(icon_widget)
    .background_color(colors::primary())
    .foreground_color(colors::primary_foreground())

// End icon
button!("Next")
    .end(arrow_icon)

// Both start and end icons
button!("Save")
    .start(save_icon)
    .end(check_icon)"#
        ).font_size(13.0))
}
