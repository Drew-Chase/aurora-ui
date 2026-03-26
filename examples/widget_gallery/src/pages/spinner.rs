use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_spinner() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(crate::page_header("Spinner", "A loading indicator that rotates continuously."))
        .child(crate::example_section("Sizes", "Spinners at different sizes."))
        .child(crate::example_card(
            row!()
                .spacing(24.0)
                .align(Align::Center)
                .child(spinner::Spinner::new())
                .child(spinner::Spinner::new().size(32.0).color(colors::info()))
                .child(spinner::Spinner::new().size(48.0).color(colors::success()).thickness(4.0))
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"// Default spinner
Spinner::new()

// Custom size and color
Spinner::new()
    .size(32.0)
    .color(colors::info())

// Large spinner with custom thickness
Spinner::new()
    .size(48.0)
    .color(colors::success())
    .thickness(4.0)"#
        ).font_size(13.0))
}
