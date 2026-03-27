use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_dialog() -> impl Widget {
    Composite::new(false, move |&open, set_state| {
        let open_setter = set_state.clone();
        let cancel_setter = set_state.clone();
        let continue_setter = set_state.clone();
        let close_setter = set_state.clone();

        Box::new(
            col!()
                .spacing(24.0)
                .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
                .child(crate::page_header(
                    "Dialog",
                    "A modal dialog with backdrop overlay.",
                ))
                .child(crate::example_section(
                    "Default",
                    "Click the button to open a centered dialog with title, content, and footer actions.",
                ))
                .child(crate::example_card(
                    button!("Open Dialog")
                        .on_click(move |_| open_setter.set(|s| *s = true)),
                ))
                .child(
                    code_block::CodeBlock::new()
                        .language("rust")
                        .code(
                            r#"Dialog::new()
    .open(show_dialog)
    .title("Are you sure?")
    .content(Text::new("This action cannot be undone."))
    .on_close(|| set_open(false))
    .footer(
        row!()
            .spacing(8.0)
            .child(button!("Cancel").on_click(|_| set_open(false)))
            .child(button!("Continue").on_click(|_| set_open(false)))
    )"#,
                        )
                        .font_size(13.0),
                )
                .child(
                    dialog::Dialog::new()
                        .open(open)
                        .title("Are you sure?")
                        .content(Text::new(
                            "This action cannot be undone. This will permanently delete your account.",
                        ))
                        .on_close(move || {
                            close_setter.set(|s| *s = false);
                        })
                        .footer(
                            row!()
                                .spacing(8.0)
                                .justify(Justify::End)
                                .child(
                                    button!("Cancel")
                                        .on_click(move |_| cancel_setter.set(|s| *s = false)),
                                )
                                .child(
                                    button!("Continue")
                                        .on_click(move |_| continue_setter.set(|s| *s = false)),
                                ),
                        ),
                ),
        )
    })
}
