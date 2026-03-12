#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::App;

fn main() {

    App::new()
        .title("Text Example")
        .min_size((310, 440))
        .font(include_bytes!("../Roboto-Regular.ttf"))
        .run(|window, _frame_info| {
        })
        .expect("Failed to run app");
}
