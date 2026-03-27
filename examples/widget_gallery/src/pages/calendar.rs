use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_calendar() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Calendar", "A month-view calendar for date selection."))
        .child(crate::example_section("Default", "Click a day to select it. Use arrows to navigate months."))
        .child(crate::example_card(
            calendar::Calendar::new()
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"Calendar::new()
    .year(2026)
    .month(3)
    .selected_day(15)
    .on_select(|day| println!("Selected day: {day}"))"#
        ).font_size(13.0))
}
