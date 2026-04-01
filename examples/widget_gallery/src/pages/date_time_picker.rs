use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_date_time_picker() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Date Time Picker",
            "A combined date and time selection input with a calendar dropdown and time spinners.",
        ))
        .child(crate::example_section(
            "Default",
            "Click to open the dropdown. Select a date from the calendar and adjust the time with spinners.",
        ))
        .child(crate::example_card(
            col!()
                .spacing(16.0)
                .child(
                    date_time_picker::DateTimePicker::new()
                        .year(2026)
                        .month(4)
                        .day(1)
                        .hour(14)
                        .minute(30)
                        .width(260.0),
                )
                .child(
                    date_time_picker::DateTimePicker::new()
                        .year(2026)
                        .month(4)
                        .day(1)
                        .hour(9)
                        .minute(15)
                        .second(30)
                        .show_seconds(true)
                        .width(280.0),
                ),
        ))
        .child(crate::example_section(
            "12-Hour Format",
            "Use format_24h(false) to display time in 12-hour AM/PM format.",
        ))
        .child(crate::example_card(
            date_time_picker::DateTimePicker::new()
                .year(2026)
                .month(4)
                .day(1)
                .hour(14)
                .minute(30)
                .format_24h(false)
                .width(280.0),
        ))
        .child(crate::example_section(
            "With Callback",
            "Fire an event when the date or time changes.",
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"DateTimePicker::new()
    .year(2026)
    .month(4)
    .day(1)
    .hour(14)
    .minute(30)
    .on_change(|year, month, day, hour, minute, second| {
        println!("{year}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}");
    })"#,
                )
                .font_size(13.0),
        )
}
