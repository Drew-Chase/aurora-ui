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
        .run(|_window, _frame_info| {
        })
        .expect("Failed to run app");
}
