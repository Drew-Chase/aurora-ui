use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_input_group() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Input Group", "A text input with attached prefix or suffix elements."))
        .child(crate::example_section("Default", "Combine inputs with labels or buttons."))
        .child(crate::example_card(
            col!()
                .spacing(16.0)
                .child(input_group::InputGroup::new(TextInput::new().placeholder("Amount")).prefix(Text::new("$")))
                .child(input_group::InputGroup::new(TextInput::new().placeholder("Search...")).suffix(button!("Go")))
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"InputGroup::new(TextInput::new().placeholder("Amount"))
    .prefix(Text::new("$"))

InputGroup::new(TextInput::new().placeholder("Search..."))
    .suffix(button!("Go"))"#
        ).font_size(13.0))
}
