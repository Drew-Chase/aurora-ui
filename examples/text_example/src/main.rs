#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::aurora_core::geometry::edges::Edges;
use aurora_ui::aurora_platform::app::App;
use aurora_ui::aurora_widgets::column;
use aurora_ui::aurora_widgets::text_widget::Text;

fn main() {
    App::new()
        .title("Text Example")
        .min_size((310, 440))
        .font(include_bytes!("../Roboto-Regular.ttf"))
        .run(|window, _frame_info| {
            window.root(
                column!()
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
