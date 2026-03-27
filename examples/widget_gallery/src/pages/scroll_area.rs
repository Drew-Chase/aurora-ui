use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_scroll_area() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header("Scroll Area", "A scrollable container for overflowing content."))
        .child(crate::example_section("Default", "Content that exceeds the height becomes scrollable."))
        .child(crate::example_card(
            scroll_area::ScrollArea::new()
                .height(200.0)
                .child(
                    col!()
                        .spacing(8.0)
                        .child(Text::new("Item 1")).child(Text::new("Item 2")).child(Text::new("Item 3"))
                        .child(Text::new("Item 4")).child(Text::new("Item 5")).child(Text::new("Item 6"))
                        .child(Text::new("Item 7")).child(Text::new("Item 8")).child(Text::new("Item 9"))
                        .child(Text::new("Item 10")).child(Text::new("Item 11")).child(Text::new("Item 12"))
                )
        ))
        .child(code_block::CodeBlock::new().language("rust").code(
r#"ScrollArea::new()
    .height(200.0)
    .child(long_content)"#
        ).font_size(13.0))
}
