use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_toggle() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(crate::page_header("Toggle", "A two-state button that can be on or off."))
        .child(crate::example_section("Variants", "Default and outline toggle styles."))
        .child(crate::example_card(
            row!()
                .spacing(12.0)
                .align(Align::Center)
                .child(toggle::Toggle::new("Bold").pressed(true))
                .child(toggle::Toggle::new("Italic"))
                .child(toggle::Toggle::new("Outline").variant(toggle::ToggleVariant::Outline))
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"// Default toggle, pressed
Toggle::new("Bold").pressed(true)

// Default toggle, unpressed
Toggle::new("Italic")

// Outline variant
Toggle::new("Outline")
    .variant(ToggleVariant::Outline)"#
        ).font_size(13.0))
}
