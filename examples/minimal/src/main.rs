#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use aurora_ui::app::App;

fn main() {
    App::new()
        .title("Minimal Example")
        .run(|_window, _frame_info| {
        })
        .expect("Failed to run app");
}
