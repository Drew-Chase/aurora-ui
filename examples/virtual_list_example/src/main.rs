#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::prelude::*;

fn main() {
    let scroll = ScrollState::new();

    App::new()
        .title("Virtual List Example")
        .size((500, 600))
        .use_system_fonts()
        .position(WindowPosition::Center)
        .resizable(false)
        .run({
            let mut initialized = false;
            move |window, _frame| {
                if initialized {
                    return;
                }
                initialized = true;

                let scroll = scroll.clone();
                window.root(
                    col!()
                        .padding(Edges::all(20.0))
                        .spacing(12.0)
                        .child(
                            Text::new("Virtual List — 1,000,000 items")
                                .font_size(20.0)
                                .font_weight(FontWeight::Bold),
                        )
                        .child(Text::new(
                            "Only visible items + a 5-item buffer are rendered each frame.",
                        ))
                        .child(
                            VirtualList::new(1_000_000, 36.0, |index| {
                                Box::new(
                                    row!()
                                        .padding(Edges::symmetric(0.0, 12.0))
                                        .child(
                                            Text::new(format!("Item {index}"))
                                                .font_size(14.0)
                                                .align(Align::Center),
                                        ),
                                )
                            })
                            .height(400.0)
                            .scroll_state(scroll)
                            .selected_bg(Color::new(0, 120, 215, 40))
                            .on_select(|idx| println!("Selected: {idx}")),
                        ),
                );
            }
        })
        .expect("Failed to run app");
}
