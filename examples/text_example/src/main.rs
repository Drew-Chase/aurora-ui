#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::App;
use aurora_ui::font_manager::FontManager;

fn main() {
    let mut font_manager = FontManager::new();
    font_manager.load_from_bytes(include_bytes!("../Roboto-Regular.ttf"));

    App::new()
        .title("Text Example")
        .min_size((310, 440))
        .run(|_window, _frame_info| {})
        .expect("Failed to run app");
}
