use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_select() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Select",
            "Displays a list of options for the user to pick from.",
        ))
        .child(crate::example_section(
            "Default",
            "Click to open the dropdown.",
        ))
        .child(crate::example_card(
            select::Select::new()
                .placeholder("Select a fruit...")
                .option("Apple")
                .option("Banana")
                .option("Cherry")
                .option("Date")
                .option("Elderberry")
                .width(250.0),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"Select::new()
    .placeholder("Select a fruit...")
    .option("Apple")
    .option("Banana")
    .option("Cherry")
    .option("Date")
    .option("Elderberry")
    .width(250.0)"#,
                )
                .font_size(13.0),
        )
}
