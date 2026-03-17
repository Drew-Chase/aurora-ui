#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::prelude::*;
fn main() {
    App::new()
        .title("Layout Example")
        .min_size((310, 440))
        .position(WindowPosition::Center)
        .run(|window, _frame_info| {
            window.root(
                col!()
                    .spacing(10.0)
                    .padding(Edges::all(10.0))
                    .justify(Justify::Center)
                    .align(Align::Center)
                    .child(
                        row!()
                            .spacing(10.0)
                            .height(100)
                            .child(BoxWidget::new().background_color(Color::BLACK.opacity(0.5)))
                            .child(BoxWidget::new().background_color(Color::BLACK))
                            .child(BoxWidget::new().background_color(Color::BLACK.opacity(0.5))),
                    )
                    .child(BoxWidget::new().height(100).background_color(Color::BLUE))
                    .child(BoxWidget::new().background_color(Color::RED))
                    .child(BoxWidget::new().height(100).background_color(Color::GREEN))
                    .child(
                        Stack::new()
                            // Rendered at the bottom of the layout rendering stack
                            .child(
                                Positioned::fixed((20.0, 20.0)).child(
                                    BoxWidget::new()
                                        .width(50)
                                        .height(50)
                                        .background_color(hsl!(348, 0.81, 0.46))
                                        .corners(Corners::all(5.0)),
                                ),
                            )
                            .child(
                                Positioned::fixed((40.0, 40.0)).child(
                                    BoxWidget::new()
                                        .width(50)
                                        .height(50)
                                        .background_color(hsl!(148, 0.9, 0.66))
                                        .corners(Corners::all(5.0)),
                                ),
                            )
                            // Rendered at the top of the layout rendering stack
                            .child(
                                Positioned::fixed((30.0, 30.0)).child(
                                    BoxWidget::new()
                                        .width(50)
                                        .height(50)
                                        .background_color(hsl!(248, 0.9, 0.46))
                                        .corners(Corners::all(5.0)),
                                ),
                            ),
                    ),
            )
        })
        .expect("Failed to run app");
}
