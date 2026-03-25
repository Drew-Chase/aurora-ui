use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_accordion() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(crate::page_header("Accordion", "A vertically stacked set of interactive headings that reveal content."))
        .child(crate::example_section("Default", "Click a heading to expand its content."))
        .child(crate::example_card(
            accordion::Accordion::new()
                .width(500.0)
                .section("Is it accessible?", typography::Typography::paragraph("Yes. It adheres to the WAI-ARIA design pattern."))
                .section("Is it styled?", typography::Typography::paragraph("Yes. It comes with default styles that match the other components."))
                .section("Is it animated?", typography::Typography::paragraph("Yes. It smoothly animates the content open and closed."))
                .expanded(0)
        ))
}
