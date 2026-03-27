use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_button_group() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Button Group", "A group of buttons displayed as a connected strip."))
        .child(crate::example_section("Default", "Buttons share borders and form a cohesive unit."))
        .child(crate::example_card(
            button_group::ButtonGroup::new()
                .button("Cut")
                .button("Copy")
                .button("Paste")
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"ButtonGroup::new()
    .button("Cut")
    .button("Copy")
    .button("Paste")"#
        ).font_size(13.0))
}
