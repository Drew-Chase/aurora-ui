use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_time_picker() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Time Picker",
            "A time input with hour/minute spinners and optional seconds.",
        ))
        .child(crate::example_section(
            "24-Hour Format",
            "Default time picker with 24-hour display.",
        ))
        .child(crate::example_card(
            row!()
                .spacing(16.0)
                .child(time_picker::TimePicker::new().hour(14).minute(30))
                .child(
                    time_picker::TimePicker::new()
                        .hour(9)
                        .minute(15)
                        .show_seconds(true)
                        .second(45),
                ),
        ))
        .child(crate::example_section(
            "12-Hour Format",
            "Time picker with AM/PM toggle.",
        ))
        .child(crate::example_card(
            row!()
                .spacing(16.0)
                .child(
                    time_picker::TimePicker::new()
                        .format_24h(false)
                        .hour(9)
                        .minute(30),
                )
                .child(
                    time_picker::TimePicker::new()
                        .format_24h(false)
                        .show_seconds(true)
                        .hour(14)
                        .minute(0)
                        .second(0),
                ),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"TimePicker::new()
    .hour(14)
    .minute(30)
    .format_24h(false)
    .show_seconds(true)
    .on_change(|hour, minute, second| {
        println!("Time: {hour:02}:{minute:02}:{second:02}");
    })"#,
                )
                .font_size(13.0),
        )
}
