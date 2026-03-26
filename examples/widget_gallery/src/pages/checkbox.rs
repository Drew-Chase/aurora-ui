use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_checkbox() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(crate::page_header("Checkbox", "A control that allows the user to toggle between checked and not checked."))
        .child(crate::example_section("Basic", "A simple checkbox with a label."))
        .child(crate::example_card(
            col!()
                .spacing(12.0)
                .child(checkbox::Checkbox::new().checked(true).label("Accept terms and conditions"))
                .child(checkbox::Checkbox::new().label("Subscribe to newsletter"))
                .child(checkbox::Checkbox::new().disabled(true).label("Disabled option"))
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"// Checked checkbox
Checkbox::new()
    .checked(true)
    .label("Accept terms and conditions")

// Unchecked checkbox
Checkbox::new()
    .label("Subscribe to newsletter")

// Disabled checkbox
Checkbox::new()
    .disabled(true)
    .label("Disabled option")"#
        ).font_size(13.0))
}
