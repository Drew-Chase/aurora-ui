use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_date_range_picker() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Date Range Picker",
            "A date range input with dual side-by-side calendars for selecting start and end dates.",
        ))
        .child(crate::example_section(
            "Default",
            "Click to open the picker. Select a start date, then an end date.",
        ))
        .child(crate::example_card(
            col!()
                .spacing(16.0)
                .child(
                    date_range_picker::DateRangePicker::new()
                        .width(340.0),
                )
                .child(
                    date_range_picker::DateRangePicker::new()
                        .start_date(2026, 4, 5)
                        .end_date(2026, 4, 15)
                        .left_month(2026, 4)
                        .width(340.0),
                ),
        ))
        .child(crate::example_section(
            "With Callback",
            "Fire an event when the range is selected.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"DateRangePicker::new()
    .left_month(2026, 4)
    .width(340.0)
    .on_change(|start, end| {
        println!("Range: {}-{}-{} to {}-{}-{}",
            start.0, start.1, start.2,
            end.0, end.1, end.2);
    })"#,
                )
                .font_size(13.0),
        )
}
