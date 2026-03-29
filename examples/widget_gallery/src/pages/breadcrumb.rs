use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_breadcrumb() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Breadcrumb",
            "Displays a breadcrumb navigation trail.",
        ))
        .child(crate::example_section(
            "Default",
            "A simple breadcrumb trail.",
        ))
        .child(crate::example_card(
            breadcrumb::Breadcrumb::new()
                .item("Home")
                .item("Components")
                .item("Breadcrumb"),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"Breadcrumb::new()
    .item("Home")
    .item("Components")
    .item("Breadcrumb")"#,
                )
                .font_size(13.0),
        )
}
