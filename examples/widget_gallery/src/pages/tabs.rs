use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_tabs() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Tabs", "A set of layered sections of content shown one at a time."))
        .child(crate::example_section("Default", "Click a tab to switch content."))
        .child(crate::example_card(
            tabs::Tabs::new()
                .width(500.0)
                .tab("Account", typography::Typography::paragraph("Make changes to your account here."))
                .tab("Password", typography::Typography::paragraph("Change your password here."))
                .tab("Settings", typography::Typography::paragraph("Manage your settings and preferences."))
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"Tabs::new()
    .width(500.0)
    .tab("Account", Typography::paragraph("Make changes to your account here."))
    .tab("Password", Typography::paragraph("Change your password here."))
    .tab("Settings", Typography::paragraph("Manage your settings."))"#
        ).font_size(13.0))
}
