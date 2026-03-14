#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use aurora_ui::prelude::*;

fn main() {
    App::new()
        .title("Text Example")
        .min_size((310, 440))
        .font(include_bytes!("../../Roboto-Regular.ttf"))
        .run(|window, _frame_info| {
            window.root(
                col!()
                    .spacing(10.0)
                    .child(Text::new("Item 1").padding(Edges::symmetric(20.0, 20.0)))
                    .child(Text::new("Item 2").padding(Edges::symmetric(20.0, 0.0)))
                    .child(Text::new("Item 3"))
                    .child(Text::new("Item 4"))
                    .child(Text::new("Item 5").padding(Edges::symmetric(0.0, 20.0))),
            );
        })
        .expect("Failed to run app");
}
