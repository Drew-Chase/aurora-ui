use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_input_otp() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Input OTP", "A one-time password input with individual character slots."))
        .child(crate::example_section("Default", "Type digits into each slot. Auto-advances to the next."))
        .child(crate::example_card(
            input_otp::InputOtp::new(6)
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"InputOtp::new(6)
    .on_complete(|code| println!("OTP entered: {code}"))"#
        ).font_size(13.0))
}
