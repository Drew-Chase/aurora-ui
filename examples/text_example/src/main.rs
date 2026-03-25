#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use aurora_ui::prelude::*;

fn main() {
    App::new()
        .title("Text Example")
        .min_size((310, 578))
        .size((454, 578))
        .font(include_bytes!("../../Roboto-Regular.ttf"))
        .run(|window, _frame_info| {
            window.root(
                col!()
                    .spacing(10.0)
                    .padding(Edges::symmetric(20.0, 0.0))
                    .child(Text::new("Item 1").font_size(144.0))
                    .child(Text::new("Item 2").font_size(64.0))
                    .child(Text::new("Item 3").font_size(48.0))
                    .child(Text::new("Item 4").font_size(32.0))
                    .child(Text::new("Item 5").font_size(24.0))
                    .child(Text::new("Item 6").font_size(18.0))
                    .child(Text::new("Item 7").font_size(16.0))
                    .child(Text::new("Item 8").font_size(14.0))
                    .child(Text::new("Item 9").font_size(12.0))
                    .child(Text::new("Item 10").font_size(10.0))
                    .child(Text::new("Item 11").font_size(10.0))
            );
        })
        .expect("Failed to run app");
}
