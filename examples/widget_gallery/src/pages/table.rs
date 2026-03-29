use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_table() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Table",
            "A responsive table component for displaying tabular data.",
        ))
        .child(crate::example_section("Default", "A basic data table."))
        .child(crate::example_card(
            table::Table::new()
                .headers(vec!["Name", "Email", "Role"])
                .row(vec!["Alice Chen", "alice@example.com", "Admin"])
                .row(vec!["Bob Smith", "bob@example.com", "User"])
                .row(vec!["Carol White", "carol@example.com", "Editor"])
                .row(vec!["Dave Jones", "dave@example.com", "User"])
                .width(500.0),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"Table::new()
    .headers(vec!["Name", "Email", "Role"])
    .row(vec!["Alice Chen", "alice@example.com", "Admin"])
    .row(vec!["Bob Smith", "bob@example.com", "User"])
    .row(vec!["Carol White", "carol@example.com", "Editor"])
    .width(500.0)"#,
                )
                .font_size(13.0),
        )
}
