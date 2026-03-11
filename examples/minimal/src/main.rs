#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use aurora_ui::app::App;
use aurora_ui::color::Color;
use aurora_ui::geometry::corners::Corners;
use aurora_ui::geometry::point::Point;
use aurora_ui::geometry::rect::Rect;
use aurora_ui::geometry::size::Size;

fn main() {
    App::new()
        .title("Minimal Example")
        .run(|window, _frame_info| {
            window.draw(|canvas| {
                canvas.fill_rounded_rect(
                    Rect::from_origin_size(Point::new(100.0, 100.0), Size::new(100.0, 100.0)),
                    Corners::new(20.0, 20.0, 20.0, 20.0),
                    Color::RED,
                );
            });
        })
        .expect("Failed to run app");
}
