use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_sheet(setter: StateSetter<crate::GalleryState>) -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Sheet",
            "A slide-in panel from the edge of the screen with an overlay backdrop.",
        ))
        .child(crate::example_section(
            "Default",
            "Click the button to open a sheet sliding in from the right side.",
        ))
        .child(crate::example_card(
            button!("Open Sheet").on_click(move |_| setter.set(|s| s.sheet_open = true)),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"Sheet::new()
    .open(show_sheet)
    .side(SheetSide::Right)
    .panel_width(400.0)
    .child(
        col!()
            .child(Text::new("Sheet content"))
            .child(button!("Close"))
    )
    .on_close(|| set_open(false))"#,
                )
                .font_size(13.0),
        )
}
