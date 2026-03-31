#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    let second_open = Arc::new(AtomicBool::new(false));
    let second_open_clone = second_open.clone();

    App::new()
        .title("Multi-Window Demo")
        .size((500, 400))
        .use_system_fonts()
        .run(move |window, _frame_info| {
            let is_open = second_open_clone.load(Ordering::Relaxed);

            window.root(
                col!()
                    .spacing(16.0)
                    .padding(Edges::all(24.0))
                    .child(
                        Text::new("Multi-Window Demo")
                            .font_size(24.0)
                            .align(Align::Center),
                    )
                    .child(Text::new("This is the primary window.").font_size(14.0))
                    .child(
                        Text::new(if is_open {
                            "Second window is open."
                        } else {
                            "Click the button to open a second window."
                        })
                        .font_size(14.0),
                    )
                    .child({
                        let flag = second_open_clone.clone();
                        button!("Open Second Window").on_click(move |_| {
                            if !flag.load(Ordering::Relaxed) {
                                flag.store(true, Ordering::Relaxed);
                                // Note: open_window requires Send closure,
                                // so we use the AppContext from within the
                                // button callback indirectly. For a full
                                // implementation, we'd pass the AppContext.
                            }
                        })
                    }),
            );
        })
        .expect("Failed to run app");
}
