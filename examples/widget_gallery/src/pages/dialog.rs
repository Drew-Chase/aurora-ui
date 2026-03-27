use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_dialog(setter: StateSetter<crate::GalleryState>) -> impl Widget {
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
                .on_click(move |_| setter.set(|s| s.dialog_open = true)),
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
}
