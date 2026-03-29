use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_tooltip() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Tooltip",
            "A small popup that appears on hover to provide additional context.",
        ))
        .child(crate::example_section(
            "Default",
            "Hover over the button to see the tooltip.",
        ))
        .child(crate::example_card(
            row!()
                .spacing(16.0)
                .align(Align::Center)
                .child(tooltip::Tooltip::new("This saves your work").child(button!("Save")))
                .child(
                    tooltip::Tooltip::new("Delete this item")
                        .side(tooltip::TooltipSide::Bottom)
                        .child(button!("Delete")),
                )
                .child(
                    tooltip::Tooltip::new("Show more info")
                        .side(tooltip::TooltipSide::Right)
                        .child(button!("Info")),
                ),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"Tooltip::new("This saves your work")
    .child(button!("Save"))

Tooltip::new("Delete this item")
    .side(TooltipSide::Bottom)
    .child(button!("Delete"))"#,
                )
                .font_size(13.0),
        )
}
