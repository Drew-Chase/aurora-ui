#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::prelude::*;

fn main() {
    App::new()
        .title("Text Input Example")
        .size((400, 300))
        .position(WindowPosition::Center)
        .background_color(Color::from_rgb(240, 240, 240))
        .use_system_fonts()
        .font_options(FontOptions::new().family("Segoe UI").size(14.0))
        .run(|window, _frame| {
            window.root(
                col!()
                    .spacing(12.0)
                    .padding(Edges::all(20.0))
                    .align(Align::Stretch)
                    .child(
                        Text::new("Login")
                            .font_size(24.0)
                            .font_weight(FontWeight::Bold)
                            .height(36.0),
                    )
                    .child(
                        TextInput::new()
                            .placeholder("Username")
                            .height(34.0)
                            .corners(Corners::all(6.0))
                            .padding(Edges::symmetric(8.0, 10.0)),
                    )
                    .child(
                        TextInput::new()
                            .placeholder("Email address")
                            .height(34.0)
                            .corners(Corners::all(6.0))
                            .padding(Edges::symmetric(8.0, 10.0)),
                    )
                    .child(
                        TextInput::new()
                            .placeholder("Password")
                            .height(34.0)
                            .corners(Corners::all(6.0))
                            .padding(Edges::symmetric(8.0, 10.0)),
                    ),
            );
        })
        .expect("Failed to run app");
}
