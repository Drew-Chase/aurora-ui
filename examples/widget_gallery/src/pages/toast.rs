use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_toast() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Toast", "A notification message for transient feedback."))
        .child(crate::example_section("Variants", "Toasts come in several semantic styles."))
        .child(crate::example_card(
            col!()
                .spacing(12.0)
                .child(toast::Toast::new("Your message has been sent.").visible(true))
                .child(toast::Toast::new("File saved successfully.").title("Success").variant(toast::ToastVariant::Success).visible(true))
                .child(toast::Toast::new("Something went wrong.").title("Error").variant(toast::ToastVariant::Error).visible(true))
                .child(toast::Toast::new("Please check your input.").title("Warning").variant(toast::ToastVariant::Warning).visible(true))
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"Toast::new("Your message has been sent.")
    .visible(true)

Toast::new("File saved successfully.")
    .title("Success")
    .variant(ToastVariant::Success)
    .visible(true)"#
        ).font_size(13.0))
}
