#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::aurora_core::color::Color;
use aurora_ui::aurora_platform::app::App;
use aurora_ui::aurora_text;

fn main() {

    App::new()
        .title("Text Example")
        .min_size((310, 440))
        .font(include_bytes!("../Roboto-Regular.ttf"))
        .run(|window, _frame_info| {
            let layout = aurora_text::text_layout::TextLayout::new(
                window.font_manager(),
                "Hello, Aurora!",
                72.0,
                Color::BLACK,
                None,
            );
            window.render_text(&layout, 50, 72+50);
        })
        .expect("Failed to run app");
}
