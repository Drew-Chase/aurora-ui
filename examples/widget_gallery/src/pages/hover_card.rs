use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_hover_card() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Hover Card",
            "A card that appears when hovering over a trigger element.",
        ))
        .child(crate::example_section(
            "Default",
            "Hover over the text to reveal the card.",
        ))
        .child(crate::example_card(
            hover_card::HoverCard::new()
                .trigger(Text::new("@aurora_ui").color(crate::theme::colors::primary()))
                .content(
                    col!()
                        .spacing(8.0)
                        .child(
                            Text::new("Aurora UI")
                                .font_size(16.0)
                                .font_weight(FontWeight::SemiBold),
                        )
                        .child(
                            Text::new("A GPU-accelerated UI framework for Rust.")
                                .font_size(13.0)
                                .color(crate::theme::colors::muted_foreground()),
                        ),
                ),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"HoverCard::new()
    .trigger(Text::new("@aurora_ui"))
    .content(
        col!()
            .child(Text::new("Aurora UI"))
            .child(Text::new("Description text"))
    )"#,
                )
                .font_size(13.0),
        )
}
