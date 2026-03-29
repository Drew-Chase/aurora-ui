use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_navigation_menu() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Navigation Menu",
            "A horizontal navigation bar with selectable items.",
        ))
        .child(crate::example_section(
            "Default",
            "Click an item to navigate.",
        ))
        .child(crate::example_card(
            navigation_menu::NavigationMenu::new()
                .item("Getting Started")
                .item("Components")
                .item("Documentation")
                .item("Examples"),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"NavigationMenu::new()
    .item("Getting Started")
    .item("Components")
    .item("Documentation")
    .item("Examples")"#,
                )
                .font_size(13.0),
        )
}
