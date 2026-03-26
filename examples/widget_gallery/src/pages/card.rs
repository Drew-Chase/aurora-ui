use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_card() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Card", "Displays a card with header, content, and footer."))
        .child(crate::example_section("Default", "A card with title and description."))
        .child(crate::example_card(
            card::Card::new()
                .width(400.0)
                .child(typography::Typography::h4("Card Title"))
                .child(typography::Typography::muted("This is a card component. Cards group related content and actions about a single subject."))
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"Card::new()
    .width(400.0)
    .child(Typography::h4("Card Title"))
    .child(Typography::muted("Cards group related content and actions about a single subject."))"#
        ).font_size(13.0))
}
