use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_kbd() -> impl Widget {
    col!()
        .spacing(24.0)
        .child(crate::page_header("Keyboard Shortcut", "A visual indicator for keyboard shortcuts."))
        .child(crate::example_section("Default", "Display keyboard shortcuts."))
        .child(crate::example_card(
            row!()
                .spacing(8.0)
                .align(Align::Center)
                .child(kbd::Kbd::new("Ctrl"))
                .child(label::Label::new("+"))
                .child(kbd::Kbd::new("S"))
                .child(BoxWidget::new().width(24))
                .child(kbd::Kbd::new("Ctrl+Shift+P"))
        ))
}
