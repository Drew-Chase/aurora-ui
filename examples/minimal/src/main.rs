#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use aurora_ui::app::App;
use aurora_ui::box_widget::BoxWidget;
use aurora_ui::layout::column::Column;

fn main() {
    App::new()
        .title("Minimal Example")
        .run(|window, _frame_info| {
            window.root(
                Column::new()
                    .spacing(10.0)
                    .padding(aurora_ui::geometry::edges::Edges::all(10.0))
                    .justify(aurora_ui::layout::Justify::Center)
                    .align(aurora_ui::layout::Align::Center)
                    .child(
                        BoxWidget::new()
                            .height(100)
                            .background_color(aurora_ui::color::Color::BLUE),
                    )
                    .child(
                        BoxWidget::new()
                            .height(100)
                            .background_color(aurora_ui::color::Color::RED),
                    )
                    .child(
                        BoxWidget::new()
                            .height(100)
                            .background_color(aurora_ui::color::Color::GREEN),
                    ),
            )
        })
        .expect("Failed to run app");
}
