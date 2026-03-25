use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_alert() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(crate::page_header("Alert", "Displays a callout for important information."))
        .child(crate::example_section("Variants", "Alerts come in several semantic variants."))
        .child(crate::example_card(
            col!()
                .spacing(12.0)
                .child(alert::Alert::new().title("Heads up!").description("You can add components to your app using the CLI."))
                .child(alert::Alert::new().variant(alert::AlertVariant::Success).title("Success").description("Your changes have been saved."))
                .child(alert::Alert::new().variant(alert::AlertVariant::Warning).title("Warning").description("This action cannot be undone."))
                .child(alert::Alert::new().variant(alert::AlertVariant::Destructive).title("Error").description("Something went wrong. Please try again."))
        ))
}
