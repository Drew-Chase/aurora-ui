use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_textarea() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Textarea",
            "A multi-line text input for longer content.",
        ))
        .child(crate::example_section(
            "Default",
            "A resizable text area with placeholder.",
        ))
        .child(crate::example_card(
            col!()
                .spacing(16.0)
                .child(
                    textarea::TextArea::new()
                        .placeholder("Type your message here...")
                        .rows(4)
                        .width(400.0),
                )
                .child(
                    textarea::TextArea::new()
                        .placeholder("Disabled textarea")
                        .rows(3)
                        .width(400.0)
                        .disabled(true),
                ),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"TextArea::new()
    .placeholder("Type your message here...")
    .rows(4)
    .width(400.0)
    .on_change(|text| println!("{text}"))"#,
                )
                .font_size(13.0),
        )
}
