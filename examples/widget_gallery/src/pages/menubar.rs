use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_menubar() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Menubar",
            "A horizontal application menu bar with dropdown sub-menus.",
        ))
        .child(crate::example_section(
            "Default",
            "Click a menu title to open its dropdown.",
        ))
        .child(crate::example_card(
            menubar::Menubar::new()
                .menu("File", vec!["New", "Open", "Save", "Exit"])
                .menu("Edit", vec!["Undo", "Redo", "Cut", "Copy", "Paste"])
                .menu("View", vec!["Zoom In", "Zoom Out", "Full Screen"]),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"Menubar::new()
    .menu("File", vec!["New", "Open", "Save", "Exit"])
    .menu("Edit", vec!["Undo", "Redo", "Cut", "Copy", "Paste"])
    .menu("View", vec!["Zoom In", "Zoom Out", "Full Screen"])"#,
                )
                .font_size(13.0),
        )
}
