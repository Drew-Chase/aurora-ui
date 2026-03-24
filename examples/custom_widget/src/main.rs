#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod composite_widget;
pub mod full_widget;
use aurora_ui::prelude::*;

fn main() -> Result<(), AppError> {
    App::new()
        .title("Custom Widget Example")
        .size((400, 300))
        .position(WindowPosition::Center)
        .use_system_fonts()
        .font_options(FontOptions::new().family("Segoe UI"))
        .run(|window, _frame_info| {
            window.root(
                col!()
                    .spacing(30.0)
                    .align(Align::Center)
                    .justify(Justify::Center)
                    // Composite widget toggle
                    .child(
                        row!()
                            .spacing(12.0)
                            .height(28)
                            .align(Align::Center)
                            .child(
                                Text::new("Composite Widget:")
                                    .font_size(16.0)
                                    .height(28.0)
                                    .justify(Justify::Center),
                            )
                            .child(composite_widget::toggle_switch()),
                    )
                    // Full widget toggle
                    .child(
                        row!()
                            .spacing(12.0)
                            .height(28)
                            .align(Align::Center)
                            .child(
                                Text::new("Full Widget:")
                                    .font_size(16.0)
                                    .height(28.0)
                                    .justify(Justify::Center),
                            )
                            .child(full_widget::ToggleSwitch::new()),
                    ),
            );
        })
}
