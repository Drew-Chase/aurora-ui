use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_typography() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Typography",
            "Styles for headings, paragraphs, and other text elements.",
        ))
        .child(crate::example_section(
            "Headings",
            "Heading levels from h1 to h4.",
        ))
        .child(crate::example_card(
            col!()
                .spacing(12.0)
                .child(typography::Typography::h1("Heading 1"))
                .child(typography::Typography::h2("Heading 2"))
                .child(typography::Typography::h3("Heading 3"))
                .child(typography::Typography::h4("Heading 4")),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"Typography::h1("Heading 1")
Typography::h2("Heading 2")
Typography::h3("Heading 3")
Typography::h4("Heading 4")"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Body text",
            "Paragraph, lead, small, and muted text.",
        ))
        .child(crate::example_card(
            col!()
                .spacing(12.0)
                .child(typography::Typography::paragraph(
                    "This is a paragraph of body text. It uses the default font size and color.",
                ))
                .child(typography::Typography::lead(
                    "This is lead text — slightly larger and lighter.",
                ))
                .child(typography::Typography::small("This is small text."))
                .child(typography::Typography::muted("This is muted text.")),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"Typography::paragraph("This is a paragraph of body text.")
Typography::lead("This is lead text — slightly larger and lighter.")
Typography::small("This is small text.")
Typography::muted("This is muted text.")"#,
                )
                .font_size(13.0),
        )
}
