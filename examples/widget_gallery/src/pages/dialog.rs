use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_dialog() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Dialog", "A modal dialog with backdrop overlay."))
        .child(crate::example_section("Default", "A centered dialog with title, content, and footer actions."))
        .child(crate::example_card(
            dialog::Dialog::new()
                .open(true)
                .title("Are you sure?")
                .content(Text::new("This action cannot be undone. This will permanently delete your account."))
                .footer(
                    row!()
                        .spacing(8.0)
                        .justify(Justify::End)
                        .child(button!("Cancel"))
                        .child(button!("Continue"))
                )
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"Dialog::new()
    .open(show_dialog)
    .title("Are you sure?")
    .content(Text::new("This action cannot be undone."))
    .footer(
        row!()
            .spacing(8.0)
            .child(button!("Cancel"))
            .child(button!("Continue"))
    )"#
        ).font_size(13.0))
}
