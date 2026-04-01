use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_stepper() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Stepper",
            "A step-by-step wizard / progress indicator showing sequential stages.",
        ))
        .child(crate::example_section(
            "Horizontal",
            "Steps laid out horizontally with connecting lines.",
        ))
        .child(crate::example_card(
            stepper::Stepper::new()
                .step("Account")
                .step("Profile")
                .step("Review")
                .step("Complete")
                .current(2),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"Stepper::new()
    .step("Account")
    .step("Profile")
    .step("Review")
    .step("Complete")
    .current(2)"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Vertical",
            "Steps stacked vertically, useful for sidebar wizards.",
        ))
        .child(crate::example_card(
            stepper::Stepper::new()
                .step("Create account")
                .step("Set up profile")
                .step("Choose plan")
                .step("Confirmation")
                .current(1)
                .orientation(stepper::StepperOrientation::Vertical),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"Stepper::new()
    .step("Create account")
    .step("Set up profile")
    .step("Choose plan")
    .step("Confirmation")
    .current(1)
    .orientation(StepperOrientation::Vertical)"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Custom Colors",
            "A stepper with custom completed and current step colors.",
        ))
        .child(crate::example_card(
            stepper::Stepper::new()
                .step("Upload")
                .step("Process")
                .step("Done")
                .current(1)
                .completed_color(colors::success())
                .current_color(colors::info())
                .step_size(36.0),
        ))
}
