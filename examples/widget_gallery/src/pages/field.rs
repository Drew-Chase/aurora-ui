use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_field() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Field", "A form field wrapper with label, input, and optional description."))
        .child(crate::example_section("Default", "Wraps any input with a label and helper text."))
        .child(crate::example_card(
            col!()
                .spacing(16.0)
                .child(field::Field::new("Username").input(TextInput::new().placeholder("Enter username")))
                .child(field::Field::new("Email").input(TextInput::new().placeholder("you@example.com")))
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"Field::new("Username")
    .input(TextInput::new().placeholder("Enter username"))

Field::new("Email")
    .input(TextInput::new().placeholder("you@example.com"))"#
        ).font_size(13.0))
}
