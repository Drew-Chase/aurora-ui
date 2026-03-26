use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_switch() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Switch", "A control that allows the user to toggle between on and off."))
        .child(crate::example_section("Default", "Toggle switches in different states."))
        .child(crate::example_card(
            row!()
                .spacing(24.0)
                .align(Align::Center)
                .child(col!().spacing(8.0).align(Align::Center).child(switch::Switch::new().checked(true)).child(label::Label::new("On").font_size(12.0)))
                .child(col!().spacing(8.0).align(Align::Center).child(switch::Switch::new()).child(label::Label::new("Off").font_size(12.0)))
                .child(col!().spacing(8.0).align(Align::Center).child(switch::Switch::new().disabled(true)).child(label::Label::new("Disabled").font_size(12.0)))
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"// Checked switch
Switch::new().checked(true)

// Unchecked switch
Switch::new()

// Disabled switch
Switch::new().disabled(true)"#
        ).font_size(13.0))
}
