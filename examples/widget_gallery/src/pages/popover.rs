use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_popover() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Popover",
            "A floating content panel anchored to a trigger widget.",
        ))
        .child(crate::example_section(
            "Default",
            "Click the button to open the popover.",
        ))
        .child(crate::example_card(
            popover::Popover::new()
                .trigger(button!("Open Popover"))
                .content(
                    col!()
                        .spacing(8.0)
                        .child(
                            Text::new("Popover Content")
                                .font_size(14.0)
                                .font_weight(FontWeight::SemiBold),
                        )
                        .child(
                            Text::new("This is the popover body.")
                                .font_size(13.0)
                                .color(crate::theme::colors::muted_foreground()),
                        ),
                ),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"Popover::new()
    .trigger(button!("Open Popover"))
    .content(
        col!()
            .child(Text::new("Popover Content"))
            .child(Text::new("Body text"))
    )
    .side(PopoverSide::Bottom)"#,
                )
                .font_size(13.0),
        )
}
