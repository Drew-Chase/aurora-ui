#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod second_window;

use aurora_ui::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    let second_open = Arc::new(AtomicBool::new(false));

    App::new()
        .title("Multi-Window Demo")
        .size((500, 400))
        .use_system_fonts()
        .run(move |window, _frame_info| {
            let ctx = window.app_context().clone();
            let flag = second_open.clone();
            let is_open = flag.load(Ordering::Relaxed);

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
                        let flag = flag.clone();
                        let ctx = ctx.clone();
                        button!("Open Second Window").on_click(move |_| {
                            if !flag.load(Ordering::Relaxed) {
                                flag.store(true, Ordering::Relaxed);
                                second_window::open(&ctx, &flag);
                            }
                        })
                    }),
            );
        })
        .expect("Failed to run app");
}
