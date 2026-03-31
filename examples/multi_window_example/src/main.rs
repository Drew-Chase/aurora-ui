#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
                                let flag_reset = flag.clone();
                                ctx.open_window(
                                    WindowConfig::new()
                                        .title("Second Window")
                                        .size((400, 300))
                                        .use_system_fonts(),
                                    move |w2, _frame| {
                                        let flag = flag_reset.clone();
                                        w2.root(
                                            col!()
                                                .spacing(12.0)
                                                .padding(Edges::all(20.0))
                                                .child(
                                                    Text::new("Second Window")
                                                        .font_size(20.0)
                                                        .align(Align::Center),
                                                )
                                                .child(
                                                    Text::new(
                                                        "This is an independent window with its own widget tree.",
                                                    )
                                                    .font_size(14.0),
                                                )
                                                .child({
                                                    let ctx = w2.app_context().clone();
                                                    let id = w2.id();
                                                    button!("Close This Window").on_click(
                                                        move |_| {
                                                            flag.store(false, Ordering::Relaxed);
                                                            ctx.close_window(id);
                                                        },
                                                    )
                                                }),
                                        );
                                    },
                                );
                            }
                        })
                    }),
            );
        })
        .expect("Failed to run app");
}
