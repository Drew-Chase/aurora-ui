#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::prelude::*;
fn main() {
    App::new()
        .title("Layout Example")
        .min_size((310, 440))
        .run(|window, _frame_info| {
            window.root(
                column!()
                    .spacing(10.0)
                    .padding(Edges::all(10.0))
                    .justify(Justify::Center)
                    .align(Align::Center)
                    .child(
                        row!()
                            .spacing(10.0)
                            .height(100)
                            .child(BoxWidget::new().background_color(Color::BLUE))
                            .child(BoxWidget::new().background_color(Color::RED))
                            .child(BoxWidget::new().background_color(Color::GREEN)),
                    )
                    .child(BoxWidget::new().height(100).background_color(Color::BLUE))
                    .child(BoxWidget::new().background_color(Color::RED))
                    .child(BoxWidget::new().height(100).background_color(Color::GREEN)),
            )
        })
        .expect("Failed to run app");
}
