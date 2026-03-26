use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;
use crate::theme;

pub fn page_counter() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Counter",
            "Build a simple counter app to learn state management with Composite and StateSetter.",
        ))
        .child(crate::example_section(
            "Live demo",
            "A working counter — click the buttons to increment and decrement.",
        ))
        .child(crate::example_card(counter_demo()))
        .child(crate::example_section(
            "Step 1: Define the state",
            "Start with a simple integer state. Composite wraps a widget tree that rebuilds when the state changes.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"// The state is just an i32 — start at 0
Composite::new(0i32, |&count, setter| {
    // This closure rebuilds whenever the state changes
    // `count` is a reference to the current state
    // `setter` lets us update it
    Box::new(/* widget tree */)
})"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Step 2: Build the UI",
            "Create a row with decrement button, count display, and increment button.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"Composite::new(0i32, |&count, setter| {
    let inc = setter.clone();
    let dec = setter.clone();

    Box::new(
        row!()
            .spacing(16.0)
            .align(Align::Center)
            .child(
                button!("-")
                    .width(40).height(36)
                    .on_click(move |_| {
                        dec.set(|c| *c -= 1);
                    })
            )
            .child(
                Text::new(&count.to_string())
                    .font_size(32.0)
                    .font_weight(FontWeight::Bold)
                    .width(80.0)
                    .align(Align::Center)
            )
            .child(
                button!("+")
                    .width(40).height(36)
                    .on_click(move |_| {
                        inc.set(|c| *c += 1);
                    })
            )
    )
})"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Step 3: Wire it into your app",
            "Pass the Composite as the root widget. The entire counter is self-contained.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"fn main() {
    let mut initialized = false;

    App::new()
        .title("Counter")
        .size((400, 300))
        .use_system_fonts()
        .run(move |window, _frame| {
            if !initialized {
                window.root(counter_widget());
                initialized = true;
            }
        })
        .expect("Failed to run app");
}

fn counter_widget() -> impl Widget {
    Composite::new(0i32, |&count, setter| {
        let inc = setter.clone();
        let dec = setter.clone();

        Box::new(
            col!()
                .align(Align::Center)
                .justify(Justify::Center)
                .child(
                    row!()
                        .spacing(16.0)
                        .align(Align::Center)
                        .child(
                            button!("-")
                                .width(40).height(36)
                                .on_click(move |_| {
                                    dec.set(|c| *c -= 1);
                                })
                        )
                        .child(
                            Text::new(&count.to_string())
                                .font_size(32.0)
                                .font_weight(FontWeight::Bold)
                                .width(80.0)
                                .align(Align::Center)
                        )
                        .child(
                            button!("+")
                                .width(40).height(36)
                                .on_click(move |_| {
                                    inc.set(|c| *c += 1);
                                })
                        )
                )
        )
    })
}"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Key concepts",
            "What you learned in this tutorial.",
        ))
        .child(crate::example_card(
            col!()
                .spacing(8.0)
                .child(
                    Text::new("• Composite holds state and rebuilds its widget tree on change")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                )
                .child(
                    Text::new("• StateSetter.set() takes a closure that mutates the state")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                )
                .child(
                    Text::new("• Clone the setter for each event handler that needs it")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                )
                .child(
                    Text::new("• The widget tree is fully rebuilt — keep build functions fast")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                ),
        ))
}

fn counter_demo() -> impl Widget {
    Composite::new(0i32, |&count, setter| {
        let inc = setter.clone();
        let dec = setter.clone();

        Box::new(
            row!()
                .spacing(16.0)
                .align(Align::Center)
                .justify(Justify::Center)
                .child(
                    button!("-")
                        .background_color(colors::secondary())
                        .hover_background_color(colors::secondary().opacity(0.8))
                        .foreground_color(colors::secondary_foreground())
                        .border_radius(theme::corners::button_radius())
                        .width(40)
                        .height(36)
                        .on_click(move |_| {
                            dec.set(|c| *c -= 1);
                        }),
                )
                .child(
                    Text::new(&count.to_string())
                        .font_size(32.0)
                        .font_weight(FontWeight::Bold)
                        .color(theme::colors::foreground())
                        .width(80.0)
                        .align(Align::Center),
                )
                .child(
                    button!("+")
                        .background_color(colors::secondary())
                        .hover_background_color(colors::secondary().opacity(0.8))
                        .foreground_color(colors::secondary_foreground())
                        .border_radius(theme::corners::button_radius())
                        .width(40)
                        .height(36)
                        .on_click(move |_| {
                            inc.set(|c| *c += 1);
                        }),
                ),
        )
    })
}
