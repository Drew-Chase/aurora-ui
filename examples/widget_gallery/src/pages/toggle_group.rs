use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_toggle_group() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Toggle Group",
            "A set of two-state buttons that can be toggled on or off.",
        ))
        .child(crate::example_section(
            "Default",
            "Select one option from the group.",
        ))
        .child(crate::example_card(
            toggle_group::ToggleGroup::new()
                .item("Left")
                .item("Center")
                .item("Right")
                .selected(1),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"ToggleGroup::new()
    .item("Left")
    .item("Center")
    .item("Right")
    .selected(1)"#,
                )
                .font_size(13.0),
        )
}
