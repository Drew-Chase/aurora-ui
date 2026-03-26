use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_collapsible() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Collapsible", "An interactive component that expands and collapses content."))
        .child(crate::example_section("Default", "Click the header to toggle content."))
        .child(crate::example_card(
            collapsible::Collapsible::new("Click to expand")
                .width(400.0)
                .child(typography::Typography::paragraph("This content is revealed when the collapsible is expanded. It smoothly animates open and closed."))
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"Collapsible::new("Click to expand")
    .width(400.0)
    .child(Typography::paragraph("This content is revealed when the collapsible is expanded."))"#
        ).font_size(13.0))
}
