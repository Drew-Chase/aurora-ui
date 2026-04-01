use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_segmented_control() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Segmented Control",
            "An iOS-style segmented toggle bar with a sliding pill indicator.",
        ))
        .child(crate::example_section(
            "Default",
            "Click a segment to switch the active selection.",
        ))
        .child(crate::example_card(
            segmented_control::SegmentedControl::new()
                .option("Day")
                .option("Week")
                .option("Month")
                .option("Year")
                .width(400.0),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"SegmentedControl::new()
    .option("Day")
    .option("Week")
    .option("Month")
    .option("Year")
    .width(400.0)"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Pre-selected",
            "Use .selected() to set the initial selection.",
        ))
        .child(crate::example_card(
            segmented_control::SegmentedControl::new()
                .option("Small")
                .option("Medium")
                .option("Large")
                .selected(1)
                .width(300.0),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"SegmentedControl::new()
    .option("Small")
    .option("Medium")
    .option("Large")
    .selected(1)
    .width(300.0)"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Custom Colors",
            "Customize background, indicator, and text colors.",
        ))
        .child(crate::example_card(
            segmented_control::SegmentedControl::new()
                .option("Light")
                .option("Dark")
                .option("System")
                .width(350.0)
                .background(colors::secondary())
                .indicator_color(colors::primary())
                .selected_text_color(colors::primary_foreground()),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"SegmentedControl::new()
    .option("Light")
    .option("Dark")
    .option("System")
    .width(350.0)
    .background(colors::secondary())
    .indicator_color(colors::primary())
    .selected_text_color(colors::primary_foreground())"#,
                )
                .font_size(13.0),
        )
}
