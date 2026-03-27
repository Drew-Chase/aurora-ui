use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_item() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Item", "A generic list item with optional icon and trailing content."))
        .child(crate::example_section("Default", "Items can hold leading icons, text, and trailing elements."))
        .child(crate::example_card(
            col!()
                .spacing(4.0)
                .child(item::Item::new("Alice Johnson").description("Software Engineer"))
                .child(item::Item::new("Bob Smith").description("Product Designer"))
                .child(item::Item::new("Carol White").description("Engineering Manager"))
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"Item::new("Alice Johnson")
    .description("Software Engineer")
    .trailing(Badge::new("Admin"))"#
        ).font_size(13.0))
}
