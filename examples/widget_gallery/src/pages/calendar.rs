use crate::theme;
use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_calendar() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Calendar",
            "A month-view calendar for date selection with hover, indicators, and range support.",
        ))
        // ── Default ─────────────────────────────────────────────────
        .child(crate::example_section(
            "Default",
            "Hover highlights days. Click to select. Arrows navigate months.",
        ))
        .child(crate::example_card(
            calendar::Calendar::new()
                .year(2026)
                .month(3)
                .today(30)
                .selected_day(15),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"Calendar::new()
    .year(2026)
    .month(3)
    .today(30)
    .selected_day(15)
    .on_select(|day| println!("Selected: {day}"))"#,
                )
                .font_size(13.0),
        )
        // ── Month/Year Selector ─────────────────────────────────────
        .child(crate::example_section(
            "Month/Year Selector",
            "Click the month name or year to open a picker overlay.",
        ))
        .child(crate::example_card(
            calendar::Calendar::new()
                .year(2026)
                .month(3)
                .month_year_selector(calendar::MonthYearSelector::Combined)
                .selector_bg(theme::colors::muted()),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"Calendar::new()
    .month_year_selector(MonthYearSelector::Combined)

// Or use Separate to pick month and year independently:
Calendar::new()
    .month_year_selector(MonthYearSelector::Separate)"#,
                )
                .font_size(13.0),
        )
        // ── Disabled Days & Date Range ──────────────────────────────
        .child(crate::example_section(
            "Disabled Days & Date Bounds",
            "Greyed-out days are not clickable. Min/max dates constrain navigation.",
        ))
        .child(crate::example_card(
            calendar::Calendar::new()
                .year(2026)
                .month(3)
                .disabled_days(vec![1, 7, 8, 14, 15, 21, 22, 28, 29])
                .min_date(2026, 3, 1)
                .max_date(2026, 3, 31),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"Calendar::new()
    .disabled_days(vec![1, 7, 8, 14, 15, 21, 22, 28, 29])
    .min_date(2026, 3, 1)
    .max_date(2026, 3, 31)"#,
                )
                .font_size(13.0),
        )
        // ── Indicators ──────────────────────────────────────────────
        .child(crate::example_section(
            "Cell Indicators",
            "Mark days with colored dots or rings. Hover to see tooltips.",
        ))
        .child(crate::example_card(
            calendar::Calendar::new()
                .year(2026)
                .month(3)
                .indicator(calendar::CellIndicator {
                    day: 5,
                    color: theme::colors::success(),
                    style: calendar::IndicatorStyle::Dot,
                    tooltip: Some("Team standup".into()),
                })
                .indicator(calendar::CellIndicator {
                    day: 12,
                    color: theme::colors::destructive(),
                    style: calendar::IndicatorStyle::Ring,
                    tooltip: Some("Deadline".into()),
                })
                .indicator(calendar::CellIndicator {
                    day: 20,
                    color: theme::colors::info(),
                    style: calendar::IndicatorStyle::Dot,
                    tooltip: Some("Release day".into()),
                })
                .indicator(calendar::CellIndicator {
                    day: 25,
                    color: theme::colors::warning(),
                    style: calendar::IndicatorStyle::Ring,
                    tooltip: None,
                }),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"Calendar::new()
    .indicator(CellIndicator {
        day: 5,
        color: colors::success(),
        style: IndicatorStyle::Dot,
        tooltip: Some("Team standup".into()),
    })
    .indicator(CellIndicator {
        day: 12,
        color: colors::destructive(),
        style: IndicatorStyle::Ring,
        tooltip: Some("Deadline".into()),
    })"#,
                )
                .font_size(13.0),
        )
        // ── Range Selection ─────────────────────────────────────────
        .child(crate::example_section(
            "Range Selection",
            "Click a start date, then click an end date to select a range.",
        ))
        .child(crate::example_card(
            calendar_range::CalendarRange::new()
                .year(2026)
                .month(3)
                .today(30),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"CalendarRange::new()
    .year(2026)
    .month(3)
    .on_range_select(|start, end| {
        println!("Range: {start}–{end}")
    })"#,
                )
                .font_size(13.0),
        )
}
