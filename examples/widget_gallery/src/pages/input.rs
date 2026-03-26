use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_input() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Input", "A text input field for user data entry."))
        .child(crate::example_section("Default", "A basic text input."))
        .child(crate::example_card(
            col!()
                .spacing(12.0)
                .child(field::Field::new("Email").width(350.0).input(TextInput::new().placeholder("Enter your email")))
                .child(field::Field::new("Password").width(350.0).input(TextInput::new().password(true).placeholder("Enter your password")))
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"Field::new("Email")
    .width(350.0)
    .input(TextInput::new().placeholder("Enter your email"))

Field::new("Password")
    .width(350.0)
    .input(
        TextInput::new()
            .password(true)
            .placeholder("Enter your password"),
    )"#
        ).font_size(13.0))
}
