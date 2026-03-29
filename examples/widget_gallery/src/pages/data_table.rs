use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_data_table() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Data Table",
            "An enhanced table with sorting and pagination.",
        ))
        .child(crate::example_section(
            "Default",
            "Click column headers to sort.",
        ))
        .child(crate::example_card(
            data_table::DataTable::new()
                .column("Name")
                .column("Email")
                .column("Role")
                .row(vec!["Alice", "alice@example.com", "Admin"])
                .row(vec!["Bob", "bob@example.com", "User"])
                .row(vec!["Carol", "carol@example.com", "Editor"]),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"DataTable::new()
    .column("Name")
    .column("Email")
    .column("Role")
    .row(vec!["Alice", "alice@example.com", "Admin"])
    .row(vec!["Bob", "bob@example.com", "User"])
    .row(vec!["Carol", "carol@example.com", "Editor"])"#,
                )
                .font_size(13.0),
        )
}
