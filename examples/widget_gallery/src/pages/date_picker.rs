use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_date_picker() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Date Picker",
            "A date input that opens a calendar dropdown.",
        ))
        .child(crate::example_section(
            "Default",
            "Click to open the calendar and select a date.",
        ))
        .child(crate::example_card(
            date_picker::DatePicker::new().placeholder("Pick a date"),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"DatePicker::new()
    .placeholder("Pick a date")
    .on_select(|year, month, day| {
        println!("Selected: {year}-{month}-{day}");
    })"#,
                )
                .font_size(13.0),
        )
}
