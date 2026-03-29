use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_separator() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Separator",
            "Visually separates content.",
        ))
        .child(crate::example_section(
            "Default",
            "A horizontal rule to divide sections.",
        ))
        .child(crate::example_card(
            col!()
                .spacing(16.0)
                .child(typography::Typography::paragraph("Content above"))
                .child(separator::Separator::new())
                .child(typography::Typography::paragraph("Content below")),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"col!()
    .child(Typography::paragraph("Content above"))
    .child(Separator::new())
    .child(Typography::paragraph("Content below"))"#,
                )
                .font_size(13.0),
        )
}
