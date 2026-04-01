use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_number_input() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Number Input",
            "A numeric input with increment/decrement buttons, min/max, and step.",
        ))
        .child(crate::example_section("Default", "Click + or - to change the value."))
        .child(crate::example_card(
            col!().spacing(16.0)
                .child(number_input::NumberInput::new().value(42.0).width(200.0))
                .child(number_input::NumberInput::new().value(3.15).min(0.0).max(10.0).step(0.1).precision(2).width(200.0))
                .child(number_input::NumberInput::new().value(0.0).disabled(true).width(200.0))
        ))
        .child(code_block::CodeBlock::new().language("rust").code(r#"NumberInput::new()
    .value(42.0)
    .min(0.0)
    .max(100.0)
    .step(1.0)
    .precision(0)
    .width(200.0)
    .on_change(|val| println!("{val}"))"#).font_size(13.0))
}
