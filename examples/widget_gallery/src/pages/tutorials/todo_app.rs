use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;
use crate::theme;

pub fn page_todo_app() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Todo App",
            "Build a todo list with add, toggle, and remove — learning dynamic state with Vec.",
        ))
        .child(crate::example_section(
            "Live demo",
            "A working todo list — type a task and press Add.",
        ))
        .child(crate::example_card(todo_demo()))
        .child(crate::example_section(
            "Step 1: Define the state",
            "A todo app needs a list of items and a text buffer for the input field.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"#[derive(Clone)]
struct TodoItem {
    text: String,
    done: bool,
}

#[derive(Clone)]
struct TodoState {
    items: Vec<TodoItem>,
    input: String,
}"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Step 2: Build the input row",
            "A text input paired with an Add button. On click, push a new item into the Vec.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"let add = setter.clone();
let input_text = state.input.clone();

row!()
    .spacing(8.0)
    .align(Align::Center)
    .child(
        TextInput::new()
            .value(&state.input)
            .placeholder("What needs to be done?")
            .on_change(move |text| {
                let text = text.to_string();
                input_set.set(move |s| s.input = text.clone());
            })
    )
    .child(
        button!("Add")
            .on_click(move |_| {
                let text = input_text.clone();
                add.set(move |s| {
                    if !text.is_empty() {
                        s.items.push(TodoItem {
                            text: text.clone(),
                            done: false,
                        });
                        s.input.clear();
                    }
                });
            })
    )"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Step 3: Render the list",
            "Loop over items to build checkboxes. Each toggle or delete mutates the Vec by index.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
r#"let mut list = col!().spacing(4.0);

for (i, item) in state.items.iter().enumerate() {
    let toggle = setter.clone();
    let remove = setter.clone();

    list = list.child(
        row!()
            .spacing(8.0)
            .align(Align::Center)
            .child(
                Checkbox::new()
                    .checked(item.done)
                    .label(&item.text)
                    .on_toggle(move |checked| {
                        toggle.set(move |s| s.items[i].done = checked);
                    })
            )
            .child(
                button!("×")
                    .on_click(move |_| {
                        remove.set(move |s| { s.items.remove(i); });
                    })
            )
    );
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
                    Text::new("• Use a struct with Vec<T> for list-based state")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                )
                .child(
                    Text::new("• Capture the loop index to mutate specific items in set()")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                )
                .child(
                    Text::new("• Clone setter once per handler — closures need owned copies")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                )
                .child(
                    Text::new("• The entire list rebuilds on any change — fine for small lists")
                        .font_size(14.0)
                        .color(theme::colors::foreground()),
                ),
        ))
}

#[derive(Clone)]
struct TodoItem {
    text: String,
    done: bool,
}

#[derive(Clone)]
struct TodoState {
    items: Vec<TodoItem>,
    input: String,
}

fn todo_demo() -> impl Widget {
    Composite::new(
        TodoState {
            items: vec![
                TodoItem { text: "Learn Aurora UI".into(), done: true },
                TodoItem { text: "Build a todo app".into(), done: false },
                TodoItem { text: "Ship it".into(), done: false },
            ],
            input: String::new(),
        },
        |state, setter| {
            let add_setter = setter.clone();
            let input_setter = setter.clone();
            let input_text = state.input.clone();

            let mut list = col!().spacing(4.0);
            for (i, item) in state.items.iter().enumerate() {
                let toggle = setter.clone();
                let remove = setter.clone();

                list = list.child(
                    row!()
                        .spacing(8.0)
                        .align(Align::Center)
                        .child(
                            checkbox::Checkbox::new()
                                .checked(item.done)
                                .label(&item.text)
                                .on_change(move |checked| {
                                    toggle.set(move |s| s.items[i].done = checked);
                                }),
                        )
                        .child(
                            button!("×")
                                .background_color(colors::destructive())
                                .hover_background_color(colors::destructive().opacity(0.8))
                                .foreground_color(colors::destructive_foreground())
                                .border_radius(theme::corners::button_radius())
                                .width(32)
                                .height(28)
                                .on_click(move |_| {
                                    remove.set(move |s| {
                                        s.items.remove(i);
                                    });
                                }),
                        ),
                );
            }

            Box::new(
                col!()
                    .spacing(12.0)
                    .child(
                        row!()
                            .spacing(8.0)
                            .align(Align::Center)
                            .child(
                                TextInput::new()
                                    .text(&state.input)
                                    .placeholder("What needs to be done?")
                                    .width(260.0)
                                    .height(36.0)
                                    .on_change(move |text| {
                                        let text = text.to_string();
                                        input_setter.set(move |s| s.input = text.clone());
                                    }),
                            )
                            .child(
                                button!("Add")
                                    .background_color(colors::primary())
                                    .hover_background_color(colors::primary().opacity(0.8))
                                    .foreground_color(colors::primary_foreground())
                                    .border_radius(theme::corners::button_radius())
                                    .width(60)
                                    .height(36)
                                    .on_click(move |_| {
                                        let text = input_text.clone();
                                        add_setter.set(move |s| {
                                            if !text.is_empty() {
                                                s.items.push(TodoItem {
                                                    text: text.clone(),
                                                    done: false,
                                                });
                                                s.input.clear();
                                            }
                                        });
                                    }),
                            ),
                    )
                    .child(list),
            )
        },
    )
}
