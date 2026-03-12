#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use aurora_ui::app::App;

fn main() {
    App::new()
        .title("Text Example")
        .min_size((310, 440))
        .run(|_window, _frame_info| {})
        .expect("Failed to run app");
}
