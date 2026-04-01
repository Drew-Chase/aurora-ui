use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_rich_text_editor() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Rich Text Editor",
            "A multi-line text editor with bold, italic, and underline formatting.",
        ))
        .child(crate::example_section(
            "Default",
            "An editor with a formatting toolbar. Use Ctrl+B, Ctrl+I, Ctrl+U for shortcuts.",
        ))
        .child(crate::example_card(
            col!()
                .spacing(16.0)
                .child(
                    rich_text_editor::RichTextEditor::new()
                        .placeholder("Write something beautiful...")
                        .width(500.0)
                        .height(200.0),
                )
                .child(
                    rich_text_editor::RichTextEditor::new()
                        .placeholder("No toolbar")
                        .toolbar(false)
                        .width(500.0)
                        .height(120.0),
                ),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"RichTextEditor::new()
    .placeholder("Write something beautiful...")
    .width(500.0)
    .height(200.0)
    .on_change(|text| println!("{text}"))"#,
                )
                .font_size(13.0),
        )
}
