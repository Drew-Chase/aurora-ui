use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_empty() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Empty State", "Placeholder shown when there is no content to display."))
        .child(crate::example_section("Default", "A centered empty state message."))
        .child(crate::example_card(
            empty::Empty::new().title("No results found").description("Try adjusting your search or filter criteria.").height(120.0)
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"Empty::new()
    .title("No results found")
    .description("Try adjusting your search or filter criteria.")
    .height(120.0)"#
        ).font_size(13.0))
}
