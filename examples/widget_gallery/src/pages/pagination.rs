use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_pagination() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Pagination",
            "Navigation between pages of content.",
        ))
        .child(crate::example_section(
            "Default",
            "A basic pagination control.",
        ))
        .child(crate::example_card(
            pagination::Pagination::new()
                .total_pages(10)
                .current_page(3),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"Pagination::new()
    .total_pages(10)
    .current_page(3)"#,
                )
                .font_size(13.0),
        )
}
