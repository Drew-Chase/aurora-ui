use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_sidebar() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Sidebar", "A layout with a collapsible side panel and main content area."))
        .child(crate::example_section("Default", "A sidebar with navigation and a main content region."))
        .child(crate::example_card(
            BoxWidget::new().height(300).child(
                sidebar::Sidebar::new()
                    .sidebar_content(
                        col!()
                            .spacing(4.0)
                            .child(Text::new("Navigation").font_size(14.0).font_weight(FontWeight::SemiBold))
                            .child(Text::new("Dashboard"))
                            .child(Text::new("Settings"))
                            .child(Text::new("Profile"))
                    )
                    .main_content(Text::new("Main content area"))
                    .sidebar_width(200.0)
            )
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"Sidebar::new()
    .sidebar_content(navigation)
    .main_content(content)
    .sidebar_width(200.0)"#
        ).font_size(13.0))
}
